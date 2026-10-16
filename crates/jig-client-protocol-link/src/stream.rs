//! Incremental SSE and canonical base64 framing (protocol/client.md, 4 and 7).
use crate::{Limits, Undecodable, translate};
use jig_client_protocol_wire::v7 as w;
use skein_http::sse;
use skein_lib::base64::{Alphabet, Decoder};
use skein_lib::stream::{Down, Read, Up};
use skein_lib::{Duration, Env, Queue, Reader, Time, Wall, bytes};

pub(crate) struct Stream {
    reader: sse::Reader,
    decoder: Decoder,
    pub(crate) name: Option<[u8; 16]>,
    last_read: Time,
    heartbeat: Duration,
    env: Env<sse::Limits>,
    above: Queue<sse::Event>,
    below: Queue<Down>,
    decoded: u32,
    line_start: bool,
    comment: bool,
}
impl Stream {
    pub(crate) fn new(limits: &Limits, now: Time) -> Self {
        let (decoded, framing) = bounds(limits).expect("validated stream bounds");
        let mut stream = Self {
            reader: sse::Reader::new(&framing),
            decoder: Decoder::new(Alphabet::Standard, decoded),
            name: None,
            last_read: now,
            heartbeat: limits.client.heartbeat,
            env: Env { now, wall: Wall::EPOCH, limits: framing },
            above: Queue::with_capacity(2),
            below: Queue::with_capacity(2),
            decoded,
            line_start: true,
            comment: false,
        };
        stream.demand(sse::Request::Next);
        stream
    }
    pub(crate) fn deadline(&self) -> Time {
        self.last_read.saturating_add(self.heartbeat.saturating_mul(2)).saturating_add(Duration::from_nanos(1))
    }
    pub(crate) fn hello(&mut self, hello: &w::Hello) -> Result<(), Undecodable> {
        if self.name.is_some() || hello.heartbeat() == Duration::ZERO {
            return Err(Undecodable::Malformed);
        }
        self.name = Some(*hello.stream());
        self.heartbeat = hello.heartbeat();
        Ok(())
    }
    pub(crate) fn progress(&mut self, now: Time) {
        self.last_read = now;
    }
    /// The reader's line scans are split at their first ending, within its chunk bound.
    pub(crate) fn piece(
        &mut self,
        piece: &[u8],
        limits: &Limits,
    ) -> Result<(Option<w::StreamEvent>, bool), Undecodable> {
        if self.line_start {
            self.comment = piece.first() == Some(&b':');
            self.line_start = false;
        }
        let ended = piece.last() == Some(&b'\r') || piece.last() == Some(&b'\n');
        let heartbeat = ended && self.comment;
        if ended {
            self.line_start = true;
            self.comment = false;
        }
        sse::up(&mut self.reader, &self.env, Up::Bytes(bytes::copy_of(piece)), &mut self.above, &mut self.below);
        let mut record = None;
        for _ in 0..self.env.limits.chunk.saturating_mul(3).saturating_add(8) {
            let Some(event) = self.above.pop() else { break };
            match event {
                sse::Event::Opened => self.data(),
                sse::Event::Data(Up::Bytes(piece)) => {
                    self.decoder.push(&piece).or(Err(Undecodable::Malformed))?;
                    self.data();
                }
                sse::Event::Data(Up::End) => {}
                sse::Event::Dispatched(_) => {
                    let decoder = core::mem::replace(&mut self.decoder, Decoder::new(Alphabet::Standard, self.decoded));
                    let body = decoder.finish().or(Err(Undecodable::Malformed))?;
                    let receiving = translate::pages::receiving(limits)?;
                    record = Some(
                        w::StreamEvent::decode(&receiving.wire, &mut Reader::new(&body))
                            .or(Err(Undecodable::Malformed))?,
                    );
                    self.demand(sse::Request::Next);
                }
                sse::Event::Data(Up::Failed(_)) | sse::Event::Failed(_) | sse::Event::Ended => {
                    return Err(Undecodable::Malformed);
                }
                sse::Event::Data(Up::Room) | sse::Event::Closed => unreachable!("reader is read-only and active"),
            }
        }
        self.clear_demands();
        Ok((record, heartbeat))
    }
    fn data(&mut self) {
        self.demand(sse::Request::Data(Down::Demand { read: Read::Arrived { max: self.env.limits.chunk }, room: 0 }));
    }
    fn demand(&mut self, request: sse::Request) {
        self.clear_demands();
        sse::down(&mut self.reader, &self.env, request, &mut self.above, &mut self.below);
    }
    fn clear_demands(&mut self) {
        for _ in 0_u32..2 {
            let Some(request) = self.below.pop() else { break };
            match request {
                Down::Demand { read: Read::Line { .. }, room: 0 } => {}
                Down::Demand { .. } | Down::Send(_) | Down::Finish => unreachable!("SSE only reads lines"),
            }
        }
    }
}
pub(crate) fn bounds(limits: &Limits) -> Option<(u32, sse::Limits)> {
    let decoded = u32::try_from(limits.client.document_bytes).ok()?.checked_add(512)?;
    let encoded = skein_lib::base64::encoded_len(Alphabet::Standard, decoded)?;
    let event = encoded.checked_add(256)?;
    Some((decoded, sse::Limits { line: event, event, field: 64, chunk: 1024 }))
}
pub(crate) fn worst_case(limits: &Limits) -> Option<u64> {
    let (decoded, framing) = bounds(limits)?;
    sse::worst_case(&framing)?
        .checked_add(u64::from(framing.chunk))?
        .checked_add(u64::from(decoded).checked_mul(2)?)?
        .checked_add(Queue::<sse::Event>::worst_case(2)?)?
        .checked_add(Queue::<Down>::worst_case(2)?)
}
