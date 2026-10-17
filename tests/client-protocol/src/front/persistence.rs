//! Each owner's records are encoded by that owner's codec.
use super::{ClientId, Link, Protocol, Queue, Side, Time, client, family, fixtures, link};
use skein_lib::{List, RequestRecord};
pub(super) type Bytes = Vec<(Box<[u8]>, Box<[u8]>)>;
fn saved(record: client::store::Record) -> link::Saved {
    match record {
        client::store::Record::Table { version, record } => link::Saved {
            version,
            record: RequestRecord {
                key: record.key,
                scope: record.scope,
                first_sent: record.first_sent,
                request: jig_client_domain::Held {
                    key: record.request.key,
                    request: match record.request.request {
                        client::Ask::Jig(value) => jig_client_domain::Ask::Jig(value),
                        client::Ask::First(value) => {
                            jig_client_domain::Ask::Family(family::request(&value).expect("family ask"))
                        }
                    },
                },
            },
        },
        client::store::Record::Jig(_) | client::store::Record::First(_) => unreachable!("table record"),
    }
}
pub(super) fn encode(records: &[client::store::Record]) -> Bytes {
    records.iter().cloned().map(encode_one).map(|record| (record.key, record.bytes)).collect()
}
pub(crate) fn encode_one(record: client::store::Record) -> link::Record {
    use jig_client_view::records as ui;
    match record {
        client::store::Record::Jig(record) => {
            let record = match record {
                jig_client_view::store::Record::Place { version, party, address } => {
                    ui::Record::Place { version, party, address }
                }
                jig_client_view::store::Record::Draft { version, party, task, words, touched } => {
                    ui::Record::Draft { version, party, task, words, touched }
                }
            };
            let (key, bytes) = ui::encode(&record);
            link::Record { key, bytes }
        }
        client::store::Record::First(record) => {
            link::Record { key: Box::from(b"first.place".as_slice()), bytes: family::saved(&record) }
        }
        record @ client::store::Record::Table { .. } => {
            let mut link = Link::new(&fixtures::link_limits()).expect("bounded store codec");
            let mut out = Queue::with_capacity(256);
            link.step(Time::ZERO, link::Event::Save { record: saved(record) }, &mut out);
            let Some(link::Request::Save { record }) = out.pop() else { panic!("save writes its record") };
            assert!(out.is_empty());
            record
        }
    }
}
pub(crate) fn child_record(record: &link::Record) -> Option<client::store::Record> {
    use jig_client_view::records as ui;
    if record.key.as_ref() == b"first.place" {
        return family::restored(&record.bytes).map(client::store::Record::First);
    }
    match ui::decode(&record.key, &record.bytes)? {
        ui::Record::Place { version, party, address } => {
            Some(client::store::Record::Jig(jig_client_view::store::Record::Place { version, party, address }))
        }
        ui::Record::Draft { version, party, task, words, touched } => {
            Some(client::store::Record::Jig(jig_client_view::store::Record::Draft {
                version,
                party,
                task,
                words,
                touched,
            }))
        }
    }
}
pub(crate) fn erased(key: client::store::Key) -> Box<[u8]> {
    use jig_client_view::records::RecordKey;
    match key {
        client::store::Key::Jig(key) => match key {
            jig_client_view::store::StoreKey::Place => RecordKey::Place.encode(),
            jig_client_view::store::StoreKey::Draft { task } => RecordKey::Draft { task }.encode(),
        },
        client::store::Key::First(_) => Box::from(b"first.place".as_slice()),
        client::store::Key::Table(key) => Box::new(key),
    }
}
pub(crate) fn restored(saved: link::Saved) -> client::store::Record {
    let link::Saved { version, record } = saved;
    client::store::Record::Table {
        version,
        record: RequestRecord {
            key: record.key,
            scope: record.scope,
            first_sent: record.first_sent,
            request: client::store::Held {
                key: record.request.key,
                request: match record.request.request {
                    jig_client_domain::Ask::Jig(value) => client::Ask::Jig(value),
                    jig_client_domain::Ask::Family(value) => {
                        let decoded =
                            jig_test_service::family::request(&value.bytes, &fixtures::owner_limits().connector)
                                .expect("family ask");
                        client::Ask::First(client::first::Ask::ChangeObject {
                            project: decoded.project,
                            object: decoded.value.object,
                            revision: jig_client_domain::contract::Revision { value: decoded.value.revision },
                            value: decoded.value.value,
                        })
                    }
                },
            },
        },
    }
}
pub(super) fn decode(bytes: &Bytes) -> Vec<client::store::Record> {
    let mut link = Link::new(&fixtures::link_limits()).expect("bounded store codec");
    let mut kept = List::with_capacity(u32::try_from(bytes.len()).expect("bounded small store"));
    let mut records = Vec::new();
    for (key, bytes) in bytes {
        let record = link::Record { key: key.clone(), bytes: bytes.clone() };
        if key.len() == 16 {
            kept.push(record).expect("exact store capacity");
        } else if let Some(record) = child_record(&record) {
            records.push(record);
        }
    }
    let mut out = Queue::with_capacity(256);
    link.step(Time::ZERO, link::Event::Start { kept }, &mut out);
    while let Some(request) = out.pop() {
        if let link::Request::Kept { record } = request {
            records.push(restored(record));
        } else {
            panic!("admitted record must restore");
        }
    }
    records
}
impl Protocol {
    pub(super) fn reload_side(&mut self, client: ClientId, now: Time) {
        self.net.cut_client(client);
        self.advance(now, fixtures::wall(now));
        let mut side = Side::new();
        let old = self.clients.get(&client).expect("bound side");
        side.authenticated = old.authenticated;
        if let Some(credential) = old.link.credential() {
            side.event(now, link::Event::SignIn { with: link::With::Held(credential.clone()) });
        }
        // Connection names survive the shell restart so late bytes cannot name a new socket.
        self.net.retire(client);
        self.clients.insert(client, side);
        self.callbacks.retain(|(id, _)| *id != client);
    }
    #[must_use]
    pub fn pending_bytes(&self, client: ClientId) -> usize {
        self.stores.get(&client).map_or(0, |bytes| bytes.iter().map(|(_, value)| value.len()).sum())
    }
}
