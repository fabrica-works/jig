//! Typed messages preserve truncation, opaque blocks and independent usage facts.
use crate::{
    Builder, JigBindings, Role, Run,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_client::{
    Domain,
    contract::{Block, Speaker, Turn, TurnName},
};
pub(super) fn turn(
    b: &mut Builder,
    client: &Domain,
    name: TurnName,
    turn: &Turn,
    words: &Words,
    bindings: &mut JigBindings,
) {
    words.text(b, JigPhrase::Attempt, &[Hole::Number(name.attempt)]);
    words.text(b, JigPhrase::TurnNumber, &[Hole::Number(u64::from(name.number))]);
    for message in &turn.messages {
        b.open(Role::Region);
        let phrase = match message.speaker {
            Speaker::Person => JigPhrase::Person,
            Speaker::Assistant => JigPhrase::Agent,
            Speaker::Tool => JigPhrase::CallCompleted,
            Speaker::System => JigPhrase::TurnInstructions,
        };
        b.name(&words.say(phrase, &[]));
        words.text(b, phrase, &[]);
        for block in &message.blocks {
            match block {
                Block::Text(text) => crate::markdown::read(text.bytes(), 2, b),
                Block::Opaque { dialect, size } => {
                    words.text(b, JigPhrase::OpaqueDialect, &[crate::words::name(dialect.bytes())]);
                    words.text(b, JigPhrase::OpaqueBytes, &[Hole::Number(*size)]);
                }
                Block::Call(call) => super::call::call(b, client, name, call, words, bindings),
                Block::Result { call, summary, failed } => {
                    words.text(b, JigPhrase::CallNumber, &[Hole::Number(*call)]);
                    words.text(b, if *failed { JigPhrase::CallFailed } else { JigPhrase::CallCompleted }, &[]);
                    b.text(Run::Plain, summary.bytes());
                }
            }
        }
        b.close();
    }
    if turn.cut {
        words.status(b, JigPhrase::TurnCut, &[]);
    }
    for (phrase, value) in [
        (JigPhrase::TurnInput, turn.used.input),
        (JigPhrase::TurnOutput, turn.used.output),
        (JigPhrase::CacheRead, turn.used.cached),
        (JigPhrase::CacheWritten, turn.used.cache_written),
    ] {
        words.text(b, phrase, &[Hole::Number(value)]);
    }
    words.text(b, JigPhrase::TurnSpent, &[Hole::Amount(turn.spent)]);
}
