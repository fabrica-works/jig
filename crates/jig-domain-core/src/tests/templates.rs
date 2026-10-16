use super::*;
const OWNER: u64 = 1;
const PROJECT: u32 = 1;
const UNKNOWN: u32 = 99;

#[test]
fn a_policy_change_naming_an_unknown_template_is_refused() {
    let (mut core, _) = make_tree();
    let change = people::PolicyChange::Templates {
        overrides: Box::new([people::TemplateOverride {
            template: UNKNOWN,
            executor: None,
            budget: None,
            deadline: None,
            priority: Some(5),
        }]),
    };
    assert_eq!(core.change_policy(&limits(), OWNER, PROJECT, change), Err(people::Refusal::Unknown));
}

#[test]
fn a_policy_change_past_the_templates_limit_is_refused() {
    let (mut core, _) = make_tree();
    let mut overrides = List::with_capacity(9);
    for _ in 0_u32..9 {
        overrides
            .push(people::TemplateOverride {
                template: UNKNOWN,
                executor: None,
                budget: None,
                deadline: None,
                priority: Some(5),
            })
            .expect("nine overrides");
    }
    let overrides = overrides.into_boxed();
    assert_eq!(
        core.change_policy(&limits(), OWNER, PROJECT, people::PolicyChange::Templates { overrides }),
        Err(people::Refusal::Limit { which: people::LimitKind::Policy })
    );
}
