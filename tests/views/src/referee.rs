//! An outside check of admission and document flow control.
use jig_core_views::{Event, Request};
use skein_lib::Token;
use std::collections::BTreeMap;
#[derive(Debug, Default)]
pub struct Referee {
    open: BTreeMap<Token, bool>,
    pub deliveries: u64,
    pub ended: u64,
}
impl Referee {
    pub fn before(&mut self, event: &Event) {
        if let Event::Delivered { watcher } = event {
            let flight = self.open.get_mut(watcher).expect("open watch");
            assert!(*flight, "outstanding delivery");
            *flight = false;
        }
    }
    pub fn saw(&mut self, requests: &[Request]) {
        for request in requests {
            match request {
                Request::Watching { watcher } => {
                    assert!(self.open.insert(*watcher, false).is_none(), "unique admission");
                }
                Request::Stream { .. } => {}
                Request::Make { watcher, .. } => {
                    assert_eq!(self.open.get(watcher), Some(&false), "construction follows delivery terminal");
                }
                Request::Deliver { watcher } => {
                    let flight = self.open.get_mut(watcher).expect("open watch");
                    assert!(!*flight, "one delivery in flight");
                    *flight = true;
                    self.deliveries += 1;
                }
                Request::Refused { watcher, .. } => {
                    assert!(!self.open.contains_key(watcher), "refusal opens nothing");
                }
                Request::Ended { watcher, .. } => {
                    assert_eq!(self.open.remove(watcher), Some(false), "terminal follows last delivery");
                    self.ended += 1;
                }
            }
        }
    }
    #[must_use]
    pub fn in_flight(&self, watcher: Token) -> bool {
        self.open.get(&watcher) == Some(&true)
    }
    #[must_use]
    pub fn live(&self) -> usize {
        self.open.len()
    }
}
