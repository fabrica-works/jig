//! Conversations and the offered first-words form (client/domain/pages.md, 4).
use crate::{
    Builder, JigBindings, Key, Owner, Role, Run, State, cards,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_client::{Domain, action::Act, contract::Runtime, fields::Purpose};
/// Render committed chat facts, window edges and the projects' offered chat forms.
pub fn conversations(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) {
    super::heading(b, words, JigPhrase::Conversations, &[]);
    b.open(Role::List);
    for row in &client.conversation_rows().rows {
        b.open(Role::Item);
        b.key(Key { owner: Owner::Jig, kind: 20, a: row.chip.task, b: 0 });
        crate::frame::control(
            b,
            words,
            bindings,
            Role::Link,
            JigPhrase::ChipNumber,
            &[Hole::Number(row.chip.task), crate::words::name(row.chip.title.bytes())],
            Act::OpenChat { task: row.chip.task },
        );
        words.text(b, JigPhrase::ContextProject, &[Hole::Number(u64::from(row.project))]);
        b.text(Run::Plain, row.first_words.bytes());
        runtime(b, client, &row.runtime, words, bindings);
        match row.runtime {
            Runtime::Held => {
                if let Some(id) = client.objects().find(jig_client::contract::ObjectKey::Task(row.chip.task)) {
                    cards::card(b, client, id, &cards::Place::Conversation, words, bindings);
                }
            }
            Runtime::Ended => {
                cards::chip::chip(b, client, &row.chip, words, bindings);
            }
            Runtime::Waiting { .. }
            | Runtime::Idle
            | Runtime::Starting
            | Runtime::Running { .. }
            | Runtime::WaitingForWords { .. }
            | Runtime::BackingOff { .. }
            | Runtime::Procedure { .. }
            | Runtime::Person { .. }
            | Runtime::Closing(_) => {}
        }
        words.text(
            b,
            if row.unread.limited { JigPhrase::MoreUpdates } else { JigPhrase::Updates },
            &[Hole::Number(u64::from(row.unread.value))],
        );
        words.text(b, JigPhrase::Delegates, &[Hole::Number(u64::from(row.delegates))]);
        words.text(b, JigPhrase::Budget, &[Hole::Amount(row.budget.total)]);
        words.text(b, JigPhrase::OwnSpend, &[Hole::Amount(row.budget.own)]);
        words.text(b, JigPhrase::DelegateSpend, &[Hole::Amount(row.budget.ended)]);
        b.close();
    }
    b.close();
    super::edges(
        b,
        words,
        bindings,
        &client.conversation_rows().edges,
        jig_client::pages::conversations::CONVERSATIONS_WINDOW,
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
    let mut field_built = false;
    if let Some(frame) = &client.frame().document {
        for project in &frame.projects {
            if !project.chats.is_empty() {
                if !field_built {
                    cards::common::field(b, client, Purpose::NewChat, JigPhrase::NewChat, words, bindings);
                    field_built = true;
                }
                b.open(Role::Region);
                b.name(&words.say(JigPhrase::Project, &[crate::words::name(project.name.bytes())]));
                for charter in &project.chats {
                    words.text(b, JigPhrase::Charter, &[Hole::Number(u64::from(charter.number))]);
                    b.text(Run::Plain, charter.name.bytes());
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
    super::excess(b, client, words);
}
pub(super) fn runtime(b: &mut Builder, client: &Domain, runtime: &Runtime, words: &Words, bindings: &mut JigBindings) {
    match runtime {
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
                &[
                    Hole::Wall(words.adjusted_wall(*at)),
                    Hole::Number(u64::from(*attempt)),
                    Hole::Number(u64::from(*tries)),
                ],
            );
            cards::chip::failure(b, *class, words);
            b.text(Run::Plain, reason.bytes());
        }
        Runtime::Procedure { panel } => cards::common::part(b, panel, words, bindings),
        Runtime::Person { addressed, .. } => cards::common::recipient(b, addressed, words),
        Runtime::Closing(closing) => cards::common::closing(b, closing, words, bindings),
        Runtime::Held => words.text(b, JigPhrase::Held, &[]),
        Runtime::Ended => words.text(b, JigPhrase::Ended, &[]),
    }
}
