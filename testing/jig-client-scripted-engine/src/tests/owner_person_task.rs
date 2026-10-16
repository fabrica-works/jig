use super::script;
use crate::{Change, Engine, Invalid, Up};
use jig_client_domain::contract::*;
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

#[test]
fn person_owner_public_read_preserves_reserved_offers_and_nested_result_admissions_above_rows() {
    let mut value = crate::owner_person_task::tests::value();
    value.acts = skein_lib::List::with_capacity(MAX_OFFERED_ACTS);
    for (act, admission) in [
        (ActKind::HandBack, Admission::Proposed { to: Recipient::Task(u64::MAX) }),
        (ActKind::Answer, Admission::Allowed),
        (ActKind::Take, Admission::Refused(Refusal::NotWaiting)),
    ] {
        value.acts.push(Offered { act, admission }).expect("ordered offer fits");
    }
    let result = value.results.get_mut(1).expect("second row").result.as_mut().expect("whole result");
    result.acts = skein_lib::List::with_capacity(MAX_OFFERED_ACTS);
    result.acts.push(Offered { act: ActKind::MarkSeen, admission: Admission::Allowed }).expect("result offer fits");
    let original = value.clone();
    let mut setup = script();
    setup.bounds.rows = 3;
    let mut engine = Engine::new(setup, 37).expect("ordinary setup fits");
    engine
        .act(
            Time::ZERO,
            Wall::EPOCH,
            Change::Object {
                project: 1,
                key: ObjectKey::PersonTask(8),
                value: ReadValue::OwnerPersonTask(Box::new(value.clone())),
            },
        )
        .expect("whole owner offers fit independently of rows");
    let _ = engine.poll(Time::from_nanos(100_000_000));
    engine
        .read(Time::from_nanos(100_000_000), 11, Token::new(97), Read::PersonTask { task: 8 })
        .expect("original read fits");
    let up = engine.poll(Time::from_nanos(100_000_000));
    let supplied = up
        .iter()
        .find_map(|event| match event {
            Up::Read { token, terminal: ReadResult::Value(ReadValue::OwnerPersonTask(document)) }
                if *token == Token::new(97) =>
            {
                Some(document)
            }
            Up::Answer { .. }
            | Up::Read { .. }
            | Up::Watch { .. }
            | Up::ApplicationAnswer { .. }
            | Up::ApplicationRead { .. }
            | Up::App { .. }
            | Up::Ended { .. } => None,
        })
        .expect("normal whole owner read returned");
    assert_eq!(supplied.as_ref(), &original);
    assert_eq!(supplied.acts.capacity(), MAX_OFFERED_ACTS);
    assert_eq!(
        supplied.acts.iter().map(|offer| offer.act).collect::<Vec<_>>(),
        vec![ActKind::HandBack, ActKind::Answer, ActKind::Take]
    );
    assert!(supplied.results.get(0).expect("first supplied row").result.is_none());
    assert_eq!(
        supplied.results.get(1).expect("second row").result.as_ref().expect("whole result").acts.capacity(),
        MAX_OFFERED_ACTS
    );
    assert_eq!(value, original);
}
