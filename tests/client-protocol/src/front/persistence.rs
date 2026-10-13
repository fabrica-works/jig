//! The world's small store uses the link's actual owner-versioned records.
use super::{ClientId, Link, Protocol, Queue, Side, Time, client, family, fixtures, link};
use skein_lib::{List, RequestRecord};
pub(super) type Bytes = Vec<(Box<[u8]>, Box<[u8]>)>;
fn saved(record: client::store::Record) -> link::Saved {
    match record {
        client::store::Record::Jig(record) => link::Saved::Jig(record),
        client::store::Record::First(record) => {
            link::Saved::Family { family: 1, version: 1, key: Box::new([]), bytes: family::saved(&record) }
        }
        client::store::Record::Table { version, record } => link::Saved::Table {
            version,
            record: RequestRecord {
                key: record.key,
                scope: record.scope,
                first_sent: record.first_sent,
                request: match record.request {
                    client::Ask::Jig(value) => link::Ask::Jig(value),
                    client::Ask::First(value) => link::Ask::Family(family::request(&value).expect("family ask")),
                },
            },
        },
    }
}
pub(super) fn encode(records: &[client::store::Record]) -> Bytes {
    let mut link = Link::new(&fixtures::link_limits()).expect("bounded store codec");
    let mut out = Queue::with_capacity(256);
    let mut bytes = Vec::new();
    for record in records {
        link.step(Time::ZERO, link::Event::Save { record: saved(record.clone()) }, &mut out);
        while let Some(request) = out.pop() {
            if let link::Request::Write { key, record } = request {
                bytes.push((key, record));
            } else {
                panic!("save must write its record");
            }
        }
        link.step(Time::ZERO, link::Event::Written, &mut out);
    }
    bytes
}
fn restored(saved: link::Saved) -> client::store::Record {
    match saved {
        link::Saved::Jig(record) => client::store::Record::Jig(record),
        link::Saved::Family { bytes, .. } => client::store::Record::First(family::restored(&bytes).expect("family")),
        link::Saved::Table { version, record } => client::store::Record::Table {
            version,
            record: RequestRecord {
                key: record.key,
                scope: record.scope,
                first_sent: record.first_sent,
                request: match record.request {
                    link::Ask::Jig(value) => client::Ask::Jig(value),
                    link::Ask::Family(value) => {
                        let decoded =
                            jig_test_service::family::request(&value.bytes, &fixtures::owner_limits().connector)
                                .expect("family ask");
                        client::Ask::First(client::first::Ask::ChangeObject {
                            project: decoded.project,
                            object: decoded.value.object,
                            revision: jig_domain_client::contract::Revision { value: decoded.value.revision },
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
    let versions = jig_client_world::test::list([
        link::StoreVersion { keeper: link::Keeper::Jig, version: jig_domain_client::store::STORE_VERSION },
        link::StoreVersion { keeper: link::Keeper::Table, version: client::store::VERSION },
        link::StoreVersion { keeper: link::Keeper::Family(1), version: 1 },
    ]);
    let mut kept = List::with_capacity(u32::try_from(bytes.len()).expect("bounded small store"));
    for (key, record) in bytes {
        kept.push(link::StoreRead { key: key.clone(), record: record.clone() }).expect("exact store capacity");
    }
    let mut out = Queue::with_capacity(256);
    link.step(Time::ZERO, link::Event::Start { versions, kept }, &mut out);
    let mut records = Vec::new();
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
        side.connections.set_token(old.connections.token().map(|token| token.expose().into()));
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
