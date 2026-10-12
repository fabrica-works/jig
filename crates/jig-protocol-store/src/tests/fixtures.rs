//! Shared keys favor number boundaries and common prefixes (protocol/store.md, 4.1).
use crate::keys::Own;
use alloc::boxed::Box;
use jig_domain_core::{self as core, CoreKey, ProjectionKey};
use jig_domain_people::{self as people, Proposer};
use skein_lib::Rng;

pub const KEY_LIMIT: u32 = 1024;
pub fn number(rng: &mut Rng) -> u64 {
    match rng.below(5) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => u64::from(u32::MAX),
        _ => u64::MAX,
    }
}
pub fn narrow(rng: &mut Rng) -> u32 {
    match rng.below(4) {
        0 => 0,
        1 => 1,
        2 => 2,
        _ => u32::MAX,
    }
}
pub fn own(rng: &mut Rng) -> Own {
    match rng.below(3) {
        0 => Own::Format,
        1 => Own::Transcript { task: number(rng) },
        _ => Own::Attempt { task: number(rng), attempt: number(rng) },
    }
}
pub fn core_key(rng: &mut Rng) -> CoreKey {
    match rng.below(8) {
        0 => CoreKey::Deployment,
        1 => CoreKey::Projection(match rng.below(4) {
            0 => ProjectionKey::Goal(number(rng)),
            1 => ProjectionKey::Task { goal: number(rng), task: number(rng) },
            2 => ProjectionKey::Lifecycle { goal: number(rng), task: number(rng), position: number(rng) },
            _ => ProjectionKey::Revision { goal: number(rng), task: number(rng), revision: number(rng) },
        }),
        2 => CoreKey::RunProof(number(rng)),
        3 => CoreKey::Call(core::CallKey {
            task: number(rng),
            attempt: number(rng),
            completion: narrow(rng),
            position: narrow(rng),
        }),
        4 => CoreKey::Turn { task: number(rng), attempt: number(rng), turn: narrow(rng) },
        5 => CoreKey::Terminal { task: number(rng), attempt: number(rng) },
        6 => CoreKey::ProposalDecision {
            proposer: match rng.below(2) {
                0 => Proposer::Task(number(rng)),
                _ => Proposer::Party(number(rng)),
            },
            proposal: number(rng),
        },
        _ => CoreKey::EscalationDecision { task: number(rng), revision: number(rng) },
    }
}
pub fn people_key(rng: &mut Rng) -> people::Key {
    match rng.below(7) {
        0 => people::Key::Person(number(rng)),
        1 => people::Key::ReadResults { party: number(rng) },
        2 => people::Key::ReadChat { party: number(rng), task: number(rng) },
        3 => people::Key::SignIn(number(rng)),
        4 => people::Key::Roles(narrow(rng)),
        5 => people::Key::Policy(narrow(rng)),
        _ => {
            let mut key = [0; 16];
            for byte in &mut key {
                *byte = match rng.below(5) {
                    0 => 0,
                    1 => 1,
                    2 => 2,
                    3 => 254,
                    _ => 255,
                };
            }
            people::Key::Answer(people::RequestKey { person: number(rng), key })
        }
    }
}
pub fn key(rng: &mut Rng) -> core::Key {
    match rng.below(4) {
        0 => core::Key::Core(core_key(rng)),
        1 => core::Key::Tasks(tasks_key(rng)),
        2 => core::Key::People(people_key(rng)),
        _ => core::Key::Notes(notes_key(rng)),
    }
}

pub fn funder(rng: &mut Rng) -> jig_domain_tasks::Funder {
    use jig_domain_tasks::Funder;
    match rng.below(4) {
        0 => Funder::Task(number(rng)),
        1 => Funder::Pool { project: narrow(rng), person: number(rng), period: number(rng) },
        2 => Funder::Recurring { project: narrow(rng), task: number(rng), period: number(rng) },
        _ => Funder::Period { project: narrow(rng), period: number(rng) },
    }
}
pub fn tasks_key(rng: &mut Rng) -> jig_domain_tasks::Key {
    use jig_domain_tasks::{Key, Newest, UpdateObject};
    match rng.below(19) {
        0 => Key::Live(number(rng)),
        1 => Key::Stub(number(rng)),
        2 => Key::Ledger(funder(rng)),
        3 => Key::Writer(number(rng)),
        4 => Key::Pool(number(rng)),
        5 => Key::PersonProposal(number(rng)),
        6 => Key::Ended(number(rng)),
        13 => Key::Result { party: number(rng), position: number(rng) },
        14 => Key::ConversationNewest { party: number(rng), task: Newest::of(number(rng)) },
        17 => Key::EndedDelegate { requester: number(rng), task: number(rng) },
        18 => Key::Delegate { requester: number(rng), task: number(rng) },
        16 => Key::Goal { project: narrow(rng), task: Newest::of(number(rng)) },
        15 => Key::ConversationOldest { party: number(rng), task: number(rng) },
        7 => Key::ClosedLedger(funder(rng)),
        8 => Key::DecidedProposal(number(rng)),
        9 => Key::History { task: number(rng), revision: number(rng) },
        10 => Key::Milestone { task: number(rng), position: number(rng) },
        11 => Key::Update {
            party: number(rng),
            at: Newest::of(number(rng)),
            task: Newest::of(number(rng)),
            object: if rng.below(2) == 0 {
                UpdateObject::Result
            } else {
                UpdateObject::Reply { message: Newest::of(number(rng)), activity: Newest::of(number(rng)) }
            },
        },
        _ => Key::Activity { task: number(rng), position: number(rng) },
    }
}
pub fn segment(rng: &mut Rng) -> Box<[u8]> {
    match rng.below(5) {
        0 => Box::new([]),
        1 => Box::new([0]),
        2 => Box::new([0, 0]),
        3 => Box::new([0, 255]),
        _ => Box::new([1]),
    }
}
pub fn pattern(rng: &mut Rng) -> jig_domain_notes::Pattern {
    use jig_domain_notes::Last;
    let mut segments = skein_lib::List::with_capacity(3);
    for _ in 0..rng.below(4) {
        segments.push(segment(rng)).expect("at most three segments");
    }
    let last = match rng.below(2) {
        0 => Last::Exact(segment(rng)),
        _ => Last::Open(segment(rng)),
    };
    jig_domain_notes::Pattern { segments: segments.into_boxed(), last }
}
pub fn notes_key(rng: &mut Rng) -> jig_domain_notes::Key {
    use jig_domain_notes::{Key, Scope};
    match rng.below(2) {
        0 => Key::Entry { name: number(rng) },
        _ => Key::Line {
            scope: match rng.below(4) {
                0 => Scope::Deployment,
                1 => Scope::Project { project: narrow(rng) },
                2 => Scope::Goal { project: narrow(rng), goal: number(rng) },
                _ => Scope::Resources {
                    project: narrow(rng),
                    connector: match rng.below(4) {
                        0 => 0,
                        1 => 1,
                        2 => 2,
                        _ => u16::MAX,
                    },
                    pattern: pattern(rng),
                },
            },
            name: number(rng),
        },
    }
}
pub fn pattern_key(segments: Box<[Box<[u8]>]>) -> jig_domain_notes::Key {
    use jig_domain_notes::{Key, Last, Pattern, Scope};
    Key::Line {
        scope: Scope::Resources {
            project: 1,
            connector: 1,
            pattern: Pattern { segments, last: Last::Exact(Box::new([])) },
        },
        name: 1,
    }
}

pub fn within(intervals: &[skein_kv::Range], key: &[u8]) -> bool {
    for interval in intervals {
        let before_end = match &interval.end {
            Some(end) => key < &**end,
            None => true,
        };
        if key >= &*interval.start && before_end {
            return true;
        }
    }
    false
}

/// Small notes bounds shared by codec round trips.
pub const NOTES_LIMITS: jig_domain_notes::Limits = jig_domain_notes::Limits {
    scopes: 2,
    entries_per_scope: 2,
    pattern_bytes: 16,
    description_bytes: 32,
    body_bytes: 64,
    references: 2,
    load_rows: 2,
    lines: 2,
    recalled: 2,
};
pub fn note_entry() -> jig_domain_notes::Entry {
    let mut references = skein_lib::List::with_capacity(NOTES_LIMITS.references);
    references.push(21).expect("two references fit");
    references.push(34).expect("two references fit");
    jig_domain_notes::Entry {
        name: 17,
        scope: jig_domain_notes::Scope::Goal { project: 3, goal: 21 },
        description: Box::from(b"an entry".as_slice()),
        body: Box::from(b"its body".as_slice()),
        references,
        author: jig_domain_notes::Author::Task { task: 21, attempt: 2 },
        revision: 4,
        changed: skein_lib::Wall::from_nanos(u64::MAX),
    }
}
pub fn note_line() -> jig_domain_notes::Line {
    use jig_domain_notes::{Last, Pattern, Scope};
    jig_domain_notes::Line {
        scope: Scope::Resources {
            project: 3,
            connector: 2,
            pattern: Pattern {
                segments: Box::new([Box::from(b"a".as_slice()), Box::from(b"\0b".as_slice())]),
                last: Last::Open(Box::from(b"c".as_slice())),
            },
        },
        name: 17,
        description: Box::from(b"a resource".as_slice()),
        revision: 4,
    }
}

pub const TASKS_LIMITS: jig_domain_tasks::Limits = jig_domain_tasks::Limits {
    tasks: 2,
    funders: 2,
    project_tasks: 2,
    tree_tasks: 2,
    depth: 2,
    delegates: 2,
    references: 2,
    subscriptions: 2,
    batch: 2,
    dependencies: 2,
    holdings: 2,
    hold_kinds: 2,
    resource_reports: 2,
    pools: 2,
    hold_segments: 2,
    hold_bytes: 16,
    hold_wait: skein_lib::Duration::from_nanos(1),
    hold_waiters: 2,
    inputs: 2,
    spec_bytes: 32,
    parameters: 3,
    result_bytes: 32,
    run_inbox: 2,
    inbox_messages: 2,
    inbox_bytes: 32,
    message_bytes: 16,
    person_stall: skein_lib::Duration::from_nanos(1),
    proposal_stall: skein_lib::Duration::from_nanos(1),
    escalation_stall: skein_lib::Duration::from_nanos(1),
    saved_resources: 2,
    contract_choices: 2,
    charters: 2,
    authority_grants: 2,
    authority_segments: 2,
    authority_bytes: 32,
    executor_kinds: 3,
    retries: jig_domain_tasks::Retries {
        transient: TASK_RETRY,
        permanent: TASK_RETRY,
        run: TASK_RETRY,
        agent: TASK_RETRY,
        lost: TASK_RETRY,
        invalid: TASK_RETRY,
    },
    facts: 2,
    room_refusals: 2,
};
const TASK_RETRY: jig_domain_tasks::Retry = jig_domain_tasks::Retry {
    retries: 1,
    base: skein_lib::Duration::from_nanos(1),
    max: skein_lib::Duration::from_nanos(2),
};
pub fn task_funding() -> jig_domain_tasks::FundingRecord {
    use jig_domain_tasks::{Funder, FundingRecord, Numbers, PeriodTimes};
    FundingRecord {
        period: Some(PeriodTimes {
            began: skein_lib::Wall::from_nanos(11),
            next: Some(skein_lib::Wall::from_nanos(29)),
        }),
        funder: Funder::Period { project: 3, period: 5 },
        parent: None,
        numbers: Numbers { budget: 100, spent: 7, spent_below: 13, reserved: 20 },
        made: 6,
        closed: false,
    }
}
pub fn task_name() -> jig_domain_tasks::Name {
    jig_domain_tasks::Name { connector: 2, path: Box::new([Box::from(b"a\0".as_slice()), Box::from(b"b".as_slice())]) }
}
pub fn task_writer() -> jig_domain_tasks::WriterSlot {
    jig_domain_tasks::WriterSlot {
        number: 9,
        resource: task_name(),
        writer: jig_domain_tasks::Writer::Run { task: 17, attempt: 3 },
        lost: true,
    }
}
pub fn task_authority() -> jig_domain_tasks::Authority {
    use jig_domain_tasks::{Authority, AuthorityExecutor, Budget, Delegation, Grant, Last, Pattern, Scopes, Tools};
    Authority {
        tools: Tools(u64::MAX),
        grants: Box::new([
            Grant {
                connector: 2,
                kind: 3,
                pattern: Pattern {
                    segments: Box::new([Box::from(b"a".as_slice())]),
                    last: Last::Exact(Box::from(b"b".as_slice())),
                },
            },
            Grant {
                connector: 4,
                kind: 5,
                pattern: Pattern { segments: Box::new([]), last: Last::Open(Box::from(b"c".as_slice())) },
            },
        ]),
        delegation: Delegation {
            kinds: Box::new([
                AuthorityExecutor::Charter(11),
                AuthorityExecutor::Procedure(13),
                AuthorityExecutor::Role(17),
            ]),
            tasks: 19,
            depth: 23,
        },
        budget: Budget { spend: 29, deadline: Some(skein_lib::Wall::from_nanos(u64::MAX)) },
        notes: Scopes(7),
        note_resources: Box::new([]),
    }
}
pub fn task_spec() -> jig_domain_tasks::Spec {
    use jig_domain_tasks::{Parameter, Spec};
    Spec {
        words: Box::from(b"spec".as_slice()),
        parameters: Box::new([
            Parameter::Number { name: 1, value: u64::MAX },
            Parameter::Bytes { name: 2, value: Box::from(b"\0value".as_slice()) },
            Parameter::Resource { name: 3, connector: u16::MAX, resource: 5 },
        ]),
        inputs: Box::new([0, u64::MAX]),
    }
}
pub fn task_contract() -> jig_domain_tasks::Contract {
    use jig_domain_tasks::{Contract, FieldRule, Verdict};
    Contract::Verdict {
        choices: Box::new([Verdict {
            code: 7,
            words: 16,
            followups: 3,
            fields: Box::new([FieldRule { name: Box::from(b"field".as_slice()), max: 11 }]),
        }]),
    }
}

pub fn creation_spec() -> jig_domain_tasks::Spec {
    let mut spec = task_spec();
    spec.inputs = Box::new([]);
    spec
}
pub fn task_creation(number: u64) -> jig_domain_tasks::New {
    use jig_domain_tasks::{Executor, Funder, Holding, New, Numbers};
    New {
        number,
        project: 3,
        executor: Executor::Agent { charter: 11 },
        spec: creation_spec(),
        contract: jig_domain_tasks::Contract::Report { words: TASKS_LIMITS.result_bytes },
        authority: task_authority(),
        numbers: Numbers { budget: 29, spent: 0, spent_below: 0, reserved: 0 },
        funder: Funder::Period { project: 3, period: 5 },
        dependencies: Box::new([17]),
        holdings: Box::new([
            Holding::Write { resource: task_name(), kind: 2 },
            Holding::Slot { pool: task_name(), kind: 3 },
        ]),
        wake: task_wake(),
        recurring: None,
        tracked: Some(1),
    }
}
pub fn task_template() -> jig_domain_tasks::RecurringTemplate {
    let mut first = task_creation(1);
    first.dependencies = Box::new([]);
    jig_domain_tasks::RecurringTemplate {
        key: 41,
        batch: Box::new([first]),
        overlap: jig_domain_tasks::RecurringOverlap::Wait,
    }
}
pub fn task_phase() -> jig_domain_tasks::Phase {
    use jig_domain_tasks::{Active, Hold, HoldSource, Phase, Was};
    Phase::Held {
        was: Was::Active(Active::Running { attempt: 3 }),
        why: Hold::Uncertain { entry: 11 },
        source: Some(Box::new(HoldSource::Resource(task_name()))),
    }
}
pub fn task_settings() -> jig_domain_tasks::Settings {
    jig_domain_tasks::Settings {
        addressed: Some(jig_domain_tasks::PersonAddress::Role(19)),
        taken_by: Some(31),
        instructions: Some(Box::from(b"override".as_slice())),
        spec: creation_spec(),
        wake: task_wake(),
        dependencies: Box::new([17]),
        authority: task_authority(),
    }
}
pub fn task_amendment() -> jig_domain_tasks::Amendment {
    jig_domain_tasks::Amendment {
        addressed: Some(jig_domain_tasks::AddressChange {
            revision: 7,
            address: jig_domain_tasks::PersonAddress::Person(31),
        }),
        instructions: Some(Box::from(b"new override".as_slice())),
        release: Some(11),
        spec: Some(creation_spec()),
        wake: Some(task_wake()),
        dependencies: Some(Box::new([17])),
        authority: Some(task_authority()),
        reason: Box::from(b"amend".as_slice()),
    }
}
pub fn task_proposal() -> jig_domain_tasks::Proposal {
    use jig_domain_tasks::{Proposal, ProposalAction, ProposalHolder, ProposalKind, ProposalState};
    Proposal {
        number: 43,
        proposer: 17,
        project: 3,
        action: ProposalAction::Release { task: 17 },
        reason: Box::from(b"proposal".as_slice()),
        as_holder: true,
        state: ProposalState::Pending {
            holder: ProposalHolder::Policy { project: 3, kind: ProposalKind::Batch },
            since: skein_lib::Wall::from_nanos(47),
        },
    }
}
pub fn task_history() -> jig_domain_tasks::History {
    use jig_domain_tasks::{AmendmentHistory, Change, History, Party};
    let before = task_settings();
    let mut after = task_settings();
    after.instructions = Some(Box::from(b"new override".as_slice()));
    after.taken_by = None;
    after.addressed = Some(jig_domain_tasks::PersonAddress::Person(31));
    History {
        task: 17,
        revision: 3,
        by: Party::Deployment { project: 3 },
        change: Change::Amended,
        reason: Box::from(b"amendment".as_slice()),
        proposal: Some(Box::new(task_proposal())),
        at: skein_lib::Wall::from_nanos(53),
        amendment: Some(Box::new(AmendmentHistory { before, after })),
    }
}
pub fn task_person_proposal() -> jig_domain_tasks::PersonProposal {
    let mut goal = task_creation(23);
    goal.funder = jig_domain_tasks::Funder::Pool { project: 3, person: 31, period: 5 };
    goal.dependencies = Box::new([]);
    jig_domain_tasks::PersonProposal {
        number: 43,
        proposer: 31,
        project: 3,
        goal,
        state: jig_domain_tasks::PersonProposalState::Pending { since: skein_lib::Wall::from_nanos(47) },
    }
}

pub fn task_wake() -> jig_domain_tasks::WakePolicy {
    use jig_domain_tasks::{ResultsWake, WakePolicy, WakeRule};
    WakePolicy {
        words: WakeRule::Batch { count: 3, age: skein_lib::Duration::from_nanos(7) },
        notices: WakeRule::Immediate,
        news: WakeRule::Never,
        results: ResultsWake::Each,
        questions: true,
        answers: false,
        timers: true,
    }
}
