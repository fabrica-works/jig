//! In-memory native stream, with seeded cuts in both directions.
use skein_channel::{Control, Lower, LowerEvent, decode_control, parse_header};
use skein_fake_channel::ScriptedPeer;
use skein_lib::{Intake, Rng, Token, stream};
use std::collections::VecDeque;
pub(crate) struct Net {
    pub peer: ScriptedPeer,
    intake: Intake,
    read: stream::Read,
    bytes: VecDeque<u8>,
    right: Option<Token>,
    pub write_room: bool,
    pub refused: Option<(u16, Box<[u8]>)>,
    events: VecDeque<LowerEvent>,
    pub read_room: bool,
    pub other_version: bool,
}
impl Net {
    pub fn new(peer: ScriptedPeer, chunk: u32) -> Self {
        Self {
            peer,
            intake: Intake::with_capacity(chunk.max(8)),
            read: stream::Read::Nothing,
            bytes: VecDeque::new(),
            right: None,
            write_room: true,
            refused: None,
            events: VecDeque::new(),
            read_room: true,
            other_version: false,
        }
    }
    pub fn lower(&mut self, lower: Lower, rng: &mut Rng) {
        match lower {
            Lower::Read(stream::Down::Demand { read, room }) => {
                assert_eq!(room, 0);
                self.read = read;
            }
            Lower::Read(stream::Down::Finish) => {}
            Lower::Read(stream::Down::Send(_)) | Lower::FinishWrite => panic!("one stream, independent output"),
            Lower::Write(write) => match write {
                stream::OutputDown::Room { right, .. } => {
                    assert!(self.right.replace(right).is_none());
                }
                stream::OutputDown::Cancel { right } => {
                    if self.right.take() == Some(right) {
                        self.events.push_back(LowerEvent::Write(stream::OutputUp::Settled {
                            right,
                            outcome: stream::OutputOutcome::Cancelled,
                        }));
                    }
                }
                stream::OutputDown::Release { .. } => {}
                stream::OutputDown::Send { bytes, .. } => {
                    let header = parse_header(&bytes[..8]).expect("a complete sent frame");
                    if header.kind < 256 {
                        let control = decode_control(header.kind, &bytes[8..], &super::fixtures::channel_limits())
                            .expect("control");
                        if let Control::Refuse { reason, text } = control {
                            self.refused = Some((reason, text));
                        }
                    }
                    let mut at = 0;
                    while at < bytes.len() {
                        let cut = usize::try_from(rng.between(1, 17)).expect("small cut").min(bytes.len() - at);
                        self.peer.feed(&bytes[at..at + cut]).expect("worker accepts engine bytes");
                        at += cut;
                    }
                }
            },
        }
    }
    pub fn tick(&mut self, rng: &mut Rng) -> Option<LowerEvent> {
        if let Some(event) = self.events.pop_front() {
            return Some(event);
        }
        if self.write_room
            && let Some(right) = self.right.take()
        {
            return Some(LowerEvent::Write(stream::OutputUp::Settled {
                right,
                outcome: stream::OutputOutcome::Granted,
            }));
        }
        if !self.read_room {
            return None;
        }
        if self.bytes.is_empty()
            && let Some(mut bytes) = self.peer.pop_output()
        {
            if self.other_version && parse_header(&bytes[..8]).expect("frame").kind == 2 {
                bytes[8..10].copy_from_slice(&2_u16.to_be_bytes());
                self.other_version = false;
            }
            self.bytes.extend(bytes);
        }
        while !self.bytes.is_empty() && self.intake.room() > 0 {
            let count = usize::try_from(rng.between(1, 17))
                .expect("small cut")
                .min(self.bytes.len())
                .min(usize::try_from(self.intake.room()).expect("small intake"));
            let cut: Vec<_> = self.bytes.drain(..count).collect();
            self.intake.append(&cut).expect("room checked");
        }
        if let Some(bytes) = self.intake.meet(self.read) {
            self.read = stream::Read::Nothing;
            return Some(LowerEvent::Read(stream::Up::Bytes(bytes)));
        }
        None
    }
}
