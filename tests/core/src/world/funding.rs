use super::{authority, core};
/// Edit the owner role used by a story, retaining its current project policy.
pub fn edit_role(config: &mut core::Config, edit: impl FnOnce(&mut authority::Role)) {
    let mut policy = config.seed_projects[0].policy.policy.clone();
    edit(&mut policy.roles[0]);

    config.seed_projects[0].policy.policy = policy;
}
