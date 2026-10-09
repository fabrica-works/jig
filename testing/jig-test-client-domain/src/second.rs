//! The second connector owns resource/kind words and the shared link only.
use jig_client::{contract::Part, link::Link};
use skein_lib::List;
#[derive(Debug)]
pub struct Side {
    parts: List<Held>,
    link: Link,
    party: Option<u64>,
    shown: u64,
}
#[derive(Debug)]
struct Held {
    part: Part,
    generation: u32,
}
impl Side {
    #[must_use]
    pub fn new(capacity: u32, _seed: u64) -> Self {
        Self { parts: List::with_capacity(capacity), link: Link::Starting, party: None, shown: 0 }
    }
    #[must_use]
    pub const fn link(&self) -> Link {
        self.link
    }
    #[must_use]
    pub const fn party(&self) -> Option<u64> {
        self.party
    }
    pub(crate) fn link_is(&mut self, link: Link) {
        self.link = link;
        self.changed();
    }
    pub(crate) fn confirmed(&mut self, party: Option<u64>) {
        self.party = party;
        self.changed();
    }
    pub(crate) fn show(&mut self, part: Part, generation: u32) {
        if self.parts.push(Held { part, generation }).is_ok() {
            self.changed();
        }
    }
    pub(crate) fn left(&mut self, generation: u32) {
        let capacity = self.parts.capacity();
        let old = core::mem::replace(&mut self.parts, List::with_capacity(capacity));
        for held in old.into_boxed() {
            if held.generation != generation {
                assert!(self.parts.push(held).is_ok(), "remaining parts fit");
            }
        }
        self.changed();
    }
    fn changed(&mut self) {
        self.shown = self.shown.checked_add(1).expect("shown count fits");
    }
    #[must_use]
    pub const fn shown(&self) -> u64 {
        self.shown
    }
    #[must_use]
    pub fn contains(&self, part: &Part) -> bool {
        for held in &self.parts {
            if &held.part == part {
                return true;
            }
        }
        false
    }
}
#[must_use]
pub fn worst_case(parts: u32, text: u32) -> Option<u64> {
    List::<Held>::worst_case(parts)?.checked_mul(2)?.checked_add(u64::from(parts).checked_mul(u64::from(text))?)
}
