//! Final task facts cross the routing world's durable delivery path.
use jig_core_world::{effects::World, fixtures, watches};
use jig_domain_core::documents::{Document, PhaseLine, Watched};
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use jig_test_domain as root;
use skein_lib::Token;

fn finish(world: &mut World, task: u64) {
    let attempt = world.assignments.iter().rev().find(|(number, _)| *number == task).expect("assigned task").1;
    world.send(root::Event::Answer {
        read: None,
        refused: Box::new([]),
        channel: Token::new(7),
        task,
        attempt,
        cumulative: 0,
        end: tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
    });
}

fn final_tree(world: &mut World, task: u64, watcher: u64) {
    watches::open(world, watcher, Watched::Tree { task });
    let parties = world.watches.as_ref().expect("party");
    let deliveries: Vec<_> = parties.deliveries.iter().filter(|(name, _, _)| *name == watcher).collect();
    assert_eq!(deliveries.len(), 1, "one final delivery");
    let Document::Tree(tree) = &deliveries[0].2 else { panic!("tree") };
    assert_eq!(tree.root.number, task);
    assert!(matches!(tree.root.phase, PhaseLine::Ended));
    assert!(tree.rows.is_empty());
    assert!(tree.folded.is_empty());
    assert_eq!(tree.next, None);
    assert_eq!(parties.endings.iter().filter(|(name, _)| *name == watcher).count(), 1);
    assert!(parties.endings.contains(&(watcher, views::End::Gone)));
    assert!(world.busy_events.is_empty());
    parties.finish();
}

#[test]
fn an_ended_roots_tree_is_delivered_once_before_ending_even_after_restart() {
    let seed = 921;
    let mut world = watches::world(seed, true);
    let task = world.assignments[0].0;
    finish(&mut world, task);
    final_tree(&mut world, task, 5);
    world.restart(seed, false);
    final_tree(&mut world, task, 6);
}

#[test]
fn an_ended_childs_tree_finishes_while_its_parents_tree_stays_live() {
    let mut world = watches::world(922, true);
    let parent = world.assignments[0];
    let mut member = fixtures::member(&world, tasks::Executor::Agent { charter: 1 }, b"delegate");
    member.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let child = fixtures::delegate(&mut world, parent, 1, member);
    watches::open(&mut world, 5, Watched::Tree { task: child });
    let Document::Tree(tree) = watches::latest(&world, 5) else { panic!("live child") };
    assert!(!matches!(tree.root.phase, PhaseLine::Ended));
    jig_core_world::stories::reads::request(
        &mut world,
        40,
        jig_domain_people::Ask::Cancel {
            project: 1,
            task: child,
            reason: b"ended delegate".as_slice().into(),
            grace: None,
        },
    );
    final_tree(&mut world, child, 6);
    let Document::Tree(tree) = watches::latest(&world, 3) else { panic!("live parent") };
    assert_eq!(tree.root.number, parent.0);
    assert!(!matches!(tree.root.phase, PhaseLine::Ended));
    assert!(tree.rows.is_empty(), "ended children leave the live subtree");
    assert!(!world.watches.as_ref().expect("party").endings.iter().any(|(watcher, _)| *watcher == 3));
}

#[test]
fn waiting_for_an_ended_task_observes_the_condition_on_the_first_delivery() {
    let mut world = watches::world(923, true);
    let task = world.assignments[0].0;
    finish(&mut world, task);
    watches::open(&mut world, 5, Watched::Task { task });
    let parties = world.watches.as_ref().expect("party");
    let documents: Vec<_> = parties.deliveries.iter().filter(|(watcher, _, _)| *watcher == 5).collect();
    assert_eq!(documents.len(), 1);
    let Document::Task(document) = &documents[0].2 else { panic!("terminal condition") };
    assert_eq!(document.chip.number, task);
    assert!(matches!(document.chip.phase, PhaseLine::Ended));
    assert!(document.result.is_some());
    assert!(parties.endings.contains(&(5, views::End::Gone)));
    parties.finish();
    assert!(world.busy_events.is_empty());
}
