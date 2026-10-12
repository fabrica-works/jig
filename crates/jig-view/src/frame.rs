//! Shared facts and application navigation (client/domain/pages.md, section 2).
//! Reads the committed frame and page summary, never computes roles or authority.
//! All interactive nodes use the last tree's token table; application navigation
//! remains in its owner slot. Build under an already open page.
use crate::{
    Builder, JigBindings, JigMeaning, Role, Run, SlotRef, State,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_domain_client::{Domain, action::Act, address::Address, pages::navigation::Destination};

/// Build the shared frame and leave the application's navigation slot.
pub fn frame(b: &mut Builder, client: &Domain, nav: SlotRef, words: &Words, bindings: &mut JigBindings) {
    let summary = client.frame_summary();
    b.open(Role::Region);
    b.name(&words.say(JigPhrase::Frame, &[]));
    words.link(b, summary.link);
    if summary.behind {
        words.status(b, JigPhrase::Behind, &[]);
    }
    match &client.frame().document {
        Some(frame) => {
            crate::frame_facts::identity(b, words, &frame.party);
            crate::frame_facts::counts(b, words, frame);
            let phrase = if summary.updates.limited { JigPhrase::MoreUpdates } else { JigPhrase::Updates };
            control(
                b,
                words,
                bindings,
                Role::Link,
                phrase,
                &[Hole::Number(u64::from(summary.updates.value))],
                Act::OpenUpdates,
            );
            control(
                b,
                words,
                bindings,
                Role::Link,
                JigPhrase::Needs,
                &[Hole::Number(u64::from(summary.needs_total))],
                Act::Go { address: Address::Inbox { project: None } },
            );
            for project in &frame.projects {
                membership(b, project, summary.project, words, bindings);
            }
        }
        None => words.status(b, JigPhrase::SignedOut, &[]),
    }
    b.open(Role::TextField);
    b.name(&words.say(JigPhrase::TaskNumber, &[]));
    b.state(State::SubmitsOnEnter);
    b.bind(bindings.insert(JigMeaning::TaskNumber));
    b.close();
    b.open(Role::Region);
    b.name(&words.say(JigPhrase::Navigation, &[]));
    for entry in &client.navigation().entries {
        if client.navigation().offered(*entry, client.frame().document.as_ref()) {
            match entry {
                Destination::Jig(address) => {
                    let phrase = match address {
                        Address::Inbox { .. } => JigPhrase::Inbox,
                        Address::Conversations { .. } => JigPhrase::Conversations,
                        Address::Task { .. } => JigPhrase::TaskPage,
                        Address::Project { .. } => JigPhrase::ProjectPage,
                        Address::Deployment => JigPhrase::Deployment,
                    };
                    match address {
                        Address::Task { task, .. } => control(
                            b,
                            words,
                            bindings,
                            Role::Link,
                            phrase,
                            &[Hole::Number(*task)],
                            Act::Go { address: *address },
                        ),
                        Address::Inbox { .. }
                        | Address::Conversations { .. }
                        | Address::Project { .. }
                        | Address::Deployment => {
                            control(b, words, bindings, Role::Link, phrase, &[], Act::Go { address: *address });
                        }
                    }
                }
                Destination::Application { .. } => {}
            }
        }
    }
    b.slot(nav, &words.say(JigPhrase::LoadingNavigation, &[]));
    b.close();
    b.close();
}
/// A named actionable node whose link and bound act share a meaning.
pub fn control(
    b: &mut Builder,
    words: &Words,
    bindings: &mut JigBindings,
    role: Role,
    phrase: JigPhrase,
    holes: &[Hole],
    action: Act,
) {
    let label = words.say(phrase, holes);
    b.open(role);
    b.name(&label);
    b.text(Run::Plain, &label);
    let binding = bindings.insert(JigMeaning::Action(action));
    if role == Role::Link {
        b.link(binding);
    }
    b.bind(binding);
    b.close();
}

fn membership(
    b: &mut Builder,
    project: &jig_domain_client::contract::ProjectRole,
    selected_project: Option<u32>,
    words: &Words,
    bindings: &mut JigBindings,
) {
    let selected = selected_project == Some(project.project);
    let label = crate::frame_facts::project_label(words, project);
    b.open(Role::Button);
    b.name(&label);
    b.text(Run::Plain, &label);
    if selected {
        b.state(State::Current);
    }
    b.bind(bindings.insert(JigMeaning::Action(Act::Go {
        address: Address::Project {
            project: project.project,
            section: jig_domain_client::address::ProjectSection::Overview,
        },
    })));
    b.close();
    crate::frame_facts::project_loss(b, words, project);
    crate::frame_facts::membership(b, words, project);
}
