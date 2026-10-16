//! Typed messages preserve truncation, opaque blocks and independent usage facts.
use crate::JigBindings;
use crate::phrases::jig::JigPhrase;
use crate::words::Words;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use jig_client_domain::contract::{Block, Speaker, TurnName, TurnProblem, TurnRead};
use jig_domain_client::Domain;
pub(super) fn turn(
    b: &mut Builder,
    client: &Domain,
    name: TurnName,
    turn: &TurnRead,
    words: &Words,
    bindings: &mut JigBindings,
) {
    words.text(b, JigPhrase::Attempt, &[Hole::Number(name.attempt)]);
    words.text(b, JigPhrase::TurnNumber, &[Hole::Number(u64::from(name.number))]);
    let turn = match turn {
        TurnRead::Decoded(turn) => turn,
        TurnRead::Undecodable { why } => {
            match why {
                TurnProblem::Version { found } => {
                    words.status(b, JigPhrase::TurnUnreadVersion, &[Hole::Number(u64::from(*found))]);
                }
                TurnProblem::Malformed => words.status(b, JigPhrase::TurnUnreadMalformed, &[]),
                TurnProblem::TooLarge => words.status(b, JigPhrase::TurnUnreadTooLarge, &[]),
            }
            return;
        }
    };
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
                Block::Text(text) => skein_ui::markdown::read(text.bytes(), 2, b),
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
