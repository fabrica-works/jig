use super::*;
pub(super) fn task_plan_keeps_folded_held_counts_and_filters_without_losing_rows() {
    use crate::page::task;
    bounded_plan_filters();
    let mut domain = Fixture::new(&LIMITS);
    let f = start(&mut domain, List::with_capacity(0));
    let initial = delivered(&mut domain, Time::ZERO, f, frame(7));
    finish_initial_inbox_reads(&mut domain, &initial);
    let opened = drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Plan }) },
    );
    let watch = open_token(&opened);
    let mut document = super::super::confirmation::task();
    document.context.as_mut().unwrap().impact.tasks.waiting = 1;
    let tree_open = drive(
        &mut domain,
        Time::ZERO,
        Watched { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(document))) },
    );
    let tree_watch = open_token(&tree_open);
    let mut nodes = List::with_capacity(1);
    nodes.push(tree_node(9, Phase::Done, 3)).unwrap();
    drive(
        &mut domain,
        Time::ZERO,
        Watched {
            watch: tree_watch,
            event: WatchEvent::Delivered(Delivery::Tree(Box::new(Tree {
                task: 8,
                nodes,
                folded: empty_phase_counts(),
                held_below: 3,
            }))),
        },
    );
    let asked = drive(&mut domain, Time::ZERO, Event::Act { action: crate::action::Act::OpenEnded { task: 9 } });
    let Request::Client {
        asked:
            jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Ended { task: 9, after: None }) },
    } = asked[0]
    else {
        panic!("ended subtree read");
    };
    let cursor = Cursor::new(Box::from(b"opaque subtree".as_slice()), 1024).expect("bounded opaque cursor");
    drive(
        &mut domain,
        Time::ZERO,
        ReadPage {
            read,
            result: ReadResult::Value(ReadValue::Ended(Box::new(ObjectPage {
                cut: false,
                rows: List::with_capacity(0),
                before: None,
                after: Some(cursor.clone()),
            }))),
        },
    );
    let more = drive(
        &mut domain,
        Time::ZERO,
        Event::Act {
            action: crate::action::Act::More {
                window: skein_lib::Token::new(task::ENDED_WINDOW),
                end: crate::action::End::After,
            },
        },
    );
    let Request::Client { asked: jig_client::Event::Read { read: jig_client::Readable::Jig(what), .. } } = &more[0]
    else {
        panic!("ended subtree continuation");
    };
    assert_eq!(*what, Read::Ended { task: 9, after: Some(cursor) });
}

fn bounded_plan_filters() {
    use crate::page::task::{self, PlanFilter};
    let mut limits = LIMITS;
    limits.rows = 2;
    let mut domain = Fixture::new(&limits);
    let mut nodes = List::with_capacity(3);
    nodes.push(tree_node(9, Phase::Done, 3)).unwrap();
    nodes.push(tree_node(10, Phase::Running, 0)).unwrap();
    nodes.push(tree_node(11, Phase::Held, 0)).unwrap();
    let watch = domain.request_watch(Subject::Tree { task: 8, window: 2 }).unwrap();
    drive(
        &mut domain,
        Time::ZERO,
        Watched {
            watch,
            event: WatchEvent::Delivered(Delivery::Tree(Box::new(Tree {
                task: 8,
                nodes,
                folded: empty_phase_counts(),
                held_below: 4,
            }))),
        },
    );
    let plan = &domain.task_page().plan;
    assert_eq!(plan.nodes.len(), 2);
    assert_eq!(plan.folded.held, 1);
    assert_eq!(plan.held_below, 4);
    task::filter(&mut domain, PlanFilter::Held);
    let plan = &domain.task_page().plan;
    assert!(plan.visible(&plan.nodes[0]));
    assert!(!plan.visible(&plan.nodes[1]));
    task::filter(&mut domain, PlanFilter::Live);
    let plan = &domain.task_page().plan;
    assert!(!plan.visible(&plan.nodes[0]));
    assert!(plan.visible(&plan.nodes[1]));
    task::filter(&mut domain, PlanFilter::All);
    assert_eq!(domain.task_page().plan.nodes.len(), 2);
}
