//! The client spine: frame facts, the shared link, page generation and watches.
//! No platform or application value is known. Each entry handles one owned event;
//! fire expires one timer, and reclaim frees entities whose obligations ended.
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

/// Client deadlines: one per watch, notice, linger and saved record, plus a tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Timer {
    /// No event arrived within the heartbeat.
    Silent(Id<Watch>),
    /// The old attempt ended and backoff permits another open.
    Reopen(Id<Watch>),
    /// A visible notice reaches its monotonic end.
    Notice(Id<crate::notices::Notice>),
    /// A leaving card has lingered.
    Linger(Id<crate::objects::Object>),
    /// A draft save has coalesced.
    Save(crate::store::StoreKey),
    /// A shown wall time needs another minute's wording.
    Minute,
}

/// Jig's client state, owned once by the application client root.
#[derive(Debug)]
pub struct Domain {
    pub(crate) started: bool,
    pub(crate) drafts: crate::drafts::Drafts,
    pub(crate) confirming: Option<crate::confirm::Confirmation>,
    pub(crate) now: Time,
    pub(crate) clock: crate::clock::State,
    pub(crate) notices: crate::notices::Notices,
    pub(crate) linger: Duration,
    pub(crate) save_delay: Duration,
    pub(crate) confirmed: Option<u64>,
    pub(crate) text_limit: u32,
    pub(crate) intents: crate::intents::Intents,
    pub(crate) fields: crate::fields::Fields,
    pub(crate) signin: crate::signin::SignIn,
    pub(crate) blocked: bool,
    pub(crate) frame: FrameState,
    pub(crate) link: Link,
    pub(crate) page: Page,
    pub(crate) inbox: crate::pages::inbox::Inbox,
    pub(crate) task_page: crate::pages::task::TaskPage,
    pub(crate) chats: crate::pages::conversations::State,
    pub(crate) navigation: crate::pages::navigation::Navigation,
    pub(crate) updates_count: u32,
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
    pub(crate) conversation: crate::conversation::Conversation,
    pub(crate) history_rows: crate::windows::HistoryWindow,
}
impl Domain {
    /// Allocates bounded state from limits; the seed is the only randomness.
    #[must_use]
    pub fn new(limits: &Limits, seed: u64) -> Self {
        assert!(crate::limits::worst_case(limits).is_some(), "client limits are valid");
        Self {
            started: false,
            drafts: crate::drafts::Drafts::new(limits.drafts),
            confirming: None,
            now: Time::ZERO,
            clock: crate::clock::State { page_times: false, armed: false },
            notices: crate::notices::Notices::new(limits.notices, limits.notice_life),
            linger: limits.linger,
            save_delay: limits.save_delay,
            confirmed: None,
            text_limit: limits.text,
            intents: crate::intents::Intents::new(limits.intents),
            fields: crate::fields::Fields::new(limits.fields, limits.text),
            signin: crate::signin::SignIn::new(limits.providers),
            blocked: false,
            frame: FrameState { document: None },
            link: Link::Starting,
            page: Page::Starting,
            inbox: crate::pages::inbox::Inbox::new(limits.rows),
            task_page: crate::pages::task::TaskPage::new(limits.rows),
            chats: crate::pages::conversations::State::new(),
            navigation: crate::pages::navigation::Navigation::new(limits.navigation_entries),
            updates_count: limits.updates_count,
            address: Address::Inbox { project: None },
            generation: 0,
            offset: Offset { minutes: 0 },
            wall: Wall::EPOCH,
            shown: 0,
            kept: List::with_capacity(limits.records),
            watches: Slab::with_capacity(limits.watches),
            watch_ids: Map::with_capacity(limits.watches),
            frame_watch: None,
            timers: Deadlines::with_capacity(crate::limits::timers(limits).expect("timer capacity is validated")),
            rng: Rng::new(seed),
            objects: crate::objects::Objects::new(limits.objects, limits.sources),
            reads: crate::reads::Reads::new(limits.reads),
            object_rows: crate::windows::ObjectWindow::new(limits.rows),
            conversation_rows: crate::windows::ConversationWindow::new(limits.rows),
            conversation: crate::conversation::Conversation::new(0, limits.rows),
            history_rows: crate::windows::HistoryWindow::new(limits.rows),
        }
    }
    /// Installs ordered navigation and its home before start; capacities are checked.
    pub fn configure_navigation(
        &mut self,
        navigation: crate::pages::navigation::Navigation,
    ) -> Result<(), crate::pages::navigation::Navigation> {
        if self.started || navigation.entries.capacity() > self.navigation.entries.capacity() {
            return Err(navigation);
        }
        self.navigation = navigation;
        Ok(())
    }
    /// Inbox context, filters and historical cursors for its shared cards.
    #[must_use]
    pub const fn inbox(&self) -> &crate::pages::inbox::Inbox {
        &self.inbox
    }
    /// The task header and its bounded plan and history choices.
    #[must_use]
    pub const fn task_page(&self) -> &crate::pages::task::TaskPage {
        &self.task_page
    }
    /// The new-chat form's request and factual refusal.
    #[must_use]
    pub const fn chats(&self) -> &crate::pages::conversations::State {
        &self.chats
    }
    /// Configured destinations, in the application's order.
    #[must_use]
    pub const fn navigation(&self) -> &crate::pages::navigation::Navigation {
        &self.navigation
    }
    /// Counts and current identity shown on every page, including application pages.
    #[must_use]
    pub fn frame_summary(&self) -> crate::frame::Summary {
        let mut summary = self.frame.summary(self.address, self.link, self.page_behind_value(), self.updates_count);
        match self.address {
            Address::Task { task, .. } => {
                if let Some(id) = self.objects.find(crate::contract::ObjectKey::Task(task))
                    && let Some(object) = self.objects.get(id)
                {
                    match &object.body {
                        Some(crate::objects::Body::Task(task)) => summary.project = Some(task.project),
                        Some(
                            crate::objects::Body::Chip(_)
                            | crate::objects::Body::Proposal(_)
                            | crate::objects::Body::Escalation(_)
                            | crate::objects::Body::Question(_)
                            | crate::objects::Body::PersonTask(_)
                            | crate::objects::Body::Result(_)
                            | crate::objects::Body::Note { .. },
                        )
                        | None => {}
                    }
                }
            }
            Address::Inbox { .. } | Address::Conversations { .. } | Address::Project { .. } | Address::Deployment => {}
        }
        summary
    }
    fn page_behind_value(&self) -> bool {
        match self.page {
            Page::Work { behind, .. } => behind,
            Page::Starting
            | Page::Elsewhere
            | Page::SigningIn
            | Page::Missing
            | Page::OutOfDate
            | Page::SomeoneElse => false,
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
    /// Visible notices and their monotonic ends.
    #[must_use]
    pub const fn notices(&self) -> &crate::notices::Notices {
        &self.notices
    }
    /// Page policy names whether its own documents show wall times.
    pub fn timed_page(&mut self, showing: bool) {
        self.clock.page_times = showing;
        crate::clock::sync(self);
    }
    /// An object policy leaves a card visible for the configured linger.
    pub fn leave(&mut self, id: Id<crate::objects::Object>, ending: crate::contract::Ending) -> bool {
        let Some(object) = self.objects.get_mut(id) else {
            return false;
        };
        let until = self.now.saturating_add(self.linger);
        object.card = crate::objects::Card::Leaving { how: crate::objects::Left::Ended { how: ending }, until };
        self.timers.arm(Timer::Linger(id), until).expect("one linger deadline per object fits");
        self.changed();
        true
    }
    /// Keeps the durable outcome on a card until its configured linger ends.
    pub fn leave_answer(&mut self, id: Id<crate::objects::Object>, outcome: crate::contract::Outcome) -> bool {
        let Some(object) = self.objects.get_mut(id) else {
            return false;
        };
        let until = self.now.saturating_add(self.linger);
        object.card = crate::objects::Card::Leaving { how: crate::objects::Left::Answered { outcome }, until };
        self.timers.arm(Timer::Linger(id), until).expect("one linger deadline per object fits");
        self.changed();
        true
    }
    /// The one object confirmation, including its observed revision and choices.
    #[must_use]
    pub const fn confirmation(&self) -> Option<&crate::confirm::Confirmation> {
        self.confirming.as_ref()
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
    /// Current-party drafts, including their saved touch order.
    #[must_use]
    pub const fn drafts(&self) -> &crate::drafts::Drafts {
        &self.drafts
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
    /// The current task's activity, provisional turns and page-owned folds.
    #[must_use]
    pub const fn conversation(&self) -> &crate::conversation::Conversation {
        &self.conversation
    }
    /// Activity including turns in the current window.
    #[must_use]
    pub const fn turn_rows(&self) -> &crate::windows::TurnWindow {
        &self.conversation.lines
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
        self.notices.reclaim();
        crate::clock::sync(self);
    }
    pub(crate) fn changed(&mut self) {
        self.shown = self.shown.saturating_add(1);
    }
    pub(crate) fn set_link(&mut self, link: Link, out: &mut Queue<Request>) {
        if self.link != link {
            let previous = self.link;
            self.link = link;
            match previous {
                Link::Starting => {}
                Link::Live | Link::Offline => crate::notices::show(self, crate::notices::Kind::Link(link)),
            }
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
        .saturating_add(limits.records)
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
    domain.now = env.now;
    domain.wall = env.wall;
    match event {
        Event::Frame { asked } => frame_answer(domain, asked, out),
        Event::Watch { watch, event } => watch_event(domain, env, Id::from_token(watch), event, out),
        Event::Ended { watch, end } => watch_end(domain, env, Id::from_token(watch), end, out),
        Event::Went { place } => {
            crate::confirm::close(domain);
            crate::pages::routing::navigate(domain, env, place, true, out);
        }
        Event::Act { act } => act_event(domain, env, act, out),
        Event::Start { .. } => {}
        Event::Ask { ask } => crate::intents::by_value(domain, ask, out),
        Event::Progress { request, progress } => crate::intents::progress(domain, request, progress, out),
        Event::Answered { request, answer } => crate::intents::answered(domain, env, request, answer, out),
        Event::SignedOut => signed_out(domain, out),
        Event::Read { read, result } => {
            if crate::objects::is_card_read(domain, read) {
                crate::objects::read_card(domain, read, result, out);
            } else {
                crate::pages::routing::read_result(domain, env, read, result, out);
            }
        }
    }
    crate::drafts::sync(domain, out);
    crate::confirm::sync(domain);
    crate::signin::returned(domain, env, out);
    crate::clock::sync(domain);
}
fn act_event(domain: &mut Domain, env: &Env<Limits>, act: Act, out: &mut Queue<Request>) {
    match act {
        Act::Go { address } => {
            crate::confirm::close(domain);
            crate::pages::routing::navigate(domain, env, Place::Jig(address), false, out);
        }
        Act::Edit { field: crate::fields::Purpose::Composer { task }, text } => {
            if let Err(which) = crate::drafts::edit(domain, task, text, out) {
                crate::pages::routing::refuse(domain, which);
                out.push(Request::Fact { fact: Fact::Refused { which } });
            }
        }
        Act::Edit { field, text } => match domain.fields.edit(field, text) {
            Ok(()) => domain.changed(),
            Err(which) => {
                crate::pages::routing::refuse(domain, which);
                out.push(Request::Fact { fact: Fact::Refused { which } });
            }
        },
        Act::Dismiss => {
            if domain.confirming.is_some() {
                crate::confirm::close(domain);
            } else {
                crate::notices::dismiss_first(domain);
            }
        }
        Act::SignIn { provider } => crate::signin::choose(domain, provider, out),
        Act::StartChat { project } => crate::pages::conversations::start(domain, env, project, out),
        Act::OpenChat { task } => crate::pages::conversations::open_chat(domain, env, task, out),
        Act::OpenUpdates => crate::pages::conversations::open_updates(domain, out),
        Act::FilterInbox { filter } => crate::pages::conversations::filter_inbox(domain, filter, out),
        Act::Filter { group, project } => crate::pages::conversations::filter_inbox(
            domain,
            crate::pages::inbox::Filter { project, kind: Some(group), goal: None },
            out,
        ),
        Act::FoldCall { name, open } => crate::conversation::routing::fold(domain, name, open),
        Act::More { window, end } => {
            crate::conversation::routing::more(domain, window, end, out);
            crate::pages::conversations::more(domain, window, end, out);
            crate::pages::task::more(domain, window, end, out);
        }
        Act::FilterPlan { filter } => crate::pages::task::filter(domain, filter),
        Act::HistoryScope { subtree } => crate::pages::task::history(domain, subtree, out),
        Act::OpenEnded { task } => crate::pages::task::open_ended(domain, task, out),
        Act::Submit { field: crate::fields::Purpose::NewChat } => {
            let project = match domain.address {
                Address::Conversations { project } | Address::Inbox { project } => project,
                Address::Project { project, .. } => Some(project),
                Address::Task { .. } | Address::Deployment => None,
            };
            if let Some(project) = project {
                crate::pages::conversations::start(domain, env, project, out);
            }
        }
        Act::Submit { field: crate::fields::Purpose::Composer { task } } => {
            if let Some(id) = domain.objects.find(crate::contract::ObjectKey::Task(task)) {
                crate::confirm::open(domain, env, id, crate::contract::ActKind::Write);
            }
        }
        Act::WriteRelease { release } => crate::confirm::write_release(domain, release),
        Act::Priority { priority } => crate::confirm::priority(domain, priority),
        Act::ChatAbout { object } => {
            if let Some(object) = domain.objects.get(Id::from_token(object)) {
                let key = object.key;
                crate::confirm::close(domain);
                crate::pages::conversations::about(domain, env, key, out);
            }
        }
        Act::Intend { object, act } => crate::confirm::open(domain, env, Id::from_token(object), act),
        Act::Choose { option } => crate::confirm::choose(domain, option),
        Act::CannotDo => crate::confirm::cannot(domain),
        Act::Again { again } => crate::confirm::again(domain, again),
        Act::Dispose { task, take_over } => crate::confirm::dispose(domain, task, take_over),
        Act::Change { change } => crate::confirm::change(domain, *change),
        Act::Confirm => crate::confirm::send(domain, out),
        Act::LookAgain { object } => crate::objects::answer::look_again(domain, Id::from_token(object), out),
        Act::Submit { .. } | Act::Fold { .. } => {}
    }
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
    domain.now = env.now;
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
                        let contexts = crate::objects::contexts(&delivery, env.limits.rows);
                        crate::objects::dropping(domain, id, &delivery);
                        crate::pages::routing::delivery(domain, env, id, delivery, out);
                        crate::objects::apply_contexts(domain, contexts);
                        crate::objects::read_dropped(domain, out);
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
    if !crate::frame::fits(&frame, &env.limits) {
        out.push(Request::Fact { fact: Fact::Refused { which: Limit::Objects } });
        return;
    }
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
    match &watch.subject {
        Subject::Run { task, .. } => {
            if *task == domain.conversation.task {
                domain.conversation.interrupted();
            }
        }
        Subject::Frame
        | Subject::Task { .. }
        | Subject::Tree { .. }
        | Subject::Chips { .. }
        | Subject::Inbox { .. }
        | Subject::Goals { .. }
        | Subject::Conversations
        | Subject::Deployment => {}
    }
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
            let task_subject = match &watch.subject {
                Subject::Task { .. } => true,
                Subject::Frame
                | Subject::Tree { .. }
                | Subject::Chips { .. }
                | Subject::Inbox { .. }
                | Subject::Goals { .. }
                | Subject::Conversations
                | Subject::Run { .. }
                | Subject::Deployment => false,
            };
            domain.retire(id);
            match owner {
                Owner::Frame => domain.set_link(Link::Offline, out),
                Owner::Page { generation } => {
                    if generation == domain.generation && task_subject {
                        match domain.address {
                            Address::Task { .. } => match end {
                                WatchEnd::Unknown | WatchEnd::NotAllowed => {
                                    domain.page = Page::Missing;
                                    domain.changed();
                                }
                                WatchEnd::SubjectEnded
                                | WatchEnd::Closed
                                | WatchEnd::Dropped
                                | WatchEnd::Busy
                                | WatchEnd::Stopping
                                | WatchEnd::SignedOut => {}
                            },
                            Address::Inbox { .. }
                            | Address::Conversations { .. }
                            | Address::Project { .. }
                            | Address::Deployment => {}
                        }
                    }
                }
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
    domain.now = env.now;
    domain.wall = env.wall;
    let Some(timer) = domain.timers.expire(env.now) else {
        return;
    };
    match timer {
        Timer::Minute => crate::clock::minute(domain),
        Timer::Notice(id) => crate::notices::dismiss(domain, id.token()),
        Timer::Save(key) => crate::store::fire_save(domain, key, out),
        Timer::Linger(id) => {
            if let Some(object) = domain.objects.get(id) {
                match object.card {
                    crate::objects::Card::Leaving { until, .. } => {
                        if until <= env.now {
                            let key = object.key;
                            domain.objects.remove(key);
                            domain.changed();
                        }
                    }
                    crate::objects::Card::Open
                    | crate::objects::Card::Deciding { .. }
                    | crate::objects::Card::Refused { .. }
                    | crate::objects::Card::Unknown { .. } => {}
                }
            }
        }
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
    crate::clock::sync(domain);
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
