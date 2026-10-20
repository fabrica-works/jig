//! A party's role edit names no models until the client wire carries them: the core keeps an
//! edited role's own and gives a new role every model the project's ceiling names
//! (domain/authority.md, section 6).
use super::*;

const ROLE: u32 = 4;
const CEILING: authority::Models = authority::Models(0b111);
const KEPT: authority::Models = authority::Models(0b010);

fn project_policy() -> authority::Policy {
    let mut ceiling = authority_value(1000, Box::new([]));
    ceiling.models = CEILING;
    let mut given = authority_value(500, Box::new([]));
    given.models = KEPT;
    authority::Policy {
        projections: Box::new([]),
        escalation_role: None,
        ceiling,
        period_spend: 1000,
        roles: Box::new([authority::Role {
            number: ROLE,
            authority: given,
            period_spend: 500,
            requests: authority::Requests::ALL,
            decides: authority::Proposals::ALL,
        }]),
        requirements: Box::new([]),
    }
}

fn edited(number: u32) -> authority::Policy {
    let value = authority_to_people(&authority_value(400, Box::new([]))).unwrap();
    let change = people::PolicyChange::Role(people::PolicyRole {
        number,
        authority: value,
        period_spend: 400,
        requests: 0,
        decides: 0,
    });
    let mut policy = project_policy();
    let mut permissions: Box<[people::PermissionRole]> = Box::new([]);
    let mut templates: Box<[people::TemplateOverride]> = Box::new([]);
    policy::apply(&mut policy, &mut permissions, &mut templates, change, 1, 2).unwrap();
    policy
}

#[test]
fn an_edited_role_keeps_its_models() {
    let policy = edited(ROLE);
    assert_eq!(policy.roles[0].authority.models, KEPT);
    assert_eq!(policy.roles[0].authority.budget.spend, 400, "the rest of the edit applies");
}

#[test]
fn a_new_role_takes_every_model_the_ceiling_names() {
    let policy = edited(ROLE + 1);
    assert_eq!(policy.roles[1].number, ROLE + 1);
    assert_eq!(policy.roles[1].authority.models, CEILING);
}
