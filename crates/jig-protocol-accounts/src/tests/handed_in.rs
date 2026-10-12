use super::fixtures::{self, ACCOUNT_CHAT, TOKEN_FIRST};
use crate::handed_in::Unfit;
use skein_lib::{Duration, Wall};
use skein_oauth_accounts as driver;

#[test]
fn a_handed_in_record_carries_no_refresh_token() {
    let lapse = Wall::from_nanos(Duration::from_secs(60).as_nanos());
    let driver::Request::HandIn { account, record } = fixtures::handed_request(TOKEN_FIRST, lapse).expect("record")
    else {
        panic!("HandIn required");
    };
    assert_eq!(account, ACCOUNT_CHAT);
    assert_eq!(record.key, ACCOUNT_CHAT);
    assert_eq!(record.generation, 1);
    assert_eq!(record.access_token.as_ref(), TOKEN_FIRST);
    assert_eq!(record.refresh_token, None);
    assert_eq!(record.metadata.as_deref(), Some(b"account-id".as_slice()));
    assert_eq!(record.expires_at, lapse);
}
#[test]
fn a_token_past_the_drivers_bound_is_unfit() {
    let token = [b'x'; 257];
    assert_eq!(fixtures::handed_request(&token, Wall::EPOCH).err(), Some(Unfit));
}
#[test]
fn a_lapse_already_past_is_handed_in_and_fails_at_its_first_grant() {
    let request = fixtures::handed_request(TOKEN_FIRST, Wall::EPOCH).expect("past lapse still admitted");
    let event = fixtures::first_grant(request, Wall::from_nanos(1));
    assert!(
        event
            == driver::Event::Failed {
                account: ACCOUNT_CHAT,
                ends: driver::Ends::Grant,
                failure: driver::Failure::Expired
            }
    );
}
