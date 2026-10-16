use super::{
    cards::{contains, drive, words},
    pages::build,
    turns::ready,
    work_pages::list,
};
use jig_client_domain::contract::*;
use jig_domain_client::Event;
use skein_lib::Wall;

fn unread_page(why: TurnProblem) -> ActivityPage {
    ActivityPage {
        cut: false,
        before: None,
        after: None,
        rows: list([ActivityLine {
            position: 1,
            at: Wall::EPOCH,
            event: Activity::Turn { name: TurnName { attempt: 2, number: 1 }, turn: TurnRead::Undecodable { why } },
        }]),
    }
}

#[test]
fn an_unreadable_turn_card_says_why() {
    for (why, saying) in [
        (TurnProblem::Version { found: 65535 }, b"Turn could not be read: transcript version 65535".as_slice()),
        (TurnProblem::Malformed, b"Turn could not be read: malformed transcript"),
        (TurnProblem::TooLarge, b"Turn could not be read: exceeds client bounds"),
    ] {
        let (mut domain, _, _, read) = ready();
        drive(
            &mut domain,
            Event::Read { read, result: ReadResult::Value(ReadValue::Activity(Box::new(unread_page(why)))) },
        );
        let tree = build(&domain, 3, words()).0;
        assert!(contains(&tree, saying));
        assert!(!contains(&tree, b"Turn spent:"));
    }
}
