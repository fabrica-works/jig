use super::*;
use skein_lib::{List, RequestRecord, Wall};
pub fn store_text() -> c::Text {
    c::Text::new(b"kept".as_slice().into(), 32).unwrap()
}
pub fn table_record(ask: c::Ask) -> RequestRecord<Ask> {
    RequestRecord {
        key: KEY,
        scope: u64::MAX,
        first_sent: Some(Wall::from_nanos(u64::MAX)),
        request: Ask::Jig(Box::new(ask)),
    }
}
fn specification() -> c::Specification {
    c::Specification { words: store_text(), parameters: List::with_capacity(0), inputs: List::with_capacity(0) }
}
fn amendment() -> c::Amendment {
    c::Amendment {
        addressed: Some(c::AddressChange { revision: read_value_revision(), address: read_value_person_address() }),
        spec: Some(specification()),
        wake: Some(read_value_owner_wake_policy()),
        dependencies: Some(List::with_capacity(0)),
        authority: Some(read_value_owner_authority()),
        instructions: Some(store_text()),
    }
}
fn policy() -> c::PolicyEdit {
    c::PolicyEdit::Role(c::PolicyEditRole {
        number: 7,
        authority: read_value_owner_authority(),
        period_spend: u64::MAX,
        requests: u16::MAX,
        decides: u8::MAX,
    })
}
pub fn store_asks() -> List<c::Ask> {
    let revision = read_value_revision();
    list([
        start_chat(),
        c::Ask::Write { project: 7, task: u64::MAX, words: store_text(), release: true },
        c::Ask::AnswerQuestion { project: 7, task: u64::MAX, message: u64::MAX, words: store_text() },
        c::Ask::AnswerPerson {
            project: 7,
            task: u64::MAX,
            revision,
            answer: c::PersonAnswer::Approved { state: b"state".as_slice().into(), words: Some(store_text()) },
        },
        c::Ask::Take { project: 7, task: u64::MAX },
        c::Ask::HandBack { project: 7, task: u64::MAX },
        c::Ask::DecideProposal {
            project: 7,
            proposal: c::ProposalKey { proposer: c::Proposer::Party(7), number: u64::MAX },
            decision: c::Decision::Rejected { reason: store_text() },
        },
        c::Ask::DecideEscalation {
            project: 7,
            task: u64::MAX,
            revision,
            decision: c::Decision::LeftHeld { reason: store_text() },
        },
        c::Ask::Release { project: 7, task: u64::MAX, revision, again: Some(true) },
        c::Ask::Amend {
            project: 7,
            task: u64::MAX,
            revision,
            change: amendment(),
            release: Some(revision),
            reason: store_text(),
        },
        c::Ask::Stop { project: 7, task: u64::MAX },
        c::Ask::Cancel { project: 7, task: u64::MAX, reason: store_text() },
        c::Ask::Close {
            project: 7,
            chat: u64::MAX,
            disposals: list([c::Disposal::Cancel { task: 8 }, c::Disposal::TakeOver { task: 9 }]),
        },
        c::Ask::TakeOver { project: 7, task: u64::MAX, reason: store_text() },
        c::Ask::SetGoal {
            project: 7,
            spec: store_text(),
            executor: c::GoalExecutor::Procedure { connector: 7, code: 9 },
            budget: u64::MAX,
            priority: u32::MAX,
            deadline: Some(Wall::from_nanos(u64::MAX)),
        },
        c::Ask::Prioritise { project: 7, goals: list([c::Priority { task: u64::MAX, priority: u32::MAX }]) },
        c::Ask::MarkSeen { seen: c::Seen::Replies { chat: u64::MAX, through: u64::MAX } },
        c::Ask::EditNote {
            project: 7,
            note: u64::MAX,
            scope: c::NoteScope::Goal { project: 7, goal: 9 },
            revision,
            description: store_text(),
            body: store_text(),
            references: list([u64::MAX]),
        },
        c::Ask::DeleteNote { project: 7, note: u64::MAX, scope: c::NoteScope::Deployment, revision },
        c::Ask::ChangePolicy { project: 7, edit: policy() },
        c::Ask::SetRoles {
            project: 7,
            holdings: list([c::Holding { party: u64::MAX, role: c::HeldRole::Policy { role: u32::MAX } }]),
        },
        c::Ask::MakeService { project: 7, name: store_text(), role: c::HeldRole::Member },
        c::Ask::SetPool { project: 7, party: u64::MAX, budget: u64::MAX },
    ])
}
fn list<T, const N: usize>(values: [T; N]) -> List<T> {
    let mut out = List::with_capacity(u32::try_from(N).unwrap());
    for value in values {
        out.push(value).ok().expect("fixture list fits");
    }
    out
}
pub const TABLE_VERSION: u16 = 1;
pub fn store_versions() -> List<StoreVersion> {
    list([
        StoreVersion { keeper: Keeper::Jig, version: jig_domain_client::store::STORE_VERSION },
        StoreVersion { keeper: Keeper::Table, version: TABLE_VERSION },
        StoreVersion { keeper: Keeper::Family(19), version: 7 },
    ])
}
pub fn saved_draft() -> Saved {
    Saved::Jig(jig_domain_client::store::Record::Draft {
        version: jig_domain_client::store::STORE_VERSION,
        party: u64::MAX,
        task: u64::MAX,
        words: store_text(),
        touched: u64::MAX,
    })
}
pub fn saved_table(ask: c::Ask) -> Saved {
    Saved::Table { version: TABLE_VERSION, record: table_record(ask) }
}
pub fn saved_family() -> Saved {
    Saved::Family { family: 19, version: 7, key: b"owner-key".as_slice().into(), bytes: Box::from([255, 0, 1]) }
}
pub fn store_read(value: Saved) -> StoreRead {
    let mut harness = Harness::new(&limits());
    harness.save(value);
    stored(&harness)
}
pub fn stored(harness: &Harness) -> StoreRead {
    for value in &harness.out {
        if let Request::Write { key, record } = value {
            return StoreRead { key: key.clone(), record: record.clone() };
        }
    }
    panic!("one store write")
}
pub fn other_store_version(mut value: StoreRead) -> StoreRead {
    let record = w::Kept::decode(&limits().wire, &mut Reader::new(&value.record)).unwrap();
    let record = w::Kept::new(
        &limits().wire,
        w::KeptParts { keeper: record.keeper().clone(), store_version: u16::MAX, record: record.record().into() },
    )
    .unwrap();
    let mut writer = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut writer).unwrap();
    value.record = writer.finish();
    value
}
impl Harness {
    pub fn save(&mut self, value: Saved) {
        self.link.step(Time::ZERO, Event::Save { record: value }, &mut self.out);
    }
    pub fn start_store(&mut self, versions: List<StoreVersion>, kept: List<StoreRead>) {
        self.link.step(Time::ZERO, Event::Start { versions, kept }, &mut self.out);
    }
    pub fn send_kept(&mut self, attempt: Token) {
        self.link.step(
            Time::ZERO,
            Event::Send { attempt, key: KEY, ask: Ask::Jig(Box::new(start_chat())) },
            &mut self.out,
        );
    }
    pub fn written(&mut self) {
        self.link.step(Time::ZERO, Event::Written, &mut self.out);
    }
    pub fn write_failed(&mut self) {
        self.link.step(Time::ZERO, Event::WriteFailed, &mut self.out);
    }
    pub fn exchanges(&self) -> u32 {
        let mut count = 0_u32;
        for value in &self.out {
            match value {
                Request::Exchange { .. } => count = count.saturating_add(1),
                Request::Kept { .. }
                | Request::Write { .. }
                | Request::Remove { .. }
                | Request::Dropped { .. }
                | Request::Answer { .. }
                | Request::Read { .. }
                | Request::OpenStream { .. }
                | Request::CloseStream
                | Request::Watch { .. }
                | Request::FamilyDelivery { .. }
                | Request::Ended { .. }
                | Request::Hello { .. }
                | Request::Signing { .. }
                | Request::ServiceCredential { .. }
                | Request::Fact { .. } => {}
            }
        }
        count
    }
    pub fn restored(&self, expected: &Saved) -> bool {
        for value in &self.out {
            match value {
                Request::Kept { record } if record == expected => return true,
                Request::Kept { .. }
                | Request::Exchange { .. }
                | Request::Write { .. }
                | Request::Remove { .. }
                | Request::Dropped { .. }
                | Request::Answer { .. }
                | Request::Read { .. }
                | Request::OpenStream { .. }
                | Request::CloseStream
                | Request::Watch { .. }
                | Request::FamilyDelivery { .. }
                | Request::Ended { .. }
                | Request::Hello { .. }
                | Request::Signing { .. }
                | Request::ServiceCredential { .. }
                | Request::Fact { .. } => {}
            }
        }
        false
    }
}

pub fn saved_places() -> List<Saved> {
    use jig_domain_client::address::{Address, ProjectSection, TaskSection};
    let mut addresses = List::with_capacity(21);
    for address in [
        Address::Inbox { project: None },
        Address::Inbox { project: Some(u32::MAX) },
        Address::Conversations { project: None },
        Address::Conversations { project: Some(u32::MAX) },
        Address::Deployment,
    ] {
        addresses.push(address).unwrap();
    }
    for section in [
        TaskSection::Overview,
        TaskSection::Activity,
        TaskSection::Plan,
        TaskSection::History,
        TaskSection::Budget,
        TaskSection::Panels,
    ] {
        addresses.push(Address::Task { task: u64::MAX, section }).unwrap();
    }
    for section in [
        ProjectSection::Overview,
        ProjectSection::Goals,
        ProjectSection::Tasks,
        ProjectSection::Notes,
        ProjectSection::Settings,
        ProjectSection::Resources,
        ProjectSection::Policy,
        ProjectSection::Roles,
        ProjectSection::Spend,
        ProjectSection::Recurring,
    ] {
        addresses.push(Address::Project { project: u32::MAX, section }).unwrap();
    }
    let mut saved = List::with_capacity(addresses.len());
    for address in addresses.into_boxed() {
        saved
            .push(Saved::Jig(jig_domain_client::store::Record::Place {
                version: jig_domain_client::store::STORE_VERSION,
                party: u64::MAX,
                address,
            }))
            .unwrap();
    }
    saved
}
pub fn one_store_read(value: StoreRead) -> List<StoreRead> {
    list([value])
}
