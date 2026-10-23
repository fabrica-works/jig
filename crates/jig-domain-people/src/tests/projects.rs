//! Project asks reach the core with the correct scope (people.md, 5.1–5.2).
use super::*;
const PARTY: u64 = 1;
const SIGN_IN: u64 = 10;
const PROJECT: u32 = 1;
const KEY: u8 = 1;

#[test]
fn project_asks_route_with_their_own_scopes_role() {
    for (ask, project, role) in [
        (Ask::MakeProject { name: "Tally".into(), policy: 1 }, DEPLOYMENT, Some(Role::Owner)),
        (Ask::Rename { project: PROJECT, name: "Tally".into() }, PROJECT, Some(Role::Maintainer)),
    ] {
        let mut test = Test::new(LIMITS);
        test.signin(PARTY, SIGN_IN, identity(0, PARTY));
        for (project, role) in [(DEPLOYMENT, Role::Owner), (PROJECT, Role::Maintainer)] {
            test.send(Event::Roles { project, holdings: Box::new([Holding { person: PARTY, role }]) });
        }
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
}
#[test]
fn project_asks_without_a_role_are_left_for_the_core() {
    for ask in
        [Ask::MakeProject { name: "Tally".into(), policy: 1 }, Ask::Rename { project: PROJECT, name: "12".into() }]
    {
        let mut test = Test::new(LIMITS);
        test.signin(PARTY, SIGN_IN, identity(0, PARTY));
        assert!(
            test.request(SIGN_IN, KEY, ask)
                .iter()
                .any(|row| { if let Request::Route { role, .. } = row { role.is_none() } else { false } })
        );
    }
}
