//! Conversations and the offered first-words form (client/domain/pages.md, 4).
use crate::phrases::jig::JigPhrase;
use crate::words::Words;
use crate::{JigBindings, Owner, cards};
use jig_client::contract::Phase;
use jig_client::words::{StatusWord, Word};
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Key, Role, Run, State};

use crate::{Borrowed, action::Act, fields::Purpose};
use jig_client::contract::Runtime;
/// Render committed chat facts, window edges and the projects' offered chat forms.
pub(crate) fn section(
    b: &mut Builder,
    client: &Borrowed<'_>,
    words: &Words,
    bindings: &mut JigBindings,
    project: Option<u32>,
) -> bool {
    let conversation_rows = client.conversation_rows();
    let text_limit = bindings.text_limit();
    if !client.shows(crate::page::Section::Conversations { project }) {
        return false;
    }
    crate::pages::work::section(b, words, JigPhrase::Conversations, crate::regions::JigRegion::Conversations);
    super::status(b, client, words, crate::page::Section::Conversations { project });
    let query = match client.frame().document {
        Some(me) => crate::queries::chats(me, project),
        None => None,
    };
    if query.is_none() {
        words.refusal(b, &jig_client::contract::Refusal::NotOffered, text_limit);
        b.close();
        return true;
    }
    b.open(Role::List);
    for row in &conversation_rows.rows {
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
        crate::document_text::loss(b, &row.chip.title, words);
        words.text(b, JigPhrase::ContextProject, &[Hole::Number(u64::from(row.project))]);
        if let Some(me) = client.frame().document {
            words.chat_status(
                b,
                &jig_client::facts::chat_status(&row.chip, &row.waits_on, row.project, me),
                client.wall(),
            );
        }
        match row.waits_on {
            jig_client::contract::WaitsOn::Party { .. } => {}
            jig_client::contract::WaitsOn::Delegates
            | jig_client::contract::WaitsOn::News
            | jig_client::contract::WaitsOn::Nothing => {
                words.text(b, JigPhrase::Word(Word::waiting(&row.waits_on)), &[]);
            }
        }
        if let Some(read) = &row.conversation {
            words.text(
                b,
                if read.unread.limited { JigPhrase::MoreUpdates } else { JigPhrase::Updates },
                &[Hole::Number(u64::from(read.unread.value))],
            );
        }
        crate::pages::work::owner_counts(b, row.phases, words);
        crate::owner_budget::render(b, &row.budget, words);
        b.close();
    }
    b.close();
    crate::pages::cut(b, words, conversation_rows.cut);
    let earlier = match conversation_rows.edges.after {
        jig_client::thread::window::Edge::More(_) => true,
        jig_client::thread::window::Edge::End => client.chats().more,
    };
    words.text(b, JigPhrase::WindowEnd, &[]);
    if earlier {
        crate::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            JigPhrase::ReadEarlier,
            &[],
            Act::More {
                window: skein_lib::Token::new(crate::page::conversations::CONVERSATIONS_WINDOW),
                end: jig_client::thread::window::End::After,
            },
        );
    }
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
        words.refusal(b, refusal, text_limit);
    }
    forms(b, client, words, bindings);
    b.close();
    true
}
fn forms(b: &mut Builder, client: &Borrowed<'_>, words: &Words, bindings: &mut JigBindings) {
    let mut field_built = false;
    if let Some(frame) = &client.frame().document {
        for project in &frame.projects {
            if let Some(template) = jig_client::shapes::template(frame, project.project, jig_client::shapes::CHAT) {
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
                if let jig_client::contract::TemplateExecutor::Charter(number) = template.executor {
                    words.text(b, JigPhrase::Charter, &[Hole::Number(u64::from(number))]);
                }
                match template.budget {
                    jig_client::contract::TemplateBudget::Amount(amount) => {
                        words.text(b, JigPhrase::Budget, &[Hole::Amount(amount)]);
                    }
                    jig_client::contract::TemplateBudget::AsAllowed { .. } => words.text(
                        b,
                        JigPhrase::PoolLeft,
                        &[Hole::Amount(match &project.creating {
                            Some(creating) => creating.within.pool_left,
                            None => 0,
                        })],
                    ),
                    jig_client::contract::TemplateBudget::Given => {}
                }
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
pub(crate) fn runtime(
    b: &mut Builder,
    client: &Borrowed<'_>,
    runtime: &Runtime,
    words: &Words,
    bindings: &mut JigBindings,
) {
    match runtime {
        Runtime::Active(value) => crate::runtime::active(b, value, words),
        Runtime::Waiting { dependencies, hold } => {
            words.text(b, JigPhrase::Word(Word::Phase(Phase::Waiting)), &[]);
            for task in dependencies {
                words.text(b, JigPhrase::ContextTask, &[Hole::Number(*task)]);
            }
            if let Some(part) = hold {
                cards::common::part(b, part, words, bindings);
            }
        }
        Runtime::Idle => {
            words.text(b, JigPhrase::Word(Word::Phase(Phase::Idle)), &[]);
            words.text(b, JigPhrase::Word(Word::Status(StatusWord::Parked)), &[]);
        }
        Runtime::Starting => words.text(b, JigPhrase::Word(Word::Status(StatusWord::Starting)), &[]),
        Runtime::Running { tool, since, .. } => {
            words.text(
                b,
                JigPhrase::ElapsedSpan,
                &[Hole::Span(skein_lib::Duration::from_nanos(
                    client.wall().as_nanos().saturating_sub(since.as_nanos()),
                ))],
            );
            match tool {
                Some(tool) => {
                    words.text(b, JigPhrase::Word(Word::Status(StatusWord::Tool)), &[crate::words::name(tool.bytes())]);
                }
                None => words.text(b, JigPhrase::Word(Word::Status(StatusWord::Writing)), &[]),
            }
        }
        Runtime::WaitingForWords { .. } => words.text(b, JigPhrase::Word(Word::Status(StatusWord::WaitingForYou)), &[]),
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
            words.text(b, JigPhrase::Word(Word::Phase(Phase::Held)), &[]);
            if let Some(was) = was {
                crate::runtime::prior(b, was, words, bindings);
            }
        }
        Runtime::Ended => words.text(b, JigPhrase::Word(Word::Phase(Phase::Ended)), &[]),
    }
}
