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
    list(source_rows())
}
fn source_rows() -> [c::OwnerAccountSource; 2] {
    [
        c::OwnerAccountSource {
            name: None,
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
            name: None,
            place: 5,
            number: 12,
            kind: c::OwnerAccountKind::Key,
            state: c::OwnerAccountState::Later { until: None },
            usable: false,
            expiring: true,
            attention: false,
            spent_until: None,
        },
    ]
}
#[expect(clippy::manual_map, reason = "optional fixture facts follow the step crate lint configuration")]
fn document(end: Option<c::OwnerAccountEnd>) -> c::OwnerAccounts {
    c::OwnerAccounts {
        readiness: List::with_capacity(0),
        readiness_cut: false,
        program_hosts: List::with_capacity(0),
        accounts: list([
            c::OwnerAccountLine {
                provider_name: None,
                manage_usage: None,
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
                provider_name: None,
                manage_usage: None,
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
        last_end: match end {
            Some(end) => Some(c::OwnerAccountLastEnd { source: 11, end }),
            None => None,
        },
        usable: true,
        settled: false,
        cut: true,
    }
}
fn ends() -> Box<[c::OwnerAccountEnd]> {
    Box::new([
        c::OwnerAccountEnd::Made { account: 7, declined: false },
        c::OwnerAccountEnd::Renewed { account: 7, declined: true },
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
    ])
}
fn bytes(end: Option<w::OwnerAccountEnd>) -> Box<[u8]> {
    delivery(jig_client_test_records::accounts_record(end))
}
/// A list's pieces written `times` over, one after another.
fn repeated(pieces: &[u8], times: usize) -> Box<[u8]> {
    let mut out = Writer::new(pieces.len().checked_mul(times).unwrap());
    for _ in 0..times {
        out.put(pieces).unwrap();
    }
    out.finish()
}
fn line_piece(line: &w::OwnerAccountLine) -> Box<[u8]> {
    let mut out = Writer::new(usize::try_from(line.measure()).unwrap());
    line.encode(&mut out).unwrap();
    let bytes = out.finish();
    let mut piece = Writer::new(bytes.len().checked_add(4).unwrap());
    skein_codec::piece::write(&mut piece, &bytes).unwrap();
    piece.finish()
}
/// The bytes `document(None)` holds once decoded: its record, its two accounts, the first's two uses,
/// display and two sources, and its one sign-in.
fn held() -> u64 {
    let parts = [
        size_of::<c::OwnerAccounts>(),
        size_of::<c::OwnerAccountLine>(),
        size_of::<c::OwnerAccountLine>(),
        size_of::<c::OwnerAccountUse>(),
        size_of::<c::OwnerAccountUse>(),
        b"Personal".len(),
        size_of::<c::OwnerAccountSource>(),
        size_of::<c::OwnerAccountSource>(),
        size_of::<c::OwnerAccountSignIn>(),
    ];
    let mut bytes = 0_usize;
    for part in parts {
        bytes = bytes.checked_add(part).unwrap();
    }
    u64::try_from(bytes).unwrap()
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
fn an_accounts_document_filling_its_decoded_room_is_admitted() {
    // A document's decoded room is twice its byte limit, the client domain's price for each it holds.
    assert_eq!(held() % 2, 0, "the fixture's decoded bytes halve evenly");
    let limits = fixtures::limits_for_document(held() / 2);
    assert_eq!(watches::delivery(&bytes(None), &limits), Ok(c::Delivery::OwnerAccounts(Box::new(document(None)))));
}
#[test]
fn an_accounts_document_past_its_decoded_room_is_refused() {
    let limits = fixtures::limits_for_document((held() / 2).checked_sub(1).unwrap());
    assert_eq!(watches::delivery(&bytes(None), &limits), Err(Undecodable::TooLarge));
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
    let mut record = jig_client_test_records::accounts_record(None).into_parts();
    record.accounts = line_piece(&line);
    let record = w::OwnerAccounts::new(&limits.wire, record).unwrap();
    let mut expected = document(None);
    let mut line = expected.accounts.get(0).unwrap().clone();
    line.uses = Some(list([c::OwnerAccountUse::Party(23); USES]));
    expected.accounts = list([line]);
    assert_eq!(watches::delivery(&delivery(record), &limits), Ok(c::Delivery::OwnerAccounts(Box::new(expected))));
}
#[test]
fn account_lists_longer_than_the_receiving_rows_are_kept() {
    // Receiving rows are two; each list here holds more, and the document's bytes bound them.
    let limits = fixtures::limits();
    let mut line = jig_client_test_records::account_line_record().into_parts();
    line.sources = repeated(&line.sources, 2);
    let line = w::OwnerAccountLine::new(&limits.wire, line).unwrap();
    let mut record = jig_client_test_records::accounts_record(None).into_parts();
    record.accounts = repeated(&line_piece(&line), 3);
    record.signing = repeated(&record.signing, 3);
    let record = w::OwnerAccounts::new(&limits.wire, record).unwrap();
    let mut expected = document(None);
    let mut line = expected.accounts.get(0).unwrap().clone();
    let [granted, later] = source_rows();
    line.sources = list([granted.clone(), later.clone(), granted, later]);
    expected.accounts = list([line.clone(), line.clone(), line]);
    expected.signing = list([(); 3].map(|()| c::OwnerAccountSignIn {
        source: 19,
        provider: 3,
        holder: c::OwnerAccountHolder::Party(23),
    }));
    assert_eq!(watches::delivery(&delivery(record), &limits), Ok(c::Delivery::OwnerAccounts(Box::new(expected))));
}
#[test]
fn a_deployment_keeps_more_accounts_than_the_receiving_rows() {
    let limits = fixtures::limits();
    let mut record = fixtures::owner_with_account(Some(5)).into_parts();
    record.accounts = repeated(&record.accounts, 3);
    let record = w::OwnerDeployment::new(&limits.wire, record).unwrap();
    let mut expected = fixtures::owner_account_expected(Some(5));
    let account = expected.accounts.get(0).unwrap().clone();
    expected.accounts = list([account.clone(), account.clone(), account]);
    assert_eq!(crate::translate::documents::owner_deployment(record, &limits), Ok(expected));
}

fn configured_line() -> c::OwnerAccountLine {
    let limits = fixtures::limits();
    let mut line = jig_client_test_records::account_line_record().into_parts();
    line.provider_name = Some(
        w::DocumentText::new(
            &limits.wire,
            w::DocumentTextParts { bytes: Box::from(b"Arbitrary provider".as_slice()), cut: true },
        )
        .unwrap(),
    );
    line.manage_usage = Some(Box::from(b"https://usage.example/".as_slice()));
    let line = w::OwnerAccountLine::new(&limits.wire, line).unwrap();
    let encoded = {
        let mut writer = Writer::new(usize::try_from(line.measure()).unwrap());
        line.encode(&mut writer).unwrap();
        writer.finish()
    };
    let mut record = jig_client_test_records::accounts_record(None).into_parts();
    record.accounts = {
        let mut writer = Writer::new(encoded.len().checked_add(4).unwrap());
        skein_codec::piece::write(&mut writer, &encoded).unwrap();
        writer.finish()
    };
    let record = w::OwnerAccounts::new(&limits.wire, record).unwrap();
    let c::Delivery::OwnerAccounts(value) = watches::delivery(&delivery(record), &limits).unwrap() else {
        panic!("accounts delivery")
    };
    value.accounts.get(0).unwrap().clone()
}
#[test]
fn configured_provider_name_reaches_accounts_client() {
    assert_eq!(configured_line().provider_name.unwrap().bytes(), b"Arbitrary provider");
}
#[test]
fn provider_name_cut_reaches_accounts_client() {
    assert!(configured_line().provider_name.unwrap().cut);
}
#[test]
fn configured_manage_address_reaches_accounts_client() {
    assert_eq!(configured_line().manage_usage.as_deref(), Some(b"https://usage.example/".as_slice()));
}

#[test]
fn account_capacity_labels_decode_without_account_payloads() {
    let limits = fixtures::limits();
    for (wire, expected) in [
        (w::OwnerPersonCapacityAction::AccountSignIn, c::OwnerPersonCapacityAction::AccountSignIn),
        (w::OwnerPersonCapacityAction::AccountSignOut, c::OwnerPersonCapacityAction::AccountSignOut),
        (w::OwnerPersonCapacityAction::AccountChange, c::OwnerPersonCapacityAction::AccountChange),
    ] {
        let mut writer = Writer::new(usize::try_from(wire.measure()).unwrap());
        wire.encode(&mut writer).unwrap();
        let bytes = writer.finish();
        let decoded = w::OwnerPersonCapacityAction::decode(&limits.wire, &mut skein_lib::Reader::new(&bytes)).unwrap();
        assert_eq!(crate::translate::documents::owner_person_capacity_action_fields(&decoded, &limits), expected);
    }
}

fn unsafe_manage_addresses() -> [&'static [u8]; 5] {
    [b"ftp://x", b"https://", b"https:///x", b"https://usage.example/a b", b"https://usage.example/\""]
}
#[test]
fn unsafe_manage_addresses_refuse_accounts_frames() {
    let limits = fixtures::limits();
    for address in unsafe_manage_addresses() {
        let mut line = jig_client_test_records::account_line_record().into_parts();
        line.manage_usage = Some(Box::from(address));
        let line = w::OwnerAccountLine::new(&limits.wire, line).unwrap();
        let mut record = jig_client_test_records::accounts_record(None).into_parts();
        record.accounts = line_piece(&line);
        let record = w::OwnerAccounts::new(&limits.wire, record).unwrap();
        assert_eq!(watches::delivery(&delivery(record), &limits), Err(Undecodable::Malformed));
    }
}
#[test]
fn unsafe_manage_addresses_refuse_deployment_frames() {
    let limits = fixtures::limits();
    for address in unsafe_manage_addresses() {
        let record = fixtures::owner_with_account(Some(5));
        let bytes = skein_codec::piece::split(&mut skein_lib::Reader::new(record.accounts())).unwrap().unwrap();
        let mut account =
            w::OwnerAccount::decode(&limits.wire, &mut skein_lib::Reader::new(&bytes)).unwrap().into_parts();
        account.manage_usage = Some(Box::from(address));
        let account = w::OwnerAccount::new(&limits.wire, account).unwrap();
        let mut out = Writer::new(usize::try_from(account.measure()).unwrap());
        account.encode(&mut out).unwrap();
        let bytes = out.finish();
        let mut pieces = Writer::new(bytes.len().checked_add(4).unwrap());
        skein_codec::piece::write(&mut pieces, &bytes).unwrap();
        let mut record = record.into_parts();
        record.accounts = pieces.finish();
        let value = w::DeliveryValue::OwnerDeployment(w::OwnerDeployment::new(&limits.wire, record).unwrap());
        let mut out = Writer::new(usize::try_from(value.measure()).unwrap());
        value.encode(&mut out).unwrap();
        assert_eq!(watches::delivery(&out.finish(), &limits), Err(Undecodable::Malformed));
    }
}

#[test]
fn a_refused_worker_position_survives_wire_receipt() {
    let limits = fixtures::limits();
    let mut record = fixtures::owner_deployment_record().into_parts();
    let mut expected = fixtures::owner_deployment();
    let position = u32::MAX;
    record.hello_refusals = list([w::OwnerHelloRefusalLine::new(
        &limits.wire,
        w::OwnerHelloRefusalLineParts {
            channel: 9,
            at: 0,
            slots: 1,
            stop_bound: Duration::ZERO,
            why: w::OwnerHelloRefusal::Position(
                w::OwnerHelloRefusalPosition::new(&limits.wire, w::OwnerHelloRefusalPositionParts { worker: position })
                    .expect("position record"),
            ),
        },
    )
    .expect("hello refusal")]);
    expected.hello_refusals = list([c::OwnerHelloRefusalLine {
        channel: 9,
        at: Wall::EPOCH,
        slots: 1,
        stop_bound: Duration::ZERO,
        why: c::OwnerHelloRefusal::Position { worker: position },
    }]);
    let record = w::OwnerDeployment::new(&limits.wire, record).expect("deployment");
    assert_eq!(crate::translate::documents::owner_deployment(record, &limits), Ok(expected));
}

#[test]
fn zero_account_receiving_counts_make_limits_unusable() {
    use jig_client_domain::AccountCounts;
    let bounds = fixtures::limits();
    for accounts in [
        AccountCounts { accounts: 0, ..bounds.client.accounts },
        AccountCounts { sources: 0, ..bounds.client.accounts },
        AccountCounts { sign_ins: 0, ..bounds.client.accounts },
    ] {
        let client = jig_client_domain::Limits { accounts, ..bounds.client };
        assert_eq!(crate::Limits::new(client, bounds.exchanges), Err(crate::Unusable::Limits), "{accounts:?}");
    }
}
