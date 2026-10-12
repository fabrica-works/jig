//! Counts remain exact past the bounded visible prefix and follow live role eligibility.
use super::*;

#[test]
fn each_need_kind_counts_beyond_cache_and_rechecks_current_recipients() {
    let mut test = Test::new(LIMITS);
    test.signin(1, 10, identity(0, 1));
    for project in [1, 2] {
        test.send(Event::Roles { project, holdings: Box::new([Holding { person: 1, role: Role::Member }]) });
    }
    let kinds = [
        EntryKind::Question { message: 1 },
        EntryKind::Proposal { number: 2 },
        EntryKind::Escalation { revision: 3 },
        EntryKind::PersonTask,
    ];
    let mut task = 0;
    for kind in kinds {
        for project in [1, 2] {
            task += 1;
            let whom = if project == 1 { Whom::Person(1) } else { Whom::Role { project, role: Role::Member.number() } };
            test.send(Event::Waiting {
                task,
                entries: Box::new([Entry { task, project, whom, kind, at: Wall::from_nanos(task) }]),
            });
        }
    }
    assert_eq!(test.d.cached_waiting(&LIMITS, 1).expect("prefix").len(), 2);
    assert_eq!(test.d.need_counts(1), Some(Needs { questions: 2, proposals: 2, escalations: 2, person_tasks: 2 }));
    assert_eq!(test.d.needs(1), Some(8));
    assert_eq!(test.d.need_counts(99), None);
    test.send(Event::Roles { project: 2, holdings: Box::new([]) });
    assert_eq!(test.d.need_counts(1), Some(Needs { questions: 1, proposals: 1, escalations: 1, person_tasks: 1 }));
    test.send(Event::Waiting { task: 1, entries: Box::new([]) });
    assert_eq!(test.d.need_counts(1), Some(Needs { questions: 0, proposals: 1, escalations: 1, person_tasks: 1 }));
    assert_eq!(test.d.needs(1), Some(3));
}
