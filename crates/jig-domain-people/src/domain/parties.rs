//! Bounded identities for the root's restart census (store.md 7.2).
impl super::Domain {
    /// People census for the host rule (accounts.md, 3.4); restoration is conservative.
    #[must_use]
    pub fn persons(&self) -> u32 {
        if !self.ready() {
            return u32::MAX;
        }
        let mut count = 0_u32;
        for (_, identity) in &self.people {
            if self.kind_of(identity) == super::Kind::Person {
                count = count.saturating_add(1);
            }
        }
        count
    }
    #[must_use]
    pub fn parties(&self) -> skein_lib::List<u64> {
        let mut parties = skein_lib::List::with_capacity(self.people.len());
        if self.ready() {
            for (party, _) in &self.people {
                parties.push(*party).expect("one entry per stored identity");
            }
        }
        parties
    }
}
