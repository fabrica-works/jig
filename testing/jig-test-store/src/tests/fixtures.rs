use jig_test_connector as d;
use skein_lib::{Rng, Token, Wall};
pub const FIRST: u16 = 1;
pub const SECOND: u16 = 2;
pub use jig_test_connector_world::LIMITS;
pub fn path() -> d::Path {
    jig_test_connector_world::path(1, 1)
}
pub fn key() -> d::Key {
    d::Key { attempt: 1, completion: 2, position: 3, deployment: [0; 16], task: 4, purpose: 5 }
}
pub fn effect() -> d::Effect {
    d::Effect { kind: 1, resources: Box::new([path()]), purpose: 2, condition: Some(3), target: 4, state: 5 }
}
pub fn record_key(random: &mut Rng) -> (u16, d::RecordKey) {
    let connector = if random.below(2) == 0 { FIRST } else { SECOND };
    let number = random.next_u64();
    let record = match random.below(12) {
        0 => d::RecordKey::ApplicationObject(number),
        1 => d::RecordKey::ApplicationAnswer(Token::new(number)),
        2 => d::RecordKey::Procedure(number),
        3 => d::RecordKey::ProcedureDelegateOutcome(number),
        4 => d::RecordKey::Result(number),
        5 => d::RecordKey::Task(number),
        6 => d::RecordKey::Adoption { project: u32::try_from(number & 255).expect("byte"), path: random_path(random) },
        7 => d::RecordKey::Subscription { topic: u16::try_from(number & 255).expect("byte"), task: random.next_u64() },
        8 => d::RecordKey::Pool(random_path(random)),
        9 => d::RecordKey::Proposal(number),
        10 => d::RecordKey::Outbox(random_key(random)),
        _ => d::RecordKey::Made(random_key(random)),
    };
    (connector, record)
}
fn random_path(random: &mut Rng) -> d::Path {
    let mut segments = skein_lib::List::with_capacity(2);
    for _ in 0..=random.below(2) {
        segments.push(Box::from([u8::try_from(random.below(256)).expect("byte")])).expect("bounded path");
    }
    d::Path::new(segments, LIMITS.segment_bytes).expect("bounded path")
}
fn random_key(random: &mut Rng) -> d::Key {
    let mut deployment = [0; 16];
    for byte in &mut deployment {
        *byte = u8::try_from(random.below(256)).expect("byte");
    }
    d::Key {
        attempt: random.next_u64(),
        completion: u32::try_from(random.below(u64::from(u32::MAX))).expect("u32"),
        position: u32::try_from(random.below(u64::from(u32::MAX))).expect("u32"),
        deployment,
        task: random.next_u64(),
        purpose: random.next_u64(),
    }
}
pub fn records() -> Box<[d::Record]> {
    Box::new([
        application_object(),
        application_answer(),
        proposal(),
        procedure(),
        delegate_outcome(),
        result(),
        task(),
        adoption(),
        subscription(),
        pool(),
        outbox(),
        made(),
    ])
}
fn application_object() -> d::Record {
    d::Record::ApplicationObject { object: 1, now: d::ApplicationObject { revision: 2, value: 3 } }
}
fn application_answer() -> d::Record {
    d::Record::ApplicationAnswer {
        token: Token::new(1),
        until: Wall::from_nanos(2),
        outcome: d::ApplicationOutcome::CoreRefused {
            why: d::application_core::Refusal::Authority {
                lacking: d::application_core::Lack::Budget { needs: 3, left: 2 },
                proposable: Some(true),
            },
        },
    }
}
fn proposal() -> d::Record {
    d::Record::Proposal { number: 1, task: 2, project: 3, effect: effect() }
}
fn procedure() -> d::Record {
    d::Record::Procedure(d::ProcedureState {
        project: 1,
        task: 2,
        number: 3,
        resource: path(),
        index: 4,
        steps: 5,
        awaiting: true,
        deadline: Some(Wall::from_nanos(6)),
    })
}
fn delegate_outcome() -> d::Record {
    d::Record::ProcedureDelegateOutcome { task: 1, step: 2, child: Some(3) }
}
fn result() -> d::Record {
    d::Record::Result { task: 1, code: 2 }
}
fn task() -> d::Record {
    d::Record::Task { task: 1, project: 2, resources: Box::new([path()]) }
}
fn adoption() -> d::Record {
    d::Record::Adoption { project: 1, path: path(), role: d::ResourceRole::Participating }
}
fn subscription() -> d::Record {
    d::Record::Subscription { topic: 1, task: 2, wake_at: 3, keep_at: 4 }
}
fn pool() -> d::Record {
    d::Record::Pool { path: path(), slots: 1 }
}
fn outbox() -> d::Record {
    d::Record::Outbox(d::OutboxEntry {
        number: 1,
        task: 2,
        key: key(),
        effect: effect(),
        attempt: Some(d::Attempt { number: 1, sent: Wall::from_nanos(2), deadline: Wall::from_nanos(3) }),
        phase: d::EffectPhase::Sent,
    })
}
fn made() -> d::Record {
    d::Record::Made { key: key(), resources: Box::new([path()]), state: 1 }
}
