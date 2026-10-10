//! Public subtree deliveries preserve source facts across page lifecycles.
use super::spine::{LIMITS, delivered, frame, start};
use crate::{
    Domain, Event, Request,
    action::Act,
    address::{Address, Place, TaskSection},
    contract::*,
    pages::task::PlanFilter,
};
use skein_lib::{Env, List, Queue, Time, Token, Wall};
fn limits() -> crate::Limits {
    crate::Limits { objects: 8, reads: 8, ..LIMITS }
}
fn drive(d: &mut Domain, event: Event) -> Vec<Request> {
    let l = limits();
    let mut q = Queue::with_capacity(crate::max_out(&l));
    crate::step(d, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: l }, event, &mut q);
    let mut out = Vec::new();
    while let Some(value) = q.pop() {
        out.push(value);
    }
    out
}
fn chip(n: u64, phase: Phase) -> Chip {
    let mut c = super::confirmation::task().chip;
    c.task = n;
    c.phase = phase;
    c.title.cut = true;
    c
}
fn counts() -> OwnerPhaseCounts {
    OwnerPhaseCounts {
        waiting: 1,
        idle: 2,
        starting: 3,
        running: 4,
        backing_off: 5,
        closing: 6,
        held: 7,
        done: 8,
        failed: 9,
        cancelled: u32::MAX,
    }
}
fn tree(root: u64) -> OwnerTree {
    let mut rows = List::with_capacity(2);
    let mut proposals = List::with_capacity(2);
    proposals.push(ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX }).unwrap();
    let mut dependencies = List::with_capacity(2);
    dependencies.push(OwnerTaskLink::Stored { number: 0, status: None }).unwrap();
    rows.push(OwnerTreeRow {
        chip: chip(u64::MAX, Phase::Running),
        parent: root,
        depth: u32::MAX,
        proposals,
        spent: u64::MAX,
        dependencies,
        cut: true,
    })
    .unwrap();
    let mut folded = List::with_capacity(2);
    folded.push(OwnerTreeFold { parent: 0, phases: counts(), held_below: 11 }).unwrap();
    folded
        .push(OwnerTreeFold {
            parent: u64::MAX,
            phases: OwnerPhaseCounts { idle: u32::MAX, ..counts() },
            held_below: 17,
        })
        .unwrap();
    OwnerTree {
        root: chip(root, Phase::Running),
        rows,
        folded,
        held_below: 19,
        next: Some(Cursor { value: Text::new(Box::from(&[255, 0, 47][..]), 32).unwrap() }),
    }
}
fn opened(root: u64) -> (Domain, Token) {
    let mut d = Domain::new(&limits(), 32);
    let f = start(&mut d, List::with_capacity(0));
    delivered(&mut d, Time::ZERO, f, frame(7));
    let asks =
        drive(&mut d, Event::Went { place: Place::Jig(Address::Task { task: root, section: TaskSection::Plan }) });
    let task = asks
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Task { task } } if *task == root => Some(*watch),
            _ => None,
        })
        .unwrap();
    let mut doc = super::confirmation::task();
    doc.chip.task = root;
    let asks = drive(&mut d, Event::Watch { watch: task, event: WatchEvent::Delivered(Delivery::Task(Box::new(doc))) });
    let tree = asks
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Tree { task, .. } } if *task == root => Some(*watch),
            _ => None,
        })
        .unwrap();
    (d, tree)
}
#[test]
fn owner_tree_root_descendants_proposals_parent_depth_spend_cuts_and_two_independent_folds_are_retained_whole() {
    let (mut d, watch) = opened(0);
    let value = tree(0);
    drive(&mut d, Event::Watch { watch, event: WatchEvent::Delivered(Delivery::OwnerTree(Box::new(value.clone()))) });
    let plan = &d.task_page().plan;
    assert_eq!(plan.owner.as_deref(), Some(&value));
    assert!(plan.nodes.is_empty());
    assert_eq!(plan.owner.as_ref().unwrap().rows.len(), 1);
    assert_eq!(plan.owner.as_ref().unwrap().rows.capacity(), 2);
    assert_eq!(plan.owner.as_ref().unwrap().folded.capacity(), 2);
    assert!(d.objects().find(ObjectKey::Task(0)).is_some());
    assert!(d.objects().find(ObjectKey::Task(u64::MAX)).is_some());
    assert!(
        d.objects().find(ObjectKey::Proposal(value.rows.get(0).unwrap().proposals.get(0).copied().unwrap())).is_some()
    );
}
#[test]
fn owner_tree_filter_and_fold_actions_preserve_hidden_source_counts_and_optional_dependency_facts() {
    let (mut d, watch) = opened(8);
    let mut value = tree(8);
    for link in [
        OwnerTaskLink::Live(chip(7, Phase::Running)),
        OwnerTaskLink::Retained { chip: chip(7, Phase::Done), at: None },
        OwnerTaskLink::Retained { chip: chip(7, Phase::Done), at: Some(Wall::EPOCH) },
        OwnerTaskLink::Stored { number: 7, status: None },
        OwnerTaskLink::Stored { number: 7, status: Some(OwnerTaskStatus::Done) },
        OwnerTaskLink::Stored { number: 7, status: Some(OwnerTaskStatus::Failed) },
        OwnerTaskLink::Stored { number: 7, status: Some(OwnerTaskStatus::Cancelled) },
    ] {
        value.rows.get_mut(0).unwrap().dependencies.clear();
        value.rows.get_mut(0).unwrap().dependencies.push(link).unwrap();
        drive(
            &mut d,
            Event::Watch { watch, event: WatchEvent::Delivered(Delivery::OwnerTree(Box::new(value.clone()))) },
        );
        for filter in [PlanFilter::Held, PlanFilter::Live, PlanFilter::All] {
            drive(&mut d, Event::Act { act: Act::FilterPlan { filter } });
            assert_eq!(d.task_page().plan.owner.as_deref(), Some(&value));
        }
        let id = d.objects().find(ObjectKey::Task(u64::MAX)).unwrap();
        drive(&mut d, Event::Act { act: Act::Fold { object: id.token(), open: false } });
        assert!(!d.task_page().plan.owner_expanded(&value.rows.get(0).unwrap().chip));
        assert_eq!(d.task_page().plan.owner.as_deref(), Some(&value));
        drive(&mut d, Event::Act { act: Act::Fold { object: id.token(), open: true } });
    }
}
#[test]
fn owner_tree_departed_same_task_watch_and_clear_are_fenced_and_older_replacement_clears_owner_state() {
    let (mut d, old) = opened(8);
    let value = tree(8);
    drive(
        &mut d,
        Event::Watch { watch: old, event: WatchEvent::Delivered(Delivery::OwnerTree(Box::new(value.clone()))) },
    );
    let closed = drive(&mut d, Event::Went { place: Place::Elsewhere });
    assert!(d.task_page().plan.owner.is_none());
    for request in closed {
        if let Request::Close { watch } = request {
            drive(&mut d, Event::Ended { watch, end: WatchEnd::Closed });
        }
    }
    crate::reclaim(&mut d);
    drive(
        &mut d,
        Event::Watch { watch: old, event: WatchEvent::Delivered(Delivery::OwnerTree(Box::new(value.clone()))) },
    );
    assert!(d.task_page().plan.owner.is_none());
    let asks = drive(&mut d, Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Plan }) });
    let task = asks
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Task { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let asks = drive(
        &mut d,
        Event::Watch {
            watch: task,
            event: WatchEvent::Delivered(Delivery::Task(Box::new(super::confirmation::task()))),
        },
    );
    let current = asks
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Tree { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    drive(
        &mut d,
        Event::Watch { watch: current, event: WatchEvent::Delivered(Delivery::OwnerTree(Box::new(value.clone()))) },
    );
    drive(&mut d, Event::Watch { watch: old, event: WatchEvent::Delivered(Delivery::OwnerTree(Box::new(tree(0)))) });
    assert_eq!(d.task_page().plan.owner.as_deref(), Some(&value));
    let older = Tree {
        task: 8,
        nodes: List::with_capacity(0),
        folded: PhaseCounts { ended: 0, waiting: 0, running: 0, held: 0, closing: 0, done: 0, failed: 0, cancelled: 0 },
        held_below: 13,
    };
    drive(&mut d, Event::Watch { watch: current, event: WatchEvent::Delivered(Delivery::Tree(Box::new(older))) });
    assert!(d.task_page().plan.owner.is_none());
    assert_eq!(d.task_page().plan.held_below, 13);
    drive(
        &mut d,
        Event::Watch { watch: current, event: WatchEvent::Delivered(Delivery::OwnerTree(Box::new(value.clone()))) },
    );
    assert_eq!(d.task_page().plan.owner.as_deref(), Some(&value));
}
