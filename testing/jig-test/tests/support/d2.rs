//! D2: removed by JP13. The one handed-in fixture's explicit rows, secret and runtime input.
use jig_domain_core as core;
use jig_protocol_config as config;
use jig_protocol_store::{codec, format, keys, transcripts};
use jig_test_domain as root;
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

pub fn preseed(directory: &Path, raw: &[u8]) {
    let bytes = fs::read(directory.join("config.json")).expect("D2 configuration");
    let document = config::document(&bytes, &config::DocumentLimits { bytes: 1 << 20, tokens: 1 << 20, depth: 32 })
        .expect("D2 document");
    let at = config::read::Object::new(&document, 0)
        .expect("bounded D2 fixture")
        .member(&document, b"engine")
        .expect("bounded D2 fixture");
    let engine =
        config::engine(&document, at, &config::Directories::working()).expect("providers-only D2 configuration");
    let record = config::handed_in(raw, &config::DocumentLimits { bytes: 1 << 20, tokens: 1 << 20, depth: 16 })
        .expect("D2 handed-in decoder");
    let fixture = jig_test_service::d2_preseed(
        jig_test_service::D2_ACCOUNT,
        jig_test_service::D2_SOURCE,
        jig_test_service::D2_PROVIDER,
        jig_test_service::D2_HOLDER,
        record.token,
        record.lapses,
        record.account_id,
        &engine.limits.driver.client.document,
    )
    .expect("D2 preseed");
    let mut configuration = jig_core_world::world::config(7);
    configuration.core = config::core(&engine, [7; 16], 7);
    let mut limits = jig_core_world::world::limits();
    limits.core = engine.limits.core;
    limits.journal = engine.limits.journal;
    let room = core::room_max(&limits.core).expect("bounded D2 fixture");
    limits.journal.writes = limits.journal.writes.max(room.writes + 128);
    limits.journal.held = limits.journal.held.max(room.held + 128);
    let mut world = jig_core_world::effects::World::first_start(configuration, limits);
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    let bounds = codec::limits(&engine.limits.core).expect("bounded D2 fixture");
    let names = codec::names(&engine.llm.models);
    let mut rows = std::collections::BTreeMap::new();
    for value in world.store.rows.values() {
        if let root::Record::Core(record) = value {
            let mut record = record.clone();
            if let core::Record::Core(core::CoreRecord::Deployment(header)) = &mut record {
                header.commits = 1;
            }
            rows.insert(
                keys::encode(&record.key(), u32::MAX).expect("bounded D2 fixture"),
                codec::encode(&record, &bounds, &names).expect("bounded D2 fixture"),
            );
        }
    }
    for record in fixture.rows {
        let record = core::Record::Accounts(record);
        rows.insert(
            keys::encode(&record.key(), u32::MAX).expect("bounded D2 fixture"),
            codec::encode(&record, &bounds, &names).expect("bounded D2 fixture"),
        );
    }
    rows.insert(
        keys::encode_own(&keys::Own::Format, u32::MAX).expect("bounded D2 fixture"),
        format::current(1, [7; 16]).encode().expect("bounded D2 fixture"),
    );
    rows.insert(
        keys::installation(u32::MAX).expect("bounded D2 fixture"),
        codec::accounts::Installation { id: [9; 16] }.encode(),
    );
    fs::create_dir_all(directory.join("state/store")).expect("bounded D2 fixture");
    fs::create_dir_all(directory.join("state/secrets")).expect("bounded D2 fixture");
    for name in ["state/store", "state/secrets"] {
        fs::set_permissions(directory.join(name), fs::Permissions::from_mode(0o700)).expect("bounded D2 fixture");
    }
    super::private(&directory.join("state/store/log-00000000000000000001"), &frame(rows.into_iter().collect()));
    let mut key = skein_kv::KeyWriter::new(64);
    key.tag(3).tag(0).u64(u64::from(jig_test_service::D2_SOURCE));
    super::private(
        &directory.join("state/secrets/log-00000000000000000001"),
        &frame(vec![(key.finish().expect("bounded D2 fixture"), fixture.value.expose().into())]),
    );
    super::private(&directory.join("d2.json"), raw);
    drop(fixture.input); // The explicit serve-d2 command calls this same typed builder.
}

/// Independent KV fixture framing; every jig value above uses its real codec.
type Row = (Box<[u8]>, Box<[u8]>);
fn frame(rows: Vec<Row>) -> Vec<u8> {
    let count = u32::try_from(rows.len()).expect("bounded D2 rows");
    let mut body = Vec::new();
    for (key, value) in rows {
        body.push(1);
        body.extend_from_slice(&u32::try_from(key.len()).expect("bounded D2 fixture").to_be_bytes());
        body.extend_from_slice(&key);
        body.extend_from_slice(&u32::try_from(value.len()).expect("bounded D2 fixture").to_be_bytes());
        body.extend_from_slice(&value);
    }
    let mut bytes = b"SKVC\0\x01".to_vec();
    bytes.extend_from_slice(&1_u64.to_be_bytes());
    bytes.extend_from_slice(&count.to_be_bytes());
    bytes.extend_from_slice(&u32::try_from(body.len()).expect("bounded D2 fixture").to_be_bytes());
    bytes.extend_from_slice(&body);
    bytes.extend_from_slice(&transcripts::frame::crc32c(&bytes).to_be_bytes());
    bytes
}
