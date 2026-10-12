//! Observe grants addressed by the fleet after its worker's hello.
use super::{Down, Message, World, fleet};
use skein_lib::Token;

impl World {
    /// The latest grant the parent routed to a worker's current channel.
    #[must_use]
    pub fn received_grant(&self) -> Option<(Token, Token, Token, fleet::Grant)> {
        self.last_grant
    }

    /// Grants emitted for placements and for a worker's later listings.
    #[must_use]
    pub fn grants_sent(&self) -> u64 {
        self.grants_sent
    }

    pub(super) fn assignment_sent(&mut self, channel: Token, run: Token, attempt: Token) {
        self.stats.assigned += 1;
        self.sent(run, attempt, Down::Assign);
        self.down(channel, Message::Assign { run: run.raw(), attempt: attempt.raw() });
    }

    pub(super) fn record_grant(&mut self, channel: Token, run: Token, attempt: Token, grant: fleet::Grant) {
        assert!(self.channels.contains_key(&channel.raw()), "grant names a worker's channel");
        self.last_grant = Some((channel, run, attempt, grant));
        self.grants_sent = self.grants_sent.checked_add(1).expect("bounded world events");
    }
}
