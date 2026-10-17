use crate::address::Address;
use crate::records::{RECORDS_VERSION, Record};
use jig_client::contract::Text;
pub const PARTY: u64 = u64::MAX;
pub const TASK: u64 = u64::MAX;
pub fn place() -> Record {
    Record::Place { version: RECORDS_VERSION, party: PARTY, address: Address::Deployment }
}
pub fn draft() -> Record {
    Record::Draft {
        version: RECORDS_VERSION,
        party: PARTY,
        task: TASK,
        words: Text::new(Box::from(b"words".as_slice()), 65536).unwrap(),
        touched: u64::MAX,
    }
}
pub fn places() -> skein_lib::List<Record> {
    use crate::address::{Address, ProjectSection, TaskSection};
    let mut addresses = skein_lib::List::with_capacity(21);
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
    let mut saved = skein_lib::List::with_capacity(addresses.len());
    for address in addresses.into_boxed() {
        saved.push(Record::Place { version: RECORDS_VERSION, party: u64::MAX, address }).unwrap();
    }
    saved
}

/// Four opaque segments expose a display cut while preserving every byte.
#[must_use]
pub(crate) fn long_resource_path<const N: usize>() -> skein_lib::List<Box<[u8]>> {
    let mut path = skein_lib::List::with_capacity(4);
    for byte in [0xF0_u8, 0xF1, 0xF2, 0xF3] {
        path.push(Box::from([byte; N])).unwrap();
    }
    path
}

/// The application's part effects, preserving their exact fields.
pub(crate) fn parts(output: &[crate::Request], capacity: u32) -> Box<[crate::Request]> {
    let mut parts = skein_lib::List::with_capacity(capacity);
    for request in output {
        if let crate::Request::ShowPart { part, scope } = request {
            parts.push(crate::Request::ShowPart { part: part.clone(), scope: *scope }).unwrap();
        }
    }
    parts.into_boxed()
}

mod client;
pub(crate) use client::MOST;
pub(crate) use client::{
    AnsweredByEngine, CardAsk, Ended, FireClient, Fixture, Input, ReadPage, Reported, SendIntent, Served, Tick,
    Watched, intent, kept,
};

pub(crate) fn one<T>(value: T) -> skein_lib::List<T> {
    let mut list = skein_lib::List::with_capacity(1);
    list.push(value).ok().unwrap();
    list
}

/// A send and the release the displayed hold names share one request.
pub(crate) fn releasing(
    ask: jig_client::contract::Ask,
    revision: Option<jig_client::contract::Revision>,
) -> jig_client::contract::Ask {
    use jig_client::contract::{Ask, Operation};
    let Some(revision) = revision else {
        return ask;
    };
    let (project, task, first) = if let Ask::Message { project, task, words } = ask {
        (project, task, Operation::Message { project, task, words })
    } else {
        let Ask::Amend { project, task, revision, change, reason } = ask else {
            panic!("message or amendment");
        };
        (project, task, Operation::Amend { project, task, revision, change, reason })
    };
    let mut operations = skein_lib::List::with_capacity(2);
    operations.push(first).unwrap();
    operations.push(Operation::Release { project, task, revision, again: None }).unwrap();
    Ask::Compound { project, operations }
}
pub(crate) fn selected_change(ask: &jig_client::contract::Ask) -> Option<&jig_client::contract::Amendment> {
    use jig_client::contract::{Ask, Operation};
    if let Ask::Amend { change, .. } = ask {
        return Some(change);
    }
    let Ask::Compound { operations, .. } = ask else {
        return None;
    };
    let [Operation::Amend { change, .. }, Operation::Release { .. }] = operations.as_slice() else {
        return None;
    };
    Some(change)
}

pub(crate) fn refusal_answer(refusal: jig_client::contract::Refusal) -> jig_client::contract::Answer {
    jig_client::contract::Answer::Refused { refusal, at: None }
}
