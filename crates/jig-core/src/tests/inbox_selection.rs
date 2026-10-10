//! Inbox selection contexts preserve the source's membership and original rows.
use super::*;

fn fixture() -> (Core, Limits) {
    let (mut core, mut limits) = watch_core();
    let template = core.tasks.task(1).unwrap().clone();
    limits.tasks.tasks = 16;
    limits.tasks.project_tasks = 16;
    limits.tasks.tree_tasks = 16;
    limits.tasks.depth = 4;
    limits.people.inbox_entries = 16;
    limits.documents.rows = 16;
    core.tasks = tasks::Domain::new(&limits.tasks, 17, Box::new([1]));
    for (number, requester) in [
        (1, tasks::Party::Person(1)),
        (2, tasks::Party::Task(1)),
        (3, tasks::Party::Task(2)),
        (4, tasks::Party::Deployment { project: 1 }),
        (5, tasks::Party::Task(99)),
    ] {
        let mut row = template.clone();
        row.number = number;
        row.requester = requester;
        send_task(&mut core, &limits, tasks::Event::Restore { record: tasks::Stored::Live(Box::new(row)) });
        assert!(core.tasks.task(number).is_some());
    }
    send_task(&mut core, &limits, tasks::Event::Restored);
    restore_document_party(&mut core, &limits, &[1]);
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    people::step(
        &mut core.people,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people },
        people::Event::Restore {
            record: people::Stored::SignIn { number: 10, person: 1, expires: Wall::from_nanos(u64::MAX) },
        },
        &mut out,
    );
    (core, limits)
}

fn waiting(core: &mut Core, limits: &Limits, task: u64, kinds: &[people::EntryKind]) {
    let mut entries = List::with_capacity(u32::try_from(kinds.len()).unwrap());
    for (index, kind) in kinds.iter().enumerate() {
        entries
            .push(people::Entry {
                task,
                project: 1,
                whom: people::Whom::Person(1),
                kind: *kind,
                at: Wall::from_nanos(u64::try_from(index).unwrap().checked_add(10).unwrap()),
            })
            .unwrap();
    }
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    people::step(
        &mut core.people,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people },
        people::Event::Waiting { task, entries: entries.into_boxed() },
        &mut out,
    );
}

fn inbox(core: &Core, limits: &Limits, rows: u32) -> Inbox {
    let Document::Inbox(value) =
        make(core, limits, 1, &Subject::Inbox { party: 1 }, &Window { rows, after: None }).unwrap()
    else {
        panic!("whole inbox document")
    };
    value
}

fn stored_page(
    core: &mut Core,
    limits: &Limits,
    group: reads::Group,
    goal: Option<u64>,
    rows: Box<[reads::Row]>,
) -> reads::Answer {
    core.reclaim();
    let filter = reads::Filter { project: Some(1), goal };
    let Held::ReadLoad { load, .. } = read_start(
        core,
        limits,
        10,
        reads::Read::Inbox {
            group,
            filter,
            from: Some(reads::Cursor::Store { scope: reads::Scope::Inbox { group, filter }, after: None }),
        },
    ) else {
        panic!("original scoped Inbox load")
    };
    loaded_answer(core, limits, load, reads::Loaded::Page { rows, next: None }).unwrap()
}

fn ended_row(core: &Core, number: u64, ancestors: &[u64]) -> reads::Row {
    let mut owner = core.tasks.task(1).unwrap().clone();
    owner.number = number;
    owner.requester = tasks::Party::Person(1);
    owner.phase = tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: Box::new([]) }));
    owner.result_position = u64::MAX;
    owner.ended_at = None;
    let object = InboxObject::Result { position: u64::MAX };
    let mut row = stored_row(Record::Tasks(tasks::Stored::Ended(Box::new(owner))), None);
    row.ancestors = ancestors.into();
    row.inbox = Some(Cursor::Inbox { task: number, object, at: None });
    row
}

fn context_for(contexts: &List<InboxSelection>, cursor: Cursor) -> &InboxSelection {
    let mut found = None;
    for context in contexts {
        if context.cursor == cursor {
            assert!(found.is_none(), "one complete context for each original cursor");
            found = Some(context);
        }
    }
    found.unwrap()
}

#[test]
fn inbox_membership_matches_the_original_read_predicate_at_every_requester_boundary() {
    let (mut core, mut limits) = fixture();
    for task in [1, 2, 3, 4, 5] {
        waiting(&mut core, &limits, task, &[people::EntryKind::Question { message: task }]);
    }
    for depth in [0, 1, 2, 4] {
        limits.tasks.depth = depth;
        let supplied = inbox(&core, &limits, 16);
        for goal in [1, 2, 3, 4, 5, 99, u64::MAX] {
            let contexts = supplied.selection.as_ref().unwrap();
            let mut expected = List::with_capacity(16);
            for entry in &supplied.needs {
                if context_for(contexts, entry.cursor).goals.as_slice().contains(&goal) {
                    expected.push(entry.clone()).unwrap();
                }
            }
            let Held::Read { answer: reads::Answer::Page(page), .. } = read_start(
                &mut core,
                &limits,
                10,
                reads::Read::Inbox {
                    group: reads::Group::Questions,
                    filter: reads::Filter { project: Some(1), goal: Some(goal) },
                    from: None,
                },
            ) else {
                panic!("live Inbox read")
            };
            let mut actual = List::with_capacity(16);
            for row in &page.rows {
                let reads::PageRow::Inbox(entry) = row else { panic!("whole selected entry") };
                actual.push(entry.clone()).unwrap();
            }
            assert_eq!(actual, expected, "goal {goal}, depth {depth}");
            assert_eq!(page.inbox_selection.as_ref().unwrap().len(), page.rows.len());
        }
        let mut selected = None;
        for entry in &supplied.needs {
            let number = match &entry.task {
                TaskLink::Live(chip) | TaskLink::Retained { chip, .. } => chip.number,
                TaskLink::Stored { number, .. } => *number,
            };
            if number == 3 {
                selected = Some(entry.cursor);
            }
        }
        let context = context_for(supplied.selection.as_ref().unwrap(), selected.unwrap());
        let expected: &[u64] = match depth {
            0 => &[3],
            1 => &[3, 2],
            _ => &[3, 2, 1],
        };
        assert_eq!(context.goals.as_slice(), expected);
    }
}

#[test]
fn inbox_context_keeps_all_six_raw_objects_groups_counts_and_full_width_identities() {
    let (mut core, limits) = fixture();
    let kinds = [
        people::EntryKind::Question { message: u64::MAX },
        people::EntryKind::Proposal { number: u64::MAX },
        people::EntryKind::Escalation { revision: u64::MAX },
        people::EntryKind::PersonTask,
        people::EntryKind::Reply { message: u64::MAX },
    ];
    waiting(&mut core, &limits, 3, &kinds);
    core.people.remember_result(
        &limits.people,
        1,
        people::ResultRef { task: u64::MAX, project: 1, at: Wall::from_nanos(u64::MAX), position: u64::MAX },
    );
    let value = inbox(&core, &limits, 16);
    assert_eq!(
        (
            value.party,
            value.needs.len(),
            value.updates.len(),
            value.cached_needs,
            value.cached_updates,
            value.results_seen,
            value.updates_time_complete
        ),
        (1, 4, 2, 4, Count::Known(2), 0, true)
    );
    let contexts = value.selection.as_ref().unwrap();
    assert_eq!(contexts.len(), 6);
    for entry in value.needs.iter().chain(value.updates.iter()) {
        let context = context_for(contexts, entry.cursor);
        let expected: &[u64] = match entry.object {
            InboxObject::Result { position } => {
                assert_eq!(position, u64::MAX);
                &[u64::MAX]
            }
            InboxObject::Question { .. }
            | InboxObject::Proposal { .. }
            | InboxObject::Escalation { .. }
            | InboxObject::PersonTask
            | InboxObject::Reply { .. } => &[3, 2, 1],
        };
        assert_eq!(context.goals.as_slice(), expected);
        assert_eq!(entry.project, Some(1));
        let Cursor::Inbox { object, task, at } = entry.cursor else { panic!("original raw cursor") };
        assert_eq!(object, entry.object);
        assert_eq!(at, entry.at);
        assert_eq!(task, *context.goals.get(0).unwrap());
        assert_eq!(context.goals.capacity(), limits.tasks.depth + 1);
    }
    assert_eq!(value.updates.get(0).unwrap().at, Some(Wall::from_nanos(u64::MAX)));
    assert_eq!(value.updates.get(0).unwrap().task, TaskLink::Stored { number: u64::MAX, status: None });
}

#[test]
fn retained_inbox_context_keeps_original_ancestors_and_missing_clocks_without_current_joins() {
    let (mut core, limits) = fixture();
    for goal in [3, 1, u64::MAX, 2] {
        let row = ended_row(&core, 3, &[u64::MAX]);
        let reads::Answer::Page(page) =
            stored_page(&mut core, &limits, reads::Group::Results, Some(goal), Box::new([row]))
        else {
            panic!("whole retained page")
        };
        assert_eq!(page.rows.len(), u32::from(goal == 3 || goal == u64::MAX));
        if let Some(reads::PageRow::Inbox(entry)) = page.rows.get(0) {
            assert_eq!(entry.at, None);
            assert_eq!(entry.unread, Some(true));
            assert_eq!(entry.activity, None);
            let TaskLink::Retained { chip, at } = &entry.task else { panic!("retained owner link") };
            assert_eq!((chip.number, *at), (3, None));
            let context = page.inbox_selection.as_ref().unwrap().get(0).unwrap();
            assert_eq!(context.cursor, entry.cursor);
            assert_eq!(context.goals.as_slice(), &[3, u64::MAX]);
            assert!(!context.goals.as_slice().contains(&1));
        }
    }
}

#[test]
fn stored_waiting_and_reply_entries_keep_separate_original_contexts_and_activity_identity() {
    let (mut core, limits) = fixture();
    let mut owner = core.tasks.task(1).unwrap().clone();
    owner.inbox = Box::new([tasks::Word {
        number: u64::MAX,
        from: tasks::Party::Task(2),
        kind: tasks::MessageKind::Question,
        words: Box::new(*b"question"),
        at: Wall::EPOCH,
        hits: 0,
        eligible: true,
    }]);
    let object = InboxObject::Question { message: u64::MAX };
    let mut row = stored_row(Record::Tasks(tasks::Stored::Live(Box::new(owner.clone()))), None);
    row.inbox = Some(Cursor::Inbox { task: 1, object, at: Some(Wall::EPOCH) });
    row.ancestors = Box::new([u64::MAX]);
    let reads::Answer::Page(needs) =
        stored_page(&mut core, &limits, reads::Group::Questions, Some(u64::MAX), Box::new([row]))
    else {
        panic!("waiting owner page")
    };
    assert_eq!(needs.rows.len(), 1);
    assert_eq!(needs.inbox_selection.as_ref().unwrap().get(0).unwrap().goals.as_slice(), &[1, u64::MAX]);
    let word = tasks::Word {
        number: u64::MAX - 1,
        from: tasks::Party::Task(2),
        kind: tasks::MessageKind::Words,
        words: Box::new(*b"reply"),
        at: Wall::from_nanos(9),
        hits: 0,
        eligible: true,
    };
    let line = tasks::ActivityLine {
        task: 1,
        position: u64::MAX,
        at: word.at,
        line: tasks::Line::Notified { recipient: tasks::Party::Person(1), word: word.clone() },
    };
    let mut row = stored_row(Record::Tasks(tasks::Stored::Activity(line)), Some(owner));
    row.inbox = Some(Cursor::Inbox { task: 1, object: InboxObject::Reply { message: word.number }, at: Some(word.at) });
    row.ancestors = Box::new([99]);
    let reads::Answer::Page(updates) =
        stored_page(&mut core, &limits, reads::Group::Replies, Some(99), Box::new([row]))
    else {
        panic!("reply owner page")
    };
    let reads::PageRow::Inbox(entry) = updates.rows.get(0).unwrap() else { panic!("reply entry") };
    assert_eq!(
        (entry.object, entry.activity, entry.at),
        (InboxObject::Reply { message: u64::MAX - 1 }, Some(u64::MAX), Some(word.at))
    );
    assert_eq!(updates.inbox_selection.as_ref().unwrap().get(0).unwrap().goals.as_slice(), &[1, 99]);
}

#[test]
fn a_party_proposals_stored_goal_context_uses_only_its_supplied_original_goal() {
    let (mut core, limits) = fixture();
    let proposal = tasks::PersonProposal {
        number: u64::MAX,
        proposer: 1,
        project: 1,
        goal: new_row(&core, u64::MAX - 1, tasks::Funder::Pool { project: 1, person: 1, period: 1 }, Some(4)),
        state: tasks::PersonProposalState::Pending { since: Wall::from_nanos(7) },
    };
    assert!(!core.person_proposal_entries(&proposal).is_empty());
    for goal in [None, Some(proposal.goal.number), Some(99)] {
        let mut row = stored_row(Record::Tasks(tasks::Stored::PersonProposal(Box::new(proposal.clone()))), None);
        row.ancestors = Box::new([99]);
        row.inbox = Some(Cursor::Inbox {
            task: proposal.goal.number,
            object: InboxObject::Proposal { proposer: Proposer::Party(1), number: proposal.number },
            at: Some(Wall::from_nanos(7)),
        });
        let reads::Answer::Page(page) = stored_page(&mut core, &limits, reads::Group::Proposals, goal, Box::new([row]))
        else {
            panic!("party-origin proposal page")
        };
        assert_eq!(page.rows.len(), u32::from(goal != Some(99)));
        if let Some(reads::PageRow::Inbox(entry)) = page.rows.get(0) {
            assert_eq!(entry.task, TaskLink::Stored { number: proposal.goal.number, status: None });
            assert_eq!(entry.object, InboxObject::Proposal { proposer: Proposer::Party(1), number: u64::MAX });
            assert_eq!(
                page.inbox_selection.as_ref().unwrap().get(0).unwrap().goals.as_slice(),
                &[proposal.goal.number]
            );
        }
    }
}

#[test]
fn whole_inbox_context_distinguishes_unsupplied_from_empty_and_refuses_depth_overflow() {
    let (mut core, mut limits) = fixture();
    let value = inbox(&core, &limits, 16);
    assert!(value.selection.as_ref().unwrap().is_empty());
    assert_eq!(value.selection.as_ref().unwrap().capacity(), 16);
    let mut absent = value.clone();
    absent.selection = None;
    assert_ne!(absent, value);
    let reads::Answer::Page(page) = stored_page(&mut core, &limits, reads::Group::Needs, None, Box::new([])) else {
        panic!("complete empty page")
    };
    assert!(page.inbox_selection.as_ref().unwrap().is_empty());
    limits.tasks.depth = u32::MAX;
    assert_eq!(inbox_selection_worst_case(16, limits.tasks.depth), None);
    assert_eq!(document_with_context_worst_case(&limits), None);
    assert_eq!(
        make(&core, &limits, 1, &Subject::Inbox { party: 1 }, &Window { rows: 16, after: None }),
        Err(Missing::TooLarge)
    );
}

#[test]
fn original_stored_ancestry_fits_at_depth_and_one_under_refuses_the_whole_page() {
    let (mut core, mut limits) = fixture();
    limits.tasks.depth = 2;
    let row = ended_row(&core, 100, &[99, u64::MAX]);
    let second = ended_row(&core, 100, &[99, u64::MAX]);
    let reads::Answer::Page(page) = stored_page(&mut core, &limits, reads::Group::Results, Some(99), Box::new([row]))
    else {
        panic!("whole depth boundary")
    };
    assert_eq!(page.inbox_selection.as_ref().unwrap().get(0).unwrap().goals.as_slice(), &[100, 99, u64::MAX]);
    assert_eq!(page.inbox_selection.as_ref().unwrap().get(0).unwrap().goals.capacity(), 3);
    limits.tasks.depth = 1;
    assert_eq!(
        stored_page(&mut core, &limits, reads::Group::Results, Some(99), Box::new([second])),
        reads::Answer::None(reads::Why::Busy)
    );
    assert_eq!(core.tasks.task(1).unwrap().requester, tasks::Party::Person(1));
}

#[test]
fn inbox_context_memory_prices_every_reserved_container_and_complete_document_envelope() {
    let (mut core, limits) = fixture();
    waiting(
        &mut core,
        &limits,
        3,
        &[people::EntryKind::Question { message: 1 }, people::EntryKind::Question { message: 2 }],
    );
    let value = inbox(&core, &limits, 16);
    let contexts = value.selection.as_ref().unwrap();
    let reserved = List::<InboxSelection>::worst_case(contexts.capacity()).unwrap();
    let roots = List::<u64>::worst_case(limits.tasks.depth + 1).unwrap();
    let bound = inbox_selection_worst_case(16, limits.tasks.depth).unwrap();
    assert_eq!(bound, reserved + 16 * roots);
    let mut held = reserved;
    for context in contexts {
        assert_eq!(context.owned_bytes(), Some(roots));
        held += context.owned_bytes().unwrap();
    }
    assert!(held <= bound);
    assert!(roots > u64::try_from(size_of_val(contexts.get(0).unwrap().goals.as_slice())).unwrap());
    assert_eq!(
        document_with_context_worst_case(&limits),
        Some(document_worst_case(&limits.documents).unwrap() + bound)
    );
    assert!(reads::answer_worst_case(&limits).unwrap() >= document_with_context_worst_case(&limits).unwrap());
    assert!(worst_case(&limits).is_some());
}
