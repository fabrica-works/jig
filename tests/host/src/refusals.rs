//! Check message refusals at the hub's public boundary (hosts.md, 11).
use jig_domain_host::{Event, LinkDown, LinkUp, Refused, Request};
use skein_lib::Token;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Default)]
pub(crate) struct Referee {
    pending: BTreeMap<(Token, Token), BTreeSet<Token>>,
    owners: BTreeMap<Token, (Token, Token)>,
    agents: BTreeMap<Token, (Token, Token)>,
}
impl Referee {
    pub(crate) fn event(&mut self, event: &Event) {
        match event {
            Event::Down(LinkDown::Message { run, attempt, name, .. }) => {
                self.pending.entry((*run, *attempt)).or_default().insert(*name);
            }
            Event::Started { owner, agent } => {
                if let Some(key) = self.owners.get(owner) {
                    self.agents.insert(*agent, *key);
                }
            }
            Event::Refused { owner, name, .. } => {
                if let Some(key) = self.owners.get(owner) {
                    self.pending.entry(*key).or_default().insert(*name);
                }
            }
            Event::Turn { .. }
            | Event::Called { .. }
            | Event::Withdrawn { .. }
            | Event::Activity { .. }
            | Event::Rejected { .. }
            | Event::Exhausted { .. }
            | Event::Finished { .. }
            | Event::RelayCancelled { .. }
            | Event::CancelAll { .. }
            | Event::Opened { .. }
            | Event::Down(_)
            | Event::Prepared { .. }
            | Event::Unprepared { .. }
            | Event::Yielded { .. }
            | Event::Faulted { .. }
            | Event::Gone { .. }
            | Event::Delivered { .. }
            | Event::Saved { .. } => {}
        }
    }
    pub(crate) fn admitted(&mut self, owner: Token, run: Token, attempt: Token) {
        self.owners.insert(owner, (run, attempt));
    }
    pub(crate) fn request(&mut self, request: &Request) {
        match request {
            Request::Message { agent, name, .. } => {
                if let Some(key) = self.agents.get(agent) {
                    self.pending.entry(*key).or_default().remove(name);
                }
            }
            Request::Up(LinkUp::Turn { run, attempt, turn }) => self.fence((*run, *attempt), turn.read, &turn.refused),
            Request::Up(LinkUp::Answer { run, attempt, answer, .. }) => {
                self.fence((*run, *attempt), answer.read, &answer.refused);
            }
            Request::Up(_)
            | Request::Hello { .. }
            | Request::Reply { .. }
            | Request::Start { .. }
            | Request::AcknowledgeAgentTurn { .. }
            | Request::Deliver { .. }
            | Request::CancelRelay { .. }
            | Request::Prepare { .. }
            | Request::Abort { .. }
            | Request::Stop { .. }
            | Request::Grant { .. }
            | Request::Save { .. }
            | Request::Release { .. } => {}
        }
    }
    fn fence(&mut self, key: (Token, Token), read: Option<Token>, refused: &[Refused]) {
        let names = self.pending.entry(key).or_default();
        for refused in refused {
            names.remove(&refused.name);
        }
        if let Some(read) = read {
            assert!(
                names.iter().all(|name| name.raw() > read.raw()),
                "each refusal arrives before the first fence that passes it"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Referee;
    use jig_domain_host::{Event, LinkUp, MessageRefusal, Request, Turn};
    use skein_lib::Token;
    #[test]
    #[should_panic(expected = "each refusal arrives before the first fence that passes it")]
    fn a_missing_refusal_at_a_passing_fence_is_rejected() {
        let mut referee = Referee::default();
        referee.admitted(Token::new(1), Token::new(31), Token::new(932));
        referee.event(&Event::Refused { owner: Token::new(1), name: Token::new(7), reason: MessageRefusal::Full });
        referee.request(&Request::Up(LinkUp::Turn {
            run: Token::new(31),
            attempt: Token::new(932),
            turn: Turn {
                turn: 1,
                spent: 0,
                read: Some(Token::new(8)),
                opens_window: false,
                refused: Box::new([]),
                body: Box::new([]),
            },
        }));
    }
}
