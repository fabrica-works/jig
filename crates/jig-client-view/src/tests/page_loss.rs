#![expect(
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary tests select public read requests and inspect semantic trees"
)]
use super::{
    cards::{contains, drive, text, words},
    pages::{build, deliver},
    work_pages::{entered, list, project_doc, task},
};
use jig_client::{
    Domain, Event, Request,
    action::Act,
    address::{Address, Place, ProjectSection, TaskSection},
    contract::*,
};
use skein_lib::{List, Token};

fn cursor() -> Cursor {
    Cursor { value: text(b"\xff/\0opaque") }
}
fn read(out: &[Request], kind: u8) -> Token {
    out.iter()
        .find_map(|request| match request {
            Request::Read { read, what } => match (kind, what) {
                (0, Read::Ended { .. })
                | (1, Read::History { .. })
                | (2, Read::Activity { .. })
                | (3, Read::Inbox { group: InboxReadGroup::Needs, .. })
                | (4, Read::Conversations { .. })
                | (5, Read::Goals { .. })
                | (6, Read::Spend { .. })
                | (7, Read::Notes { .. }) => Some(*read),
                _ => None,
            },
            _ => None,
        })
        .unwrap()
}

fn ready(kind: u8) -> (Domain, Token) {
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
                Request::Open { watch, subject: Subject::Tree { .. } } => Some(*watch),
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
        out.extend(drive(&mut domain, Event::Act { act: Act::OpenEnded { task: 9 } }));
    }
    if kind >= 5 {
        let project = out
            .iter()
            .find_map(|request| match request {
                Request::Read { read, what: Read::Project { .. } } => Some(*read),
                _ => None,
            })
            .unwrap();
        drive(
            &mut domain,
            Event::Read { read: project, result: ReadResult::Value(ReadValue::Project(Box::new(project_doc()))) },
        );
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
        4 => ReadValue::Conversations(Box::new(ConversationPage {
            cut,
            rows: List::with_capacity(2),
            before: None,
            after,
        })),
        5 => ReadValue::Goals(Box::new(GoalPage { cut, rows: List::with_capacity(2), before: None, after })),
        6 => ReadValue::Spend(Box::new(SpendPage { cut, rows: List::with_capacity(2), before: None, after })),
        7 => ReadValue::Notes(Box::new(NotePage { cut, rows: List::with_capacity(2), before: None, after })),
        _ => panic!("eight page kinds"),
    }
}
fn held(domain: &Domain, kind: u8) -> bool {
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
fn continuation(domain: &Domain, kind: u8) -> Option<Cursor> {
    match kind {
        0 => domain.task_page().ended_after.clone(),
        1 => match &domain.history_rows().edges.after {
            jig_client::windows::Edge::More(cursor) => Some(cursor.clone()),
            jig_client::windows::Edge::End => None,
        },
        // Activity's reverse document end is the newer end of the held log.
        2 => match &domain.conversation().lines.edges.before {
            jig_client::windows::Edge::More(cursor) => Some(cursor.clone()),
            jig_client::windows::Edge::End => None,
        },
        3 => domain.inbox().needs_window.after.clone(),
        4 => match &domain.conversation_rows().edges.after {
            jig_client::windows::Edge::More(cursor) => Some(cursor.clone()),
            jig_client::windows::Edge::End => None,
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
            drive(&mut domain, Event::Read { read: token, result: ReadResult::Value(source) });
            assert_eq!(held(&domain, kind), cut, "native page kind {kind}");
            assert_eq!(continuation(&domain, kind), after, "opaque continuation for page kind {kind}");
            let tree = build(&domain, 3, &words).0;
            assert_eq!(contains(&tree, b"This page is incomplete"), cut, "public page kind {kind}");
            assert_eq!(
                tree.nodes()
                    .iter()
                    .filter(|n| n.role == crate::Role::Status && n.name.as_deref() == Some(b"This page is incomplete"))
                    .count(),
                usize::from(cut)
            );
            assert!(tree.len() <= crate::page_nodes(&super::cards::CLIENT, 0).unwrap());
            drive(&mut domain, Event::Went { place: Place::Jig(Address::Conversations { project: None }) });
            assert!(!contains(&build(&domain, 3, &words).0, b"This page is incomplete"));
        }
    }
}
#[test]
fn a_complete_conversation_page_preserves_independent_row_text_loss_and_opaque_continuation() {
    let (mut domain, token) = ready(4);
    let mut chip = super::work_pages::chip(8);
    chip.title.cut = true;
    let row = ConversationLine {
        runtime: Runtime::Idle,
        delegates: 0,
        budget: OwnerBudget {
            numbers: OwnerNumbers { budget: 20, spent: 0, spent_below: 0, reserved: 0 },
            left: 20,
            funder: Funder::Pool { project: 4, person: 1, period: 3 },
            deadline: None,
        },
        chip,
        project: 4,
        first_words: text(b"whole first words"),
        unread: Count { value: 0, limited: false },
        seen: 7,
        last_reply: 7,
        changed: skein_lib::Wall::EPOCH,
    };
    drive(
        &mut domain,
        Event::Read {
            read: token,
            result: ReadResult::Value(ReadValue::Conversations(Box::new(ConversationPage {
                cut: false,
                rows: list([row.clone()]),
                before: None,
                after: Some(cursor()),
            }))),
        },
    );
    assert!(!domain.conversation_rows().cut);
    assert_eq!(domain.conversation_rows().rows.get(0), Some(&row));
    assert_eq!(domain.conversation_rows().edges.after, jig_client::windows::Edge::More(cursor()));
    let tree = build(&domain, 3, &words()).0;
    assert!(!contains(&tree, b"This page is incomplete"));
    assert!(contains(&tree, b"Read later"));
    assert!(contains(&tree, b"Title is incomplete"));
}

#[test]
fn sliding_a_read_window_keeps_supplied_page_loss_until_that_source_is_reset() {
    for (kind, window) in
        [(1, jig_client::pages::task::HISTORY_WINDOW), (4, jig_client::pages::conversations::CONVERSATIONS_WINDOW)]
    {
        let (mut domain, token) = ready(kind);
        drive(&mut domain, Event::Read { read: token, result: ReadResult::Value(value(kind, true, Some(cursor()))) });
        let out = drive(
            &mut domain,
            Event::Act { act: Act::More { window: Token::new(window), end: jig_client::action::End::After } },
        );
        let token = read(&out, kind);
        drive(&mut domain, Event::Read { read: token, result: ReadResult::Value(value(kind, false, None)) });
        assert!(held(&domain, kind), "a later page says nothing about the previous page's missing content");
        assert!(contains(&build(&domain, 3, &words()).0, b"This page is incomplete"));
        if kind == 1 {
            drive(&mut domain, Event::Act { act: Act::HistoryScope { subtree: true } });
            assert!(!held(&domain, kind));
        }
    }
}
