//! First connector's client side. Its own page and small-store position are
//! child state, never root state (client/domain/root.md, sections 7 and 8).
use jig_client::{
    contract::{Part, Revision},
    frame::FrameShown,
    link::Link,
};
use skein_lib::{Env, List, Queue, RequestProgress, Time, Token};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address {
    pub project: u32,
    pub item: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    Touch { project: u32, item: u64, revision: Revision },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    Touched { item: u64, revision: Revision },
    Moved { item: u64, revision: Revision },
    Gone,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Go { address: Address },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Place,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    Place { version: u16, address: Address },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Out {}
#[derive(Debug)]
pub enum Event {
    Show { part: Part, generation: u32 },
    PartsLeft { generation: u32 },
    Went { address: Address },
    Left,
    Act { act: Act },
    Answered { request: Token, answer: Answer },
    Progress { request: Token, progress: RequestProgress },
    Link { link: Link },
    Confirmed { party: Option<u64> },
    FrameIs { asked: Token, frame: FrameShown },
    Restore { record: Record },
}
#[derive(Debug)]
pub enum Request {
    Frame { asked: Token },
    Show { address: Address },
    Save { record: Record },
    Send { request: Token, scope: u64, ask: Ask },
    Out(Out),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub parts: u32,
    pub text: u32,
}
#[derive(Debug)]
pub struct Side {
    parts: List<Held>,
    address: Option<Address>,
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
    pub fn new(limits: &Limits, _seed: u64) -> Self {
        Self { parts: List::with_capacity(limits.parts), address: None, link: Link::Starting, party: None, shown: 0 }
    }
    #[must_use]
    pub const fn address(&self) -> Option<Address> {
        self.address
    }
    #[must_use]
    pub const fn link(&self) -> Link {
        self.link
    }
    #[must_use]
    pub const fn party(&self) -> Option<u64> {
        self.party
    }
    fn changed(&mut self) {
        self.shown = self.shown.checked_add(1).expect("shown count fits");
    }
}
#[must_use]
pub const fn shown(side: &Side) -> u64 {
    side.shown
}
pub fn step(side: &mut Side, _env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Show { part, generation } => {
            for held in &side.parts {
                if held.part == part && held.generation == generation {
                    return;
                }
            }
            if side.parts.push(Held { part, generation }).is_ok() {
                side.changed();
            }
        }
        Event::PartsLeft { generation } => {
            let capacity = side.parts.capacity();
            let old = core::mem::replace(&mut side.parts, List::with_capacity(capacity));
            for held in old.into_boxed() {
                if held.generation != generation {
                    assert!(side.parts.push(held).is_ok(), "remaining parts fit");
                }
            }
            side.changed();
        }
        Event::Went { address } => {
            side.address = Some(address);
            side.changed();
            out.push(Request::Frame { asked: Token::new(0) });
        }
        Event::Left => {
            side.address = None;
            side.changed();
        }
        Event::Act { act } => match act {
            Act::Go { address } => out.push(Request::Show { address }),
        },
        Event::Link { link } => {
            side.link = link;
            side.changed();
        }
        Event::Confirmed { party } => {
            side.party = party;
            side.changed();
        }
        Event::FrameIs { asked: _, frame } => {
            side.party = frame.party;
            side.link = frame.link;
            side.changed();
        }
        Event::Restore { record } => match record {
            Record::Place { version, address } => {
                if version == 1 {
                    side.address = Some(address);
                }
            }
        },
        Event::Answered { request: _, answer: _ } | Event::Progress { request: _, progress: _ } => {}
    }
}
#[must_use]
pub const fn next_deadline(_side: &Side) -> Option<Time> {
    None
}
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    List::<Held>::worst_case(limits.parts)?
        .checked_mul(2)?
        .checked_add(u64::from(limits.parts).checked_mul(u64::from(limits.text))?)
}
#[must_use]
pub const fn max_out(_limits: &Limits) -> u32 {
    4
}
