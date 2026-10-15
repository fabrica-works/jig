//! Conversations and the offered first-words form (client/domain/pages.md, 4).
use crate::ui::{Builder, Key, Role, Run, State};
use crate::{
    JigBindings, Owner, cards,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};

use jig_domain_client::{Domain, action::Act, contract::Runtime, fields::Purpose};
/// Render committed chat facts, window edges and the projects' offered chat forms.
pub(crate) fn section(
    b: &mut Builder,
    client: &Domain,
    words: &Words,
    bindings: &mut JigBindings,
    project: Option<u32>,
) -> bool {
    if !client.shows(jig_domain_client::pages::Section::Conversations { project }) {
        return false;
    }
    crate::pages::work::section(b, words, JigPhrase::Conversations, crate::regions::JigRegion::Conversations);
    super::status(b, client, words, jig_domain_client::pages::Section::Conversations { project });
    b.open(Role::List);
    for row in &client.conversation_rows().rows {
        b.open(Role::Item);
        b.key(Key { owner: Owner::Jig.opaque(), kind: 20, a: row.chip.task, b: 0 });
        crate::frame::control(
            b,
            words,
            bindings,
            Role::Link,
            JigPhrase::ChipNumber,
            &[Hole::Number(row.chip.task), crate::words::name(row.chip.title.bytes())],
            Act::OpenChat { task: row.chip.task },
        );
        if let Some(runtime) = &row.runtime {
            title_loss(b, &row.chip.title, runtime, words);
        }
        words.text(b, JigPhrase::ContextProject, &[Hole::Number(u64::from(row.project))]);
        crate::document_text::plain(b, &row.first_words, words);
        if let Some(value) = &row.runtime {
            runtime(b, client, value, words, bindings);
            match value {
                Runtime::Held { .. } => {
                    if let Some(id) = client.objects().find(jig_domain_client::contract::ObjectKey::Task(row.chip.task))
                        && !client.owes(id)
                    {
                        cards::card(b, client, id, &cards::Place::Conversation, words, bindings);
                    }
                }
                Runtime::Ended => {
                    cards::chip::chip(b, client, &row.chip, words, bindings);
                }
                Runtime::Active(_)
                | Runtime::Waiting { .. }
                | Runtime::Idle
                | Runtime::Starting
                | Runtime::Running { .. }
                | Runtime::WaitingForWords { .. }
                | Runtime::BackingOff { .. }
                | Runtime::Procedure { .. }
                | Runtime::Person { .. }
                | Runtime::Closing(_) => {}
            }
        } else {
            cards::chip::chip(b, client, &row.chip, words, bindings);
        }
        if let Some(unread) = row.unread {
            words.text(
                b,
                if unread.limited { JigPhrase::MoreUpdates } else { JigPhrase::Updates },
                &[Hole::Number(u64::from(unread.value))],
            );
        }
        words.text(b, JigPhrase::CachedReplies, &[Hole::Number(u64::from(row.cached_replies))]);
        words.text(b, JigPhrase::Delegates, &[Hole::Number(u64::from(row.delegates))]);
        crate::owner_budget::render(b, &row.budget, words);
        b.close();
    }
    b.close();
    crate::pages::cut(b, words, client.conversation_rows().cut);
    crate::pages::edges(
        b,
        words,
        bindings,
        &client.conversation_rows().edges,
        jig_domain_client::pages::conversations::CONVERSATIONS_WINDOW,
    );
    let state = client.chats();
    if let Some(token) = state.starting {
        b.open(Role::Status);
        b.name(&words.say(JigPhrase::StartChat, &[]));
        b.state(State::Busy);
        if let Some(intent) = client.intents().get(skein_lib::Id::from_token(token)) {
            words.progress(b, intent.progress);
        }
        b.close();
    }
    if let Some(refusal) = &state.refusal {
        words.refusal(b, refusal);
    }
    forms(b, client, words, bindings);
    b.close();
    true
}
fn forms(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) {
    let mut field_built = false;
    if let Some(frame) = &client.frame().document {
        for offers in &frame.charters {
            let mut membership = None;
            for project in &frame.projects {
                if project.project == offers.project {
                    membership = Some(project);
                    break;
                }
            }
            let Some(project) = membership else {
                continue;
            };
            if !offers.chats.is_empty() {
                if !field_built {
                    b.open(Role::Form);
                    b.region(crate::regions::JigRegion::NewChat.region());
                    b.name(&words.say(JigPhrase::NewChat, &[]));
                    cards::common::field(b, client, Purpose::NewChat, JigPhrase::NewChat, words, bindings);
                    field_built = true;
                }
                b.open(Role::Region);
                b.name(&crate::frame_facts::project_label(words, project));
                crate::frame_facts::project_loss(b, words, project);
                for charter in &offers.chats {
                    words.text(b, JigPhrase::Charter, &[Hole::Number(u64::from(charter.number))]);
                    crate::frame_facts::charter_name(b, words, charter);
                }
                words.text(b, JigPhrase::Budget, &[Hole::Amount(project.chat_budget)]);
                crate::frame::control(
                    b,
                    words,
                    bindings,
                    Role::Button,
                    JigPhrase::StartChat,
                    &[],
                    Act::StartChat { project: project.project },
                );
                b.close();
            }
        }
    }
    if field_built {
        b.close();
    }
}
pub(crate) fn runtime(b: &mut Builder, client: &Domain, runtime: &Runtime, words: &Words, bindings: &mut JigBindings) {
    match runtime {
        Runtime::Active(value) => crate::runtime::active(b, value, words),
        Runtime::Waiting { dependencies, hold } => {
            words.text(b, JigPhrase::Waiting, &[]);
            for task in dependencies {
                words.text(b, JigPhrase::ContextTask, &[Hole::Number(*task)]);
            }
            if let Some(part) = hold {
                cards::common::part(b, part, words, bindings);
            }
        }
        Runtime::Idle => {
            words.text(b, JigPhrase::Idle, &[]);
            words.text(b, JigPhrase::ParkedResume, &[]);
        }
        Runtime::Starting => words.text(b, JigPhrase::TaskStarting, &[]),
        Runtime::Running { tool, since, .. } => {
            words.text(
                b,
                JigPhrase::ElapsedSpan,
                &[Hole::Span(skein_lib::Duration::from_nanos(
                    client.wall().as_nanos().saturating_sub(since.as_nanos()),
                ))],
            );
            match tool {
                Some(tool) => words.text(b, JigPhrase::Tool, &[crate::words::name(tool.bytes())]),
                None => words.text(b, JigPhrase::Writing, &[]),
            }
        }
        Runtime::WaitingForWords { .. } => words.text(b, JigPhrase::WaitingWords, &[]),
        Runtime::BackingOff { at, attempt, tries, class, reason } => {
            words.text(
                b,
                JigPhrase::RetryAt,
                &[Hole::Wall(words.adjusted_wall(*at)), Hole::Number(*attempt), Hole::Number(u64::from(*tries))],
            );
            cards::chip::failure(b, *class, words);
            b.text(Run::Plain, reason.bytes());
        }
        Runtime::Procedure { panel } => cards::common::part(b, panel, words, bindings),
        Runtime::Person { addressed, .. } => cards::common::recipient(b, addressed, words),
        Runtime::Closing(closing) => cards::common::closing(b, closing, words, bindings),
        Runtime::Held { was } => {
            words.text(b, JigPhrase::Held, &[]);
            if let Some(was) = was {
                crate::runtime::prior(b, was, words, bindings);
            }
        }
        Runtime::Ended => words.text(b, JigPhrase::Ended, &[]),
    }
}

// Ended rows already render the complete chip, including its title-loss caption.
fn title_loss(b: &mut Builder, title: &jig_domain_client::contract::DocumentText, runtime: &Runtime, words: &Words) {
    match runtime {
        Runtime::Ended => {}
        Runtime::Active(_)
        | Runtime::Waiting { .. }
        | Runtime::Idle
        | Runtime::Starting
        | Runtime::Running { .. }
        | Runtime::WaitingForWords { .. }
        | Runtime::BackingOff { .. }
        | Runtime::Procedure { .. }
        | Runtime::Person { .. }
        | Runtime::Closing(_)
        | Runtime::Held { .. } => {
            if title.cut {
                words.text(b, JigPhrase::ChipTitleCut, &[]);
            }
        }
    }
}
