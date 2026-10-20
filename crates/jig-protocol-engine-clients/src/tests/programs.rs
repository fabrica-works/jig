//! Program document facts reach their allocated client records.
use crate::translate::programs;
use jig_client_protocol_wire::v10 as w;
use jig_domain_core::documents as d;
use jig_domain_fleet as fleet;
use skein_lib::List;
fn host() -> d::Host {
    d::Host {
        name: Some(b"configured-worker".as_slice().into()),
        awaiting_logins: true,
        programs: List::with_capacity(0),
        channel: 17,
        kind: fleet::HostKind::Worker,
        slots: 1,
        used: 0,
        free: 1,
        draining: false,
        connected: true,
        grace_until: None,
        grace_remaining: None,
        runs: List::with_capacity(0),
        cut: false,
    }
}
#[test]
fn configured_names_cross_as_whole_bytes() {
    let wire = crate::translate::deployment::host(&host(), &w::CEILINGS).expect("host");
    assert_eq!(wire.name().as_deref(), Some(b"configured-worker".as_slice()));
}
#[test]
fn login_readiness_crosses_the_wire() {
    let wire = crate::translate::deployment::host(&host(), &w::CEILINGS).expect("host");
    assert!(wire.awaiting_logins());
    assert!(wire.connected());
    assert!(!wire.draining());
}
#[test]
fn program_details_past_the_wire_bound_are_refused() {
    let row = d::ProgramLogin {
        name: b"personal".as_slice().into(),
        state: Some(fleet::LoginState::Failed {
            reason: fleet::SourceFailure::Exited,
            detail: Box::from([b'x'; 4097]),
        }),
        sign_out: None,
        cleanup: false,
        cut: false,
    };
    assert_eq!(
        programs::login(&row, &w::CEILINGS).err(),
        Some(w::Problem { path: w::Path::OwnerProgramFailedDetail, reason: skein_codec::Reason::Bound })
    );
}
#[test]
fn program_names_past_the_wire_bound_are_refused() {
    let row = d::ProgramLogin { name: Box::from([b'a'; 33]), state: None, sign_out: None, cleanup: false, cut: false };
    assert_eq!(
        programs::login(&row, &w::CEILINGS).err(),
        Some(w::Problem { path: w::Path::OwnerProgramLoginName, reason: skein_codec::Reason::Bound })
    );
}
