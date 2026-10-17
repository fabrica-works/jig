use super::*;
use skein_lib::List;
pub const WATCH: Token = Token::new(41);
pub const HEARTBEAT: Duration = Duration::from_nanos(100);
pub const STREAM: [u8; 16] = [7; 16];
pub fn frame_subject() -> Watchable {
    Watchable::Jig(c::Subject::Me)
}
pub fn family_subject() -> Watchable {
    Watchable::Family {
        scope: jig_client_domain::contract::Scope::Project(1),
        window: jig_client_domain::contract::Window { rows: 3, after: None },
        record: family_read(),
    }
}
pub fn hello_event(heartbeat: Duration) -> Box<[u8]> {
    let lim = limits().wire;
    let mut served = List::with_capacity(2);
    served.push(w::Served::new(&lim, w::ServedParts { owner: w::Owner::Jig, first: 8, last: 8 }).unwrap()).unwrap();
    served
        .push(
            w::Served::new(
                &lim,
                w::ServedParts {
                    owner: w::Owner::Family(w::Family::new(&lim, w::FamilyParts { number: 19 }).unwrap()),
                    first: 23,
                    last: 23,
                },
            )
            .unwrap(),
        )
        .unwrap();
    event_bytes(&w::StreamEvent::Hello(
        w::Hello::new(&lim, w::HelloParts { stream: STREAM, served, heartbeat }).unwrap(),
    ))
}
pub fn event_bytes(value: &w::StreamEvent) -> Box<[u8]> {
    let mut binary = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut binary).unwrap();
    binary.finish()
}
pub fn family_delivery(name: u64) -> Box<[u8]> {
    let lim = limits().wire;
    let family = family_read();
    let owner = w::Owner::Family(w::Family::new(&lim, w::FamilyParts { number: family.family }).unwrap());
    let document = w::Owned::new(&lim, w::OwnedParts { owner, version: family.version, record: family.bytes }).unwrap();
    event_bytes(&w::StreamEvent::Delivery(w::Delivery::new(&lim, w::DeliveryParts { watch: name, document }).unwrap()))
}
pub fn ending(name: u64, why: w::WatchEnd) -> Box<[u8]> {
    event_bytes(&w::StreamEvent::Ended(w::Ended::new(&limits().wire, w::EndedParts { watch: name, why }).unwrap()))
}
pub fn open_answer(value: &w::OpenAnswer) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut writer).unwrap();
    writer.finish()
}
pub fn close_answer() -> Box<[u8]> {
    let value = w::CloseAnswer::Closed;
    let mut writer = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut writer).unwrap();
    writer.finish()
}
pub fn past_two_heartbeats() -> Time {
    Time::from_nanos(201)
}
impl Harness {
    pub fn clear(&mut self) {
        self.out = Queue::with_capacity(32);
    }
    pub fn open(&mut self, watch: Token, subject: Watchable) -> Token {
        self.link.step(Time::ZERO, Event::Open { watch, subject }, &mut self.out);
        watch
    }
    pub fn streaming(subject: Watchable) -> Self {
        let mut harness = Self::new(&limits());
        harness.open(WATCH, subject);
        harness.stream_record(&hello_event(HEARTBEAT));
        harness
    }
    pub fn stream_at(&mut self, now: Time, bytes: &[u8]) {
        self.link.step(now, Event::Streamed { message: Streamed::Record(Box::from(bytes)) }, &mut self.out);
    }
    pub fn stream_record(&mut self, bytes: &[u8]) {
        self.stream_at(Time::ZERO, bytes);
    }
    pub fn advance(&mut self, now: Time) {
        self.link.fire(now, &mut self.out);
    }
    pub fn opening(&self) -> (Token, u64) {
        for request in &self.out {
            match request {
                Request::Exchange { exchange, message: Message::Watch(body), .. } => {
                    let record = w::Open::decode(&limits().wire, &mut Reader::new(body)).unwrap();
                    return (*exchange, record.watch());
                }
                Request::Exchange { .. }
                | Request::Answer { .. }
                | Request::NotKept { .. }
                | Request::Read { .. }
                | Request::OpenStream { .. }
                | Request::CloseStream
                | Request::Watch { .. }
                | Request::FamilyDelivery { .. }
                | Request::Ended { .. }
                | Request::Hello { .. }
                | Request::Save { .. }
                | Request::Erase { .. }
                | Request::Kept { .. }
                | Request::Dropped
                | Request::Signing { .. }
                | Request::ServiceCredential { .. }
                | Request::Fact { .. } => {}
            }
        }
        panic!("no opening")
    }
    pub fn ended(&self, watch: Token) -> Option<c::WatchEnd> {
        for request in &self.out {
            match request {
                Request::Ended { watch: token, end } if *token == watch => return Some(*end),
                Request::Exchange { .. }
                | Request::Answer { .. }
                | Request::NotKept { .. }
                | Request::Read { .. }
                | Request::OpenStream { .. }
                | Request::CloseStream
                | Request::Watch { .. }
                | Request::FamilyDelivery { .. }
                | Request::Ended { .. }
                | Request::Hello { .. }
                | Request::Save { .. }
                | Request::Erase { .. }
                | Request::Kept { .. }
                | Request::Dropped
                | Request::Signing { .. }
                | Request::ServiceCredential { .. }
                | Request::Fact { .. } => {}
            }
        }
        None
    }
    pub fn closed_count(&self) -> u32 {
        let mut count = 0_u32;
        for request in &self.out {
            match request {
                Request::Ended { .. } => count = count.checked_add(1).unwrap(),
                Request::Exchange { .. }
                | Request::Answer { .. }
                | Request::NotKept { .. }
                | Request::Read { .. }
                | Request::OpenStream { .. }
                | Request::CloseStream
                | Request::Watch { .. }
                | Request::FamilyDelivery { .. }
                | Request::Hello { .. }
                | Request::Save { .. }
                | Request::Erase { .. }
                | Request::Kept { .. }
                | Request::Dropped
                | Request::Signing { .. }
                | Request::ServiceCredential { .. }
                | Request::Fact { .. } => {}
            }
        }
        count
    }
    pub fn close(&mut self) -> Token {
        self.link.step(Time::ZERO, Event::Close { watch: WATCH }, &mut self.out);
        match self.out.iter().last().unwrap() {
            Request::Exchange { exchange, message: Message::Unwatch(_), .. } => *exchange,
            other @ (Request::Exchange { .. }
            | Request::Answer { .. }
            | Request::NotKept { .. }
            | Request::Read { .. }
            | Request::OpenStream { .. }
            | Request::CloseStream
            | Request::Watch { .. }
            | Request::FamilyDelivery { .. }
            | Request::Ended { .. }
            | Request::Hello { .. }
            | Request::Save { .. }
            | Request::Erase { .. }
            | Request::Kept { .. }
            | Request::Dropped
            | Request::Signing { .. }
            | Request::ServiceCredential { .. }
            | Request::Fact { .. }) => panic!("no close: {other:?}"),
        }
    }
    pub fn expected_family(&self) {
        assert!(self.has(&Request::FamilyDelivery { watch: WATCH, record: family_read() }));
    }
}
impl Harness {
    pub fn gone_name(&self) -> [u8; 16] {
        for request in &self.out {
            match request {
                Request::Exchange { message: Message::StreamGone(body), .. } => {
                    return *w::Gone::decode(&limits().wire, &mut Reader::new(body)).unwrap().stream();
                }
                Request::Exchange { .. }
                | Request::Answer { .. }
                | Request::NotKept { .. }
                | Request::Read { .. }
                | Request::OpenStream { .. }
                | Request::CloseStream
                | Request::Watch { .. }
                | Request::FamilyDelivery { .. }
                | Request::Ended { .. }
                | Request::Hello { .. }
                | Request::Save { .. }
                | Request::Erase { .. }
                | Request::Kept { .. }
                | Request::Dropped
                | Request::Signing { .. }
                | Request::ServiceCredential { .. }
                | Request::Fact { .. } => {}
            }
        }
        panic!("no stream-gone exchange")
    }
    pub fn assert_run_subject(&self) {
        for request in &self.out {
            match request {
                Request::Exchange { message: Message::Watch(body), .. } => {
                    let lim = limits().wire;
                    let open = w::Open::decode(&lim, &mut Reader::new(body)).unwrap();
                    let subject = w::Subject::decode(&lim, &mut Reader::new(open.subject().record())).unwrap();
                    let turn = w::ProvisionalTurnName::new(
                        &lim,
                        w::ProvisionalTurnNameParts { attempt: 9, number: 2, retry: 3 },
                    )
                    .unwrap();
                    assert_eq!(
                        subject,
                        w::Subject::Run(
                            w::SubjectRun::new(&lim, w::SubjectRunParts { task: 7, turn: Some(turn), offset: 123 })
                                .unwrap()
                        )
                    );
                    return;
                }
                Request::Exchange { .. }
                | Request::Answer { .. }
                | Request::NotKept { .. }
                | Request::Read { .. }
                | Request::OpenStream { .. }
                | Request::CloseStream
                | Request::Watch { .. }
                | Request::FamilyDelivery { .. }
                | Request::Ended { .. }
                | Request::Hello { .. }
                | Request::Save { .. }
                | Request::Erase { .. }
                | Request::Kept { .. }
                | Request::Dropped
                | Request::Signing { .. }
                | Request::ServiceCredential { .. }
                | Request::Fact { .. } => {}
            }
        }
        panic!("no run opening")
    }
}

pub fn not_served() -> w::NotServed {
    let lim = limits().wire;
    let owner = w::Owner::Family(w::Family::new(&lim, w::FamilyParts { number: 19 }).unwrap());
    let mut served = List::with_capacity(1);
    served.push(w::Served::new(&lim, w::ServedParts { owner: owner.clone(), first: 24, last: 24 }).unwrap()).unwrap();
    w::NotServed::new(&lim, w::NotServedParts { owner, served }).unwrap()
}
