//! A chronological log of committed activity and provisional turns (view.md, 9).
//! Reads the conversation owner's windows, word snapshots and call folds. The
//! same task/attempt/turn key names live and committed text; no stream facts are
//! committed by the view. Call objects use shared cards and parts leave slots.
use crate::{Builder, JigBindings, Key, Owner, Role, Run, phrases::jig::JigPhrase, words::Words};
use jig_domain_client::{
    Domain,
    contract::{LiveCall, Party, TurnName},
    conversation::{Conversation, Words as WordState},
};
mod activity;
mod call;
mod turn;
/// Show an agent task's activity in its page's already opened section.
pub fn conversation(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) {
    let state = client.conversation();
    b.open(Role::List);
    b.open(Role::Item);
    b.key(Key { owner: Owner::Jig.opaque(), kind: 28, a: state.task, b: 0 });
    b.open(Role::Log);
    b.region(crate::regions::JigRegion::Conversation.region());
    b.name(&words.say(JigPhrase::ConversationLog, &[]));
    if state.reading.is_some() {
        b.state(crate::State::Busy);
    }
    // The owner retains newest first; a log reads from oldest to newest.
    for offset in 0..state.lines.rows.len() {
        let index = state
            .lines
            .rows
            .len()
            .checked_sub(1)
            .expect("line exists")
            .checked_sub(offset)
            .expect("reverse line exists");
        let line = state.lines.rows.get(index).expect("retained line exists");
        let key = match (line.event.committed_turn(), line.event.word_message()) {
            (Some(name), _) => turn_key(name),
            (None, Some((message, _))) => word_key(state, message),
            (None, None) => Key { owner: Owner::Jig.opaque(), kind: 27, a: state.task, b: line.position },
        };
        b.open(Role::Item);
        b.key(key);
        b.open(Role::Region);
        b.name(&words.say(JigPhrase::ConversationLog, &[]));
        words.wall(b, line.at);
        activity::activity(b, client, &line.event, words, bindings);
        b.close();
        b.close();
    }
    pending(b, state, words);
    for live in &state.live {
        b.open(Role::Item);
        b.key(turn_key(live.name.turn()));
        b.open(Role::Region);
        b.name(&words.say(JigPhrase::Provisional, &[]));
        words.status(b, JigPhrase::Provisional, &[]);
        if let Some(began) = live.began {
            words.wall(b, began);
        }
        crate::markdown::read(live.text.bytes(), 2, b);
        if live.cut {
            words.status(b, JigPhrase::TurnCut, &[]);
        }
        if let Some(value) = &live.call {
            match value {
                LiveCall::Decoded(_) => call::live(b, client, live.name.turn(), value, words, bindings),
                LiveCall::Raw(_) => {
                    if state.current_run.is_none() {
                        call::live(b, client, live.name.turn(), value, words, bindings);
                    }
                }
            }
        }
        b.close();
        b.close();
    }
    if let Some(run) = &state.current_run
        && let Some(value) = &run.call
    {
        b.open(Role::Item);
        b.open(Role::Region);
        b.name(&words.say(JigPhrase::Provisional, &[]));
        words.status(b, JigPhrase::Provisional, &[]);
        call::raw(b, client, value, words, bindings);
        b.close();
        b.close();
    }
    b.close();
    b.close();
    b.close();
    crate::pages::cut(b, words, state.page_cut);
    edges(b, state, words, bindings);
}
fn pending(b: &mut Builder, state: &Conversation, words: &Words) {
    for sent in &state.words.rows {
        let message = match sent.state {
            WordState::BeingSent => None,
            WordState::Sent { message } | WordState::Read { message } => Some(message),
        };
        let mut in_activity = false;
        for line in &state.lines.rows {
            if let Some((seen, _)) = line.event.word_message()
                && message == Some(seen)
            {
                in_activity = true;
            }
        }
        if in_activity {
            continue;
        }
        b.open(Role::Item);
        match sent.request {
            Some(request) => b.key(Key { owner: Owner::Jig.opaque(), kind: 26, a: state.task, b: request.raw() }),
            None => {
                if let Some(message) = message {
                    b.key(word_key(state, message));
                }
            }
        }
        b.open(Role::Region);
        b.name(&words.say(JigPhrase::Person, &[]));
        if let Some(by) = &sent.by {
            party(b, by, words);
            if sent.by_name_cut == Some(true) {
                words.text(b, JigPhrase::DocumentTextCut, &[]);
            }
        }
        b.text(Run::Plain, sent.words.bytes());
        word_state(b, sent.state, words);
        b.close();
        b.close();
    }
}
fn word_key(state: &Conversation, message: u64) -> Key {
    for sent in &state.words.rows {
        match sent.state {
            WordState::Sent { message: number } | WordState::Read { message: number } => {
                if message == number
                    && let Some(request) = sent.request
                {
                    return Key { owner: Owner::Jig.opaque(), kind: 26, a: state.task, b: request.raw() };
                }
            }
            WordState::BeingSent => {}
        }
    }
    Key { owner: Owner::Jig.opaque(), kind: 25, a: state.task, b: message }
}
// Keys are matched only below matched parents (view.md, 6). The log's enclosing item
// task key supplies the first identity; these two words keep full attempt/turn.
fn turn_key(name: TurnName) -> Key {
    Key { owner: Owner::Jig.opaque(), kind: 24, a: name.attempt, b: u64::from(name.number) }
}

fn party(b: &mut Builder, by: &Party, words: &Words) {
    words.actor(b, jig_domain_client::contract::Actor::Party(by.number));
    b.text(Run::Plain, by.name.bytes());
}
fn word_state(b: &mut Builder, state: WordState, words: &Words) {
    let phrase = match state {
        WordState::BeingSent => JigPhrase::WordsBeingSent,
        WordState::Sent { .. } => JigPhrase::WordsSent,
        WordState::Read { .. } => JigPhrase::WordsRead,
    };
    words.status(b, phrase, &[]);
}

fn edges(b: &mut Builder, state: &Conversation, words: &Words, bindings: &mut JigBindings) {
    use jig_domain_client::{
        action::{Act, End},
        windows::Edge,
    };
    for (edge, end, phrase) in [
        (&state.lines.edges.after, End::Before, JigPhrase::ReadEarlier),
        (&state.lines.edges.before, End::After, JigPhrase::ReadLater),
    ] {
        match edge {
            Edge::End => words.text(
                b,
                match end {
                    End::Before => JigPhrase::WindowStart,
                    End::After => JigPhrase::WindowEnd,
                },
                &[],
            ),
            Edge::More(_) => crate::frame::control(
                b,
                words,
                bindings,
                Role::Button,
                phrase,
                &[],
                Act::More { window: skein_lib::Token::new(jig_domain_client::conversation::ACTIVITY_WINDOW), end },
            ),
        }
    }
}
