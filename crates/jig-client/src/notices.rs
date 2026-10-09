//! Short-lived outcomes with no card, on monotonic deadlines.
//! Notices never decide whether a request was made. The root table's answer
//! remains its one durable answer. See client/domain/client.md, 13.
use crate::{
    Domain,
    contract::{Answer, Limit, ObjectKey},
    domain::Timer,
    link::Link,
};
use skein_lib::{Duration, Id, Map, Slab, Time, Token};
/// Facts the view phrases as a short-lived notice.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// The frame's link went away or came back.
    Link(Link),
    /// A durable answer for a card no longer shown.
    Answer { object: ObjectKey, answer: Answer },
    /// A durable page request answer whose page no longer shows it.
    PageAnswer { answer: Answer },
    /// A refusal with no page or card to show it on.
    Refused(Limit),
}
/// One visible notice and its monotonic life.
#[derive(Debug)]
pub struct Notice {
    pub kind: Kind,
    pub made: Time,
    pub until: Time,
}
/// Bounded notices with retirement room for replacing the oldest in one step.
#[derive(Debug)]
pub struct Notices {
    slab: Slab<Notice>,
    ids: Map<Id<Notice>, ()>,
    life: Duration,
}
impl Notices {
    /// Allocates the configured visible notices and replacement room.
    #[must_use]
    pub fn new(capacity: u32, life: Duration) -> Self {
        Self {
            slab: Slab::with_capacity(capacity.checked_mul(2).expect("notice capacity is validated")),
            ids: Map::with_capacity(capacity),
            life,
        }
    }
    /// Handles of currently visible notices.
    #[must_use]
    pub const fn ids(&self) -> &Map<Id<Notice>, ()> {
        &self.ids
    }
    /// A currently visible notice.
    #[must_use]
    pub fn get(&self, id: Id<Notice>) -> Option<&Notice> {
        self.ids.get(&id)?;
        self.slab.get(id)
    }
    fn oldest(&self) -> Option<Id<Notice>> {
        let mut oldest = None;
        for (id, ()) in &self.ids {
            match oldest {
                Some(previous) => {
                    let made = self.slab.get(*id).expect("registered notice exists").made;
                    let before = self.slab.get(previous).expect("oldest notice exists").made;
                    if made < before {
                        oldest = Some(*id);
                    }
                }
                None => oldest = Some(*id),
            }
        }
        oldest
    }
    fn remove(&mut self, id: Id<Notice>) -> bool {
        if self.ids.remove(&id).is_some() {
            self.slab.retire(id);
            true
        } else {
            false
        }
    }
    /// Frees retired notices at the application's reclaim point.
    pub fn reclaim(&mut self) {
        self.slab.reclaim();
    }
}
/// Shows one bounded notice, replacing the oldest at the visible limit.
pub fn show(domain: &mut Domain, kind: Kind) {
    if domain.notices.ids.capacity() == 0 {
        return;
    }
    if domain.notices.ids.len() == domain.notices.ids.capacity() {
        let old = domain.notices.oldest().expect("full nonempty notice set has an oldest");
        let removed = domain.notices.remove(old);
        assert!(removed, "oldest notice was visible");
        domain.timers.cancel(Timer::Notice(old));
        domain.changed();
    }
    let until = domain.now.saturating_add(domain.notices.life);
    let notice = Notice { kind, made: domain.now, until };
    if let Ok(id) = domain.notices.slab.insert(notice) {
        let old = domain.notices.ids.insert(id, ()).expect("notice admission was checked");
        assert!(old.is_none(), "new notice is not registered");
        domain.timers.arm(Timer::Notice(id), until).expect("one timer per visible notice fits");
        domain.changed();
    }
}
/// Dismisses or expires a visible notice once.
pub fn dismiss(domain: &mut Domain, token: Token) {
    let id = Id::from_token(token);
    if domain.notices.remove(id) {
        domain.timers.cancel(Timer::Notice(id));
        domain.changed();
    }
}
pub(crate) fn dismiss_first(domain: &mut Domain) {
    if let Some(id) = domain.notices.oldest() {
        dismiss(domain, id.token());
    }
}
/// Visible and retired notices and their maximum durable answer payloads.
#[must_use]
pub fn worst_case(limits: &crate::Limits) -> Option<u64> {
    let slots = limits.notices.checked_mul(2)?;
    Slab::<Notice>::worst_case(slots)?
        .checked_add(Map::<Id<Notice>, ()>::worst_case(limits.notices)?)?
        .checked_add(u64::from(slots).checked_mul(limits.document_bytes)?)
}
