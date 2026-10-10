use super::spine::{LIMITS, delivered, drive, frame, start};
use crate::{Domain, Event, Request, contract::*, intents::ByValue};
use skein_lib::{List, Time, Token};
#[test]
fn a_whole_policy_edit_enters_as_one_request_and_a_duplicate_joins_that_request() {
    let edit = PolicyEdit::Permissions {
        mappings: {
            let mut mappings = List::with_capacity(2);
            mappings.push(PolicyPermission { connector: u16::MAX, permission: u16::MAX, role: u32::MAX }).unwrap();
            mappings.push(PolicyPermission { connector: 0, permission: 0, role: 0 }).unwrap();
            mappings
        },
    };
    let ask = Ask::ChangePolicy { project: u32::MAX, edit };
    let by_value = ByValue { asked: Token::new(73), scope: u64::MAX, ask: Box::new(ask.clone()) };
    let mut domain = Domain::new(&LIMITS, 1);
    let f = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, f, frame(7));
    let out = drive(&mut domain, Time::ZERO, Event::Ask { ask: by_value.clone() });
    assert_eq!(out.len(), 1);
    let Request::Send { request, scope, ask: sent } = &out[0] else { panic!("whole keyed send") };
    assert_eq!(*scope, u64::MAX);
    assert_eq!(**sent, ask);
    assert!(drive(&mut domain, Time::ZERO, Event::Ask { ask: by_value }).is_empty());
    assert_eq!(
        drive(&mut domain, Time::ZERO, Event::Answered { request: *request, answer: Answer::Done(Outcome::Changed) })
            .as_ref(),
        &[Request::Answered { asked: Token::new(73), answer: Answer::Done(Outcome::Changed) }]
    );
}
