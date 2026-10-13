use super::*;
pub const READ_CURSOR: &[u8] = &[255; 48];
pub fn history_read() -> c::Read {
    c::Read::History { task: 7, subtree: true, after: Some(raw_cursor(READ_CURSOR)) }
}
pub fn notes_read() -> c::Read {
    c::Read::Notes {
        scope: c::NoteScope::Goal { project: 7, goal: 9 },
        description: Some(text(b"filter")),
        after: Some(raw_cursor(READ_CURSOR)),
    }
}
pub fn notes_scope_record() -> w::NoteScope {
    w::NoteScope::Goal(w::NoteScopeGoal::new(&limits().wire, w::NoteScopeGoalParts { project: 7, goal: 9 }).unwrap())
}

pub fn family_read() -> Family {
    Family { family: 19, version: 23, bytes: Box::from([255, 0, 1]) }
}

pub const READ: Token = Token::new(29);
impl Harness {
    pub fn read(&mut self, what: Readable) -> Token {
        self.link.step(Time::ZERO, Event::Read { read: READ, what }, &mut self.out);
        match self.out.iter().last().expect("one exchange") {
            Request::Exchange { exchange, .. } => *exchange,
            other
            @ (Request::Abandon { .. } | Request::Answer { .. } | Request::Read { .. } | Request::Fact { .. }) => {
                panic!("expected read exchange: {other:?}")
            }
        }
    }
    pub fn reading() -> (Self, Token) {
        let mut harness = Self::new(&limits());
        let exchange = harness.read(Readable::Jig(history_read()));
        (harness, exchange)
    }
    pub fn reading_family() -> (Self, Token) {
        let mut harness = Self::new(&limits());
        let exchange = harness.read(Readable::Family(family_read()));
        (harness, exchange)
    }
    pub fn read_answered(&self) -> Option<&Readout> {
        for request in &self.out {
            match request {
                Request::Read { read, result } if *read == READ => return Some(result),
                Request::Read { .. }
                | Request::Exchange { .. }
                | Request::Abandon { .. }
                | Request::Answer { .. }
                | Request::Fact { .. } => {}
            }
        }
        None
    }
    pub fn read_terminals(&self) -> u32 {
        let mut count: u32 = 0;
        for request in &self.out {
            match request {
                Request::Read { read, .. } if *read == READ => count = count.saturating_add(1),
                Request::Read { .. }
                | Request::Exchange { .. }
                | Request::Abandon { .. }
                | Request::Answer { .. }
                | Request::Fact { .. } => {}
            }
        }
        count
    }
    pub fn read_unreachable(&self) {
        assert_eq!(self.read_answered(), Some(&Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable))));
    }
}
pub fn encoded_read_answer(value: &w::ReadAnswer) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut writer).unwrap();
    writer.finish()
}
pub fn read_none_answer(value: w::ReadNone) -> Box<[u8]> {
    encoded_read_answer(&w::ReadAnswer::None(
        w::ReadAnswerNone::new(&limits().wire, w::ReadAnswerNoneParts { value }).unwrap(),
    ))
}
pub fn family_read_answer(family: u16, version: u32) -> Box<[u8]> {
    let lim = limits().wire;
    let owner = w::Owner::Family(w::Family::new(&lim, w::FamilyParts { number: family }).unwrap());
    let value = w::Owned::new(&lim, w::OwnedParts { owner, version, record: family_read().bytes }).unwrap();
    encoded_read_answer(&w::ReadAnswer::Value(value))
}
