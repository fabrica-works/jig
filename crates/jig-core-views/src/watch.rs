//! One document request or delivery in flight per watch (`domain/engine.md`, 11).
//! Changes coalesce into one mark. A change during construction also marks the
//! watch: a document already being made may reflect the preceding state.
//! Closing waits for construction and delivery terminals before releasing the
//! parent's watcher token. Documents themselves never enter this child.
//!
//! ```text
//! state       event         next             emits
//! absent      watch         Making           Watching, Make
//! Idle        changed       Making           Make
//! Making      changed       Making(changed)  none
//! Making      made ready    Sending          Deliver
//! Sending     changed       Sending(changed) none
//! Sending     delivered     Idle             none
//! Sending(*)  delivered     Making           Make
//! Idle        close         closed           Ended
//! Making      close         ClosingMake      none
//! ClosingMake made          closed           Ended
//! Sending     close         ClosingDelivery  none
//! ClosingDelivery delivered closed           Ended, or wait for stream
//! ClosingStream stream taken closed           Ended
//! ```
use crate::{Domain, End, Fact, Follow, Limits, Made, Refusal, Request, Window};
use skein_lib::{Id, List, Queue, Token};

#[derive(Debug)]
pub(crate) struct Watcher {
    pub(crate) token: Token,
    party: Token,
    subject: Token,
    window: Window,
    pub(crate) state: State,
    pub(crate) follow: Option<Follow>,
    pub(crate) flight: Option<crate::text::Flight>,
    pub(crate) revision: Option<u64>,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum State {
    Idle,
    Making { changed: bool },
    Sending { changed: bool },
    ClosingMake { end: End },
    ClosingStream { end: End },
    ClosingDelivery { end: End },
}

#[expect(clippy::too_many_arguments, reason = "one admitted watch carries its optional stream binding")]
pub(crate) fn open(
    domain: &mut Domain,
    limits: &Limits,
    token: Token,
    party: Token,
    subject: Token,
    window: Window,
    follow: Option<Follow>,
    out: &mut Queue<Request>,
) {
    assert!(!domain.names.contains_key(&token), "unique open watcher tokens");
    let mut count = 0_u32;
    for (_, id) in &domain.names {
        if domain.watchers.get(*id).expect("named watch").party == party {
            count = count.checked_add(1).expect("bounded watches");
        }
    }
    if domain.names.len() >= limits.watchers
        || count >= limits.watchers_per_party
        || domain.watchers.is_full()
        || crate::text::full(domain, limits, follow)
    {
        domain.facts.push(Fact::Refused { refusal: Refusal::Busy });
        out.push(Request::Refused { watcher: token, refusal: Refusal::Busy });
        return;
    }
    crate::text::attach(domain, limits, follow);
    let watch = Watcher {
        token,
        party,
        subject,
        window,
        state: State::Making { changed: false },
        follow,
        flight: None,
        revision: None,
    };
    let id = domain.watchers.insert(watch).expect("admission checked");
    domain.names.insert(token, id).expect("one name per watch");
    domain.facts.push(Fact::Watching);
    out.push(Request::Watching { watcher: token });
    make(domain.watchers.get(id).expect("inserted"), &mut domain.facts, out);
    crate::text::offer_one(domain, id, out);
}
fn make(watch: &Watcher, facts: &mut crate::facts::Facts, out: &mut Queue<Request>) {
    facts.push(Fact::Making);
    out.push(Request::Make { watcher: watch.token, subject: watch.subject, window: watch.window });
}
pub(crate) fn changed(domain: &mut Domain, limits: &Limits, subjects: &List<Token>, out: &mut Queue<Request>) {
    assert!(subjects.len() <= limits.subjects_per_change, "parent bounds change subjects");
    let mut count = 0_u32;
    for (_, id) in &domain.names {
        let watch = domain.watchers.get_mut(*id).expect("named watch");
        if !subjects.as_slice().contains(&watch.subject) {
            continue;
        }
        count = count.checked_add(1).expect("bounded watches");
        watch.state = match watch.state {
            State::Idle => {
                make(watch, &mut domain.facts, out);
                State::Making { changed: false }
            }
            State::Making { .. } => State::Making { changed: true },
            State::Sending { .. } => State::Sending { changed: true },
            State::ClosingMake { end } => State::ClosingMake { end },
            State::ClosingStream { end } => State::ClosingStream { end },
            State::ClosingDelivery { end } => State::ClosingDelivery { end },
        };
    }
    domain.facts.push(Fact::Changed { watchers: count });
}
pub(crate) fn made(domain: &mut Domain, token: Token, outcome: Made, out: &mut Queue<Request>) {
    let Some(&id) = domain.names.get(&token) else {
        return;
    };
    let watch = domain.watchers.get_mut(id).expect("named watch");
    match watch.state {
        State::Making { changed } => match outcome {
            Made::Ready => {
                watch.state = State::Sending { changed };
                domain.facts.push(Fact::Delivered);
                out.push(Request::Deliver { watcher: token });
            }
            Made::Gone { why } => finish(domain, id, why, out),
        },
        State::ClosingMake { end } => finish(domain, id, end, out),
        State::Idle | State::Sending { .. } | State::ClosingDelivery { .. } | State::ClosingStream { .. } => {}
    }
}
pub(crate) fn delivered(domain: &mut Domain, token: Token, out: &mut Queue<Request>) {
    let Some(&id) = domain.names.get(&token) else {
        return;
    };
    let watch = domain.watchers.get_mut(id).expect("named watch");
    match watch.state {
        State::Sending { changed } => {
            watch.state = if changed {
                make(watch, &mut domain.facts, out);
                State::Making { changed: false }
            } else {
                State::Idle
            };
        }
        State::ClosingDelivery { end } => finish(domain, id, end, out),
        State::Idle | State::Making { .. } | State::ClosingMake { .. } | State::ClosingStream { .. } => {}
    }
}
pub(crate) fn close(domain: &mut Domain, token: Token, why: End, out: &mut Queue<Request>) {
    let Some(&id) = domain.names.get(&token) else {
        return;
    };
    let watch = domain.watchers.get_mut(id).expect("named watch");
    match watch.state {
        State::Idle => finish(domain, id, why, out),
        State::Making { .. } => watch.state = State::ClosingMake { end: why },
        State::Sending { .. } => watch.state = State::ClosingDelivery { end: why },
        State::ClosingMake { .. } | State::ClosingDelivery { .. } | State::ClosingStream { .. } => {}
    }
}
pub(crate) fn gone(domain: &mut Domain, subject: Token, why: End, out: &mut Queue<Request>) {
    let mut names = List::with_capacity(domain.names.len());
    for (token, id) in &domain.names {
        if domain.watchers.get(*id).expect("named watch").subject == subject {
            names.push(*token).expect("bounded names");
        }
    }
    for token in &names {
        close(domain, *token, why, out);
    }
}
pub(crate) fn finish(domain: &mut Domain, id: Id<Watcher>, end: End, out: &mut Queue<Request>) {
    let watch = domain.watchers.get_mut(id).expect("ending watch");
    if watch.flight.is_some() {
        watch.state = State::ClosingStream { end };
        return;
    }
    let token = watch.token;
    let follow = watch.follow;
    crate::text::detach(domain, follow);
    domain.names.remove(&token);
    domain.watchers.retire(id);
    domain.facts.push(Fact::Ended { end });
    out.push(Request::Ended { watcher: token, end });
}
