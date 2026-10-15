//! Activity's own facts; objects are shown by their shared cards (objects.md, 10).
use crate::phrases::jig::JigPhrase;
use crate::words::Words;
use crate::{JigBindings, cards};
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Run};

use jig_domain_client::{
    Domain,
    contract::{Activity, RunBeginning, RunEnding},
};
pub(super) fn activity(b: &mut Builder, client: &Domain, value: &Activity, words: &Words, bindings: &mut JigBindings) {
    match value {
        Activity::Owner { source, decoded } => {
            crate::owner_activity::render(b, source, words);
            if let Some(archive) = decoded {
                words.text(b, JigPhrase::ArchiveCumulativeSpend, &[Hole::Amount(archive.cumulative)]);
                words.text(b, JigPhrase::ArchiveAcceptedAt, &[Hole::Wall(words.adjusted_wall(archive.at))]);
                match archive.read {
                    Some(read) => words.text(b, JigPhrase::ArchiveReadThrough, &[Hole::Number(read)]),
                    None => words.text(b, JigPhrase::ArchiveReadAbsent, &[]),
                }
            }
            if let Some(archive) = decoded {
                super::turn::turn(b, client, archive.name, &archive.turn, words, bindings);
            }
            if let Some((message, _)) = value.word_message() {
                super::word_state(b, client.conversation().words.state(message), words);
            }
        }
        Activity::Words { by, message, words: value } => {
            super::party(b, by, words);
            b.text(Run::Plain, value.bytes());
            super::word_state(b, client.conversation().words.state(*message), words);
        }
        Activity::Turn { name, turn } => super::turn::turn(b, client, *name, turn, words, bindings),
        Activity::RunBegan { attempt, how, why } => {
            words.text(b, JigPhrase::Attempt, &[Hole::Number(*attempt)]);
            match how {
                RunBeginning::Resumed => words.text(b, JigPhrase::RunResumed, &[]),
                RunBeginning::Fresh { tail } => {
                    words.text(b, JigPhrase::RunFreshTail, &[Hole::Number(u64::from(*tail))]);
                }
            }
            b.text(Run::Plain, why.bytes());
        }
        Activity::RunEnded { attempt, how, spent } => {
            words.text(b, JigPhrase::Attempt, &[Hole::Number(*attempt)]);
            match how {
                RunEnding::Parked => words.text(b, JigPhrase::Parked, &[]),
                RunEnding::Finished => words.text(b, JigPhrase::RunFinished, &[]),
                RunEnding::Cancelled => words.text(b, JigPhrase::RunCancelled, &[]),
                RunEnding::Failed { class, reason } => {
                    cards::chip::failure(b, *class, words);
                    b.text(Run::Plain, reason.bytes());
                }
            }
            words.amount(b, *spent);
        }
        Activity::Result { result } => {
            if let Some(id) = client.objects().find(jig_domain_client::contract::ObjectKey::Result(result.task.task)) {
                cards::card(b, client, id, &cards::Place::Conversation, words, bindings);
            } else {
                cards::result::facts(b, client, result, words, bindings);
            }
        }
        Activity::News { topic, summary, part } => {
            words.text(b, JigPhrase::NewsTopic, &[Hole::Number(u64::from(*topic))]);
            b.text(Run::Plain, summary.bytes());
            if let Some(part) = part {
                cards::common::part(b, part, words, bindings);
            }
        }
        Activity::Decision { object, by, how } => {
            words.actor(b, *by);
            words.decision(b, how);
            if let Some(id) = client.objects().find(*object) {
                cards::card(b, client, id, &cards::Place::Conversation, words, bindings);
            }
        }
    }
}
