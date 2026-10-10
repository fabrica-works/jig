use super::script;
use crate::{Change, Engine, Invalid, Up};
use jig_client::contract::*;
use skein_lib::{Time, Token, Wall};
#[test]
fn person_owner_public_read_preserves_whole_optional_results_and_original_object_identity() {
    let value = crate::owner_person_task::tests::value();
    let mut engine = Engine::new(script(), 37).expect("ordinary fixture admitted");
    let commit = engine
        .act(
            Time::ZERO,
            Wall::EPOCH,
            Change::Object {
                project: 1,
                key: ObjectKey::PersonTask(8),
                value: ReadValue::OwnerPersonTask(Box::new(value.clone())),
            },
        )
        .expect("whole owner document admitted");
    assert!(engine.poll(Time::from_nanos(100_000_000)).is_empty());
    engine
        .read(Time::from_nanos(100_000_000), 11, Token::new(97), Read::PersonTask { task: 8 })
        .expect("original read admitted");
    let up = engine.poll(Time::from_nanos(100_000_000));
    assert!(up.iter().any(|up| matches!(up,Up::Read {terminal:ReadResult::Value(ReadValue::OwnerPersonTask(document)),..} if document.as_ref()==&value)));
    assert_eq!(
        engine
            .act(
                Time::from_nanos(100_000_000),
                Wall::EPOCH,
                Change::Object {
                    project: 1,
                    key: ObjectKey::PersonTask(9),
                    value: ReadValue::OwnerPersonTask(Box::new(value))
                }
            )
            .err(),
        Some(Invalid::Identity)
    );
    assert_eq!(engine.commits().last().expect("committed document exists").number, commit);
}
