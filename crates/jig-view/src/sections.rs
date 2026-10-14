//! Sections are whole regions, shown only where the domain names them (view.md, 4.1).
use crate::{Builder, JigBindings, Role, words::Words};
use jig_domain_client::{Domain, pages::Section};
mod conversations;
mod deployment;
pub mod inbox;
pub mod task;
/// Build one section, or nothing and false when the domain does not show it.
pub fn section(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings, section: Section) -> bool {
    if !client.shows(section) {
        return false;
    }
    match section {
        Section::Task { task, section } => return task::section(b, client, words, bindings, task, section),
        Section::Inbox { group, project } => return inbox::group(b, client, words, bindings, group, project),
        Section::Conversations { project } => return conversations::section(b, client, words, bindings, project),
        Section::Project { section, .. } => {
            b.open(Role::Region);
            b.region(crate::pages::project::region(section).region());
            b.name(&words.say(crate::phrases::jig::JigPhrase::ProjectPage, &[]));
            crate::pages::project(b, client, words, bindings);
        }
        Section::Deployment => return deployment::section(b, client, words, bindings),
    }
    b.close();
    true
}
