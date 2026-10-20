//! Complete accounts receiving admission and field preservation.
use super::{c, fixtures, w};
use crate::{Undecodable, translate::watches};
use skein_lib::{Duration, List, Wall, Writer};
fn list<T, const N: usize>(items: [T; N]) -> List<T> {
    let mut out = List::with_capacity(u32::try_from(N).expect("literal rows"));
    for item in items {
        out.push(item).ok().expect("literal rows");
    }
    out
}
fn source_lines() -> List<c::OwnerAccountSource> {
    list([
        c::OwnerAccountSource {
            place: 2,
            number: 11,
            kind: c::OwnerAccountKind::SignIn,
            state: c::OwnerAccountState::Granted,
            usable: true,
            expiring: false,
            attention: true,
            spent_until: Some(Wall::from_nanos(101)),
        },
        c::OwnerAccountSource {
            place: 5,
            number: 12,
            kind: c::OwnerAccountKind::Key,
            state: c::OwnerAccountState::Later { until: None },
            usable: false,
            expiring: true,
            attention: false,
            spent_until: None,
        },
    ])
}
fn document(end: Option<c::OwnerAccountEnd>) -> c::OwnerAccounts {
    c::OwnerAccounts {
        accounts: list([
            c::OwnerAccountLine {
                number: 7,
                place: 3,
                uses: Some(list([c::OwnerAccountUse::Party(23), c::OwnerAccountUse::Project(29)])),
                plan: true,
                provider: Some(3),
                display: c::DocumentText {
                    bytes: c::Text::new(Box::from(b"Personal".as_slice()), 32).unwrap(),
                    cut: true,
                },
                holder: c::OwnerAccountHolder::Party(23),
                sources: source_lines(),
                cut: true,
            },
            c::OwnerAccountLine {
                number: 9,
                place: 4,
                uses: None,
                plan: false,
                provider: None,
                display: c::DocumentText { bytes: c::Text::new(Box::from([]), 32).unwrap(), cut: false },
                holder: c::OwnerAccountHolder::Deployment,
                sources: List::with_capacity(0),
                cut: false,
            },
        ]),
        signing: list([c::OwnerAccountSignIn { source: 19, provider: 3, holder: c::OwnerAccountHolder::Party(23) }]),
        last_end: end.map(|end| c::OwnerAccountLastEnd { source: 11, end }),
        usable: true,
        settled: false,
        cut: true,
    }
}
fn ends() -> Vec<c::OwnerAccountEnd> {
    vec![
        c::OwnerAccountEnd::Made { account: 7, declined: false },
        c::OwnerAccountEnd::Renewed { account: 7, declined: true },
        c::OwnerAccountEnd::Refused { refusal: c::OwnerAccountRefusal::Configured },
        c::OwnerAccountEnd::Refused { refusal: c::OwnerAccountRefusal::NotLimited },
        c::OwnerAccountEnd::Refused { refusal: c::OwnerAccountRefusal::Uses },
        c::OwnerAccountEnd::Refused {
            refusal: c::OwnerAccountRefusal::NotKept { why: c::OwnerAccountKeyFailure::NotStored },
        },
        c::OwnerAccountEnd::Refused {
            refusal: c::OwnerAccountRefusal::Limit { bound: c::OwnerAccountBound::HolderAccounts },
        },
        c::OwnerAccountEnd::Refused { refusal: c::OwnerAccountRefusal::Held { provider: 3 } },
        c::OwnerAccountEnd::Refused { refusal: c::OwnerAccountRefusal::Provider { provider: 3 } },
        c::OwnerAccountEnd::Refused { refusal: c::OwnerAccountRefusal::Subject },
        c::OwnerAccountEnd::Refused { refusal: c::OwnerAccountRefusal::Unknown },
        c::OwnerAccountEnd::Refused { refusal: c::OwnerAccountRefusal::Numbers },
        c::OwnerAccountEnd::Refused { refusal: c::OwnerAccountRefusal::Kind { kind: c::OwnerAccountKind::Program } },
        c::OwnerAccountEnd::Refused { refusal: c::OwnerAccountRefusal::Closing },
        c::OwnerAccountEnd::Refused { refusal: c::OwnerAccountRefusal::NotStarted },
        c::OwnerAccountEnd::Failed {
            failure: c::OwnerAccountFailure::Later { after: Some(Duration::from_nanos(103)) },
        },
        c::OwnerAccountEnd::Failed { failure: c::OwnerAccountFailure::Later { after: None } },
        c::OwnerAccountEnd::Failed { failure: c::OwnerAccountFailure::SignInAgain },
        c::OwnerAccountEnd::Failed { failure: c::OwnerAccountFailure::MissingScope { scope: 31 } },
        c::OwnerAccountEnd::Failed { failure: c::OwnerAccountFailure::Misconfigured },
        c::OwnerAccountEnd::Failed {
            failure: c::OwnerAccountFailure::Unloaded {
                at: c::OwnerAccountPlace::File,
                why: c::OwnerAccountUnread::Stalled,
            },
        },
        c::OwnerAccountEnd::Failed { failure: c::OwnerAccountFailure::Cancelled },
    ]
}
fn bytes(end: Option<w::OwnerAccountEnd>) -> Box<[u8]> {
    delivery(jig_client_test_records::accounts_record(end))
}
fn delivery(record: w::OwnerAccounts) -> Box<[u8]> {
    let value = w::DeliveryValue::OwnerAccounts(record);
    let mut out = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut out).unwrap();
    out.finish()
}
#[test]
fn accounts_delivery_preserves_every_field_and_terminal_outcome() {
    let limits = fixtures::limits();
    assert_eq!(watches::subject(&c::Subject::Accounts, &limits.wire), Ok(w::Subject::Accounts));
    for (end, expected) in jig_client_test_records::account_end_records().into_iter().zip(ends()) {
        assert_eq!(
            watches::delivery(&bytes(Some(end)), &limits),
            Ok(c::Delivery::OwnerAccounts(Box::new(document(Some(expected)))))
        );
    }
    assert_eq!(watches::delivery(&bytes(None), &limits), Ok(c::Delivery::OwnerAccounts(Box::new(document(None)))));
}
#[test]
fn accounts_delivery_at_the_receiving_byte_limit_is_admitted() {
    let bytes = bytes(None);
    let limits = fixtures::limits_for_document(u64::try_from(bytes.len()).unwrap());
    assert_eq!(watches::delivery(&bytes, &limits), Ok(c::Delivery::OwnerAccounts(Box::new(document(None)))));
}
#[test]
fn accounts_delivery_one_byte_over_the_receiving_limit_is_refused() {
    let bytes = bytes(None);
    let limits = fixtures::limits_for_document(u64::try_from(bytes.len()).unwrap().checked_sub(1).unwrap());
    assert_eq!(watches::delivery(&bytes, &limits), Err(Undecodable::TooLarge));
}

#[test]
fn account_uses_keep_their_constant_bound_with_fewer_receiving_rows() {
    // The schema mirrors jig_domain_accounts::USES; receiving rows remain two.
    const USES: usize = 16;
    let limits = fixtures::limits();
    let mut line = jig_client_test_records::account_line_record().into_parts();
    line.uses = Some(list([(); USES].map(|()| {
        w::OwnerAccountUse::Party(
            w::OwnerAccountUseParty::new(&limits.wire, w::OwnerAccountUsePartyParts { party: 23 }).unwrap(),
        )
    })));
    let line = w::OwnerAccountLine::new(&limits.wire, line).unwrap();
    let mut out = Writer::new(usize::try_from(line.measure()).unwrap());
    line.encode(&mut out).unwrap();
    let bytes = out.finish();
    let mut pieces = Writer::new(bytes.len().checked_add(4).unwrap());
    skein_codec::piece::write(&mut pieces, &bytes).unwrap();
    let mut record = jig_client_test_records::accounts_record(None).into_parts();
    record.accounts = pieces.finish();
    let record = w::OwnerAccounts::new(&limits.wire, record).unwrap();
    let mut expected = document(None);
    let mut line = expected.accounts.get(0).unwrap().clone();
    line.uses = Some(list([c::OwnerAccountUse::Party(23); USES]));
    expected.accounts = list([line]);
    assert_eq!(watches::delivery(&delivery(record), &limits), Ok(c::Delivery::OwnerAccounts(Box::new(expected))));
}
