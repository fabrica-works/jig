//! Deployment diagnostics come from the owning connector, without new records.
use super::*;
use connector::{DiagnosticAvailability, EffectPhase};

fn deployment(world: &mut World, token: Token, read: bool) -> ClientDocument {
    let requests = if read {
        world.event(Event::ReadDocument {
            scope: ClientScope::Deployment,
            read: token,
            party: 1,
            window: connector::ClientWindow { rows: 16, after: None },
        })
    } else {
        world.event(Event::Document {
            watcher: Token::new(70),
            scope: ClientScope::Deployment,
            subject: token,
            party: 1,
            window: connector::ClientWindow { rows: 16, after: None },
        })
    };
    match requests.as_slice() {
        [Request::ReadAnswered { read: echoed, answer: Ok(value) }] => {
            assert!(read);
            assert_eq!(*echoed, token);
            value.clone()
        }
        [Request::Document { watcher, subject, document: Ok(value) }] => {
            assert!(!read);
            assert_eq!(*watcher, Token::new(70));
            assert_eq!(*subject, token);
            value.clone()
        }
        _ => panic!("one complete owning Deployment terminal"),
    }
}

#[test]
fn deployment_source_keeps_every_outbox_phase_and_true_capacities_with_explicit_unkept_diagnostics() {
    for phase in [
        EffectPhase::Kept,
        EffectPhase::Sent,
        EffectPhase::Uncertain,
        EffectPhase::Retry,
        EffectPhase::Repeat,
        EffectPhase::Held,
    ] {
        let mut world = world(86);
        let original = connector::OutboxEntry {
            number: u64::MAX,
            task: 4,
            key: world.key(4, 7),
            effect: connector::Effect {
                kind: kind(86, 1),
                resources: Box::new([path(86, 1)]),
                purpose: 7,
                condition: Some(0),
                target: 5,
                state: 5,
            },
            attempt: Some(connector::Attempt {
                number: u32::MAX,
                sent: Wall::EPOCH,
                deadline: Wall::from_nanos(u64::MAX),
            }),
            phase,
        };
        world.event(Event::Restore { record: connector::Record::Outbox(original.clone()) });
        let mut restored = original;
        if phase == EffectPhase::Sent {
            restored.phase = EffectPhase::Uncertain;
        }
        let records = world.records.clone();
        let commits = world.commits;
        for read in [false, true] {
            let token = prepare(&mut world, 60, ClientSubject::Deployment);
            let ClientDocument::Deployment {
                outbox,
                subject_limit,
                staged_limit,
                request_budget,
                system_last_response,
            } = deployment(&mut world, token, read)
            else {
                panic!("Deployment remains whole")
            };
            assert_eq!(outbox.as_slice(), std::slice::from_ref(&restored));
            assert_eq!(outbox.capacity(), jig_test_connector_world::LIMITS.entries);
            assert_eq!(subject_limit, jig_test_connector_world::LIMITS.values);
            assert_eq!(staged_limit, jig_test_connector_world::LIMITS.staged);
            assert_eq!(request_budget, DiagnosticAvailability::NotKept);
            assert_eq!(system_last_response, DiagnosticAvailability::NotKept);
        }
        assert_eq!(world.records, records, "diagnostics add no persistence");
        assert_eq!(world.commits, commits, "documents decide nothing");
    }
}

#[test]
fn deployment_source_empty_outbox_wrong_scope_and_consumed_read_keep_the_original_token_contract() {
    let mut world = world(87);
    let token = prepare(&mut world, 60, ClientSubject::Deployment);
    let wrong = world.event(Event::Document {
        watcher: Token::new(70),
        scope: ClientScope::Project(1),
        subject: token,
        party: 1,
        window: connector::ClientWindow { rows: 16, after: None },
    });
    assert!(matches!(wrong.as_slice(), [Request::Document {
        watcher, subject, document: Err(connector::DocumentMissing::Unknown),
    }] if *watcher == Token::new(70) && *subject == token));
    let ClientDocument::Deployment { outbox, request_budget, system_last_response, .. } =
        deployment(&mut world, token, true)
    else {
        panic!("Deployment")
    };
    assert!(outbox.is_empty());
    assert_eq!(outbox.capacity(), jig_test_connector_world::LIMITS.entries);
    assert_eq!(request_budget, DiagnosticAvailability::NotKept);
    assert_eq!(system_last_response, DiagnosticAvailability::NotKept);
    let consumed = world.event(Event::ReadDocument {
        scope: ClientScope::Deployment,
        read: token,
        party: 1,
        window: connector::ClientWindow { rows: 16, after: None },
    });
    assert!(matches!(consumed.as_slice(), [Request::ReadAnswered {
        read, answer: Err(connector::DocumentMissing::Unknown),
    }] if *read == token));
}
