//! Outside expectations for document freshness and provisional stream prefixes.
//! Observations come from the scripted parent and watcher terminals, never the
//! child's state (`domain/engine.md`, 11; `client/domain/contract.md`, 7).
use jig_core_views::{Call, End, Event, Follow, Limits, Request, TurnName, Window};
use skein_lib::{Time, Token, Wall};
use skein_world::domain::{Expectations, Judge, Referee as Harness};
use std::collections::{BTreeMap, BTreeSet};

/// A document made by the scripted owner after the named generation changed.
#[derive(Clone, Debug)]
pub struct Document {
    pub generation: u64,
    pub bytes: Vec<u8>,
}
#[derive(Debug)]
struct Watch {
    party: Token,
    subject: Token,
    window: Window,
    follow: Option<Follow>,
    making: bool,
    document_flight: bool,
    made: Option<u64>,
    last: Option<u64>,
    stream_flight: Option<(TurnName, u32)>,
    positions: BTreeMap<TurnName, u32>,
}
#[derive(Debug)]
enum Seen {
    Asked { watcher: Token, party: Token, subject: Token, window: Window, follow: Option<Follow> },
    Changed { subject: Token, generation: u64 },
    Made { watcher: Token, document: Document },
    MadeTerminal { watcher: Token },
    DocumentTaken { watcher: Token },
    StreamTaken { watcher: Token },
    Text { run: Token, turn: TurnName, offset: u32, text: Vec<u8>, call: Option<Call> },
    Called { run: Token, call: Call },
    Watching { watcher: Token },
    Make { watcher: Token, subject: Token, window: Window },
    Deliver { watcher: Token },
    Stream { watcher: Token, turn: TurnName, from: u32, to: u32, text: Vec<u8>, cut: bool, call: Option<Call> },
    Ended { watcher: Token, end: End },
    Refused { watcher: Token },
}
#[derive(Debug)]
struct Checks {
    limits: Limits,
    asked: BTreeMap<Token, Watch>,
    open: BTreeMap<Token, Watch>,
    generations: BTreeMap<Token, u64>,
    text: BTreeMap<(Token, TurnName), (Vec<u8>, Option<Call>)>,
    deliveries: u64,
    ended: Vec<(Token, End)>,
}
impl Expectations for Checks {
    type Seen = Seen;
    type Name = Token;
    type Stimulus = ();
    #[expect(clippy::too_many_lines, reason = "each public observation checks one outside invariant")]
    fn observe(&mut self, seen: Seen, _judge: &mut Judge<Token, ()>) {
        match seen {
            Seen::Asked { watcher, party, subject, window, follow } => {
                let mut positions = BTreeMap::new();
                if let Some(follow) = follow
                    && let Some(turn) = follow.turn
                {
                    positions.insert(turn, follow.offset);
                }
                assert!(
                    self.asked
                        .insert(
                            watcher,
                            Watch {
                                party,
                                subject,
                                window,
                                follow,
                                making: false,
                                document_flight: false,
                                made: None,
                                last: None,
                                stream_flight: None,
                                positions
                            }
                        )
                        .is_none(),
                    "unique admission request"
                );
            }
            Seen::Changed { subject, generation } => {
                self.generations.insert(subject, generation);
            }
            Seen::Watching { watcher } => {
                let watch = self.asked.remove(&watcher).expect("parent admitted this watch");
                assert!(self.open.insert(watcher, watch).is_none(), "unique open watch");
                assert!(
                    self.open.len() <= usize::try_from(self.limits.watchers).expect("watch capacity"),
                    "global watch capacity"
                );
                let mut parties = BTreeMap::<Token, u32>::new();
                let mut runs = BTreeSet::new();
                for watch in self.open.values() {
                    *parties.entry(watch.party).or_default() += 1;
                    if let Some(follow) = watch.follow {
                        runs.insert(follow.run);
                    }
                }
                assert!(parties.values().all(|count| *count <= self.limits.watchers_per_party), "party watch capacity");
                assert!(
                    runs.len() <= usize::try_from(self.limits.watchers).expect("watch capacity"),
                    "followed runs are bounded by open watches, independently of retained-run admission"
                );
            }
            Seen::Make { watcher, subject, window } => {
                let watch = self.open.get_mut(&watcher).expect("open watch");
                assert_eq!(watch.subject, subject, "subject echoed");
                assert_eq!(watch.window, window, "window echoed");
                assert!(!watch.document_flight && !watch.making, "one construction or document in flight");
                watch.making = true;
            }
            Seen::Made { watcher, document } => {
                let watch = self.open.get_mut(&watcher).expect("open watch");
                assert!(watch.making, "owner was asked to make");
                assert!(
                    document.bytes.len() <= usize::try_from(watch.window.rows).expect("window bound"),
                    "document respects its window"
                );
                assert_eq!(
                    document.generation,
                    *self.generations.get(&watch.subject).unwrap_or(&0),
                    "owner made the current state"
                );
                watch.made = Some(document.generation);
            }
            Seen::MadeTerminal { watcher } => {
                let watch = self.open.get_mut(&watcher).expect("open watch");
                assert!(watch.making, "outstanding construction");
                watch.making = false;
            }
            Seen::Deliver { watcher } => {
                let watch = self.open.get_mut(&watcher).expect("open watch");
                assert!(!watch.document_flight && !watch.making, "one document in flight");
                watch.last = Some(watch.made.take().expect("owner made a document"));
                watch.document_flight = true;
                self.deliveries += 1;
            }
            Seen::DocumentTaken { watcher } => {
                let watch = self.open.get_mut(&watcher).expect("open watch");
                assert!(watch.document_flight, "document terminal has its flight");
                watch.document_flight = false;
            }
            Seen::Text { run, turn, offset, text, call } => {
                let source = self.text.entry((run, turn)).or_default();
                let offset = usize::try_from(offset).expect("text offset");
                assert!(offset <= source.0.len(), "the scripted host writes consecutive offsets");
                assert_eq!(source.0.get(offset..), text.get(..source.0.len() - offset), "repeated host prefixes agree");
                source.0.truncate(offset);
                source.0.extend_from_slice(&text);
                if let Some(call) = call {
                    source.1 = Some(observed_call(call, source.1.as_ref()));
                }
            }
            Seen::Called { run, call } => {
                if let Some((_, source)) = self.text.iter_mut().filter(|((owner, _), _)| *owner == run).next_back() {
                    source.1 = Some(observed_call(call, source.1.as_ref()));
                }
            }
            Seen::Stream { watcher, turn, from, to, text, cut, call } => {
                let watch = self.open.get_mut(&watcher).expect("open watch");
                assert!(watch.stream_flight.is_none(), "one provisional stream in flight");
                let follow = watch.follow.expect("watch follows this run");
                let source = self.text.get(&(follow.run, turn)).expect("host wrote this named turn");
                let requested = *watch.positions.get(&turn).unwrap_or(&0);
                let end = u32::try_from(source.0.len()).expect("source length");
                let beginning = end.saturating_sub(self.limits.text_bytes);
                let position = if (beginning..=end).contains(&requested) { requested } else { 0 };
                if cut {
                    assert!(from > position, "cut says which prefix was omitted");
                } else {
                    assert_eq!(from, position, "each turn keeps its own offset");
                }
                assert_eq!(
                    to.checked_sub(from),
                    Some(u32::try_from(text.len()).expect("bounded text")),
                    "absolute positions describe bytes"
                );
                assert!(
                    text.len() <= usize::try_from(self.limits.text_bytes).expect("text bound"),
                    "bounded retained suffix"
                );
                assert_eq!(
                    source.0.get(usize::try_from(from).expect("from")..usize::try_from(to).expect("to")),
                    Some(text.as_slice()),
                    "provisional bytes are a prefix-consistent source copy"
                );
                if let Some(call) = &call {
                    assert!(
                        call.identity.id.len() + call.tool.len() + call.input.len()
                            <= usize::try_from(self.limits.text_bytes).expect("call bound"),
                        "bounded provisional call"
                    );
                }
                assert_eq!(&call, &source.1, "current call belongs to this turn");
                watch.stream_flight = Some((turn, to));
            }
            Seen::StreamTaken { watcher } => {
                let watch = self.open.get_mut(&watcher).expect("open watch");
                let (turn, to) = watch.stream_flight.take().expect("stream terminal has its flight");
                watch.positions.insert(turn, to);
            }
            Seen::Ended { watcher, end } => {
                let watch = self.open.remove(&watcher).expect("one end per admitted watch");
                assert!(
                    !watch.document_flight && !watch.making && watch.stream_flight.is_none(),
                    "end follows every outstanding terminal"
                );
                self.ended.push((watcher, end));
            }
            Seen::Refused { watcher } => {
                assert!(!self.open.contains_key(&watcher), "refusal opens nothing");
                self.asked.remove(&watcher).expect("refused admission was asked");
            }
        }
    }
}

/// The world's referee, using skein's observation harness.
#[derive(Debug)]
pub struct Referee {
    harness: Harness<Checks>,
}
impl Default for Referee {
    fn default() -> Self {
        Self::new(crate::world::LIMITS)
    }
}
impl Referee {
    #[must_use]
    pub fn new(limits: Limits) -> Self {
        Self {
            harness: Harness::new(Checks {
                limits,
                asked: BTreeMap::new(),
                open: BTreeMap::new(),
                generations: BTreeMap::new(),
                text: BTreeMap::new(),
                deliveries: 0,
                ended: Vec::new(),
            }),
        }
    }
    fn observe(&mut self, seen: Seen) {
        self.harness.observe(Time::ZERO, seen, &mut Vec::new());
    }
    pub fn changed(&mut self, subject: Token, generation: u64) {
        self.observe(Seen::Changed { subject, generation });
    }
    pub fn made(&mut self, watcher: Token, document: Document) {
        self.observe(Seen::Made { watcher, document });
    }
    pub fn before(&mut self, event: &Event) {
        match event {
            Event::Watch { watcher, party, subject, window, follow } => self.observe(Seen::Asked {
                watcher: *watcher,
                party: *party,
                subject: *subject,
                window: *window,
                follow: *follow,
            }),
            Event::Made { watcher, .. } => self.observe(Seen::MadeTerminal { watcher: *watcher }),
            Event::Delivered { watcher } => self.observe(Seen::DocumentTaken { watcher: *watcher }),
            Event::StreamDelivered { watcher } => self.observe(Seen::StreamTaken { watcher: *watcher }),
            Event::Text { run, turn, offset, text, call } => self.observe(Seen::Text {
                run: *run,
                turn: *turn,
                offset: *offset,
                text: text.to_vec(),
                call: call.clone(),
            }),
            Event::Called { run, call } => self.observe(Seen::Called { run: *run, call: call.clone() }),
            Event::Changed { .. }
            | Event::Unwatch { .. }
            | Event::Gone { .. }
            | Event::RunEnded { .. }
            | Event::TurnCommitted { .. }
            | Event::EndWatch { .. } => {}
        }
    }
    pub fn saw(&mut self, requests: &[Request]) {
        for request in requests {
            let seen = match request {
                Request::Watching { watcher } => Seen::Watching { watcher: *watcher },
                Request::Make { watcher, subject, window } => {
                    Seen::Make { watcher: *watcher, subject: *subject, window: *window }
                }
                Request::Deliver { watcher } => Seen::Deliver { watcher: *watcher },
                Request::Stream { watcher, turn, from, to, text, cut, call } => Seen::Stream {
                    watcher: *watcher,
                    turn: *turn,
                    from: *from,
                    to: *to,
                    text: text.to_vec(),
                    cut: *cut,
                    call: call.clone(),
                },
                Request::Ended { watcher, end } => Seen::Ended { watcher: *watcher, end: *end },
                Request::Refused { watcher, .. } => Seen::Refused { watcher: *watcher },
            };
            self.observe(seen);
        }
    }
    #[must_use]
    pub fn in_flight(&self, watcher: Token) -> bool {
        self.harness.expectations().open.get(&watcher).is_some_and(|watch| watch.document_flight)
    }
    #[must_use]
    pub fn stream_in_flight(&self, watcher: Token) -> bool {
        self.harness.expectations().open.get(&watcher).is_some_and(|watch| watch.stream_flight.is_some())
    }
    #[must_use]
    pub fn live(&self) -> usize {
        self.harness.expectations().open.len()
    }
    #[must_use]
    pub fn deliveries(&self) -> u64 {
        self.harness.expectations().deliveries
    }
    #[must_use]
    pub fn ended(&self) -> &[(Token, End)] {
        &self.harness.expectations().ended
    }
    #[must_use]
    pub fn open_names(&self) -> Vec<Token> {
        self.harness.expectations().open.keys().copied().collect()
    }
    #[must_use]
    pub fn open_subjects(&self) -> Vec<Token> {
        self.harness.expectations().open.values().map(|watch| watch.subject).collect()
    }
    pub fn assert_caught_up(&self) {
        let checks = self.harness.expectations();
        for watch in checks.open.values() {
            assert_eq!(
                watch.last,
                Some(*checks.generations.get(&watch.subject).unwrap_or(&0)),
                "last delivery follows the final subject change"
            );
        }
    }
}

fn observed_call(mut call: Call, previous: Option<&Call>) -> Call {
    call.began = Some(match previous {
        Some(previous) if previous.identity == call.identity => previous.began.expect("source first seen"),
        Some(_) | None => Wall::EPOCH,
    });
    call
}
