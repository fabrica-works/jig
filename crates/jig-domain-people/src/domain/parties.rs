//! Bounded identities for the root's restart census (store.md 7.2).
impl super::Domain {
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
