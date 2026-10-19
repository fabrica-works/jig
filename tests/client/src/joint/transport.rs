//! Protocol correlation for the real Front (testing.md, 5; root.md, 4).
//! These routes retain original shell tokens and authenticated session numbers.
//! They do not inspect engine tasks, choose outcomes, or own request durability.
use crate::harness::ClientId;
use jig_client_domain::contract::{Read, Subject, VersionRange};
use jig_client_test_domain::family;
use jig_domain_core::Event;
use jig_domain_people as people;
use jig_domain_people::ApplicationScope;
use skein_lib::{ReplyTo, Token};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pace {
    Document,
    Stream,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidContinuation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyedOwner {
    Jig,
    Application { connector: u16 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    SignIn { client: ClientId },
    SignOut { client: ClientId, sign_in: u64 },
    Keyed { client: ClientId, attempt: Token, key: [u8; 16], owner: KeyedOwner },
    Read { client: ClientId, read: Token, what: Read },
    ApplicationRead { client: ClientId, read: Token, subject: family::Subject },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Watched {
    Jig(Subject),
    Application { subject: family::Subject, scope: ApplicationScope },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WatchOwner {
    Jig,
    Application,
}
impl Watched {
    fn owner(&self) -> WatchOwner {
        match self {
            Self::Jig(_) => WatchOwner::Jig,
            Self::Application { .. } => WatchOwner::Application,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Watch {
    pub client: ClientId,
    pub native: Token,
    pub subject: Watched,
}

#[derive(Clone, Debug)]
struct Client {
    party: u64,
    identity: people::Identity,
    sign_in: Option<u64>,
}

/// One transport's globally unique reply/watch namespace, shared across tabs.
pub struct Transport {
    versions: VersionRange,
    next: u64,
    clients: BTreeMap<ClientId, Client>,
    replies: BTreeMap<Token, Reply>,
    watches: BTreeMap<Token, Watch>,
    closing: BTreeSet<Token>,
    documents: BTreeSet<Token>,
    streams: BTreeSet<Token>,
}

impl Transport {
    /// The caller supplies the genuine configured protocol Hello range.
    #[must_use]
    pub fn new(versions: VersionRange) -> Self {
        Self {
            versions,
            next: 1,
            clients: BTreeMap::new(),
            replies: BTreeMap::new(),
            watches: BTreeMap::new(),
            closing: BTreeSet::new(),
            documents: BTreeSet::new(),
            streams: BTreeSet::new(),
        }
    }

    pub fn bind(&mut self, client: ClientId, party: u64, identity: people::Identity) {
        assert!(self.clients.insert(client, Client { party, identity, sign_in: None }).is_none(), "bind once");
    }

    #[must_use]
    pub fn is_bound(&self, client: ClientId) -> bool {
        self.clients.contains_key(&client)
    }

    /// The caller supplies the replacement authenticated identity independently
    /// of its expected party number, after retiring every old protocol route.
    pub fn replace_identity(&mut self, client: ClientId, party: u64, identity: people::Identity) {
        assert!(self.watches.values().all(|watch| watch.client != client), "old watches are detached first");
        assert!(
            self.replies.values().all(|reply| match reply {
                Reply::SignIn { client: owner }
                | Reply::SignOut { client: owner, .. }
                | Reply::Keyed { client: owner, .. }
                | Reply::Read { client: owner, .. }
                | Reply::ApplicationRead { client: owner, .. } => *owner != client,
            }),
            "old replies are detached first"
        );
        let old = self.clients.get_mut(&client).expect("bound client");
        *old = Client { party, identity, sign_in: None };
    }

    #[must_use]
    pub fn hello(&self) -> jig_client::Up {
        jig_client::Up::Served { served: jig_client_protocol_link::serves(self.versions), versions: None }
    }

    /// The configured Jig range, independent of a delivered Me.
    #[must_use]
    pub fn served(&self) -> VersionRange {
        self.versions
    }

    /// Replace only the protocol's configured served range, supplied outside Me.
    pub fn versions(&mut self, versions: VersionRange) {
        self.versions = versions;
    }

    #[must_use]
    pub fn clients(&self) -> Vec<ClientId> {
        self.clients.keys().copied().collect()
    }

    fn token(&mut self) -> Token {
        let number = self.next;
        self.next = self.next.checked_add(1).expect("finite transport token namespace");
        Token::new(number)
    }

    pub fn reply_to(&mut self, reply: Reply) -> ReplyTo {
        self.routed_reply(reply).1
    }

    /// Retain the transport's own token before moving its single-use reply door.
    pub fn routed_reply(&mut self, reply: Reply) -> (Token, ReplyTo) {
        let token = self.token();
        assert!(self.replies.insert(token, reply).is_none(), "fresh reply token");
        (token, ReplyTo::new(token))
    }

    #[must_use]
    pub fn reply(&self, token: Token) -> Option<&Reply> {
        self.replies.get(&token)
    }

    /// Explicit outstanding protocol destinations, independent of people,
    /// sessions and whether an owner's answer is final.
    #[must_use]
    pub fn reply_tokens(&self) -> Vec<Token> {
        self.replies.keys().copied().collect()
    }

    pub fn take_reply(&mut self, token: Token) -> Option<Reply> {
        self.replies.remove(&token)
    }

    /// The core supplies the actual independent person and session on its reply.
    pub fn signed_in(&mut self, client: ClientId, party: u64, sign_in: u64) {
        let state = self.clients.get_mut(&client).expect("bound client");
        assert_eq!(state.party, party, "the supplied authenticated identity names this party");
        state.sign_in = Some(sign_in);
    }

    #[must_use]
    pub fn sign_in(&self, client: ClientId) -> Option<u64> {
        self.clients.get(&client).expect("bound client").sign_in
    }

    #[must_use]
    pub fn party(&self, client: ClientId) -> u64 {
        self.clients.get(&client).expect("bound client").party
    }

    pub fn signed_out(&mut self, client: ClientId) {
        self.clients.get_mut(&client).expect("bound client").sign_in = None;
    }

    pub fn start_sign_in(&mut self, client: ClientId) -> Event {
        let identity = self.clients.get(&client).expect("bound client").identity.clone();
        let reply_to = self.reply_to(Reply::SignIn { client });
        Event::SignIn { reply_to, identity }
    }

    pub fn open(&mut self, client: ClientId, native: Token, subject: Watched) -> Token {
        assert!(
            self.owner_watch_token(client, native, subject.owner()).is_none(),
            "one live native watch per child and tab"
        );
        let token = self.token();
        assert!(self.watches.insert(token, Watch { client, native, subject }).is_none(), "fresh watch token");
        token
    }

    /// The jig child's opaque native watch name.
    #[must_use]
    pub fn watch_token(&self, client: ClientId, native: Token) -> Option<Token> {
        self.owner_watch_token(client, native, WatchOwner::Jig)
    }

    /// The application's child has an independent native-token namespace.
    #[must_use]
    pub fn application_watch_token(&self, client: ClientId, native: Token) -> Option<Token> {
        self.owner_watch_token(client, native, WatchOwner::Application)
    }

    fn owner_watch_token(&self, client: ClientId, native: Token, owner: WatchOwner) -> Option<Token> {
        self.watches.iter().find_map(|(token, watch)| {
            if watch.client == client
                && watch.native == native
                && watch.subject.owner() == owner
                && !self.closing.contains(token)
            {
                Some(*token)
            } else {
                None
            }
        })
    }

    #[must_use]
    pub fn watch(&self, token: Token) -> Option<&Watch> {
        self.watches.get(&token)
    }

    /// Retain the original route until its owning terminal. Closing cancels
    /// only flights actually admitted by this transport, independently at each pace.
    pub fn close_watch(&mut self, token: Token) -> Vec<Event> {
        assert!(self.watches.contains_key(&token), "original live route");
        self.closing.insert(token);
        self.closed_flights(token)
    }

    #[must_use]
    pub fn is_closing(&self, token: Token) -> bool {
        self.closing.contains(&token)
    }

    /// After whole admission, a packet for a closed route is discarded here.
    /// Its real flight ends without creating a callback for another route.
    pub fn closed_flights(&mut self, token: Token) -> Vec<Event> {
        let mut events = Vec::new();
        if self.closing.contains(&token) {
            if self.documents.remove(&token) {
                events.push(Event::Delivered { watcher: token });
            }
            if self.streams.remove(&token) {
                events.push(Event::StreamDelivered { watcher: token });
            }
        }
        events
    }

    pub fn end_watch(&mut self, token: Token) -> Option<Watch> {
        self.closing.remove(&token);
        self.documents.remove(&token);
        self.streams.remove(&token);
        self.watches.remove(&token)
    }

    /// Record an actual owner delivery before queuing its shell callback.
    /// The normal document and provisional stream have independent pacing.
    pub fn delivered(&mut self, watcher: Token, pace: Pace) -> bool {
        if !self.watches.contains_key(&watcher) {
            return false;
        }
        let inserted = match pace {
            Pace::Document => self.documents.insert(watcher),
            Pace::Stream => self.streams.insert(watcher),
        };
        assert!(inserted, "one owner delivery in flight at each pace");
        true
    }

    /// The shell took a normal document. This never acknowledges provisional text.
    pub fn taken(&mut self, client: ClientId, native: Token) -> Option<Event> {
        self.taken_owner(client, native, WatchOwner::Jig)
    }

    /// An application callback releases only its own normal flight.
    pub fn application_taken(&mut self, client: ClientId, native: Token) -> Option<Event> {
        self.taken_owner(client, native, WatchOwner::Application)
    }

    fn taken_owner(&mut self, client: ClientId, native: Token, owner: WatchOwner) -> Option<Event> {
        let watcher = self.owner_watch_token(client, native, owner)?;
        self.documents.remove(&watcher).then_some(Event::Delivered { watcher })
    }

    /// Keep the client's exact consumed position for a later reopened watch,
    /// and release only the actual provisional flight. A stale suffix may leave
    /// the client's newer position unchanged; no source sequence is substituted.
    /// # Errors
    /// Rejects a continuation for an unrelated task or a non-run watch.
    pub fn continued(
        &mut self,
        client: ClientId,
        native: Token,
        subject: Subject,
    ) -> Result<Option<Event>, InvalidContinuation> {
        let Some(watcher) = self.watch_token(client, native) else { return Ok(None) };
        let watch = self.watches.get_mut(&watcher).expect("live route");
        if let (Watched::Jig(Subject::Run { task: original, .. }), Subject::Run { task, .. }) =
            (&watch.subject, &subject)
        {
            if original != task {
                return Err(InvalidContinuation);
            }
        } else {
            return Err(InvalidContinuation);
        }
        watch.subject = Watched::Jig(subject);
        Ok(self.streams.remove(&watcher).then_some(Event::StreamDelivered { watcher }))
    }

    /// Original owner tokens of every open protocol watch, including siblings.
    #[must_use]
    pub fn watches(&self) -> Vec<Token> {
        self.watches.keys().filter(|token| !self.closing.contains(token)).copied().collect()
    }

    /// Old tab callbacks lose their route. The Front sends these actual owner
    /// watch tokens to Unwatch; retained keyed decisions stay in the core store.
    pub fn detach(&mut self, client: ClientId) -> Vec<Token> {
        self.replies.retain(|_, reply| match reply {
            Reply::SignIn { client: owner }
            | Reply::SignOut { client: owner, .. }
            | Reply::Keyed { client: owner, .. }
            | Reply::Read { client: owner, .. }
            | Reply::ApplicationRead { client: owner, .. } => *owner != client,
        });
        let watches: Vec<_> = self
            .watches
            .iter()
            .filter_map(|(token, watch)| if watch.client == client { Some(*token) } else { None })
            .collect();
        for token in &watches {
            self.end_watch(*token);
        }
        watches
    }
}

/// Carry Jig contract operations through the existing real-core routes.
/// # Errors
/// Returns storage work and unsupported family carriers intact to their owner.
pub fn contract_request(down: jig_client::Down) -> Result<jig_client_test_domain::Request, jig_client::Down> {
    Ok(jig_client_test_domain::Request::Client { request: jig_client::Request::Down { down } })
}

/// Translate only original transport callbacks; shell handoffs stay with their caller.
#[must_use]
pub fn contract_event(event: &jig_client_test_domain::Event) -> Option<jig_client::Up> {
    use jig_client::Up;
    use jig_client_test_domain::Event;
    match event {
        Event::Client { event: jig_client::Event::Up { up } } => Some(up.clone()),
        Event::SignedOut => Some(Up::SignedOut),
        Event::Client { .. }
        | Event::Start { .. }
        | Event::Went { .. }
        | Event::Act { .. }
        | Event::First(_)
        | Event::Jig(_) => None,
    }
}

#[cfg(test)]
mod tests {
    const UTF8_TEXTS: [&[u8]; 1] = [b"~\0"];

    use super::*;

    fn identity(subject: &[u8]) -> people::Identity {
        people::Identity {
            key: people::IdentityKey { provider: u16::MAX, subject: subject.into() },
            login: Box::from([255, 0]),
            name: Box::new([]),
        }
    }

    #[test]
    fn closing_keeps_original_correlation_until_real_terminals_and_finishes_only_admitted_flights() {
        let mut front = Transport::new(VersionRange { first: 9, last: 9 });
        front.bind(ClientId(0), 7, identity(b"same"));
        front.bind(ClientId(1), 7, identity(b"same"));
        let native = Token::new(9);
        let subject = Subject::Run { task: 1, turn: None, offset: 0 };
        let owner = front.open(ClientId(0), native, Watched::Jig(subject.clone()));
        let other = front.open(ClientId(1), native, Watched::Jig(subject.clone()));
        assert!(front.close_watch(owner).is_empty(), "closing does not invent any flight");
        assert!(front.watch_token(ClientId(0), native).is_none());
        assert_eq!(front.watch_token(ClientId(1), native), Some(other));
        assert_eq!(front.watches(), vec![other]);
        assert_eq!(
            front.watch(owner).expect("closing retains the original route").subject,
            Watched::Jig(subject.clone())
        );
        assert!(front.delivered(owner, Pace::Document));
        assert!(front.delivered(owner, Pace::Stream));
        assert!(front.delivered(other, Pace::Document));
        assert!(matches!(front.closed_flights(owner).as_slice(), [
            Event::Delivered { watcher: a }, Event::StreamDelivered { watcher: b }
        ] if *a == owner && *b == owner));
        assert!(front.closed_flights(owner).is_empty(), "each real terminal occurs once");
        assert!(matches!(front.taken(ClientId(1), native), Some(Event::Delivered { watcher }) if watcher == other));
        assert!(front.continued(ClientId(0), native, subject).expect("closed original continuation").is_none());
        assert_eq!(front.end_watch(owner).expect("actual original terminal").native, native);
        assert!(front.watch(owner).is_none());
        assert!(!front.delivered(owner, Pace::Document));
        assert!(!front.is_closing(owner));
    }

    #[test]
    fn normal_run_taken_and_provisional_continue_release_only_their_original_independent_flights() {
        use jig_client_domain::contract::ProvisionalTurnName;
        let mut front = Transport::new(VersionRange { first: 9, last: 9 });
        front.bind(ClientId(0), 7, identity(b"same"));
        front.bind(ClientId(1), 7, identity(b"same"));
        let native = Token::new(9);
        let initial = Subject::Run { task: u64::MAX, turn: None, offset: 0 };
        let owner = front.open(ClientId(0), native, Watched::Jig(initial.clone()));
        let other = front.open(ClientId(1), native, Watched::Jig(initial));
        assert!(front.delivered(owner, Pace::Document));
        assert!(front.delivered(owner, Pace::Stream));
        assert!(front.delivered(other, Pace::Document));
        assert!(matches!(front.taken(ClientId(0), native), Some(Event::Delivered { watcher }) if watcher == owner));
        assert!(front.taken(ClientId(0), native).is_none(), "normal callback is taken once");
        let exact = Subject::Run {
            task: u64::MAX,
            turn: Some(ProvisionalTurnName { attempt: u64::MAX, number: u32::MAX, retry: u32::MAX }),
            offset: u32::MAX,
        };
        assert!(
            matches!(front.continued(ClientId(0), native, exact.clone()).expect("named run"), Some(Event::StreamDelivered { watcher }) if watcher == owner)
        );
        assert_eq!(front.watch(owner).expect("original watch").subject, Watched::Jig(exact.clone()));
        assert!(front.continued(ClientId(0), native, exact.clone()).expect("duplicate callback").is_none());
        let resumed = crate::joint::wire::watches::watch(Token::new(77), 23, &exact, 2).expect("exact resume");
        assert!(
            matches!(resumed, Event::WatchRun { watcher, sign_in: 23, task: u64::MAX, turn: Some(jig_domain_views::TurnName { attempt: u64::MAX, turn: u32::MAX, retry: u32::MAX }), offset: u32::MAX } if watcher == Token::new(77))
        );
        assert!(matches!(front.taken(ClientId(1), native), Some(Event::Delivered { watcher }) if watcher == other));
    }

    #[test]
    fn unrelated_continue_and_old_tab_callbacks_cannot_release_or_relabel_a_live_run_flight() {
        let mut front = Transport::new(VersionRange { first: 9, last: 9 });
        front.bind(ClientId(0), 7, identity(b"same"));
        front.bind(ClientId(1), 7, identity(b"same"));
        let native = Token::new(3);
        let original = Subject::Run { task: 41, turn: None, offset: 0 };
        let owner = front.open(ClientId(0), native, Watched::Jig(original.clone()));
        let other = front.open(ClientId(1), native, Watched::Jig(original.clone()));
        assert!(front.delivered(owner, Pace::Document));
        assert!(front.delivered(owner, Pace::Stream));
        assert!(front.delivered(other, Pace::Stream));
        assert!(matches!(
            front.continued(ClientId(0), native, Subject::Run { task: 42, turn: None, offset: 0 }),
            Err(InvalidContinuation)
        ));
        assert!(matches!(front.continued(ClientId(0), native, Subject::Me), Err(InvalidContinuation)));
        assert_eq!(front.watch(owner).expect("unchanged original").subject, Watched::Jig(original.clone()));
        assert_eq!(front.detach(ClientId(0)), vec![owner]);
        assert!(front.taken(ClientId(0), native).is_none());
        assert!(front.continued(ClientId(0), native, original.clone()).expect("detached callback").is_none());
        assert!(!front.delivered(owner, Pace::Stream), "detached owner delivery has no route");
        assert!(
            matches!(front.continued(ClientId(1), native, original).expect("other tab remains live"), Some(Event::StreamDelivered { watcher }) if watcher == other)
        );
    }

    #[test]
    fn equal_tab_tokens_keep_distinct_owner_routes_and_actual_session_numbers() {
        let mut front = Transport::new(VersionRange { first: 1, last: u32::MAX });
        front.bind(ClientId(0), 7, identity(b"opaque/one"));
        front.bind(ClientId(1), 7, identity(b"opaque/one"));
        front.signed_in(ClientId(0), 7, 41);
        front.signed_in(ClientId(1), 7, u64::MAX);
        assert_eq!(front.sign_in(ClientId(0)), Some(41));
        assert_eq!(front.sign_in(ClientId(1)), Some(u64::MAX));
        let native = Token::new(9);
        let a = front.open(ClientId(0), native, Watched::Jig(Subject::Me));
        let b = front.open(ClientId(1), native, Watched::Jig(Subject::Me));
        assert_ne!(a, b);
        assert_eq!(front.watch(a).expect("first owner watch").native, native);
        assert_eq!(front.watch(b).expect("second owner watch").client, ClientId(1));
        let reply = Reply::Keyed { client: ClientId(1), attempt: native, key: [255; 16], owner: KeyedOwner::Jig };
        let to = front.reply_to(reply.clone()).into_token();
        assert_ne!(to, a);
        assert_ne!(to, b);
        assert_eq!(front.take_reply(to), Some(reply));
        assert_eq!(front.take_reply(to), None);
    }

    #[test]
    fn reload_detaches_only_old_tab_routes_without_reusing_or_closing_another_tabs_watch() {
        let mut front = Transport::new(VersionRange { first: 9, last: 9 });
        front.bind(ClientId(0), 7, identity(b"same"));
        front.bind(ClientId(1), 7, identity(b"same"));
        let a = front.open(ClientId(0), Token::new(3), Watched::Jig(Subject::Me));
        let b = front.open(ClientId(1), Token::new(3), Watched::Jig(Subject::Me));
        let old = front.reply_to(Reply::Read { client: ClientId(0), read: Token::new(3), what: Read::Me }).into_token();
        assert_eq!(front.detach(ClientId(0)), vec![a]);
        assert_eq!(front.take_reply(old), None);
        assert_eq!(front.watch(a), None);
        assert!(front.watch(b).is_some());
        let next = front.open(ClientId(0), Token::new(3), Watched::Jig(Subject::Me));
        assert_ne!(next, a);
        assert_ne!(next, b);
        assert_eq!(front.watch_token(ClientId(0), Token::new(3)), Some(next));
    }

    #[test]
    fn original_read_queries_and_application_subjects_remain_distinct_under_equal_native_tokens() {
        let mut front = Transport::new(VersionRange { first: 9, last: 9 });
        front.bind(ClientId(0), 7, identity(b"seven"));
        front.bind(ClientId(1), 8, identity(b"eight"));
        let native = Token::new(u64::MAX);
        let what = Read::Spend {
            project: u32::MAX,
            after: Some(
                jig_client_domain::contract::Cursor::new(Box::from(UTF8_TEXTS[0]), 2).expect("whole opaque cursor"),
            ),
        };
        let jig = Reply::Read { client: ClientId(0), read: native, what: what.clone() };
        let subject =
            family::Subject { scope: family::Scope::Project(u32::MAX), kind: family::Kind::OwnerEffect(u64::MAX) };
        let application = Reply::ApplicationRead { client: ClientId(1), read: native, subject };
        let first = front.reply_to(jig.clone()).into_token();
        let second = front.reply_to(application.clone()).into_token();
        assert_ne!(first, second);
        assert_eq!(front.take_reply(second), Some(application));
        assert_eq!(front.take_reply(first), Some(jig));
        assert_eq!(front.take_reply(first), None);
        assert_eq!(
            what,
            Read::Spend {
                project: u32::MAX,
                after: Some(
                    jig_client_domain::contract::Cursor::new(Box::from(UTF8_TEXTS[0]), 2)
                        .expect("original whole cursor")
                )
            }
        );
        assert_eq!(subject.scope, family::Scope::Project(u32::MAX));
        assert_eq!(subject.kind, family::Kind::OwnerEffect(u64::MAX));
    }

    #[test]
    fn hello_and_sign_in_keep_configured_protocol_range_and_supplied_opaque_identity() {
        let versions = VersionRange { first: u32::MAX, last: u32::MAX };
        let mut front = Transport::new(versions);
        let supplied = identity(&[255, b'/', 0]);
        front.bind(ClientId(0), u64::MAX, supplied.clone());
        assert_eq!(front.hello(), jig_client::Up::Served { served: false, versions: None });
        assert_eq!(front.sign_in(ClientId(0)), None);
        let Event::SignIn { reply_to, identity: actual } = front.start_sign_in(ClientId(0)) else {
            panic!("authentic sign-in input")
        };
        assert_eq!(actual, supplied);
        assert_eq!(front.take_reply(reply_to.into_token()), Some(Reply::SignIn { client: ClientId(0) }));
        assert_eq!(front.sign_in(ClientId(0)), None);
        assert_eq!(front.party(ClientId(0)), u64::MAX);
    }
    #[test]
    fn equal_child_watch_tokens_keep_original_routes_and_independent_pacing() {
        check_equal_child_watch_tokens_keep_original_routes_and_independent_pacing();
    }
    fn check_equal_child_watch_tokens_keep_original_routes_and_independent_pacing() {
        for native in [Token::new(0), Token::new(u64::MAX)] {
            let mut front = Transport::new(VersionRange { first: 9, last: 9 });
            front.bind(ClientId(0), 7, identity(b"same"));
            front.bind(ClientId(1), 7, identity(b"same"));
            let run = Subject::Run { task: 7, turn: None, offset: 0 };
            let family = Watched::Application {
                subject: family::Subject { scope: family::Scope::Project(3), kind: family::Kind::Hold(7) },
                scope: ApplicationScope::Project(3),
            };
            let jig = front.open(ClientId(0), native, Watched::Jig(run.clone()));
            let application = front.open(ClientId(0), native, family.clone());
            let sibling = front.open(ClientId(1), native, family.clone());
            assert_ne!(jig, application);
            assert_ne!(application, sibling);
            assert_eq!(front.watch_token(ClientId(0), native), Some(jig));
            assert_eq!(front.application_watch_token(ClientId(0), native), Some(application));
            assert_eq!(front.application_watch_token(ClientId(1), native), Some(sibling));
            for owner in [jig, application, sibling] {
                assert!(front.delivered(owner, Pace::Document));
            }
            assert!(front.delivered(jig, Pace::Stream));
            assert!(matches!(front.application_taken(ClientId(0), native),
                Some(Event::Delivered { watcher }) if watcher == application));
            assert!(front.application_taken(ClientId(0), native).is_none());
            assert!(front.close_watch(application).is_empty());
            assert_eq!(front.application_watch_token(ClientId(0), native), None);
            assert_eq!(front.watch_token(ClientId(0), native), Some(jig));
            let replacement = front.open(ClientId(0), native, family.clone());
            assert_ne!(replacement, application);
            assert_eq!(
                front.end_watch(application).expect("closed application watch has its original route").subject,
                family
            );
            assert_eq!(front.application_watch_token(ClientId(0), native), Some(replacement));
            assert!(
                matches!(front.continued(ClientId(0), native, run).expect("the Jig run watch retains its supplied subject"),
                Some(Event::StreamDelivered { watcher }) if watcher == jig)
            );
            assert!(matches!(front.taken(ClientId(0), native),
                Some(Event::Delivered { watcher }) if watcher == jig));
            assert!(front.taken(ClientId(0), native).is_none());
            assert!(matches!(front.application_taken(ClientId(1), native),
                Some(Event::Delivered { watcher }) if watcher == sibling));
            assert!(front.delivered(replacement, Pace::Document));
            front.end_watch(jig).expect("the Jig watch is independent of its application sibling");
            assert!(matches!(front.application_taken(ClientId(0), native),
                Some(Event::Delivered { watcher }) if watcher == replacement));
            assert_eq!(front.watch(sibling).expect("the other tab retains its original route").subject, family);
        }
    }
}
