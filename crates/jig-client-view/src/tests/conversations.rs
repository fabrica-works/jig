use super::{
    cards::drive,
    fixtures::{Fixture, ReadPage, task_line},
    pages::deliver,
    work_pages::{entered, list},
};
use crate::{Event, Request, action::Act, address::Address};
use jig_client::contract::*;
use jig_client::thread::window::End;
use skein_lib::{Token, Wall};
const FIRST: u64 = 8;
const SECOND: u64 = 9;
const PERSON: u64 = 1;
const PROJECT: u32 = 4;
const LATER: Wall = Wall::from_nanos(99);
fn line(task: u64, at: Wall) -> TaskLine {
    let mut chip = super::work_pages::chip(task);
    chip.phase = Phase::Idle;
    chip.details = Box::new(ChipPhase::Owner {
        phase: OwnerChipPhase::Active(RunStage::Idle),
        details: Box::new(super::owner_chip::details()),
    });
    task_line(chip, PROJECT, OwnerRequester::Person(PERSON), at)
}
fn older(domain: &mut Fixture) -> (jig_client::Handle, Read) {
    let out = drive(
        domain,
        Event::Act {
            action: Act::More { window: Token::new(crate::page::conversations::CONVERSATIONS_WINDOW), end: End::After },
        },
    );
    for request in out {
        if let Request::Client { asked: jig_client::Event::Read { handle, read: jig_client::Readable::Jig(read) } } =
            request
        {
            return (handle, read);
        }
    }
    panic!("one ended task query read");
}
#[test]
fn conversations_keep_the_owners_order_without_reading_the_ended_group() {
    let (mut domain, out) = entered(Address::Conversations { project: None });
    let mut watched = None;
    for request in &out {
        if let Request::Client { asked: jig_client::Event::Read { .. } } = request {
            panic!("opening conversations must not read ended tasks");
        }
        if let Request::Client {
            asked:
                jig_client::Event::Watch { handle, subject: jig_client::Watchable::Jig(Subject::Tasks { query, .. }) },
        } = request
        {
            assert_eq!(*query, crate::queries::conversations(PERSON, 1, None));
            watched = Some(*handle);
        }
    }
    let watch = watched.expect("live task query watched");
    let rows = list([line(FIRST, Wall::EPOCH), line(SECOND, LATER)]);
    deliver(&mut domain, watch, Delivery::Tasks(Box::new(Tasks { rows: rows.clone(), more: false })));
    assert_eq!(domain.conversation_rows().rows, rows);
}
#[test]
fn older_reads_the_ended_query_then_its_exact_opaque_continuation() {
    let (mut domain, _) = entered(Address::Conversations { project: Some(PROJECT) });
    let (handle, read) = older(&mut domain);
    let mut query = crate::queries::conversations(PERSON, 1, Some(PROJECT));
    query.phase = Some(PhaseClass::Ended);
    query.order = QueryOrder::Ended;
    assert_eq!(read, Read::Tasks { query, live: None, ended: None });
    let next = Cursor::new(b"opaque next".as_slice().into(), 32).unwrap();
    let rows = list([line(FIRST, Wall::EPOCH)]);
    drive(
        &mut domain,
        ReadPage {
            read: handle,
            result: ReadResult::Value(ReadValue::Tasks(Box::new(TaskPage {
                live: list([]),
                live_after: None,
                ended: EndedGroup::Rows { rows: rows.clone(), after: Some(next.clone()), cut: true },
            }))),
        },
    );
    assert_eq!(domain.conversation_rows().rows, rows);
    assert!(domain.conversation_rows().cut);
    let words = super::cards::words();
    let (tree, _) = super::pages::build(&domain, 1, words);
    let mut earlier = false;
    for node in tree.nodes() {
        earlier |= node.name.as_deref() == Some(b"Read earlier");
        assert_ne!(node.name.as_deref(), Some(b"Read later".as_slice()));
    }
    assert!(earlier);
    assert_eq!(older(&mut domain).1, Read::Tasks { query, live: None, ended: Some(next) });
}

#[test]
fn a_project_tasks_section_keeps_queries_filtered_by_requester_and_label() {
    let query = crate::queries::conversations(PERSON, 1, Some(PROJECT));
    let subject = Subject::Tasks { query, window: 2 };
    let section = crate::page::Section::Project { project: PROJECT, section: crate::address::ProjectSection::Tasks };
    assert!(crate::page::shown::uses(section, &subject));
}
