use jig_core_world::effects::World;
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_protocol_store::{KeptFamily, KeptMap, Owner, PatchSet, PatchSets};
use jig_test_domain as root;
use jig_test_patches_smith as fixture;
use skein_lib::{Duration, Token};

pub const SEED: u64 = 118;
const TURN: KeptFamily = KeptFamily {
    name: "fixture-turn",
    texts: &[fixture::patches::kept::fixture_turn::TEXTS[0]],
    map: KeptMap::Store,
    keys: &[],
    frames: Some(fixture::ROOT),
};
const SMITH: PatchSet =
    PatchSet { owner: Owner::Smith, name: fixture::NAME, families: &[TURN], patches: &[], fresh: Some(fixture::FRESH) };
pub const BEFORE: PatchSets = PatchSets { sets: &[SMITH] };
pub const AFTER: PatchSets =
    PatchSets { sets: &[PatchSet { patches: &["patch 1\nowner fixture-smith 1\nmark fresh-windows\n"], ..SMITH }] };
// The fixture binary renders the said field of each loaded v1 turn.
pub const TAIL: &[u8] = b"closed windowwindow continuation";

fn turn_text(bytes: &[u8]) -> Option<Box<[u8]>> {
    let schema = skein_codec::schema::parse(TURN.texts[0].as_bytes()).ok()?;
    let root = schema.find(fixture::ROOT.as_bytes())?;
    let mut value = skein_codec::value::Value::with_room(3);
    let turn = value.decode(&schema, root, bytes.into()).ok()?;
    let said = value.field(turn, fixture::patches::kept::fixture_turn::v1::turn::SAID).ok()?;
    Some(value.content(said).ok()?.into())
}

pub fn turn(said: &[u8]) -> Box<[u8]> {
    // Independent v1 encoding: version, said's text and replay's bytes.
    let mut bytes = 1_u16.to_be_bytes().to_vec();
    for field in [said, b"fixture replay".as_slice()] {
        bytes.extend(u32::try_from(field.len()).expect("fixture field size").to_be_bytes());
        bytes.extend(field);
    }
    let schema = skein_codec::schema::parse(TURN.texts[0].as_bytes()).expect("kept v1 schema");
    let root = schema.find(fixture::ROOT.as_bytes()).expect("fixture root");
    // One record and its two fields: validate the independent bytes against
    // the kept text, then take the value's canonical encoding.
    let mut value = skein_codec::value::Value::with_room(3);
    value.decode(&schema, root, bytes.into_boxed_slice()).expect("fixture v1 turn");
    value.bytes().into()
}

pub fn parked() -> World {
    let mut world = World::with_patch_sets(SEED, false, BEFORE);
    world.store.transcript_text(turn_text);
    world.turn(1, 1, None, &turn(b"earlier window"));
    world.end(tasks::End::Parked, 1);
    let read = world.say(86);
    world.window_turn(1, 1, Some(read), &turn(b"closed window"));
    world.turn(2, 2, Some(read), &turn(b"window continuation"));
    world.end(tasks::End::Parked, 2);
    world
}

pub fn restart(world: &mut World, sets: PatchSets) {
    world.restart_with_patch_sets(SEED, false, sets);
    hello(world);
    world.grant_account(1);
}
pub fn hello(world: &mut World) {
    world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Hello {
        worker: 0,
        channel: Token::new(7),
        hello: jig_domain_fleet::Hello {
            bounds: jig_core_world::fixtures::bounds(),
            stop_bound: Duration::from_millis(100),
            slots: 1,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
}

pub fn proof(world: &World) -> &core::RunProof {
    let task = world.assigned.as_ref().expect("chat assignment").task;
    let key = root::Key::Core(core::Key::Core(core::CoreKey::RunProof(task)));
    let root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof))) = &world.store.rows[&key] else {
        panic!("durable run proof")
    };
    proof
}
pub fn tail(world: &World) -> Option<&[u8]> {
    world.assigned.as_ref().expect("chat assignment").brief.iter().find_map(|section| {
        if let jig_domain_brief::GatherPlaced::Core { kind: jig_domain_brief::Core::TranscriptTail, text, .. } = section
        {
            Some(text.as_ref())
        } else {
            None
        }
    })
}
pub fn tries(world: &World) -> tasks::Tries {
    let task = proof(world).task;
    let key = root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)));
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) = &world.store.rows[&key] else {
        panic!("live chat")
    };
    row.tries
}
pub fn activity(world: &World) -> Vec<tasks::Line> {
    world
        .store
        .rows
        .values()
        .filter_map(|row| {
            if let root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(row))) = row {
                Some(row.line.clone())
            } else {
                None
            }
        })
        .collect()
}
pub fn fresh(world: &World) {
    let assignment = world.assigned.as_ref().expect("fresh assignment");
    assert!(!assignment.run.policy.resume);
    assert!(assignment.transcript.is_empty());
    assert_eq!(assignment.conversation, assignment.attempt);
    assert_eq!(proof(world).conversation, assignment.attempt);
    assert_eq!(proof(world).window, None);
    assert!(matches!(activity(world).last(), Some(tasks::Line::RunBegan { resumed: false, .. })));
}
