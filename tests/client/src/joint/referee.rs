//! Independent protocol obligations from actual shell consumption and root inputs.
//! Durable business rules remain the harness's Rules and the core observer
//! (client/domain/testing.md, section 6).
use skein_lib::Token;
use std::collections::BTreeSet;

/// The original normal watch whose consumed delivery was not acknowledged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissingAcknowledgement(pub Token);

/// Observes traffic only; it does not read a root or change pacing.
#[derive(Default)]
pub struct Referee {
    consumed: BTreeSet<Token>,
    acknowledged: u64,
}
impl Referee {
    /// A shell consumed a normal delivery on this exact original owner route.
    pub fn consumed(&mut self, watcher: Token) {
        assert!(self.consumed.insert(watcher), "a consumed delivery has one acknowledgement");
    }
    /// The independently observed root input acknowledges that delivery.
    pub fn acknowledged(&mut self, watcher: Token) {
        if self.consumed.remove(&watcher) {
            self.acknowledged += 1;
        }
    }
    /// # Errors
    /// Names the original route of an acknowledgement dropped by the peer.
    pub fn check(&self) -> Result<(), MissingAcknowledgement> {
        self.consumed.first().map_or(Ok(()), |watcher| Err(MissingAcknowledgement(*watcher)))
    }
    /// Number of consumed normal deliveries actually acknowledged to the root.
    #[must_use]
    pub fn acknowledgements(&self) -> u64 {
        self.acknowledged
    }
}
