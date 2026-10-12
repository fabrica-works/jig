//! Refusals ride with the next turn or answer (hosts.md, 6.4).
use crate::hosted::NamedEvent;
use crate::{MessageRefusal, Refused};
use alloc::boxed::Box;
use skein_lib::{List, Queue};

#[derive(Debug)]
pub(crate) struct Refusals {
    kept: List<Refused>,
    pub(crate) lost: u64,
}
impl Refusals {
    pub(crate) fn new(inbox: u32) -> Self {
        Self { kept: List::with_capacity(inbox), lost: 0 }
    }
    pub(crate) fn take(&mut self) -> Box<[Refused]> {
        let capacity = self.kept.capacity();
        let kept = core::mem::replace(&mut self.kept, List::with_capacity(capacity));
        kept.into_boxed()
    }
}
pub(crate) fn keep(refusals: &mut Refusals, refused: Refused) {
    if refusals.kept.push(refused).is_err() {
        refusals.lost = refusals.lost.saturating_add(1);
    }
}
pub(crate) fn hold(held: &mut Queue<NamedEvent>, event: NamedEvent, refusals: &mut Refusals) {
    let name = event.name;
    if held.try_push(event).is_err() {
        keep(refusals, Refused { name, reason: MessageRefusal::Full });
    }
}
pub(crate) fn ending(mut held: Queue<NamedEvent>, refusals: &mut Refusals) {
    for _ in 0..held.len() {
        let event = held.pop().expect("the queue held this many messages");
        keep(refusals, Refused { name: event.name, reason: MessageRefusal::Ending });
    }
}
