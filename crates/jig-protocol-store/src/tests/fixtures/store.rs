//! Memory component fixtures (protocol/store.md, section 14).
use super::{
    APPLICATION, CORE_LIMITS, DEPLOYMENT, KEY_LIMIT, calls, core_rows, identity, note_entry, note_line, task_history,
    task_record,
};
use crate::{Event, Limits, Request, Store, Stores, Write, codec, format, keys};
use alloc::boxed::Box;
use jig_domain_core::{CoreKey, CoreRecord, Key, Record};
use skein_io::kernel::Fd;
use skein_lib::{Duration, JournalLimits, List, Queue, Token};

pub const OTHER_DEPLOYMENT: [u8; 16] = [9; 16];
pub const ROOT_FD: Fd = Fd::new(0);

fn kv_limits() -> skein_kv::Limits {
    skein_kv::Limits {
        key: KEY_LIMIT,
        value: 16_777_216,
        ops: 16,
        commit: 134_217_728,
        queued: 2,
        queued_bytes: 268_435_456,
        budget: 16_384,
        segment: 268_435_456,
        snapshot_after: 134_217_728,
        chunk: 16_778_260,
        page: skein_kv::Page { rows: 2, bytes: 16_778_240 },
        deadline: Duration::from_nanos(1_000_000_000),
    }
}

fn limits() -> Limits {
    Limits {
        core: CORE_LIMITS,
        store: kv_limits(),
        secrets: kv_limits(),
        journal: JournalLimits { commits: 2, writes: 4, held: 4, now: 4, release: 4 },
    }
}

pub fn stores() -> Stores {
    stores_with(&limits())
}

fn stores_with(limits: &Limits) -> Stores {
    Stores { store: skein_kv::Store::memory(&limits.store), secrets: skein_kv::Store::memory(&limits.secrets) }
}

pub fn plant(stores: &mut Stores, ops: &[skein_kv::Op]) {
    let mut above = Queue::with_capacity(skein_kv::MAX_OUT.above);
    let mut below = Queue::with_capacity(skein_kv::MAX_OUT.below);
    skein_kv::down(
        &mut stores.store,
        &crate::memory_env(kv_limits()),
        skein_kv::Request::Commit { owner: Token::new(0), ops: Box::from(ops) },
        &mut above,
        &mut below,
    );
    assert!(below.is_empty(), "memory commits issue no files");
    let Some(skein_kv::Event::Committed { .. }) = above.pop() else {
        panic!("fixture commit must fit");
    };
    assert!(above.is_empty());
}

pub fn format_put(deployment: [u8; 16]) -> skein_kv::Op {
    skein_kv::Op::Put {
        key: keys::encode_own(&keys::Own::Format, KEY_LIMIT).expect("format key fits"),
        value: format::current(APPLICATION.version, deployment).encode().expect("format fits"),
    }
}

pub fn header_put(deployment: [u8; 16], commits: u64) -> skein_kv::Op {
    let record = deployment_record(deployment, commits);
    skein_kv::Op::Put {
        key: keys::encode(&Key::Core(CoreKey::Deployment), KEY_LIMIT).expect("header key fits"),
        value: codec::encode(&record, &codec::limits(&CORE_LIMITS).expect("fixture limits")).expect("header fits"),
    }
}

fn deployment_record(deployment: [u8; 16], commits: u64) -> Record {
    let mut header = core_rows::deployment();
    header.id = deployment;
    header.commits = commits;
    Record::Core(CoreRecord::Deployment(header))
}

pub fn over(stores: Stores) -> Store {
    Store::over(&limits(), APPLICATION, stores)
}

pub fn run(store: &mut Store, request: Request) -> Box<[Event]> {
    let mut above = Queue::with_capacity(crate::MAX_OUT_DOWN);
    crate::down(store, request, &mut above);
    let mut events = List::with_capacity(crate::MAX_OUT_DOWN);
    for _ in 0..crate::MAX_OUT_DOWN {
        if let Some(event) = above.pop() {
            events.push(event).expect("one terminal");
        }
    }
    events.into_boxed()
}

pub fn opened() -> Store {
    opened_with_budget(16_384)
}

pub fn opened_with_budget(budget: u64) -> Store {
    let mut limits = limits();
    limits.store.budget = budget;
    let mut store = Store::over(&limits, APPLICATION, stores_with(&limits));
    let events = run(&mut store, Request::Open { owner: Token::new(1), root: ROOT_FD });
    let [Event::Opened { owner: _, last: 0 }] = &*events else {
        panic!("empty store opens: {events:?}");
    };
    store
}

pub fn commit(number: u64, writes: Box<[Write]>) -> Request {
    Request::Commit { number, writes }
}

pub fn header(commits: u64) -> Write {
    save(header_record(commits))
}

pub fn save(record: Record) -> Write {
    Write::Save(record)
}

pub fn get_kv(store: &mut Store, key: &[u8]) -> Option<Box<[u8]>> {
    let mut above = Queue::with_capacity(skein_kv::MAX_OUT.above);
    let mut below = Queue::with_capacity(skein_kv::MAX_OUT.below);
    skein_kv::down(
        &mut store.stores.store,
        &crate::memory_env(store.limits.store),
        skein_kv::Request::Get { owner: Token::new(0), key: Box::from(key) },
        &mut above,
        &mut below,
    );
    assert!(below.is_empty(), "memory gets issue no files");
    let Some(skein_kv::Event::Got { value, .. }) = above.pop() else {
        panic!("memory get answers immediately");
    };
    assert!(above.is_empty());
    value
}

pub const ALICE: u64 = 11;
pub const LIVE: u64 = 21;
pub const ENDED: u64 = 22;
pub const PROJECT: u32 = 3;

pub fn committed(store: &mut Store, writes: Box<[Write]>) {
    let number = store.stores.store.last().checked_add(1).expect("fixture next commit");
    let events = run(store, commit(number, writes));
    let [Event::Committed { number: found, .. }] = &*events else {
        panic!("fixture commit fits: {events:?}");
    };
    assert_eq!(*found, number);
}

pub fn header_record(commits: u64) -> Record {
    deployment_record(DEPLOYMENT, commits)
}

pub fn person(number: u64) -> Record {
    Record::People(jig_domain_people::Stored::Person { number, identity: identity() })
}

pub fn live_task(number: u64) -> Record {
    let mut row = task_record();
    row.number = number;
    Record::Tasks(jig_domain_tasks::Stored::Live(Box::new(row)))
}

pub fn ended_task(number: u64) -> Record {
    let mut row = task_record();
    row.number = number;
    row.phase = jig_domain_tasks::Phase::Ended(jig_domain_tasks::Ending::Done(jig_domain_tasks::TaskResult::Report {
        words: Box::from(b"result".as_slice()),
    }));
    Record::Tasks(jig_domain_tasks::Stored::Ended(Box::new(row)))
}

pub fn history(task: u64, revision: u64) -> Record {
    let mut row = task_history();
    row.task = task;
    row.revision = revision;
    Record::Tasks(jig_domain_tasks::Stored::History(row))
}

pub fn run_proof(task: u64) -> Record {
    let mut row = core_rows::run();
    row.task = task;
    Record::Core(CoreRecord::RunProof(row))
}

pub fn call(task: u64, completion: u32) -> Record {
    let CoreRecord::Call(mut row) = calls::call(jig_domain_core::CallPart::Controlled) else {
        panic!("call fixture");
    };
    row.key.task = task;
    row.key.completion = completion;
    Record::Core(CoreRecord::Call(row))
}

pub fn notes_entry(name: u64) -> Record {
    let mut row = note_entry();
    row.name = name;
    Record::Notes(jig_domain_notes::Record::Entry(row))
}

pub fn result_index(party: u64, position: u64, task: u64) -> Record {
    Record::Tasks(jig_domain_tasks::Stored::Result(jig_domain_tasks::ResultRow { party, position, task }))
}

pub fn conversation_index(party: u64, task: u64) -> Record {
    Record::Tasks(jig_domain_tasks::Stored::ConversationOldest(jig_domain_tasks::Conversation { party, task }))
}

pub fn notes_line(scope: &jig_domain_notes::Scope, name: u64) -> Record {
    let mut row = note_line();
    row.scope = scope.clone();
    row.name = name;
    Record::Notes(jig_domain_notes::Record::Line(row))
}
