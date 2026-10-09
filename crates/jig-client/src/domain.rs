//! The client spine: frame facts, the shared link, page generation and watches.
//! No platform or application value is known. Each entry handles one owned event;
//! fire expires one timer, and reclaim frees only watches whose terminals arrived.
//! Nothing is accepted before Start. See client/domain/client.md, 3, 4 and 8.
//!
//! | Client | Event | Next | Emits |
//! |---|---|---|---|
//! | unstarted | start | starting | frame open |
//! | starting/offline | frame delivery | live | confirmed, retention, link |
//! | live | frame dropped/silent | offline | link, retry/close |
//! | active | unsupported frame version | out of date | park and close |
//! | active | different party from kept work | someone else's work | confirm nobody |
//! | active | frame hand off | unchanged | frame answer |
use crate::{
    action::Act,
    address::{Address, Place},
    boundary::{Event, Offset, Request},
    contract::{CLIENT_VERSION, Delivery, Frame, Limit, Subject, VersionRange, WatchEnd, WatchEvent},
    facts::Fact,
    frame::{FrameShown, FrameState},
    limits::Limits,
    link::Link,
    pages::Page,
    store::Record,
    watches::{Following, Owner, Watch},
};
use skein_lib::{Deadlines, Duration, Env, Id, List, Map, Queue, Rng, Slab, Time, Token, Wall};

/// A watch timer; one timer per watch is armed at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Timer {
    /// No event arrived within the heartbeat.
    Silent(Id<Watch>),
    /// The old attempt ended and backoff permits another open.
    Reopen(Id<Watch>),
}

/// Jig's client state, owned once by the application client root.
#[derive(Debug)]
pub struct Domain {
    pub(crate) started: bool,
    pub(crate) confirmed: Option<u64>,
    pub(crate) text_limit: u32,
    pub(crate) intents: crate::intents::Intents,
    pub(crate) fields: crate::fields::Fields,
    pub(crate) signin: crate::signin::SignIn,
    pub(crate) blocked: bool,
    pub(crate) frame: FrameState,
    pub(crate) link: Link,
    pub(crate) page: Page,
    pub(crate) address: Address,
    pub(crate) generation: u32,
    pub(crate) offset: Offset,
    pub(crate) wall: Wall,
    pub(crate) shown: u64,
    pub(crate) kept: List<Record>,
    pub(crate) watches: Slab<Watch>,
    pub(crate) watch_ids: Map<Id<Watch>, ()>,
    pub(crate) frame_watch: Option<Id<Watch>>,
    pub(crate) timers: Deadlines<Timer>,
    pub(crate) rng: Rng,
    pub(crate) objects: crate::objects::Objects,
    pub(crate) reads: crate::reads::Reads,
    pub(crate) object_rows: crate::windows::ObjectWindow,
    pub(crate) conversation_rows: crate::windows::ConversationWindow,
    pub(crate) turn_rows: crate::windows::TurnWindow,
    pub(crate) history_rows: crate::windows::HistoryWindow,
}
impl Domain {
    /// Allocates bounded state from limits; the seed is the only randomness.
    #[must_use]
    pub fn new(limits: &Limits, seed: u64) -> Self {
        assert!(crate::limits::worst_case(limits).is_some(), "client limits are valid");
        Self {
            started: false,
            confirmed: None,
            text_limit: limits.text,
            intents: crate::intents::Intents::new(limits.intents),
            fields: crate::fields::Fields::new(limits.fields, limits.text),
            signin: crate::signin::SignIn::new(limits.providers),
            blocked: false,
            frame: FrameState { document: None },
            link: Link::Starting,
            page: Page::Starting,
            address: Address::Inbox { project: None },
            generation: 0,
            offset: Offset { minutes: 0 },
            wall: Wall::EPOCH,
            shown: 0,
            kept: List::with_capacity(limits.records),
            watches: Slab::with_capacity(limits.watches),
            watch_ids: Map::with_capacity(limits.watches),
            frame_watch: None,
            timers: Deadlines::with_capacity(limits.watches),
            rng: Rng::new(seed),
            objects: crate::objects::Objects::new(limits.objects, limits.sources),
            reads: crate::reads::Reads::new(limits.reads),
            object_rows: crate::windows::ObjectWindow::new(limits.rows),
            conversation_rows: crate::windows::ConversationWindow::new(limits.rows),
            turn_rows: crate::windows::TurnWindow::new(limits.rows),
            history_rows: crate::windows::HistoryWindow::new(limits.rows),
        }
    }
    /// Installs one provider before start, refusing duplicates and oversized names.
    pub fn provider(&mut self, provider: crate::signin::Provider) -> Result<(), crate::signin::Provider> {
        if self.started
            || u64::try_from(provider.name.bytes().len()).expect("name length fits u64") > u64::from(self.text_limit)
        {
            return Err(provider);
        }
        for kept in &self.signin.providers {
            if kept.number == provider.number {
                return Err(provider);
            }
        }
        self.signin.providers.push(provider)
    }
    /// Provider names for the sign-in page.
    #[must_use]
    pub const fn signin(&self) -> &crate::signin::SignIn {
        &self.signin
    }
    /// Pending intents and any terminal awaiting reclaim.
    #[must_use]
    pub const fn intents(&self) -> &crate::intents::Intents {
        &self.intents
    }
    /// Current fields, with counts of domain writes.
    #[must_use]
    pub const fn fields(&self) -> &crate::fields::Fields {
        &self.fields
    }
    /// Writes a field from domain policy and advances the view's shown count.
    pub fn write_field(&mut self, purpose: crate::fields::Purpose, text: crate::contract::Text) -> Result<(), Limit> {
        self.fields.write(purpose, text)?;
        self.changed();
        Ok(())
    }
    /// This domain's restored and saved records.
    #[must_use]
    pub const fn records(&self) -> &List<Record> {
        &self.kept
    }
    /// What the frame most recently said, without interpreting authority.
    #[must_use]
    pub const fn frame(&self) -> &FrameState {
        &self.frame
    }
    /// The one page the view builds.
    #[must_use]
    pub const fn page(&self) -> &Page {
        &self.page
    }
    /// The link judged from the frame watch alone.
    #[must_use]
    pub const fn link(&self) -> Link {
        self.link
    }
    /// The count of changes affecting the built view.
    #[must_use]
    pub const fn shown(&self) -> u64 {
        self.shown
    }
    /// The display wall clock at the last entry.
    #[must_use]
    pub const fn wall(&self) -> Wall {
        self.wall
    }
    /// The page's objects held once by key.
    #[must_use]
    pub fn objects(&self) -> &crate::objects::Objects {
        &self.objects
    }
    /// Object names in the page's current bounded window.
    #[must_use]
    pub const fn object_rows(&self) -> &crate::windows::ObjectWindow {
        &self.object_rows
    }
    /// Activity including turns in the current window.
    #[must_use]
    pub const fn turn_rows(&self) -> &crate::windows::TurnWindow {
        &self.turn_rows
    }
    /// Conversations in the current bounded window.
    #[must_use]
    pub const fn conversation_rows(&self) -> &crate::windows::ConversationWindow {
        &self.conversation_rows
    }
    /// History in the current bounded window.
    #[must_use]
    pub const fn history_rows(&self) -> &crate::windows::HistoryWindow {
        &self.history_rows
    }
    /// The person's configured display time offset.
    #[must_use]
    pub const fn offset(&self) -> Offset {
        self.offset
    }
    /// The next timer the application's loop should fire.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.timers.next()
    }
    /// Reclaims entities only after their terminals arrived.
    pub fn reclaim(&mut self) {
        // A watch closed during backoff has no outstanding terminal.
        for _ in 0..self.watch_ids.len() {
            let mut finished = None;
            for (id, ()) in &self.watch_ids {
                let watch = self.watches.get(*id).expect("registered watch exists");
                match watch.state {
                    Following::Closed => {
                        finished = Some(*id);
                        break;
                    }
                    Following::Opening { .. } | Following::Live | Following::Reopening { .. } | Following::Closing => {}
                }
            }
            match finished {
                Some(id) => {
                    let removed = self.watch_ids.remove(&id);
                    assert!(removed.is_some(), "closed watch was registered");
                    self.watches.retire(id);
                    if self.frame_watch == Some(id) {
                        self.frame_watch = None;
                    }
                    let count = self.objects.len();
                    let excess = self.objects.replace(crate::objects::Source::Watch(id), &[]);
                    assert!(excess == 0, "empty source replacement never refuses");
                    if self.objects.len() != count {
                        self.changed();
                    }
                }
                None => break,
            }
        }
        self.watches.reclaim();
        self.objects.reclaim();
        self.reads.reclaim();
        self.intents.reclaim();
    }
    pub(crate) fn changed(&mut self) {
        self.shown = self.shown.saturating_add(1);
    }
    pub(crate) fn set_link(&mut self, link: Link, out: &mut Queue<Request>) {
        if self.link != link {
            self.link = link;
            self.changed();
            out.push(Request::Link { link });
        }
    }
    fn arm(&mut self, timer: Timer, at: Time) {
        self.timers.arm(timer, at).expect("one timer per watch fits the timer capacity");
    }
    pub(crate) fn open(
        &mut self,
        env: &Env<Limits>,
        subject: Subject,
        owner: Owner,
        out: &mut Queue<Request>,
    ) -> Option<Id<Watch>> {
        let watch = Watch { subject, owner, state: Following::Opening { attempt: 0 }, behind: false };
        match self.watches.insert(watch) {
            Ok(id) => {
                let previous = self.watch_ids.insert(id, ()).expect("the registry has the slab capacity");
                assert!(previous.is_none(), "a newly issued watch is not already registered");
                let subject = self.watches.get(id).expect("inserted watch exists").subject.clone();
                out.push(Request::Open { watch: id.token(), subject });
                self.arm(Timer::Silent(id), env.now.saturating_add(env.limits.heartbeat));
                Some(id)
            }
            Err(_) => {
                out.push(Request::Fact { fact: Fact::Refused { which: Limit::Watches } });
                None
            }
        }
    }
    fn retire(&mut self, id: Id<Watch>) {
        match self.watches.get_mut(id) {
            Some(watch) => {
                match watch.state {
                    Following::Closed => return,
                    Following::Opening { .. } | Following::Live | Following::Reopening { .. } | Following::Closing => {}
                }
                watch.state = Following::Closed;
            }
            None => return,
        }
        self.timers.cancel(Timer::Silent(id));
        self.timers.cancel(Timer::Reopen(id));
        let removed = self.watch_ids.remove(&id);
        assert!(removed.is_some(), "retired watch was registered");
        self.watches.retire(id);
        let count = self.objects.len();
        let excess = self.objects.replace(crate::objects::Source::Watch(id), &[]);
        assert!(excess == 0, "empty source replacement never refuses");
        if self.objects.len() != count {
            self.changed();
        }
        if self.frame_watch == Some(id) {
            self.frame_watch = None;
        }
    }
    pub(crate) fn close_page(&mut self, out: &mut Queue<Request>) {
        for (id, ()) in &self.watch_ids {
            let watch = self.watches.get_mut(*id).expect("registered watch exists");
            match watch.owner {
                Owner::Frame => {}
                Owner::Page { .. } => close_watch(watch, *id, &mut self.timers, out),
            }
        }
    }
    pub(crate) fn close_all(&mut self, out: &mut Queue<Request>) {
        for (id, ()) in &self.watch_ids {
            let watch = self.watches.get_mut(*id).expect("registered watch exists");
            close_watch(watch, *id, &mut self.timers, out);
        }
    }
    fn page_behind(&mut self) {
        let mut behind = false;
        for (id, ()) in &self.watch_ids {
            let watch = self.watches.get(*id).expect("registered watch exists");
            match watch.owner {
                Owner::Frame => {}
                Owner::Page { generation } => {
                    if generation == self.generation && watch.behind {
                        behind = true;
                    }
                }
            }
        }
        match &mut self.page {
            Page::Work { behind: current, .. } => {
                if *current != behind {
                    *current = behind;
                    self.changed();
                }
            }
            Page::Starting
            | Page::Elsewhere
            | Page::SigningIn
            | Page::Missing
            | Page::OutOfDate
            | Page::SomeoneElse => {}
        }
    }
    pub(crate) fn show_work(&mut self, env: &Env<Limits>, out: &mut Queue<Request>) {
        crate::pages::routing::enter(self, env, out);
    }
}

fn close_watch(watch: &mut Watch, id: Id<Watch>, timers: &mut Deadlines<Timer>, out: &mut Queue<Request>) {
    timers.cancel(Timer::Silent(id));
    timers.cancel(Timer::Reopen(id));
    watch.state = match watch.state {
        Following::Opening { .. } | Following::Live | Following::Reopening { waiting: true, .. } => {
            out.push(Request::Close { watch: id.token() });
            Following::Closing
        }
        Following::Reopening { waiting: false, .. } | Following::Closed => Following::Closed,
        Following::Closing => Following::Closing,
    };
}

/// The maximum effects emitted by any one entry, with all watches closing.
#[must_use]
pub fn max_out(limits: &Limits) -> u32 {
    limits
        .watches
        .saturating_mul(2)
        .saturating_add(limits.objects)
        .saturating_add(limits.parts.saturating_mul(2))
        .saturating_add(16)
}

/// Routes one event, accepting no input until Start.
pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    if !domain.started {
        match event {
            Event::Start { address, kept, offset } => start(domain, env, address, kept, offset, out),
            Event::Went { .. }
            | Event::Act { .. }
            | Event::Ask { .. }
            | Event::Progress { .. }
            | Event::Answered { .. }
            | Event::Read { .. }
            | Event::Watch { .. }
            | Event::Ended { .. }
            | Event::SignedOut
            | Event::Frame { .. } => {}
        }
        return;
    }
    domain.wall = env.wall;
    match event {
        Event::Frame { asked } => frame_answer(domain, asked, out),
        Event::Watch { watch, event } => watch_event(domain, env, Id::from_token(watch), event, out),
        Event::Ended { watch, end } => watch_end(domain, env, Id::from_token(watch), end, out),
        Event::Went { place } => crate::pages::routing::navigate(domain, env, place, true, out),
        Event::Act { act } => match act {
            Act::Go { address } => crate::pages::routing::navigate(domain, env, Place::Jig(address), false, out),
            Act::Edit { field, text } => match domain.fields.edit(field, text) {
                Ok(()) => domain.changed(),
                Err(which) => {
                    crate::pages::routing::refuse(domain, which);
                    out.push(Request::Fact { fact: Fact::Refused { which } });
                }
            },
            Act::SignIn { provider } => crate::signin::choose(domain, provider, out),
            Act::Submit { .. }
            | Act::Intend { .. }
            | Act::Choose { .. }
            | Act::Confirm
            | Act::Dismiss
            | Act::Fold { .. }
            | Act::Filter { .. }
            | Act::More { .. } => {}
        },
        Event::Start { .. } => {}
        Event::Ask { ask } => crate::intents::by_value(domain, ask, out),
        Event::Progress { request, progress } => crate::intents::progress(domain, request, progress),
        Event::Answered { request, answer } => crate::intents::answered(domain, request, answer, out),
        Event::SignedOut => signed_out(domain, out),
        Event::Read { read, result } => crate::pages::routing::read_result(domain, env, read, result, out),
    }
    crate::signin::returned(domain, env, out);
}
fn start(
    domain: &mut Domain,
    env: &Env<Limits>,
    address: Address,
    kept: List<Record>,
    offset: Offset,
    out: &mut Queue<Request>,
) {
    domain.started = true;
    domain.wall = env.wall;
    domain.address = address;
    domain.offset = offset;
    crate::store::restore(domain, env.limits.text, kept, out);
    domain.frame_watch = domain.open(env, Subject::Frame, Owner::Frame, out);
    domain.changed();
}
fn frame_answer(domain: &Domain, asked: Token, out: &mut Queue<Request>) {
    let party = domain.frame.party();
    let address = match domain.page {
        Page::Work { address, .. } => Some(address),
        Page::Starting | Page::Elsewhere | Page::SigningIn | Page::Missing | Page::OutOfDate | Page::SomeoneElse => {
            None
        }
    };
    out.push(Request::FrameIs { asked, frame: FrameShown { party, address, link: domain.link } });
}

fn accepts(state: Following) -> bool {
    match state {
        Following::Opening { .. } | Following::Live => true,
        Following::Reopening { .. } | Following::Closing | Following::Closed => false,
    }
}
fn watch_event(domain: &mut Domain, env: &Env<Limits>, id: Id<Watch>, event: WatchEvent, out: &mut Queue<Request>) {
    let Some(watch) = domain.watches.get_mut(id) else {
        return;
    };
    if domain.blocked || !accepts(watch.state) {
        return;
    }
    let owner = watch.owner;
    match event {
        WatchEvent::Opened | WatchEvent::Alive => {}
        WatchEvent::Delivered(delivery) => {
            watch.state = Following::Live;
            watch.behind = false;
            match owner {
                Owner::Frame => match delivery {
                    Delivery::Frame(frame) => frame_delivery(domain, env, *frame, out),
                    Delivery::Task(_)
                    | Delivery::Tree(_)
                    | Delivery::Chips(_)
                    | Delivery::Inbox(_)
                    | Delivery::Goals(_)
                    | Delivery::Conversations(_)
                    | Delivery::Run(_)
                    | Delivery::Deployment(_) => {}
                },
                Owner::Page { generation } => {
                    if generation == domain.generation {
                        crate::pages::routing::delivery(domain, env, id, delivery, out);
                    }
                }
            }
            domain.page_behind();
        }
    }
    if !domain.blocked {
        domain.arm(Timer::Silent(id), env.now.saturating_add(env.limits.heartbeat));
    }
}
fn frame_delivery(domain: &mut Domain, env: &Env<Limits>, frame: Frame, out: &mut Queue<Request>) {
    let range: VersionRange = frame.versions;
    if !range.contains(CLIENT_VERSION) {
        domain.blocked = true;
        domain.confirmed = None;
        domain.page = Page::OutOfDate;
        domain.frame.document = Some(frame);
        domain.close_all(out);
        domain.set_link(Link::Offline, out);
        out.push(Request::Confirmed { party: None });
        domain.changed();
        return;
    }
    domain.signin.service = match frame.party.kind {
        crate::contract::PartyKind::Service => true,
        crate::contract::PartyKind::Person => false,
    };
    let first = domain.frame.document.is_none();
    let party = frame.party.number;
    let mut mismatch = false;
    for record in &domain.kept {
        let owner = match record {
            Record::Place { party, .. } | Record::Draft { party, .. } => *party,
        };
        if owner != party {
            mismatch = true;
        }
    }
    let previous = domain.frame.party();
    let retention = frame.retention;
    domain.frame.document = Some(frame);
    domain.confirmed = if mismatch { None } else { Some(party) };
    domain.changed();
    if first || previous != Some(party) || mismatch {
        let confirmed = if mismatch { None } else { Some(party) };
        out.push(Request::Confirmed { party: confirmed });
    }
    out.push(Request::Retention { retention });
    domain.set_link(Link::Live, out);
    if mismatch {
        domain.close_page(out);
        domain.page = Page::SomeoneElse;
        domain.changed();
    } else {
        match domain.page {
            Page::Starting | Page::SomeoneElse | Page::SigningIn => domain.show_work(env, out),
            Page::Work { .. } | Page::Elsewhere | Page::Missing | Page::OutOfDate => {}
        }
    }
}
fn backoff(rng: &mut Rng, limits: &Limits, attempt: u32) -> Duration {
    let shift = attempt.saturating_sub(1).min(31);
    let factor = 1_u64.checked_shl(shift).expect("backoff shift is bounded");
    let cap = limits.backoff.first.saturating_mul(factor).as_nanos().min(limits.backoff.most.as_nanos());
    let jitter = cap.checked_div(4).expect("four is positive");
    let upper = cap.saturating_add(jitter).min(limits.backoff.most.as_nanos());
    Duration::from_nanos(rng.between(cap, upper))
}
fn retry(domain: &mut Domain, env: &Env<Limits>, id: Id<Watch>, out: &mut Queue<Request>) {
    let Some(watch) = domain.watches.get_mut(id) else {
        return;
    };
    let attempt = match watch.state {
        Following::Opening { attempt } | Following::Reopening { attempt, .. } => attempt.saturating_add(1),
        Following::Live => 1,
        Following::Closing | Following::Closed => {
            domain.retire(id);
            return;
        }
    };
    watch.state = Following::Reopening { attempt, waiting: false };
    watch.behind = true;
    let frame = match watch.owner {
        Owner::Frame => true,
        Owner::Page { .. } => false,
    };
    domain.timers.cancel(Timer::Silent(id));
    let span = backoff(&mut domain.rng, &env.limits, attempt);
    domain.arm(Timer::Reopen(id), env.now.saturating_add(span));
    if frame {
        domain.set_link(Link::Offline, out);
    }
    domain.page_behind();
}
fn watch_end(domain: &mut Domain, env: &Env<Limits>, id: Id<Watch>, end: WatchEnd, out: &mut Queue<Request>) {
    let Some(watch) = domain.watches.get(id) else {
        return;
    };
    let owner = watch.owner;
    let recovering = match watch.state {
        Following::Reopening { waiting: true, .. } => true,
        Following::Reopening { waiting: false, .. } => return,
        Following::Opening { .. } | Following::Live | Following::Closing | Following::Closed => false,
    };
    match watch.state {
        Following::Closed => return,
        Following::Closing => {
            domain.retire(id);
            crate::signin::returned(domain, env, out);
            return;
        }
        Following::Opening { .. } | Following::Live | Following::Reopening { .. } => {}
    }
    match end {
        WatchEnd::Dropped | WatchEnd::Busy | WatchEnd::Stopping => {
            retry(domain, env, id, out);
        }
        WatchEnd::Closed => {
            if recovering {
                retry(domain, env, id, out);
            } else {
                domain.retire(id);
            }
        }
        WatchEnd::Unknown | WatchEnd::NotAllowed | WatchEnd::SubjectEnded => {
            domain.retire(id);
            match owner {
                Owner::Frame => domain.set_link(Link::Offline, out),
                Owner::Page { .. } => {}
            }
        }
        WatchEnd::SignedOut => {
            domain.retire(id);
            signed_out(domain, out);
        }
    }
    crate::signin::returned(domain, env, out);
}
pub(crate) fn signed_out(domain: &mut Domain, out: &mut Queue<Request>) {
    crate::signin::signed_out(domain, out);
}
/// Fires at most one expired deadline, respecting watch terminal obligations.
pub fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    domain.wall = env.wall;
    let Some(timer) = domain.timers.expire(env.now) else {
        return;
    };
    match timer {
        Timer::Silent(id) => {
            let Some(watch) = domain.watches.get_mut(id) else {
                return;
            };
            if !accepts(watch.state) {
                return;
            }
            let attempt = match watch.state {
                Following::Opening { attempt } => attempt,
                Following::Live => 0,
                Following::Reopening { .. } | Following::Closing | Following::Closed => return,
            };
            watch.state = Following::Reopening { attempt, waiting: true };
            watch.behind = true;
            let frame = match watch.owner {
                Owner::Frame => true,
                Owner::Page { .. } => false,
            };
            out.push(Request::Close { watch: id.token() });
            if frame {
                domain.set_link(Link::Offline, out);
            }
            domain.page_behind();
        }
        Timer::Reopen(id) => {
            let Some(watch) = domain.watches.get_mut(id) else {
                return;
            };
            match watch.state {
                Following::Reopening { attempt, waiting: false } => {
                    watch.state = Following::Opening { attempt };
                    let subject = watch.subject.clone();
                    out.push(Request::Open { watch: id.token(), subject });
                    domain.arm(Timer::Silent(id), env.now.saturating_add(env.limits.heartbeat));
                }
                Following::Opening { .. }
                | Following::Live
                | Following::Reopening { waiting: true, .. }
                | Following::Closing
                | Following::Closed => {}
            }
        }
    }
}
/// The next deadline of the client, for roots using free entry points.
#[must_use]
pub fn next_deadline(domain: &Domain) -> Option<Time> {
    domain.next_deadline()
}
/// Frees retired slots at the loop's reclaim point.
pub fn reclaim(domain: &mut Domain) {
    domain.reclaim();
}
