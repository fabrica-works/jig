//! Admission follows the scope named by the application.
use super::*;

const PARTY: u64 = 1;
const SIGN_IN: u64 = 10;
const PROJECT: u32 = 1;
const OTHER_PROJECT: u32 = 2;
const KEY: u8 = 1;
const CONNECTOR: u16 = 7;

fn signed_in(holding: Option<u32>) -> Test {
    let mut test = Test::new(LIMITS);
    test.signin(PARTY, SIGN_IN, identity(0, PARTY));
    if let Some(project) = holding {
        test.send(Event::Roles { project, holdings: Box::new([Holding { person: PARTY, role: Role::Owner }]) });
    }
    test
}

fn application(scope: ApplicationScope) -> Ask {
    Ask::Application { scope, connector: CONNECTOR, digest: [8; 32] }
}

fn assert_admitted(test: &mut Test, scope: ApplicationScope, project: u32, role: Option<Role>) {
    let ask = application(scope);
    let rows = test.request(SIGN_IN, KEY, ask.clone());
    let routed = rows.iter().find_map(|row| {
        if let Request::Route { person, project, role, ask, .. } = row {
            Some((*person, *project, *role, ask.as_ref()))
        } else {
            None
        }
    });
    assert_eq!(routed, Some((PARTY, project, role, &ask)));
}

#[test]
fn a_deployment_application_request_without_a_deployment_role_is_refused() {
    let mut test = signed_in(Some(PROJECT));
    assert_eq!(
        reply(&test.request(SIGN_IN, KEY, application(ApplicationScope::Deployment))),
        Reply::Outcome(Outcome::Refused(Refusal::Role { held: None }))
    );
}

#[test]
fn a_project_application_request_is_admitted_on_a_role_in_that_project() {
    assert_admitted(&mut signed_in(Some(PROJECT)), ApplicationScope::Project(PROJECT), PROJECT, Some(Role::Owner));
}

#[test]
fn a_project_application_request_without_a_role_there_is_refused() {
    let mut test = signed_in(Some(OTHER_PROJECT));
    assert_eq!(
        reply(&test.request(SIGN_IN, KEY, application(ApplicationScope::Project(PROJECT)))),
        Reply::Outcome(Outcome::Refused(Refusal::Role { held: None }))
    );
}

#[test]
fn a_deployment_application_request_on_a_deployment_role_is_admitted() {
    assert_admitted(&mut signed_in(Some(0)), ApplicationScope::Deployment, 0, Some(Role::Owner));
}

#[test]
fn an_own_application_request_is_admitted_on_the_sign_in_alone() {
    for holding in [None, Some(PROJECT), Some(0)] {
        assert_admitted(&mut signed_in(holding), ApplicationScope::Own, 0, None);
    }
}
