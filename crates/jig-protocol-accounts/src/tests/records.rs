use super::fixtures::{self, ACCOUNT_CHAT, ACCOUNT_DELEGATE, TOKEN_FIRST, TOKEN_SECOND};
use crate::records::{Keep, Next, Records};
use crate::translate::Up;
use jig_domain_accounts as accounts;
use skein_lib::{Duration, Secret, Wall, bytes, sha256};
use skein_oauth as oauth;
use skein_oauth_accounts as driver;

fn limits() -> oauth::Limits {
    fixtures::driver_limits().client.document
}

fn record(account: u32, token: &[u8], generation: u64) -> oauth::SavedToken {
    let mut record = crate::handed_in::record(
        account,
        Secret::new(bytes::copy_of(token)),
        Wall::from_nanos(Duration::from_secs(60).as_nanos()),
        Some(bytes::copy_of(b"account-id")),
        &limits(),
    )
    .expect("record fits");
    record.generation = generation;
    record
}

fn encoded(record: &oauth::SavedToken) -> Box<[u8]> {
    oauth::encode_record(record, &limits()).expect("record encodes")
}

fn keep(records: &mut Records, record: oauth::SavedToken) -> Keep {
    let Next::Store(keep) = records.keep(ACCOUNT_CHAT, record, &limits()) else {
        panic!("the record goes to the store");
    };
    keep
}

/// The driver's answer to a grant for a source it holds no record for.
fn no_record() -> driver::Event {
    driver::Event::Failed { account: ACCOUNT_CHAT, ends: driver::Ends::Grant, failure: driver::Failure::Expired }
}

#[test]
fn a_stored_record_starts_its_source() {
    let mut records = Records::new(2);
    let first = record(ACCOUNT_CHAT, TOKEN_FIRST, 1);
    let loaded = records.load(ACCOUNT_CHAT, &encoded(&first), &limits());
    assert!(loaded == Some(first));
}
#[test]
fn a_renewed_record_replaces_the_one_the_store_holds() {
    let mut records = Records::new(2);
    let first = encoded(&record(ACCOUNT_CHAT, TOKEN_FIRST, 1));
    assert!(records.load(ACCOUNT_CHAT, &first, &limits()).is_some());
    let renewed = record(ACCOUNT_CHAT, TOKEN_SECOND, 2);
    let keep = keep(&mut records, renewed.clone());
    assert_eq!(
        keep,
        Keep { account: ACCOUNT_CHAT, value: Secret::new(encoded(&renewed)), replaces: Some(sha256::digest(&first)) }
    );
    assert!(
        records.kept(ACCOUNT_CHAT)
            == Some(driver::Request::Kept { account: ACCOUNT_CHAT, generation: 2, keeping: driver::Keeping::Kept })
    );
    let next = self::keep(&mut records, record(ACCOUNT_CHAT, TOKEN_FIRST, 3));
    assert_eq!(next.replaces, Some(sha256::digest(&encoded(&renewed))));
}
#[test]
fn a_record_the_store_finds_stale_is_not_kept() {
    let mut records = Records::new(2);
    let first = keep(&mut records, record(ACCOUNT_CHAT, TOKEN_FIRST, 2));
    assert_eq!(first.replaces, None);
    assert!(
        records.stale(ACCOUNT_CHAT)
            == Some(driver::Request::Kept { account: ACCOUNT_CHAT, generation: 2, keeping: driver::Keeping::NotKept })
    );
    let again = keep(&mut records, record(ACCOUNT_CHAT, TOKEN_FIRST, 3));
    assert_eq!(again.replaces, None, "a stale write changes nothing the records hold");
}
#[test]
fn a_second_record_while_one_is_on_its_way_is_not_kept() {
    let mut records = Records::new(2);
    drop(keep(&mut records, record(ACCOUNT_CHAT, TOKEN_FIRST, 2)));
    assert!(records.keeping());
    let refused = records.keep(ACCOUNT_CHAT, record(ACCOUNT_CHAT, TOKEN_SECOND, 3), &limits());
    assert!(
        refused
            == Next::Driver(driver::Request::Kept {
                account: ACCOUNT_CHAT,
                generation: 3,
                keeping: driver::Keeping::NotKept
            })
    );
    assert!(records.kept(ACCOUNT_CHAT).is_some());
    assert!(!records.keeping());
}
#[test]
fn a_handed_in_record_is_kept_before_it_is_handed_in() {
    let mut records = Records::new(2);
    let handed = record(ACCOUNT_CHAT, TOKEN_FIRST, 1);
    let Ok(Next::Store(keep)) = records.hand_in(ACCOUNT_CHAT, handed.clone(), &limits()) else {
        panic!("a record the store lacks is kept first");
    };
    assert_eq!(keep, Keep { account: ACCOUNT_CHAT, value: Secret::new(encoded(&handed)), replaces: None });
    assert!(records.kept(ACCOUNT_CHAT) == Some(driver::Request::HandIn { account: ACCOUNT_CHAT, record: handed }));
}
#[test]
fn a_handed_in_record_the_store_holds_is_handed_in_without_keeping() {
    let mut records = Records::new(2);
    let handed = record(ACCOUNT_CHAT, TOKEN_FIRST, 1);
    assert!(records.load(ACCOUNT_CHAT, &encoded(&handed), &limits()).is_some());
    let handing = records.hand_in(ACCOUNT_CHAT, handed.clone(), &limits());
    assert!(handing == Ok(Next::Driver(driver::Request::HandIn { account: ACCOUNT_CHAT, record: handed })));
    assert!(!records.keeping());
}
#[test]
fn a_handed_in_record_the_store_finds_stale_is_not_handed_in() {
    let mut records = Records::new(2);
    let Ok(Next::Store(_)) = records.hand_in(ACCOUNT_CHAT, record(ACCOUNT_CHAT, TOKEN_FIRST, 1), &limits()) else {
        panic!("a record the store lacks is kept first");
    };
    assert!(records.stale(ACCOUNT_CHAT).is_none());
    assert!(records.kept(ACCOUNT_CHAT).is_none(), "nothing is left waiting");
}
#[test]
fn a_stored_record_that_cannot_be_read_fails_its_source_as_unloaded() {
    let unloaded = Up::Child(accounts::FromComponent::Failed {
        account: ACCOUNT_CHAT,
        failure: accounts::Failure::Unloaded { at: accounts::Place::File, why: accounts::Unread::Unreadable },
    });
    let mut records = Records::new(2);
    assert!(records.load(ACCOUNT_CHAT, b"damaged record", &limits()).is_none());
    assert_eq!(records.up(no_record(), fixtures::NOW), unloaded);
    let another = encoded(&record(ACCOUNT_DELEGATE, TOKEN_FIRST, 1));
    assert!(records.load(ACCOUNT_CHAT, &another, &limits()).is_none(), "a record names its own source");
    assert_eq!(records.up(no_record(), fixtures::NOW), unloaded);
    let replacing = keep(&mut records, record(ACCOUNT_CHAT, TOKEN_FIRST, 2));
    assert_eq!(replacing.replaces, Some(sha256::digest(&another)), "the unread record is still the one replaced");
    assert!(records.kept(ACCOUNT_CHAT).is_some());
    assert_eq!(
        records.up(no_record(), fixtures::NOW),
        Up::Child(accounts::FromComponent::Failed { account: ACCOUNT_CHAT, failure: accounts::Failure::SignInAgain })
    );
}
