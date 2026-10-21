//! Program document facts reach their allocated client records.
use crate::translate::programs;
use jig_client_protocol_wire::v11 as w;
use jig_domain_core::documents as d;
use jig_domain_fleet as fleet;
use skein_lib::List;
fn host() -> d::Host {
    d::Host {
        name: Some(b"configured-worker".as_slice().into()),
        awaiting_logins: true,
        rule: jig_domain_core::HostRule::Open,
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

#[test]
fn a_refused_worker_position_reaches_the_wire_record() {
    let mut value = super::fixtures::deployment_document();
    value.hello_refusals.get_mut(0).expect("fixture refusal").why = fleet::HelloRefusal::Position { worker: u32::MAX };
    let record = crate::translate::deployment::document(&value, &w::CEILINGS).expect("deployment record");
    let w::OwnerHelloRefusal::Position(row) = record.hello_refusals().get(0).expect("wire refusal").why() else {
        panic!("position refusal");
    };
    assert_eq!(row.worker(), u32::MAX);
}

#[test]
fn a_host_rule_reaches_the_wire() {
    use jig_domain_core::HostRule;
    for (rule, expected) in [
        (HostRule::Open, w::OwnerHostRule::Open),
        (
            HostRule::Person(7),
            w::OwnerHostRule::Person(
                w::OwnerHostPerson::new(&w::CEILINGS, w::OwnerHostPersonParts { party: 7 }).expect("person"),
            ),
        ),
        (HostRule::Several, w::OwnerHostRule::Several),
        (HostRule::Unknown, w::OwnerHostRule::Unknown),
    ] {
        let mut value = host();
        value.rule = rule;
        let wire = crate::translate::deployment::host(&value, &w::CEILINGS).expect("host");
        assert_eq!(wire.rule(), &expected);
    }
}
