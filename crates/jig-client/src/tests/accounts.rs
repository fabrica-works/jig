//! Account moves use the once path and the ordinary written request (client/domain/client.md, 6).
use super::fixtures::STOP;
use crate::{
    Ask, Event, accounts,
    contract::{self as c, AccountChange, AccountHolder},
};
use skein_lib::Secret;
const NAME: skein_lib::RequestKey = [7; 16];
const ACCOUNT: u32 = 19;
const MOVE: u32 = 23;
fn ask(change: AccountChange) -> Ask {
    Ask::Jig(Box::new(c::Ask::AccountChange { holder: AccountHolder::Own, change }))
}
#[test]
fn move_out_builds_a_carry_with_no_value() {
    let Event::Carry { key, name, request, value } = accounts::move_out(STOP, NAME, AccountHolder::Own, ACCOUNT) else {
        panic!("move out carries once");
    };
    assert_eq!((key, name, request, value), (STOP, NAME, ask(AccountChange::MoveOut { account: ACCOUNT }), None));
}
#[test]
fn written_builds_an_ordinary_named_send() {
    let Event::Send { key, name, request } = accounts::written(STOP, NAME, AccountHolder::Own, MOVE) else {
        panic!("written goes through the table");
    };
    assert_eq!((key, name, request), (STOP, Some(NAME), ask(AccountChange::Written { number: MOVE })));
}
#[test]
fn move_in_carries_the_file_beside_its_digest() {
    let file = Secret::new(Box::from(b"moved record".as_slice()));
    let Event::Carry { key, name, request, value } = accounts::move_in(STOP, NAME, AccountHolder::Own, file.clone())
    else {
        panic!("move in carries once");
    };
    assert_eq!(
        (key, name, request, value),
        (STOP, NAME, ask(AccountChange::MoveIn { digest: skein_lib::sha256::digest(file.expose()) }), Some(file))
    );
}
#[test]
fn key_helpers_still_carry_some_value() {
    let file = Secret::new(Box::from(b"key".as_slice()));
    for event in [
        accounts::add_key(STOP, NAME, AccountHolder::Own, 1, None, file.clone()),
        accounts::replace_key(STOP, NAME, AccountHolder::Own, 1, file.clone()),
    ] {
        let Event::Carry { value, .. } = event else { panic!("key carries once") };
        assert_eq!(value, Some(file.clone()));
    }
}
