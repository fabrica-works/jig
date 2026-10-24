use super::*;
use skein_fake_machine::{How, Item, Machine};
use skein_lib::Token;

#[test]
fn a_repair_snapshot_preserves_the_durable_prefix_through_another_restart() {
    let limits = crate::fixtures::store_limits();
    let codec = codec::limits(&limits.root.core).expect("fixture codec");
    let row = core::Record::Core(core::CoreRecord::Terminal(core::TerminalRecord {
        task: 1,
        attempt: 1,
        cumulative: 0,
        end: jig_domain_tasks::End::Finished {
            result: jig_domain_tasks::TaskResult::Report { words: b"durable".as_slice().into() },
            cancel_delegates: false,
        },
    }));
    let key = keys::encode(&row.key(), 1024).expect("fixture key");
    let value = codec::encode(&row, &codec, crate::fixtures::store_names()).expect("fixture row");
    let first = log_frame(1, &key, &value);
    let second = log_frame(2, &key, &value);
    let mut machine = Machine::for_user(1000);
    let root = machine.lay(&[Item::file(b"log", b"")]);
    let file = machine.open(root, b"log", How::Write).expect("log");
    machine.write(file, 0, &first).expect("first write");
    machine.sync(file);
    machine.write(file, first.len() as u64, &second).expect("second write");
    machine.crash(6);
    let root = machine.reopen_root(0);
    let file = machine.open(root, b"log", How::Read).expect("recovered log");
    let bytes = machine.read(file, 0, 4096).expect("recovery read");
    assert!(bytes.len() > first.len() && bytes.len() < first.len() + second.len(), "power loss tears second write");
    let mut store = Store::new();
    let mut events = vec![Observed::Bytes { path: log_path(), at: 0, bytes }, Observed::Recovered(log_path())];
    assert_eq!(store.observe(&mut events, &codec, crate::fixtures::store_names(), &limits.root.connector).len(), 1);
    assert_eq!(store.applied, 1);
    let snapshot = snapshot(&key, &value);
    let mut capture = Capture::default();
    capture.names.insert(1, b"generations/7/store/snapshot.tmp".to_vec());
    let write = Op::Write { fd: Fd::new(1), bytes: snapshot.clone().into(), from: 0, at: 0 };
    assert!(!capture.pending(&write), "repair snapshot write reaches the observer without yielding log frames");
    capture.complete(&Complete {
        op: Token::new(1),
        kind: write,
        result: Ok(Done::Count(u32::try_from(snapshot.len()).expect("fixture length"))),
    });
    capture.complete(&Complete { op: Token::new(2), kind: Op::Sync { fd: Fd::new(1) }, result: Ok(Done::Nothing) });
    assert!(
        store.observe(&mut capture.events, &codec, crate::fixtures::store_names(), &limits.root.connector).is_empty()
    );
    assert_eq!(store.rows.get(&root::Record::Core(row.clone()).key()), Some(&root::Record::Core(row.clone())));
    store.recovering();
    capture.names.insert(1, b"generations/7/store/snapshot".to_vec());
    capture.complete(&Complete {
        op: Token::new(3),
        kind: Op::Read { fd: Fd::new(1), buf: snapshot.clone().into(), at: 0 },
        result: Ok(Done::Count(u32::try_from(snapshot.len()).expect("fixture length"))),
    });
    assert_eq!(
        store.observe(&mut capture.events, &codec, crate::fixtures::store_names(), &limits.root.connector).len(),
        1
    );
    assert_eq!(store.applied, 1);
    assert_eq!(store.recovered, [1].into());
    assert_eq!(store.rows.get(&root::Record::Core(row.clone()).key()), Some(&root::Record::Core(row)));
}

fn log_path() -> Vec<u8> {
    b"generations/7/store/log-00000000000000000001".to_vec()
}

fn log_frame(number: u64, key: &[u8], value: &[u8]) -> Vec<u8> {
    let mut body = vec![1];
    body.extend_from_slice(&(u32::try_from(key.len()).expect("fixture length")).to_be_bytes());
    body.extend_from_slice(key);
    body.extend_from_slice(&(u32::try_from(value.len()).expect("fixture length")).to_be_bytes());
    body.extend_from_slice(value);
    let mut bytes = b"SKVC\0\x01".to_vec();
    bytes.extend_from_slice(&number.to_be_bytes());
    bytes.extend_from_slice(&1_u32.to_be_bytes());
    bytes.extend_from_slice(&(u32::try_from(body.len()).expect("fixture length")).to_be_bytes());
    bytes.extend_from_slice(&body);
    bytes.extend_from_slice(&transcripts::frame::crc32c(&bytes).to_be_bytes());
    bytes
}

fn snapshot(key: &[u8], value: &[u8]) -> Vec<u8> {
    let mut bytes = b"SKVS\0\x01".to_vec();
    bytes.extend_from_slice(&2_u64.to_be_bytes());
    bytes.extend_from_slice(&transcripts::frame::crc32c(&bytes).to_be_bytes());
    let mut chunk = 1_u32.to_be_bytes().to_vec();
    chunk.extend_from_slice(&(u32::try_from(8 + key.len() + value.len()).expect("fixture length")).to_be_bytes());
    chunk.extend_from_slice(&(u32::try_from(key.len()).expect("fixture length")).to_be_bytes());
    chunk.extend_from_slice(&(u32::try_from(value.len()).expect("fixture length")).to_be_bytes());
    chunk.extend_from_slice(key);
    chunk.extend_from_slice(value);
    chunk.extend_from_slice(&transcripts::frame::crc32c(&chunk).to_be_bytes());
    bytes.extend_from_slice(&chunk);
    let mut trailer = b"SNPE".to_vec();
    trailer.extend_from_slice(&1_u64.to_be_bytes());
    trailer.extend_from_slice(&((key.len() + value.len()) as u64).to_be_bytes());
    trailer.extend_from_slice(&transcripts::frame::crc32c(&trailer).to_be_bytes());
    trailer.extend_from_slice(b"END!");
    bytes.extend_from_slice(&trailer);
    bytes
}
