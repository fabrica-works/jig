use super::{Queue, authority, core};
/// Edit the owner role used by a story, retaining its current project policy.
pub fn edit_role(config: &mut core::Config, edit: impl FnOnce(&mut authority::Role)) {
    let mut policy = config.authority.policy(1).expect("fixture policy").clone();
    edit(&mut policy.roles[0]);
    let mut facts = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut config.authority, authority::Event::Policy { project: 1, policy }, &mut facts);
    assert_eq!(facts.pop(), Some(authority::PolicyFact::Changed { project: 1 }));
}
