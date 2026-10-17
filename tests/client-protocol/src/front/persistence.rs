//! Each owner's records are encoded by that owner's codec.
use super::{ClientId, Protocol, Side, Time, client, family, fixtures};
use jig_client_protocol_link as link;
pub(super) type Bytes = Vec<(Box<[u8]>, Box<[u8]>)>;
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
        client::store::Record::Client(record) => record,
        client::store::Record::Unlinked(_) => unreachable!("byte transport keeps the client's encoded records"),
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
        client::store::Key::Client(key) => key,
        client::store::Key::Unlinked(_) => unreachable!("byte transport uses encoded keys"),
    }
}
pub(super) fn decode(bytes: &Bytes) -> Vec<client::store::Record> {
    bytes
        .iter()
        .filter_map(|(key, bytes)| {
            let record = link::Record { key: key.clone(), bytes: bytes.clone() };
            if key.len() == 16 { Some(client::store::Record::Client(record)) } else { child_record(&record) }
        })
        .collect()
}
impl Protocol {
    pub(super) fn reload_side(&mut self, client: ClientId, now: Time) {
        self.net.cut_client(client);
        self.advance(now, fixtures::wall(now));
        let credential = self.clients[&client].wire.credential();
        let mut side = Side::new();
        side.wire.restore(credential);
        // Connection names survive the shell restart so late bytes cannot name a new socket.
        self.net.retire(client);
        self.clients.insert(client, side);
        self.callbacks.retain(|(id, _)| *id != client);
        self.receipts.retain(|(id, _, _, _)| *id != client);
    }
    #[must_use]
    pub fn pending_bytes(&self, client: ClientId) -> usize {
        self.stores.get(&client).map_or(0, |bytes| bytes.iter().map(|(_, value)| value.len()).sum())
    }
}
