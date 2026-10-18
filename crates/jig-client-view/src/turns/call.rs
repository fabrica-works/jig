//! Calls fold only input/result detail; made objects stay live as cards (view.md, 9).
use crate::phrases::jig::JigPhrase;
use crate::words::Words;
use crate::{JigBindings, cards};
use jig_client::words::{StatusWord, Word};
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run, State};

use crate::{Borrowed, action::Act};
use jig_client::contract::{Call, CurrentCall, LiveCall, TurnName};
use jig_client::thread::{CallName, Fold};
pub(super) fn call(
    b: &mut Builder,
    client: &Borrowed<'_>,
    turn: TurnName,
    call: &Call,
    words: &Words,
    bindings: &mut JigBindings,
) {
    let name = CallName { attempt: turn.attempt, turn: turn.number, call: call.number };
    let open = client.conversation().fold(name) == Fold::Open;
    b.open(Role::Region);
    b.name(call.tool.bytes());
    words.text(b, JigPhrase::CallNumber, &[Hole::Number(call.number)]);
    words.text(b, JigPhrase::Word(Word::Status(StatusWord::Tool)), &[crate::words::name(call.tool.bytes())]);
    words.text(
        b,
        if call.failed {
            JigPhrase::CallFailed
        } else if call.outcome.is_some() {
            JigPhrase::CallCompleted
        } else {
            JigPhrase::CallPending
        },
        &[],
    );
    if let Some(summary) = &call.outcome {
        b.text(Run::Plain, summary.bytes());
    }
    let label = words
        .say(if open { JigPhrase::FoldCall } else { JigPhrase::OpenCall }, &[crate::words::name(call.tool.bytes())]);
    b.open(Role::Button);
    b.name(&label);
    if open {
        b.state(State::Expanded);
    }
    b.bind(bindings.insert(crate::JigMeaning::Action(Act::FoldCall { name, open: !open })));
    b.close();
    if open {
        b.text(Run::Code, call.input.bytes());
        if let Some(result) = &call.outcome {
            b.text(Run::Plain, result.bytes());
        }
    }
    for object in &call.objects {
        if let Some(id) = client.objects().find(*object) {
            cards::card(b, client, id, &cards::Place::Conversation, words, bindings);
        } else {
            words.status(b, JigPhrase::LoadingObject, &[]);
        }
    }
    for part in &call.parts {
        cards::common::part(b, part, words, bindings);
    }
    b.close();
}

pub(super) fn live(
    b: &mut Builder,
    client: &Borrowed<'_>,
    turn: TurnName,
    value: &LiveCall,
    words: &Words,
    bindings: &mut JigBindings,
) {
    match value {
        LiveCall::Decoded(value) => call(b, client, turn, value, words, bindings),
        LiveCall::Raw(value) => raw(b, client, value, words, bindings),
    }
}
pub(super) fn raw(
    b: &mut Builder,
    client: &Borrowed<'_>,
    call: &CurrentCall,
    words: &Words,
    bindings: &mut JigBindings,
) {
    let open = client.conversation().current_fold(&call.identity) == Fold::Open;
    b.open(Role::Region);
    let region = words
        .book
        .say(words.language, crate::live_call_words::LiveCallPhrase::ObservedCurrentCall.id(), &[])
        .expect("checked observed call label")
        .text;
    b.name(&region);
    crate::literal_segment::render(b, call.identity.id.bytes());
    if let Some(began) = call.began {
        words.wall(b, began);
    }
    crate::literal_segment::render(b, call.tool.bytes());
    let phrase = if open {
        crate::live_call_words::LiveCallPhrase::FoldCurrentCallInput
    } else {
        crate::live_call_words::LiveCallPhrase::OpenCurrentCallInput
    };
    let label = words.book.say(words.language, phrase.id(), &[]).expect("checked live input control").text;
    b.open(Role::Button);
    b.name(&label);
    if open {
        b.state(State::Expanded);
    }
    b.bind(
        bindings
            .insert(crate::JigMeaning::Action(Act::FoldCurrentCall { identity: call.identity.clone(), open: !open })),
    );
    b.close();
    if open {
        crate::literal_segment::render(b, call.input.bytes());
    }
    if call.cut {
        words.status(b, JigPhrase::TurnCut, &[]);
    }
    b.close();
}
