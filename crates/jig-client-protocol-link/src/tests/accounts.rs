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
            translate::answers::owner_outcome(&wire, &limits),
            c::OwnerOutcome::AccountSignInStarted { source: u32::MAX, address }
        );
    }
    for confirmed in [false, true] {
        let wire = w::OwnerOutcome::AccountSignedOut(
            w::OwnerOutcomeAccountSignedOut::new(&limits.wire, w::OwnerOutcomeAccountSignedOutParts { confirmed })
                .unwrap(),
        );
        assert_eq!(translate::answers::owner_outcome(&wire, &limits), c::OwnerOutcome::AccountSignedOut { confirmed });
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
