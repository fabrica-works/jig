use super::{objects, spine::drive};
use crate::tests::fixtures::{Ended, Fixture, ReadPage};
use crate::{
    Event, Request,
    address::{Address, Place, TaskSection},
    objects::Body,
    page::Page,
};
use jig_client::contract::*;
use skein_lib::{List, Time};

fn task_watch(domain: &mut Fixture, task: u64) -> jig_client::Handle {
    let output = drive(domain, Time::ZERO, Event::Went { place: Place::Jig(Address::task(task)) });
    let mut token = None;
    for request in &output {
        if let Request::Client {
            asked:
                jig_client::Event::Watch {
                    handle: watch,
                    subject: jig_client::Watchable::Jig(Subject::Task { task: found }),
                },
        } = request
            && *found == task
        {
            assert!(token.replace(*watch).is_none());
        }
    }
    token.unwrap()
}
fn terminal(domain: &mut Fixture, watch: jig_client::Handle, end: WatchEnd) -> Box<[Request]> {
    drive(domain, Time::ZERO, Ended { watch, end })
}
fn task_read(output: &[Request]) -> jig_client::Handle {
    let mut token = None;
    for request in output {
        if let Request::Client {
            asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Task { task: 8 }) },
        } = request
        {
            assert!(token.replace(*read).is_none());
        }
    }
    token.unwrap()
}
fn work_address(domain: &Fixture) -> &Address {
    match domain.page() {
        Page::Work { address, .. } => address,
        Page::Starting | Page::Elsewhere | Page::SigningIn | Page::Missing | Page::OutOfDate | Page::SomeoneElse => {
            panic!("expected work page")
        }
    }
}
fn no_reads(output: &[Request]) {
    for request in output {
        if let Request::Client { asked: jig_client::Event::Read { .. } } = request {
            panic!("unexpected read");
        }
    }
}
fn historical() -> OwnerHistorical {
    let mut document = super::confirmation::task();
    document.context = None;
    OwnerHistorical {
        key: ObjectKey::Task(8),
        value: OwnerHistoricalValue::Task {
            result: Box::new(Result {
                task: document.chip.clone(),
                ending: Ending::Done,
                value: Some(ResultValue::Report(DocumentText { bytes: objects::text(UTF8_TEXT), cut: true })),
                at: None,
                details: None,
                acts: List::with_capacity(0),
            }),
            document: Box::new(document),
        },
        at: None,
        by: None,
    }
}
#[test]
fn a_fresh_unknown_task_watch_reads_the_whole_retained_task_and_independent_result() {
    for end in [WatchEnd::Unknown, WatchEnd::SubjectEnded] {
        let (mut domain, _) = objects::live();
        let watch = task_watch(&mut domain, 8);
        let read = task_read(&terminal(&mut domain, watch, end));
        assert_eq!(work_address(&domain), &Address::task(8));
        assert_eq!(domain.task_page().header, domain.objects().find(ObjectKey::Task(8)));
        assert!(domain.task_page().header.is_some());
        let original = historical();
        drive(
            &mut domain,
            Time::ZERO,
            ReadPage { read, result: ReadResult::Value(ReadValue::Historical(Box::new(original.clone()))) },
        );
        let id = domain.objects().find(ObjectKey::Task(8)).unwrap();
        assert_eq!(domain.task_page().header, Some(id));
        assert_eq!(domain.objects().get(id).unwrap().body, Some(Body::Historical(Box::new(original.clone()))));
        assert_eq!(work_address(&domain), &Address::task(8));
        assert!(terminal(&mut domain, watch, end).is_empty());
        drive(&mut domain, Time::ZERO, ReadPage { read, result: ReadResult::None(ReadNone::Gone) });
        assert_eq!(domain.objects().get(id).unwrap().body, Some(Body::Historical(Box::new(original))));
        assert_eq!(work_address(&domain), &Address::task(8));
    }
}
#[test]
fn revisiting_an_already_retained_task_attaches_and_refreshes_its_new_page_header() {
    let (mut domain, _) = objects::live();
    let watch = task_watch(&mut domain, 8);
    let read = task_read(&terminal(&mut domain, watch, WatchEnd::SubjectEnded));
    let original = historical();
    drive(
        &mut domain,
        Time::ZERO,
        ReadPage { read, result: ReadResult::Value(ReadValue::Historical(Box::new(original.clone()))) },
    );
    let output = drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Activity }) },
    );
    let watch = super::spine::open_token(&output);
    assert!(domain.task_page().header.is_none());
    let before = domain.shown();
    let read = task_read(&terminal(&mut domain, watch, WatchEnd::Unknown));
    assert!(domain.shown() > before);
    let id = domain.objects().find(ObjectKey::Task(8)).unwrap();
    assert_eq!(domain.task_page().header, Some(id));
    assert_eq!(domain.objects().get(id).unwrap().body, Some(Body::Historical(Box::new(original.clone()))));
    drive(
        &mut domain,
        Time::ZERO,
        ReadPage { read, result: ReadResult::Value(ReadValue::Historical(Box::new(original.clone()))) },
    );
    assert_eq!(domain.task_page().header, Some(id));
    assert_eq!(domain.objects().get(id).unwrap().body, Some(Body::Historical(Box::new(original))));
}
#[test]
fn an_unknown_task_becomes_missing_only_after_its_retained_read_refuses_the_object() {
    for none in [ReadNone::Unknown, ReadNone::Gone, ReadNone::Refused] {
        let (mut domain, _) = objects::live();
        let watch = task_watch(&mut domain, 8);
        let read = task_read(&terminal(&mut domain, watch, WatchEnd::Unknown));
        assert_eq!(work_address(&domain), &Address::task(8));
        drive(&mut domain, Time::ZERO, ReadPage { read, result: ReadResult::None(none) });
        assert_eq!(domain.page(), &Page::Missing);
    }
}
#[test]
fn a_forbidden_task_watch_does_not_probe_retained_work() {
    let (mut domain, _) = objects::live();
    let watch = task_watch(&mut domain, 8);
    let output = terminal(&mut domain, watch, WatchEnd::NotAllowed);
    no_reads(&output);
    assert_eq!(domain.page(), &Page::Missing);
}
#[test]
fn an_unknown_task_watch_closed_by_navigation_does_not_read_or_hide_the_new_page() {
    let (mut domain, _) = objects::live();
    let watch = task_watch(&mut domain, 8);
    task_watch(&mut domain, 9);
    let output = terminal(&mut domain, watch, WatchEnd::Unknown);
    no_reads(&output);
    assert_eq!(work_address(&domain), &Address::task(9));
}
#[test]
fn a_retained_task_reply_abandoned_by_navigation_cannot_replace_the_new_page() {
    for result in
        [ReadResult::Value(ReadValue::Historical(Box::new(historical()))), ReadResult::None(ReadNone::Unknown)]
    {
        let (mut domain, _) = objects::live();
        let watch = task_watch(&mut domain, 8);
        let read = task_read(&terminal(&mut domain, watch, WatchEnd::Unknown));
        task_watch(&mut domain, 9);
        drive(&mut domain, Time::ZERO, ReadPage { read, result });
        assert_eq!(work_address(&domain), &Address::task(9));
        assert!(domain.objects().find(ObjectKey::Task(8)).is_none());
    }
}
#[test]
fn an_unknown_non_task_page_watch_does_not_issue_a_retained_task_read() {
    let (mut domain, inbox) = objects::live();
    let output = terminal(&mut domain, inbox, WatchEnd::Unknown);
    for request in &output {
        if let Request::Client {
            asked: jig_client::Event::Read { read: jig_client::Readable::Jig(Read::Task { .. }), .. },
        } = request
        {
            panic!("unexpected retained Task read");
        }
    }
    assert_eq!(work_address(&domain), &Address::Inbox { project: None });
}

const UTF8_TEXT: &[u8] = b"~\0\x07";
