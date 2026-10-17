use super::spine::{LIMITS, delivered, drive, frame, open_token, start};
use crate::tests::fixtures::{Fixture, ReadPage, Watched};
use crate::{
    Event, Request,
    address::{Address, Place, TaskSection},
    objects::{Card, Objects, Source},
    windows::*,
};
use jig_client::contract::*;
use jig_client::thread::window::{Edge, Edges, End, TurnWindow};
use skein_lib::{List, Time, Token, Wall};

fn rows<T>(values: impl IntoIterator<Item = T>) -> List<T> {
    let mut rows = List::with_capacity(3);
    for value in values {
        rows.push(value).ok().unwrap();
    }
    rows
}
fn text(bytes: &[u8]) -> Text {
    Text::new(Box::from(bytes), 32).unwrap()
}
fn edges() -> Edges {
    Edges {
        before: Edge::More(Cursor::new(Box::from(b"before".as_slice()), 1024).expect("bounded opaque cursor")),
        after: Edge::More(Cursor::new(Box::from(b"after".as_slice()), 1024).expect("bounded opaque cursor")),
    }
}
fn chip(task: u64) -> Chip {
    Chip {
        task,
        title: DocumentText::complete(text(b"title")),
        executor: Executor::Party { party: 7 },
        phase: Phase::Idle,
        details: Box::new(super::objects::phase_details(Phase::Idle)),
    }
}
fn conversation(task: u64) -> ConversationLine {
    ConversationLine {
        cached_replies: 0,
        runtime: Some(Runtime::Idle),
        delegates: 0,
        budget: OwnerBudget {
            numbers: OwnerNumbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 },
            left: 0,
            funder: Funder::Pool { project: 1, person: 7, period: 3 },
            deadline: None,
        },
        chip: chip(task),
        project: 1,
        first_words: DocumentText::complete(text(b"words")),
        unread: Some(Count { value: 0, limited: false }),
        seen: 0,
        last_reply: 0,
        changed: Some(Wall::EPOCH),
    }
}
fn activity(position: u64) -> ActivityLine {
    ActivityLine {
        position,
        at: Wall::EPOCH,
        event: Activity::Decision { object: ObjectKey::Task(1), by: Actor::Party(7), how: Decision::Accepted },
    }
}
fn history(position: u64) -> HistoryRow {
    HistoryRow::Line(Box::new(HistoryLine {
        position,
        task: 1,
        by: Actor::Party(7),
        at: Wall::EPOCH,
        change: HistoryChange::Released,
    }))
}

#[test]
fn navigation_closes_abandons_leaves_and_opens_in_order_keeping_pending_cards() {
    let mut domain = Fixture::new(&LIMITS);
    let frame_watch = start(&mut domain, List::with_capacity(0));
    let initial = delivered(&mut domain, Time::ZERO, frame_watch, frame(7));
    let inbox = open_token(&initial);
    super::pages::finish_initial_inbox_reads(&mut domain, &initial);
    let source = Source::Window { number: 0 };
    let pending = domain.objects.name(ObjectKey::Task(5), source).unwrap();
    domain.objects.get_mut(pending).unwrap().card =
        Card::Deciding { request: Some(Token::new(99)), progress: crate::intents::Progress::InFlight };
    let address = Address::Task { task: 8, section: TaskSection::Activity };
    let moved = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(address) });
    assert_eq!(moved.len(), 5);
    assert_eq!(moved[0], Request::Client { asked: jig_client::Event::Unwatch { handle: inbox } });
    assert_eq!(moved[1], Request::PartsLeft);
    let Request::Client {
        asked:
            jig_client::Event::Watch {
                handle: task_watch,
                subject: jig_client::Watchable::Jig(Subject::Task { task: 8 }),
                ..
            },
    } = moved[2]
    else {
        panic!("task first");
    };
    assert_eq!(moved[3], Request::Show { place: Place::Jig(address), how: crate::How::Replace });
    assert_eq!(
        moved[4],
        Request::Save {
            records: crate::tests::fixtures::one(crate::store::Record::Place {
                version: crate::store::STORE_VERSION,
                party: 7,
                address
            })
        }
    );
    let mut document = super::confirmation::task();
    document.chip.executor = Executor::Agent { charter: 1 };
    let activity_requests = drive(
        &mut domain,
        Time::ZERO,
        Watched { watch: task_watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(document))) },
    );
    let Request::Client {
        asked: jig_client::Event::Watch { subject: jig_client::Watchable::Jig(Subject::Run { task: 8, .. }), .. },
    } = activity_requests[0]
    else {
        panic!("agent run after its document");
    };
    let Request::Client {
        asked:
            jig_client::Event::Read {
                handle: read,
                read: jig_client::Readable::Jig(Read::Activity { task: 8, before: None }),
            },
    } = activity_requests[1]
    else {
        panic!("activity read after its document");
    };
    assert_eq!(domain.objects().find(ObjectKey::Task(5)), Some(pending));
    assert!(domain.objects().get(pending).unwrap().sources.is_empty());
    let gone = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
    assert_eq!(gone.len(), 4);
    assert_eq!(gone[2], Request::Client { asked: jig_client::Event::Unwatch { handle: read } });
    assert_eq!(gone[3], Request::PartsLeft);
    let shown = domain.shown();
    let page = ActivityPage { cut: false, rows: rows([activity(9)]), before: None, after: None };
    assert!(
        drive(
            &mut domain,
            Time::ZERO,
            ReadPage { read, result: ReadResult::Value(ReadValue::Activity(Box::new(page))) }
        )
        .is_empty()
    );
    assert_eq!(domain.shown(), shown);
    assert!(domain.turn_rows().rows.is_empty());
    assert_eq!(domain.frame().party(), Some(7));
}

#[test]
fn source_replacement_keeps_shared_names_and_admits_in_order() {
    let mut objects = Objects::new(2, 2);
    let a = Source::Window { number: 1 };
    let b = Source::Window { number: 2 };
    let first = ObjectKey::Task(1);
    let second = ObjectKey::Task(2);
    let third = ObjectKey::Task(3);
    assert_eq!(objects.replace(a, &[first, second, third]), 1);
    let id = objects.find(first).unwrap();
    assert_eq!(objects.replace(b, &[first]), 0);
    let c = Source::Window { number: 3 };
    assert_eq!(objects.replace(c, &[first]), 1);
    assert!(!objects.named(first, c));
    assert_eq!(objects.replace(a, &[second]), 0);
    assert_eq!(objects.find(first), Some(id));
    assert_eq!(objects.replace(b, &[]), 0);
    assert!(objects.find(first).is_none());
    assert_eq!(objects.replace(a, &[third, first, second]), 1);
    assert!(objects.find(third).is_some());
    assert!(objects.find(first).is_some());
    assert!(objects.find(second).is_none());
    objects.reclaim();
    assert_eq!(objects.replace(a, &[first]), 0);
    assert_eq!(objects.len(), 1);
}

#[test]
fn deployment_parts_are_shown_with_the_page_generation() {
    let mut domain = Fixture::new(&LIMITS);
    let f = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, f, frame(7));
    let moved = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Deployment) });
    let watch = open_token(&moved);
    let part = Part::Deployment { connector: 2 };
    let deployment = Deployment {
        hosts: List::with_capacity(0),
        accounts: List::with_capacity(0),
        capacity: List::with_capacity(0),
        proposals: 0,
        refusals: List::with_capacity(0),
        parts: rows([part.clone()]),
    };
    let out = drive(
        &mut domain,
        Time::ZERO,
        Watched { watch, event: WatchEvent::Delivered(Delivery::Deployment(Box::new(deployment))) },
    );
    assert_eq!(out.as_ref(), &[Request::ShowPart { part, scope: PartScope::Deployment }]);
}

fn object_window_slides_from_both_ends_and_gates_live_rows() {
    let mut window = ObjectWindow::new(2);
    assert_eq!(
        window.slide(
            rows([ObjectKey::Task(1), ObjectKey::Task(2)]),
            Edges { before: Edge::End, after: Edge::End },
            End::After
        ),
        0
    );
    assert_eq!(window.slide(rows([ObjectKey::Task(3)]), edges(), End::After), 1);
    assert_eq!(window.rows.as_slice(), &[ObjectKey::Task(2), ObjectKey::Task(3)]);
    assert_eq!(window.edges, edges());
    assert!(!window.live(rows([ObjectKey::Task(4)]), edges(), End::After));
    assert_eq!(window.slide(rows([ObjectKey::Task(1)]), edges(), End::Before), 1);
    assert_eq!(window.rows.as_slice(), &[ObjectKey::Task(1), ObjectKey::Task(2)]);
    assert_eq!(window.edges, edges());
    window.edges.before = Edge::End;
    assert!(window.live(rows([ObjectKey::Task(0)]), edges(), End::Before));
    assert_eq!(window.rows.as_slice(), &[ObjectKey::Task(0), ObjectKey::Task(1)]);
}

fn conversation_window_slides_from_both_ends_and_gates_live_rows() {
    let mut window = ConversationWindow::new(2);
    assert_eq!(
        window.slide(
            rows([conversation(1), conversation(2)]),
            Edges { before: Edge::End, after: Edge::End },
            End::After
        ),
        0
    );
    assert_eq!(window.slide(rows([conversation(3)]), edges(), End::After), 1);
    assert_eq!(window.rows.as_slice(), &[conversation(2), conversation(3)]);
    assert_eq!(window.edges, edges());
    assert!(!window.live(rows([conversation(4)]), edges(), End::After));
    assert_eq!(window.slide(rows([conversation(1)]), edges(), End::Before), 1);
    assert_eq!(window.rows.as_slice(), &[conversation(1), conversation(2)]);
    assert_eq!(window.edges, edges());
    window.edges.before = Edge::End;
    assert!(window.live(rows([conversation(0)]), edges(), End::Before));
    assert_eq!(window.rows.as_slice(), &[conversation(0), conversation(1)]);
}

fn turn_window_slides_from_both_ends_and_gates_live_rows() {
    let mut window = TurnWindow::new(2);
    assert_eq!(
        window.slide(rows([activity(1), activity(2)]), Edges { before: Edge::End, after: Edge::End }, End::After),
        0
    );
    assert_eq!(window.slide(rows([activity(3)]), edges(), End::After), 1);
    assert_eq!(window.rows.as_slice(), &[activity(2), activity(3)]);
    assert_eq!(window.edges, edges());
    assert!(!window.live(rows([activity(4)]), edges(), End::After));
    assert_eq!(window.slide(rows([activity(1)]), edges(), End::Before), 1);
    assert_eq!(window.rows.as_slice(), &[activity(1), activity(2)]);
    assert_eq!(window.edges, edges());
    window.edges.before = Edge::End;
    assert!(window.live(rows([activity(0)]), edges(), End::Before));
    assert_eq!(window.rows.as_slice(), &[activity(0), activity(1)]);
}

fn history_window_slides_from_both_ends_and_gates_live_rows() {
    let mut window = HistoryWindow::new(2);
    assert_eq!(
        window.slide(rows([history(1), history(2)]), Edges { before: Edge::End, after: Edge::End }, End::After),
        0
    );
    assert_eq!(window.slide(rows([history(3)]), edges(), End::After), 1);
    assert_eq!(window.rows.as_slice(), &[history(2), history(3)]);
    assert_eq!(window.edges, edges());
    assert!(!window.live(rows([history(4)]), edges(), End::After));
    assert_eq!(window.slide(rows([history(1)]), edges(), End::Before), 1);
    assert_eq!(window.rows.as_slice(), &[history(1), history(2)]);
    assert_eq!(window.edges, edges());
    window.edges.before = Edge::End;
    assert!(window.live(rows([history(0)]), edges(), End::Before));
    assert_eq!(window.rows.as_slice(), &[history(0), history(1)]);
}

#[test]
fn every_concrete_window_slides_from_both_ends_and_gates_live_rows() {
    object_window_slides_from_both_ends_and_gates_live_rows();
    conversation_window_slides_from_both_ends_and_gates_live_rows();
    turn_window_slides_from_both_ends_and_gates_live_rows();
    history_window_slides_from_both_ends_and_gates_live_rows();
}

#[test]
fn outbox_parts_keep_their_owner_scope_in_pages_and_retained_application_objects() {
    for owner in [PartScope::Project(41), PartScope::Deployment] {
        let mut domain = Fixture::new(&LIMITS);
        let f = start(&mut domain, List::with_capacity(0));
        delivered(&mut domain, Time::ZERO, f, frame(7));
        let part = Part::Outbox { connector: 2, scope: owner, entry: u64::MAX };
        let moved = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Deployment) });
        let watch = open_token(&moved);
        let deployment = Deployment {
            hosts: List::with_capacity(0),
            accounts: List::with_capacity(0),
            capacity: List::with_capacity(0),
            proposals: 0,
            refusals: List::with_capacity(0),
            parts: rows([part.clone()]),
        };
        let out = drive(
            &mut domain,
            Time::ZERO,
            Watched { watch, event: WatchEvent::Delivered(Delivery::Deployment(Box::new(deployment))) },
        );
        assert_eq!(out.as_ref(), &[Request::ShowPart { part: part.clone(), scope: owner }]);
        drive(
            &mut domain,
            Time::ZERO,
            Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Overview }) },
        );
        assert_eq!(crate::application::scope(&domain.ui, &part, &domain.client), Some(owner));
        let task = super::confirmation::task();
        assert_eq!(crate::application::owner_scope(&part, PartScope::Project(task.project)), owner);
        // The same supplied scope survives a jig object retained on an application page.
        drive(&mut domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
        let shown = drive(
            &mut domain,
            Time::ZERO,
            Event::Named { sections: List::with_capacity(0), objects: rows([ObjectKey::Task(8)]) },
        );
        let mut task_read = None;
        for request in &shown {
            if let Request::Client {
                asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Task { task: 8 }) },
            } = request
            {
                task_read = Some(*read);
            }
        }
        let read = task_read.unwrap();
        let mut task = super::confirmation::task();
        task.resources = rows([TaskResource::Part(part.clone())]);
        let shown = drive(
            &mut domain,
            Time::ZERO,
            ReadPage { read, result: ReadResult::Value(ReadValue::Task(Box::new(task))) },
        );
        assert!(shown.contains(&Request::ShowPart { part, scope: owner }));
    }
}
