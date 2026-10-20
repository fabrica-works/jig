//! LLM account sign-in asks and retained answers at the link boundary.
use super::*;

#[test]
fn a_sign_in_ask_is_translated_both_ways() {
    for holder in [c::AccountHolder::Own, c::AccountHolder::Deployment] {
        let ask = c::Ask::SignIn { holder, provider: u32::MAX };
        let wire = translate::asks::ask(&ask, &fixtures::limits().wire).unwrap();
        let w::Ask::SignIn(row) = &wire else { panic!("account sign-in ask") };
        assert_eq!(row.provider(), u32::MAX);
        assert_eq!(
            row.holder(),
            &match holder {
                c::AccountHolder::Own => w::AccountHolder::Own,
                c::AccountHolder::Deployment => w::AccountHolder::Deployment,
            }
        );
        assert_eq!(translate::store::restore_ask(wire, &fixtures::limits()), Ok(ask));
    }
}
#[test]
fn a_sign_out_ask_is_translated_both_ways() {
    for holder in [c::AccountHolder::Own, c::AccountHolder::Deployment] {
        let ask = c::Ask::SignOut { holder, source: u32::MAX };
        let wire = translate::asks::ask(&ask, &fixtures::limits().wire).unwrap();
        let w::Ask::SignOut(row) = &wire else { panic!("account sign-in end ask") };
        assert_eq!(row.source(), u32::MAX);
        assert_eq!(
            row.holder(),
            &match holder {
                c::AccountHolder::Own => w::AccountHolder::Own,
                c::AccountHolder::Deployment => w::AccountHolder::Deployment,
            }
        );
        assert_eq!(translate::store::restore_ask(wire, &fixtures::limits()), Ok(ask));
    }
}
#[test]
fn a_sign_in_is_refused_in_a_compound() {
    for ask in [
        c::Ask::SignIn { holder: c::AccountHolder::Own, provider: 4 },
        c::Ask::SignOut { holder: c::AccountHolder::Deployment, source: 9 },
    ] {
        let wire = translate::asks::ask(&ask, &fixtures::limits().wire).unwrap();
        let mut out = skein_lib::Writer::new(usize::try_from(wire.measure()).unwrap());
        wire.encode(&mut out).unwrap();
        let bytes = out.finish();
        let mut out = skein_lib::Writer::new(bytes.len() + 4);
        out.put(&u32::try_from(bytes.len()).unwrap().to_be_bytes()).unwrap();
        out.put(&bytes).unwrap();
        let compound = w::Ask::Compound(
            w::Compound::new(&fixtures::limits().wire, w::CompoundParts { project: 1, operations: out.finish() })
                .unwrap(),
        );
        translate::store::restore_ask(compound, &fixtures::limits())
            .expect_err("LLM account sign-in asks are not compound operations");
    }
}
#[test]
fn the_account_outcomes_are_read() {
    let limits = fixtures::limits();
    for length in [0, 1, 2048] {
        let address: Box<[u8]> = skein_lib::bytes::zeroed(length);
        let wire = w::OwnerOutcome::AccountSignInStarted(
            w::OwnerOutcomeAccountSignInStarted::new(
                &limits.wire,
                w::OwnerOutcomeAccountSignInStartedParts { source: u32::MAX, address: address.clone() },
            )
            .unwrap(),
        );
        assert_eq!(
            translate::answers::owner_outcome(&wire, &limits).unwrap(),
            c::OwnerOutcome::AccountSignInStarted { source: u32::MAX, address }
        );
    }
    for confirmed in [false, true] {
        let wire = w::OwnerOutcome::AccountSignedOut(
            w::OwnerOutcomeAccountSignedOut::new(
                &limits.wire,
                w::OwnerOutcomeAccountSignedOutParts { confirmed, failed_hosts: skein_lib::List::with_capacity(0) },
            )
            .unwrap(),
        );
        assert_eq!(
            translate::answers::owner_outcome(&wire, &limits).unwrap(),
            c::OwnerOutcome::AccountSignedOut { confirmed, failed_hosts: Box::new([]) }
        );
    }
}
#[test]
fn the_four_account_limits_are_read() {
    for (wire, client) in [
        (w::AdmissionLimit::Accounts, c::AdmissionLimit::Accounts),
        (w::AdmissionLimit::HolderAccounts, c::AdmissionLimit::HolderAccounts),
        (w::AdmissionLimit::Sources, c::AdmissionLimit::Sources),
        (w::AdmissionLimit::SignIns, c::AdmissionLimit::SignIns),
    ] {
        assert_eq!(translate::answers::admission_limit(&wire, &fixtures::limits()), client);
    }
}

#[test]
fn all_eight_account_changes_translate_both_ways() {
    let limits = fixtures::limits();
    for holder in [c::AccountHolder::Own, c::AccountHolder::Deployment] {
        for change in [
            c::AccountChange::AddKey { provider: 3, join: Some(7), digest: [19; 32] },
            c::AccountChange::ReplaceKey { source: 9, digest: [23; 32] },
            c::AccountChange::RemoveKey { source: 9 },
            c::AccountChange::Resume { source: 9 },
            c::AccountChange::Remove { account: 7 },
            c::AccountChange::MoveAccount { account: 7, place: 2 },
            c::AccountChange::MoveSource { account: 7, source: 9, place: 2 },
            c::AccountChange::SetUses {
                account: 7,
                uses: Some(Box::new([c::OwnerAccountUse::Party(5), c::OwnerAccountUse::Project(1)])),
            },
            c::AccountChange::SetUses { account: 7, uses: None },
        ] {
            let ask = c::Ask::AccountChange { holder, change };
            let wire = translate::asks::ask(&ask, &limits.wire).unwrap();
            assert_eq!(translate::store::restore_ask(wire, &limits), Ok(ask));
        }
    }
}
#[test]
fn the_key_outcomes_and_provider_refusal_are_read() {
    let limits = fixtures::limits();
    for (wire, client) in [
        (
            w::OwnerOutcome::AccountKeyKept(
                w::OwnerOutcomeAccountKeyKept::new(&limits.wire, w::OwnerOutcomeAccountKeyKeptParts { source: 9 })
                    .unwrap(),
            ),
            c::OwnerOutcome::AccountKeyKept { source: 9 },
        ),
        (w::OwnerOutcome::AccountChanged, c::OwnerOutcome::AccountChanged),
        (
            w::OwnerOutcome::AccountRemoved(
                w::OwnerOutcomeAccountRemoved::new(
                    &limits.wire,
                    w::OwnerOutcomeAccountRemovedParts {
                        confirmed: true,
                        failed_hosts: skein_lib::List::with_capacity(0),
                    },
                )
                .unwrap(),
            ),
            c::OwnerOutcome::AccountRemoved { confirmed: true, failed_hosts: Box::new([]) },
        ),
    ] {
        assert_eq!(translate::answers::owner_outcome(&wire, &limits).unwrap(), client);
    }
    assert_eq!(translate::answers::refusal(&w::Refusal::Provider, &limits), Ok(c::Refusal::Provider));
}

#[test]
fn a_declared_program_name_is_validated_on_receipt() {
    let limits = fixtures::limits();
    let wire = w::OwnerOutcome::AccountProgramDeclared(
        w::OwnerOutcomeAccountProgramDeclared::new(
            &limits.wire,
            w::OwnerOutcomeAccountProgramDeclaredParts { source: 0, name: b"../escape".as_slice().into() },
        )
        .unwrap(),
    );
    assert_eq!(
        translate::answers::owner_outcome(&wire, &limits).err(),
        Some(w::Problem { path: w::Path::OwnedRecord, reason: skein_codec::Reason::Malformed })
    );
}
#[test]
fn program_host_results_share_the_native_answer_budget() {
    let mut limits = fixtures::limits();
    limits.client.document_bytes = 1;
    let mut failed_hosts = skein_lib::List::with_capacity(1);
    failed_hosts
        .push(
            w::AccountHostFailure::new(
                &limits.wire,
                w::AccountHostFailureParts { name: b"worker".as_slice().into(), why: w::AccountHostWhy::Lost },
            )
            .unwrap(),
        )
        .unwrap();
    let wire = w::OwnerOutcome::AccountSignedOut(
        w::OwnerOutcomeAccountSignedOut::new(
            &limits.wire,
            w::OwnerOutcomeAccountSignedOutParts { confirmed: false, failed_hosts },
        )
        .unwrap(),
    );
    assert_eq!(
        translate::answers::owner_outcome(&wire, &limits).err(),
        Some(w::Problem { path: w::Path::OwnedRecord, reason: skein_codec::Reason::Bound })
    );
}
