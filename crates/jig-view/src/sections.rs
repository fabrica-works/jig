//! Sections are whole regions, shown only where the domain names them (view.md, 4.1).
use crate::{Builder, JigBindings, Role, regions::JigRegion, words::Words};
use jig_domain_client::{Domain, pages::Section};
pub mod task;
/// Build one section, or nothing and false when the domain does not show it.
pub fn section(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings, section: Section) -> bool {
    if !client.shows(section) {
        return false;
    }
    match section {
        Section::Task { task, section } => return task::section(b, client, words, bindings, task, section),
        Section::Inbox { group, .. } => {
            b.open(Role::Region);
            b.region(
                match group {
                    jig_domain_client::contract::InboxReadGroup::Needs => JigRegion::InboxNeeds,
                    jig_domain_client::contract::InboxReadGroup::Updates => JigRegion::InboxUpdates,
                }
                .region(),
            );
            b.name(&words.say(crate::phrases::jig::JigPhrase::Inbox, &[]));
            crate::pages::inbox(b, client, words, bindings);
        }
        Section::Conversations { .. } => {
            b.open(Role::Region);
            b.region(JigRegion::Conversations.region());
            b.name(&words.say(crate::phrases::jig::JigPhrase::Conversations, &[]));
            crate::pages::conversations(b, client, words, bindings);
        }
        Section::Project { section, .. } => {
            b.open(Role::Region);
            b.region(crate::pages::project::region(section).region());
            b.name(&words.say(crate::phrases::jig::JigPhrase::ProjectPage, &[]));
            crate::pages::project(b, client, words, bindings);
        }
        Section::Deployment => {
            b.open(Role::Region);
            b.region(JigRegion::Deployment.region());
            b.name(&words.say(crate::phrases::jig::JigPhrase::Deployment, &[]));
            crate::pages::deployment(b, client, words, bindings);
        }
    }
    b.close();
    true
}
