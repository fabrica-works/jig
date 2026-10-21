#![expect(
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary tests select public read requests and inspect semantic trees"
)]
use super::{
    cards::{contains, drive, words},
    pages::{build, deliver},
    work_pages::{entered, list, project_doc, task},
};
use crate::tests::fixtures::{Fixture, ReadPage};
use crate::{
    Event, Request,
    action::Act,
    address::{Address, Place, ProjectSection, TaskSection},
};
use jig_client::contract::*;
use jig_client::words::{DecisionWord as D, Word};
use skein_lib::{List, Token};

fn cursor() -> Cursor {
    Cursor::new(Box::from(UTF8_TEXTS[0]), 1024).expect("bounded opaque cursor")
}
fn read(out: &[Request], kind: u8) -> jig_client::Handle {
    out.iter()
        .find_map(|request| match request {
            Request::Client {
                asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(what) },
            } => match (kind, what) {
                (0, Read::Ended { .. })
                | (1, Read::History { .. })
                | (2, Read::Activity { .. })
                | (3, Read::Inbox { group: InboxReadGroup::Needs, .. })
                | (4, Read::Tasks { .. })
                | (5, Read::Tasks { .. })
                | (6, Read::Spend { .. })
                | (7, Read::Notes { .. }) => Some(*read),
                _ => None,
            },
            _ => None,
        })
        .unwrap()
}

fn ready(kind: u8) -> (Fixture, jig_client::Handle) {
    let (mut domain, mut out) = match kind {
        0 => task(TaskSection::Plan, Executor::Agent { charter: 2 }),
        1 => task(TaskSection::History, Executor::Agent { charter: 2 }),
        2 => task(TaskSection::Activity, Executor::Agent { charter: 2 }),
        3 => entered(Address::Inbox { project: None }),
        4 => entered(Address::Conversations { project: None }),
        5..=7 => entered(Address::Project {
            project: 4,
            section: match kind {
                5 => ProjectSection::Goals,
                6 => ProjectSection::Spend,
                _ => ProjectSection::Notes,
            },
        }),
        _ => panic!("eight page kinds"),
    };
    if kind == 0 {
        let tree_watch = out
            .iter()
            .find_map(|request| match request {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Tree { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
        let mut chip = super::work_pages::chip(9);
        chip.phase = Phase::Done;
        let counts =
            PhaseCounts { waiting: 0, running: 0, held: 0, closing: 0, ended: 0, done: 0, failed: 0, cancelled: 0 };
        deliver(
            &mut domain,
            tree_watch,
            Delivery::Tree(Box::new(Tree {
                task: 8,
                nodes: list([TreeNode {
                    chip,
                    parent: None,
                    dependencies: list([]),
                    proposals: list([]),
                    spend: 0,
                    folded: counts,
                    held_below: 0,
                }]),
                folded: counts,
                held_below: 0,
            })),
        );
        out.extend(drive(&mut domain, Event::Act { action: Act::OpenEnded { task: 9 } }));
    }
    if kind >= 5 {
        let project = out
            .iter()
            .find_map(|request| match request {
                Request::Client {
                    asked:
                        jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Project { .. }) },
                } => Some(*read),
                _ => None,
            })
            .unwrap();
        drive(
            &mut domain,
            ReadPage { read: project, result: ReadResult::Value(ReadValue::Project(Box::new(project_doc()))) },
        );
    }
    if kind == 4 {
        out.extend(drive(
            &mut domain,
            Event::Act {
                action: Act::More {
                    window: Token::new(crate::page::conversations::CONVERSATIONS_WINDOW),
                    end: jig_client::thread::window::End::After,
                },
            },
        ));
    }
    let token = read(&out, kind);
    (domain, token)
}
fn value(kind: u8, cut: bool, after: Option<Cursor>) -> ReadValue {
    match kind {
        0 => ReadValue::Ended(Box::new(ObjectPage { cut, rows: List::with_capacity(2), before: None, after })),
        1 => ReadValue::History(Box::new(HistoryPage { cut, rows: List::with_capacity(2), before: None, after })),
        2 => ReadValue::Activity(Box::new(ActivityPage { cut, rows: List::with_capacity(2), before: None, after })),
        3 => ReadValue::Inbox(Box::new(InboxPage { cut, rows: List::with_capacity(2), before: None, after })),
        4 => ReadValue::Tasks(Box::new(TaskPage {
            live: List::with_capacity(0),
            live_after: None,
            ended: EndedGroup::Rows { rows: List::with_capacity(2), cut, after },
        })),
        5 => ReadValue::Tasks(Box::new(TaskPage {
            live: List::with_capacity(0),
            live_after: None,
            ended: EndedGroup::Rows { rows: List::with_capacity(2), after, cut },
        })),
        6 => ReadValue::Spend(Box::new(SpendPage { cut, rows: List::with_capacity(2), before: None, after })),
        7 => ReadValue::Notes(Box::new(NotePage { cut, rows: List::with_capacity(2), before: None, after })),
        _ => panic!("eight page kinds"),
    }
}
fn held(domain: &Fixture, kind: u8) -> bool {
    match kind {
        0 => domain.task_page().ended_cut,
        1 => domain.history_rows().cut,
        2 => domain.conversation().page_cut,
        3 => domain.inbox().needs_window.cut,
        4 => domain.conversation_rows().cut,
        5 => domain.project_page().ended_cut,
        6 => domain.project_page().spend_cut,
        7 => domain.project_page().notes_cut,
        _ => panic!("eight page kinds"),
    }
}
fn continuation(domain: &Fixture, kind: u8) -> Option<Cursor> {
    match kind {
        0 => domain.task_page().ended_after.clone(),
        1 => match &domain.history_rows().edges.after {
            jig_client::thread::window::Edge::More(cursor) => Some(cursor.clone()),
            jig_client::thread::window::Edge::End => None,
        },
        // Activity's reverse document end is the newer end of the held log.
        2 => match &domain.conversation().lines.edges.before {
            jig_client::thread::window::Edge::More(cursor) => Some(cursor.clone()),
            jig_client::thread::window::Edge::End => None,
        },
        3 => domain.inbox().needs_window.after.clone(),
        4 => match &domain.conversation_rows().edges.after {
            jig_client::thread::window::Edge::More(cursor) => Some(cursor.clone()),
            jig_client::thread::window::Edge::End => None,
        },
        5 => domain.project_page().ended_after.clone(),
        6 => domain.project_page().spend_after.clone(),
        7 => domain.project_page().notes_after.clone(),
        _ => panic!("eight page kinds"),
    }
}

#[test]
fn every_native_page_retains_source_loss_and_displays_it_independently_of_continuation() {
    let words = words();
    for kind in 0..8 {
        for (cut, after) in [(true, Some(cursor())), (true, None), (false, Some(cursor())), (false, None)] {
            let (mut domain, token) = ready(kind);
            let source = value(kind, cut, after.clone());
            drive(&mut domain, ReadPage { read: token, result: ReadResult::Value(source) });
            assert_eq!(held(&domain, kind), cut, "native page kind {kind}");
            assert_eq!(continuation(&domain, kind), after, "opaque continuation for page kind {kind}");
            let tree = build(&domain, 3, words).0;
            assert_eq!(contains(&tree, b"This page is incomplete"), cut, "public page kind {kind}");
            assert_eq!(
                tree.nodes()
                    .iter()
                    .filter(
                        |n| n.role == skein_ui::Role::Status && n.name.as_deref() == Some(b"This page is incomplete")
                    )
                    .count(),
                usize::from(cut)
            );
            assert!(tree.len() <= crate::page_nodes(&super::cards::CLIENT, 0).unwrap());
            drive(&mut domain, Event::Went { place: Place::Jig(Address::Conversations { project: None }) });
            assert!(!contains(&build(&domain, 3, words).0, b"This page is incomplete"));
        }
    }
}

#[test]
fn sliding_a_read_window_keeps_supplied_page_loss_until_that_source_is_reset() {
    for (kind, window) in
        [(1, crate::page::task::HISTORY_WINDOW), (4, crate::page::conversations::CONVERSATIONS_WINDOW)]
    {
        let (mut domain, token) = ready(kind);
        drive(&mut domain, ReadPage { read: token, result: ReadResult::Value(value(kind, true, Some(cursor()))) });
        let out = drive(
            &mut domain,
            Event::Act {
                action: Act::More { window: Token::new(window), end: jig_client::thread::window::End::After },
            },
        );
        let token = read(&out, kind);
        drive(&mut domain, ReadPage { read: token, result: ReadResult::Value(value(kind, false, None)) });
        assert!(held(&domain, kind), "a later page says nothing about the previous page's missing content");
        assert!(contains(&build(&domain, 3, words()).0, b"This page is incomplete"));
        if kind == 1 {
            drive(&mut domain, Event::Act { action: Act::HistoryScope { subtree: true } });
            assert!(!held(&domain, kind));
        }
    }
}

fn row_selected_words() -> crate::words::Words {
    use skein_ui::phrases::{PhraseLimits, Phrasebook, Segment};
    let first = crate::phrases::english::language();
    let mut second = first.clone();
    for phrase in &mut second.phrases {
        for form in &mut phrase.forms {
            form.segments.reverse();
            for segment in &mut form.segments {
                if let Segment::Text(bytes) = segment {
                    let mut selected = b"ALT ".to_vec();
                    selected.extend_from_slice(bytes);
                    *bytes = selected.into_boxed_slice();
                }
            }
        }
    }
    let mut segments = 0_u32;
    let mut text_bytes = 0_u32;
    for language in [&first, &second] {
        for phrase in &language.phrases {
            for form in &phrase.forms {
                for segment in &form.segments {
                    segments = segments.checked_add(1).unwrap();
                    if let Segment::Text(bytes) = segment {
                        text_bytes = text_bytes.checked_add(u32::try_from(bytes.len()).unwrap()).unwrap();
                    }
                }
            }
        }
    }
    crate::words::Words::new(
        Phrasebook::load(
            crate::phrases::jig::shapes(super::cards::CLIENT.text),
            Box::new([first, second]),
            &PhraseLimits {
                languages: 2,
                phrases: crate::owner_words::phrase_count(),
                holes: 4,
                segments,
                text_bytes,
                hole_bytes: super::cards::CLIENT.text,
            },
        )
        .unwrap(),
        1,
        0,
    )
}

#[test]
fn owner_goal_rows_render_every_distinct_stage_without_generic_ended_and_keep_public_finite_capacity() {
    let (mut domain, read) = ready(5);
    let row = TaskLine {
        chip: super::work_pages::chip(9),
        tracked: Some(u32::MAX),
        phases: OwnerPhaseCounts {
            waiting: u32::MAX,
            idle: u32::MAX - 1,
            starting: u32::MAX - 2,
            running: u32::MAX - 3,
            backing_off: u32::MAX - 4,
            closing: u32::MAX - 5,
            held: u32::MAX - 6,
            done: u32::MAX - 7,
            failed: u32::MAX - 8,
            cancelled: u32::MAX - 9,
        },
        budget: super::cards_objects::task_doc(&[]).budget,
        held_below: u32::MAX,
        changed: skein_lib::Wall::EPOCH,
        last_activity: skein_lib::Wall::EPOCH,
        ended_at: None,
        project: 1,
        requester: OwnerRequester::Person(7),
        label: 2,
        flags: TaskFlags { ended_by: EndedBy::Result, conversation: false, listed: true, retention: Retention::Kept },
        waits_on: WaitsOn::Nothing,
        conversation: None,
    };
    drive(
        &mut domain,
        ReadPage {
            read,
            result: ReadResult::Value(ReadValue::Tasks(Box::new(TaskPage {
                live: List::with_capacity(0),
                live_after: None,
                ended: EndedGroup::Rows { rows: list([row.clone()]), after: Some(cursor()), cut: false },
            }))),
        },
    );
    let selected = row_selected_words();
    let tree = build(&domain, 3, &selected).0;
    for phrase_and_count in [
        (crate::phrases::jig::JigPhrase::Word(Word::Decision(D::TasksWaiting)), u32::MAX),
        (crate::phrases::jig::JigPhrase::TasksIdle, u32::MAX - 1),
        (crate::phrases::jig::JigPhrase::TasksStarting, u32::MAX - 2),
        (crate::phrases::jig::JigPhrase::Word(Word::Decision(D::TasksRunning)), u32::MAX - 3),
        (crate::phrases::jig::JigPhrase::TasksBackingOff, u32::MAX - 4),
        (crate::phrases::jig::JigPhrase::Word(Word::Decision(D::TasksClosing)), u32::MAX - 5),
        (crate::phrases::jig::JigPhrase::Word(Word::Decision(D::TasksHeld)), u32::MAX - 6),
        (crate::phrases::jig::JigPhrase::Word(Word::Decision(D::TasksDone)), u32::MAX - 7),
        (crate::phrases::jig::JigPhrase::Word(Word::Decision(D::TasksFailed)), u32::MAX - 8),
        (crate::phrases::jig::JigPhrase::Word(Word::Decision(D::TasksCancelled)), u32::MAX - 9),
    ] {
        assert!(contains(
            &tree,
            &selected.say(phrase_and_count.0, &[skein_ui::phrases::Hole::Number(u64::from(phrase_and_count.1))])
        ));
    }
    assert!(!contains(
        &tree,
        &selected.say(
            crate::phrases::jig::JigPhrase::Word(Word::Decision(D::TasksEnded)),
            &[skein_ui::phrases::Hole::Number(1)]
        )
    ));
    assert_eq!(domain.project_page().listed_goals.get(0), Some(&row));
    assert!(tree.len() <= crate::page_nodes(&super::cards::CLIENT, 0).unwrap());
}

const UTF8_TEXTS: [&[u8]; 2] = [b"\x7e/\0opaque", b"\x7e\0whole words"];
