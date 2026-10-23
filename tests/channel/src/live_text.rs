//! The process reply's prefixes precede its turn (protocol/channel.md, 12).
use crate::processes::Proc;
use jig_client::{Asker, Handle, contract as c};
use jig_client_protocol_world::native::Received;

pub(crate) const RUN: Handle = Handle { asker: Asker(78), number: 71 };
const TASK: u64 = 1;
const TURN: u32 = 1;
// Three literal prefixes of the provider's reply, each observed before release.
pub(crate) const PIECES: usize = 3;
const PREFIXES: [&[u8]; PIECES] = [b"Hello", b"Hello from", b"Hello from worker"];
const HOLDS: [&[u8]; PIECES] = [b"watch-open", b"second-piece", b"third-piece"];

#[derive(Default)]
pub(crate) struct Reply {
    opened: bool,
    pub(crate) prefixes: Vec<Box<[u8]>>,
    pub(crate) committed: bool,
}

pub(crate) fn line() -> skein_fake_llm_domain::api::Line {
    use skein_fake_llm_domain::api::{Line, Piece};
    Line::Pieces {
        pieces: Box::new([
            Piece { text: b"Hello".as_slice().into(), hold: Some(HOLDS[0].into()) },
            Piece { text: b" from".as_slice().into(), hold: Some(HOLDS[1].into()) },
            Piece { text: b" worker".as_slice().into(), hold: Some(HOLDS[2].into()) },
        ]),
    }
}

impl Reply {
    pub(crate) fn act(&mut self, procs: &mut [Proc], store: &crate::world::Store) {
        let client =
            procs.iter().find_map(|p| if let Proc::Client(c) = p { Some(c) } else { None }).expect("native watcher");
        let state = client.received.iter().rev().find_map(|r| {
            if let Received::Delivery(c::Delivery::RunState(state)) = r
                && state.task == TASK
            {
                Some(state)
            } else {
                None
            }
        });
        let mut release = None;
        if !self.opened && state.is_some() {
            self.opened = true;
            release = Some(HOLDS[0]);
        } else if self.prefixes.len() < PREFIXES.len() {
            if let Some(text) = client.text(RUN) {
                if self.prefixes.last().is_some_and(|previous| previous.as_ref() == text.text.bytes()) {
                    return;
                }
                assert_eq!(text.text.bytes(), PREFIXES[self.prefixes.len()], "prefix before next release");
                assert_eq!((text.turn.number, text.turn.retry, text.from, text.cut), (TURN, 0, 0, false));
                assert_eq!(text.through as usize, text.text.bytes().len());
                assert_eq!(text.attempt, text.turn.attempt);
                assert_eq!(state.expect("opened run watch").committed_turn, 0);
                assert!(text.call.is_none());
                assert!(crate::world::turns(store).is_empty(), "each prefix precedes the durable turn");
                self.prefixes.push(text.text.bytes().into());
                release = HOLDS.get(self.prefixes.len()).copied();
            }
        } else if state.is_some_and(|state| state.committed_turn >= TURN) {
            assert!(state.expect("committed state").writing.is_none(), "committed reply has no writing");
            assert!(client.text(RUN).is_none(), "committed turn clears retained provisional text");
            self.committed = true;
        }
        if let Some(hold) = release {
            for proc in procs {
                if let Proc::Peer(peer) = proc {
                    peer.release(hold).expect("named provider hold");
                }
            }
        }
    }
}
