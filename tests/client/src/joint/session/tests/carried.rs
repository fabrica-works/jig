//! A request carrying a value through the real root (protocol/client.md, 5;
//! store.md, 9): kept before the core sees it, named to the core by number and
//! digest alone, and erased once the core leaves it unused.
use super::*;
use jig_test_connector as connector;
use skein_lib::{RequestEnvelope, Secret};

const VALUE: &[u8] = b"a value no store row holds";

fn adopted(session: &mut Session, now: Time, wall: Wall) {
    let resource = connector::Path::new(crate::test::list([Box::from([1_u8]), Box::from([1_u8])]), 16)
        .expect("two connector path segments");
    session.driver.send(
        now,
        wall,
        root::Event::Connector {
            number: 1,
            event: connector::Event::Adopt { project: 1, resource, role: connector::ResourceRole::Owned },
        },
    );
}
fn carrying(attempt: Token, value: &Secret) -> Request {
    let (project, request) =
        jig_client_test_domain::family::wire::request(&jig_client_test_domain::family::Ask::Touch {
            project: 1,
            item: 1,
            revision: jig_client::contract::Revision { value: 0 },
            parameters: Some(42),
        });
    Request::Client {
        request: jig_client::Request::Down {
            down: jig_client::Down::Send {
                attempt,
                key: [77; 16],
                request: jig_client::Ask::Family {
                    scope: jig_client::contract::Scope::Own,
                    record: jig_client::Family {
                        family: jig_client_test_domain::FIRST,
                        version: jig_client_test_protocol_wire::VERSION,
                        bytes: jig_client_test_protocol_wire::codec::family::request_record(project, request),
                    },
                },
                value: Some(value.clone()),
            },
        },
    }
}
fn answered(session: &mut Session, now: Time, wall: Wall, attempt: Token) -> Option<jig_client::Answered> {
    let callbacks = session.poll(now, wall).expect("the carried request's answer");
    let [(ClientId(0), Event::Client { event: jig_client::Event::Up { up } })] = callbacks.as_slice() else {
        panic!("one answer: {callbacks:?}")
    };
    let jig_client::Up::Answer { attempt: actual, envelope: RequestEnvelope::Final, answer, .. } = up else {
        panic!("a final answer: {up:?}")
    };
    assert_eq!(*actual, attempt);
    answer.clone()
}

#[test]
fn a_carried_value_is_kept_before_the_core_sees_it_and_a_replay_s_value_is_erased() {
    let (mut session, now, wall) = session();
    adopted(&mut session, now, wall);
    session.bind(ClientId(0), 1, identity(), now, wall);
    session.poll(now, wall).expect("genuine authentication");
    let value = Secret::new(Box::from(VALUE));
    session.send(ClientId(0), now, wall, carrying(Token::new(31), &value)).expect("a carried request");
    let first = answered(&mut session, now, wall, Token::new(31));
    let kept: Vec<_> =
        session.driver.transport.carried().iter().map(|(number, kept)| (*number, kept.clone())).collect();
    assert_eq!(kept, vec![(1 << 63, value.clone())], "the admitted request's value stays kept");
    session.send(ClientId(0), now, wall, carrying(Token::new(32), &value)).expect("the same key again");
    assert_eq!(answered(&mut session, now, wall, Token::new(32)), first, "the first answer again");
    assert_eq!(
        session.driver.transport.carried().keys().copied().collect::<Vec<_>>(),
        vec![1 << 63],
        "the replay's value, kept under the next number, is erased"
    );
    let rows = format!("{:?}", session.driver.world.store.rows);
    let bytes = format!("{VALUE:?}");
    assert!(!rows.contains(bytes.trim_matches(['[', ']'])), "the value never reaches the core's store");
}
