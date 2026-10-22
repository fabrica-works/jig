use super::fixtures::{self, ACCOUNT_CHAT, ACCOUNT_DELEGATE, TOKEN_FIRST, TOKEN_SECOND, record};
use crate::records::{Keep, Next, Records};
use crate::translate::Up;
use jig_domain_accounts as accounts;
use skein_lib::{Secret, sha256};
use skein_oauth as oauth;
use skein_oauth_accounts as driver;

fn limits() -> oauth::Limits {
    fixtures::driver_limits().client.document
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
fn a_stored_record_that_cannot_be_read_fails_its_source_as_unloaded() {
    let unloaded = Up::Child(accounts::FromComponent::Failed {
        account: ACCOUNT_CHAT,
        failure: accounts::Failure::Unloaded { at: accounts::Place::File, why: accounts::Unread::Unreadable },
    });
    let mut records = Records::new(2);
    assert!(records.load(ACCOUNT_CHAT, b"damaged record", &limits()).is_none());
    assert_eq!(records.up(no_record(), fixtures::NOW, b""), unloaded);
    let another = encoded(&record(ACCOUNT_DELEGATE, TOKEN_FIRST, 1));
    assert!(records.load(ACCOUNT_CHAT, &another, &limits()).is_none(), "a record names its own source");
    assert_eq!(records.up(no_record(), fixtures::NOW, b""), unloaded);
    let replacing = keep(&mut records, record(ACCOUNT_CHAT, TOKEN_FIRST, 2));
    assert_eq!(replacing.replaces, Some(sha256::digest(&another)), "the unread record is still the one replaced");
    assert!(records.kept(ACCOUNT_CHAT).is_some());
    assert_eq!(
        records.up(no_record(), fixtures::NOW, b""),
        Up::Child(accounts::FromComponent::Failed { account: ACCOUNT_CHAT, failure: accounts::Failure::SignInAgain })
    );
}

#[test]
fn an_acknowledged_erase_forgets_the_record_and_retains_its_place() {
    let mut records = Records::new(1);
    let first = encoded(&record(ACCOUNT_CHAT, TOKEN_FIRST, 1));
    assert!(records.load(ACCOUNT_CHAT, &first, &limits()).is_some());
    drop(keep(&mut records, record(ACCOUNT_CHAT, TOKEN_SECOND, 2)));
    records.erased(ACCOUNT_CHAT);
    records.erased(ACCOUNT_CHAT);
    records.erased(ACCOUNT_DELEGATE);
    assert!(!records.keeping());
    assert!(records.kept(ACCOUNT_CHAT).is_none());
    assert!(records.admit(ACCOUNT_DELEGATE).is_err(), "the original place remains admitted");
    let fresh = keep(&mut records, record(ACCOUNT_CHAT, TOKEN_SECOND, 3));
    assert_eq!(fresh.replaces, None);
    assert!(
        records.kept(ACCOUNT_CHAT)
            == Some(driver::Request::Kept { account: ACCOUNT_CHAT, generation: 3, keeping: driver::Keeping::Kept })
    );
    let Up::Granted { generation: 3, .. } = records.up(
        fixtures::granted(ACCOUNT_CHAT, TOKEN_SECOND, 3, skein_lib::Duration::from_secs(60)),
        fixtures::NOW,
        b"",
    ) else {
        panic!("a fresh kept record grants normally");
    };
    assert!(records.load(ACCOUNT_CHAT, b"unread", &limits()).is_none());
    records.erased(ACCOUNT_CHAT);
    assert_eq!(
        records.up(no_record(), fixtures::NOW, b""),
        Up::Child(accounts::FromComponent::Failed { account: ACCOUNT_CHAT, failure: accounts::Failure::SignInAgain })
    );
}
