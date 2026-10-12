use super::fixtures::{self, ACCOUNT_CHAT, ACCOUNT_DELEGATE, TOKEN_FIRST, TOKEN_SECOND};
use crate::{
    ACCOUNT_ID_BYTES,
    table::{self, Table, Unfit},
};
use skein_lib::{Secret, Time, bytes};
fn table() -> Table {
    Table::new(2, 32)
}
fn grant(table: &mut Table, account: u32, generation: u64, token: &[u8], expires: u64) {
    table.granted(account, generation, fixtures::value(token), Time::from_nanos(expires)).expect("fits");
}
#[test]
fn a_third_generation_pushes_out_the_oldest() {
    let mut table = table();
    grant(&mut table, ACCOUNT_CHAT, 1, TOKEN_FIRST, 1000);
    grant(&mut table, ACCOUNT_CHAT, 2, TOKEN_SECOND, 1000);
    grant(&mut table, ACCOUNT_CHAT, 3, b"token-third", 1000);
    assert!(table.value(ACCOUNT_CHAT, 1, Time::ZERO).is_none());
    assert_eq!(table.value(ACCOUNT_CHAT, 2, Time::ZERO).expect("second generation").token.expose(), TOKEN_SECOND);
}
#[test]
fn a_value_is_gone_at_its_expiry_and_not_before() {
    let mut table = table();
    grant(&mut table, ACCOUNT_CHAT, 1, TOKEN_FIRST, 10);
    assert_eq!(table.next_expiry(), Some(Time::from_nanos(10)));
    table.expire(Time::from_nanos(9));
    assert_eq!(table.value(ACCOUNT_CHAT, 1, Time::from_nanos(9)).expect("unexpired").token.expose(), TOKEN_FIRST);
    assert!(table.value(ACCOUNT_CHAT, 1, Time::from_nanos(10)).is_none());
    table.expire(Time::from_nanos(10));
    assert_eq!(table.next_expiry(), None);
}
#[test]
fn a_stale_generation_reads_as_absent() {
    let mut table = table();
    grant(&mut table, ACCOUNT_CHAT, 3, TOKEN_FIRST, 1000);
    grant(&mut table, ACCOUNT_CHAT, 2, TOKEN_SECOND, 1000);
    grant(&mut table, ACCOUNT_CHAT, 1, b"stale", 1000);
    assert!(table.value(ACCOUNT_CHAT, 1, Time::ZERO).is_none());
    assert!(table.value(ACCOUNT_DELEGATE, 3, Time::ZERO).is_none());
}
#[test]
fn a_value_past_the_token_bound_is_refused() {
    let mut table = table();
    let too_large = fixtures::value(&[b'x'; 33]);
    assert_eq!(table.granted(ACCOUNT_CHAT, 1, too_large, Time::from_nanos(1000)), Err(Unfit));
    assert!(table.value(ACCOUNT_CHAT, 1, Time::ZERO).is_none());
}
#[test]
fn an_account_past_the_tables_bound_is_refused() {
    let mut table = table();
    assert_eq!(table.granted(2, 1, fixtures::value(TOKEN_FIRST), Time::from_nanos(1000)), Err(Unfit));
}
#[test]
fn the_tables_debug_never_shows_a_value() {
    let mut table = table();
    let mut value = fixtures::value(TOKEN_FIRST);
    value.account_id = Secret::new(bytes::copy_of(b"provider-account"));
    table.granted(ACCOUNT_CHAT, 1, value, Time::from_nanos(1000)).expect("fits");
    let mut output = std::io::Cursor::new([0_u8; 1024]);
    std::io::Write::write_fmt(&mut output, format_args!("{table:?}")).expect("debug fits");
    let output = output.into_inner();
    assert_eq!(bytes::find(&output, TOKEN_FIRST), None);
    assert_eq!(bytes::find(&output, b"provider-account"), None);
}
#[test]
fn zero_accounts_have_no_usable_worst_case() {
    assert_eq!(table::worst_case(0, 32), None);
}

#[test]
fn a_new_grant_keeps_the_older_unexpired_generation() {
    let mut table = table();
    grant(&mut table, ACCOUNT_CHAT, 1, TOKEN_FIRST, 20);
    grant(&mut table, ACCOUNT_CHAT, 2, TOKEN_SECOND, 10);
    table.expire(Time::from_nanos(10));
    grant(&mut table, ACCOUNT_CHAT, 3, b"token-third", 30);
    assert_eq!(table.value(ACCOUNT_CHAT, 1, Time::from_nanos(10)).expect("still valid").token.expose(), TOKEN_FIRST);
}

#[test]
fn an_account_id_past_its_bound_is_refused() {
    let mut table = table();
    let mut value = fixtures::value(TOKEN_FIRST);
    value.account_id = Secret::new(bytes::zeroed(usize::try_from(ACCOUNT_ID_BYTES + 1).expect("id bytes")));
    assert_eq!(table.granted(ACCOUNT_CHAT, 1, value, Time::from_nanos(1000)), Err(Unfit));
    assert!(table.value(ACCOUNT_CHAT, 1, Time::ZERO).is_none());
}
