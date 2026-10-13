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
            other @ (Request::Abandon { .. }
            | Request::Answer { .. }
            | Request::Read { .. }
            | Request::OpenStream
            | Request::CloseStream
            | Request::Watch { .. }
            | Request::FamilyDelivery { .. }
            | Request::Ended { .. }
            | Request::Hello { .. }
            | Request::Write { .. }
            | Request::Remove { .. }
            | Request::Kept { .. }
            | Request::Dropped { .. }
            | Request::ServiceCredential { .. }
            | Request::Fact { .. }) => {
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
                | Request::OpenStream
                | Request::CloseStream
                | Request::Watch { .. }
                | Request::FamilyDelivery { .. }
                | Request::Ended { .. }
                | Request::Hello { .. }
                | Request::Write { .. }
                | Request::Remove { .. }
                | Request::Kept { .. }
                | Request::Dropped { .. }
                | Request::ServiceCredential { .. }
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
                | Request::OpenStream
                | Request::CloseStream
                | Request::Watch { .. }
                | Request::FamilyDelivery { .. }
                | Request::Ended { .. }
                | Request::Hello { .. }
                | Request::Write { .. }
                | Request::Remove { .. }
                | Request::Kept { .. }
                | Request::Dropped { .. }
                | Request::ServiceCredential { .. }
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

pub fn encoded_read_value(value: &w::ReadValue) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut writer).unwrap();
    writer.finish()
}
pub fn jig_read_record_answer(record: Box<[u8]>, version: u32) -> Box<[u8]> {
    let value = w::Owned::new(&w::CEILINGS, w::OwnedParts { owner: w::Owner::Jig, version, record }).unwrap();
    encoded_read_answer(&w::ReadAnswer::Value(value))
}
pub fn jig_read_value_answer(value: &w::ReadValue) -> Box<[u8]> {
    jig_read_record_answer(encoded_read_value(value), jig_protocol_client_wire::VERSION)
}
pub fn frame_read_value() -> w::ReadValue {
    w::ReadValue::Frame(frame_record())
}
pub fn expected_frame_read() -> Readout {
    Readout::Jig(c::ReadResult::Value(c::ReadValue::Frame(Box::new(frame()))))
}
pub fn read_value_cases() -> Box<[(w::ReadValue, c::ReadValue)]> {
    let mut cases = skein_lib::List::with_capacity(26);
    read_value_cases_first(&mut cases);
    read_value_cases_last(&mut cases);
    cases.into_boxed()
}
fn read_value_cases_first(cases: &mut skein_lib::List<(w::ReadValue, c::ReadValue)>) {
    cases
        .push((
            w::ReadValue::OwnerProject(owner_project_record()),
            c::ReadValue::OwnerProject(Box::new(owner_project())),
        ))
        .unwrap();
    cases
        .push((
            w::ReadValue::Historical(Box::new(read_wire_owner_historical())),
            c::ReadValue::Historical(Box::new(read_value_owner_historical())),
        ))
        .unwrap();
    cases.push((w::ReadValue::Frame(frame_record()), c::ReadValue::Frame(Box::new(frame())))).unwrap();
    cases.push((w::ReadValue::Task(task_record()), c::ReadValue::Task(Box::new(task())))).unwrap();
    cases
        .push((w::ReadValue::Proposal(read_wire_proposal()), c::ReadValue::Proposal(Box::new(read_value_proposal()))))
        .unwrap();
    cases
        .push((
            w::ReadValue::OwnerProposal(read_wire_owner_proposal()),
            c::ReadValue::OwnerProposal(Box::new(read_value_owner_proposal())),
        ))
        .unwrap();
    cases
        .push((
            w::ReadValue::Escalation(read_wire_escalation()),
            c::ReadValue::Escalation(Box::new(read_value_escalation())),
        ))
        .unwrap();
    cases
        .push((
            w::ReadValue::OwnerEscalation(read_wire_owner_escalation()),
            c::ReadValue::OwnerEscalation(Box::new(read_value_owner_escalation())),
        ))
        .unwrap();
    cases
        .push((w::ReadValue::Question(read_wire_question()), c::ReadValue::Question(Box::new(read_value_question()))))
        .unwrap();
    cases
        .push((
            w::ReadValue::PersonTask(read_wire_person_task()),
            c::ReadValue::PersonTask(Box::new(read_value_person_task())),
        ))
        .unwrap();
    cases
        .push((
            w::ReadValue::OwnerPersonTask(read_wire_owner_person_task()),
            c::ReadValue::OwnerPersonTask(Box::new(read_value_owner_person_task())),
        ))
        .unwrap();
    cases
        .push((w::ReadValue::Result(read_wire_result()), c::ReadValue::Result(Box::new(read_value_result()))))
        .unwrap();
    cases
        .push((w::ReadValue::Ended(read_wire_object_page()), c::ReadValue::Ended(Box::new(read_value_object_page()))))
        .unwrap();
}
fn read_value_cases_last(cases: &mut skein_lib::List<(w::ReadValue, c::ReadValue)>) {
    cases
        .push((
            w::ReadValue::OwnerEnded(read_wire_owner_ended_page()),
            c::ReadValue::OwnerEnded(Box::new(read_value_owner_ended_page())),
        ))
        .unwrap();
    cases
        .push((
            w::ReadValue::History(read_wire_history_page()),
            c::ReadValue::History(Box::new(read_value_history_page())),
        ))
        .unwrap();
    cases
        .push((
            w::ReadValue::Activity(decoded_activity_record()),
            c::ReadValue::Activity(Box::new(decoded_activity_page())),
        ))
        .unwrap();
    cases
        .push((w::ReadValue::Inbox(read_wire_inbox_page()), c::ReadValue::Inbox(Box::new(read_value_inbox_page()))))
        .unwrap();
    cases
        .push((
            w::ReadValue::OwnerInbox(read_wire_owner_inbox_page()),
            c::ReadValue::OwnerInbox(Box::new(read_value_owner_inbox_page())),
        ))
        .unwrap();
    cases
        .push((
            w::ReadValue::Conversations(read_wire_conversation_page()),
            c::ReadValue::Conversations(Box::new(read_value_conversation_page())),
        ))
        .unwrap();
    cases
        .push((w::ReadValue::Goals(read_wire_goal_page()), c::ReadValue::Goals(Box::new(read_value_goal_page()))))
        .unwrap();
    cases
        .push((w::ReadValue::Spend(read_wire_spend_page()), c::ReadValue::Spend(Box::new(read_value_spend_page()))))
        .unwrap();
    cases
        .push((
            w::ReadValue::OwnerSpend(read_wire_owner_spend_page()),
            c::ReadValue::OwnerSpend(Box::new(read_value_owner_spend_page())),
        ))
        .unwrap();
    cases
        .push((w::ReadValue::Project(read_wire_project()), c::ReadValue::Project(Box::new(read_value_project()))))
        .unwrap();
    cases
        .push((w::ReadValue::Notes(read_wire_note_page()), c::ReadValue::Notes(Box::new(read_value_note_page()))))
        .unwrap();
    cases
        .push((w::ReadValue::Deployment(deployment_record()), c::ReadValue::Deployment(Box::new(deployment()))))
        .unwrap();
    cases
        .push((
            w::ReadValue::OwnerDeployment(owner_deployment_record()),
            c::ReadValue::OwnerDeployment(Box::new(owner_deployment())),
        ))
        .unwrap();
}

pub fn values_past_client_limits() -> [w::ReadValue; 2] {
    [w::ReadValue::Proposal(proposal_with_words(&[b'a'; 33])), w::ReadValue::Ended(object_page_piece(result_rows(3)))]
}
pub fn malformed_read_values() -> [Box<[u8]>; 4] {
    let value = encoded_read_value(&frame_read_value());
    let mut trailing = Writer::new(value.len().checked_add(1).unwrap());
    trailing.put(&value).unwrap();
    trailing.put(&[0]).unwrap();
    [
        Box::from([255]),
        trailing.finish(),
        encoded_read_value(&w::ReadValue::Ended(object_page_piece(Box::from([0, 0, 0])))),
        encoded_read_value(&w::ReadValue::Ended(object_page_piece(trailing_result_row()))),
    ]
}
pub fn read_byte_limits(document_bytes: u64) -> Limits {
    let mut client = limits().client;
    client.document_bytes = document_bytes;
    Limits::new(client, 4, limits().deadline).unwrap()
}
