//! Sections are whole regions, shown only where the domain names them (view.md, 4.1).
use crate::{Builder, JigBindings, words::Words};
use jig_domain_client::{Domain, pages::Section};
mod conversations;
mod deployment;
pub mod inbox;
pub mod project;
pub mod task;
/// Build one section, or nothing and false when the domain does not show it.
pub fn section(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings, section: Section) -> bool {
    if !client.shows(section) {
        return false;
    }
    match section {
        Section::Task { task, section } => task::section(b, client, words, bindings, task, section),
        Section::Inbox { group, project } => inbox::group(b, client, words, bindings, group, project),
        Section::Conversations { project } => conversations::section(b, client, words, bindings, project),
        Section::Project { project, section } => project::section(b, client, words, bindings, project, section),
        Section::Deployment => deployment::section(b, client, words, bindings),
    }
}

fn status(b: &mut Builder, client: &Domain, words: &Words, section: Section) {
    use crate::phrases::jig::JigPhrase;
    if let Some(status) = client.section_status(section) {
        if status.behind {
            words.status(b, JigPhrase::Behind, &[]);
        }
        if status.excess > 0 {
            words.status(b, JigPhrase::MoreRows, &[]);
        }
        if let Some(which) = status.refused {
            words.refusal(b, &jig_domain_client::contract::Refusal::Limit { which });
        }
        if status.missing {
            words.status(b, JigPhrase::MissingPage, &[]);
        }
    }
}
