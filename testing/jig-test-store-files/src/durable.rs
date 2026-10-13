//! The real store under the conformance harness (store.md 14).
use crate::Store;
impl jig_conformance::Durable for Store {
    type Write = jig_test_domain::Write;
    fn submit(&mut self, number: u64, writes: Box<[Self::Write]>, delay: u32) {
        Self::submit(self, number, writes, delay);
    }
    fn tick(&mut self, hold: bool) -> Result<Option<u64>, u64> {
        Self::tick(self, hold)
    }
    fn release_held(&mut self) -> Option<u64> {
        Self::release_held(self)
    }
    fn crash(&mut self, seed: u64) {
        Self::crash(self, seed);
    }
    fn fork(&self) -> Self {
        Self::fork(self)
    }
    fn applied(&self) -> u64 {
        self.applied
    }
    fn idle(&self) -> bool {
        self.pending.is_empty() && !self.loading()
    }
    fn holding(&self) -> bool {
        !self.held.is_empty()
    }
}
