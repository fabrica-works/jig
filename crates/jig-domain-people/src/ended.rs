//! Ended positions are supplied for one decision, never retained (people.md, 6).
use crate::domain::{Pending, answer_decided};
use crate::{Ask, Domain, EntryKind, Key, Limits, Outcome, Request, Seen, Stored, Update};
use skein_lib::{Env, Id, Queue, Token};

/// The loaded position and owner header for one authenticated ended mark.
#[derive(Debug)]
pub struct EndedMark {
    pub request: Token,
    pub project: u32,
    pub last_reply: u64,
    pub seen: u64,
}

impl Domain {
    /// A different pending mark on this conversation owns its point loads.
    #[must_use]
    pub fn conversation_mark_loading(&self, task: u64, except: Token) -> bool {
        for (_, id) in &self.flights {
            let flight = self.pending.get(*id).expect("pending flight");
            if id.token() != except
                && let Ask::MarkSeen { seen: Seen::Conversation { task: number, .. } } = flight.ask
                && number == task
            {
                return true;
            }
        }
        false
    }

    /// Decide the mark against its durable position without caching that position.
    pub fn mark_ended(&mut self, env: &Env<Limits>, mark: EndedMark, out: &mut Queue<Request>) {
        let Some((party, seen @ Seen::Conversation { task, through })) = self.seen_context(mark.request) else {
            return;
        };
        let next = mark.seen.max(through);
        let delta = mark
            .last_reply
            .saturating_sub(mark.seen)
            .checked_sub(mark.last_reply.saturating_sub(next))
            .expect("marks only reduce unread");
        let count = *self.chat_updates.get(&(party, mark.project)).unwrap_or(&0);
        self.chat_updates
            .insert((party, mark.project), count.checked_sub(u128::from(delta)).expect("counted ended turns"))
            .expect("known party aggregate");
        mark_cached(self, party, task, next, mark.last_reply);
        out.push(Request::Save { record: Stored::EndedRead { task, party, through: next } });
        answer_decided(self, env, Id::<Pending>::from_token(mark.request), Outcome::MarkedSeen { seen }, out);
    }
}

pub(super) fn ended_state(
    domain: &mut Domain,
    party: u64,
    task: u64,
    through: u64,
    ending: bool,
    out: &mut Queue<Request>,
) {
    let _old = domain.conversations_seen.remove(&(party, task));
    if ending {
        out.push(Request::Erase { key: Key::ReadConversation { party, task } });
        out.push(Request::Save { record: Stored::EndedRead { task, party, through } });
    }
    mark_cached(domain, party, task, through, u64::MAX);
}

fn mark_cached(domain: &mut Domain, party: u64, task: u64, through: u64, last_reply: u64) {
    if let Some(entries) = domain.unread.get_mut(&party) {
        for entry in entries.as_mut() {
            if entry.task == task
                && let Some(forgettable) = &mut entry.forgettable
            {
                forgettable.turns_read |= through >= last_reply;
            }
        }
    }
    if let Some(recent) = domain.recent.get_mut(&party) {
        for item in recent.as_mut() {
            if let Update::ChatReply(reply) = item.update
                && reply.entry.task == task
                && let EntryKind::ChatReply { message } = reply.entry.kind
                && message <= through
            {
                item.gone = true;
            }
        }
    }
}
