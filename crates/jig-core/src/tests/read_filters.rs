use super::*;

fn note_row(name: u64, scope: notes::Scope, description: &[u8]) -> reads::Row {
    stored_row(
        Record::Notes(notes::Record::Entry(notes::Entry {
            name,
            scope,
            description: skein_lib::bytes::copy_of(description),
            body: Box::new(*b"whole note body"),
            references: List::with_capacity(0),
            author: notes::Author::Party { party: 1 },
            revision: 7,
            changed: Wall::from_nanos(20),
        })),
        None,
    )
}

fn scoped(scope: notes::Scope, description: Option<&[u8]>, from: Option<reads::Cursor>) -> reads::Read {
    reads::Read::ScopedNotes { scope, description: description.map(skein_lib::bytes::copy_of), from }
}

#[test]
fn scoped_note_search_pages_preserve_filtered_positions_and_whole_query_identity() {
    let (mut core, mut limits) = watch_core();
    limits.documents.rows = 2;
    let scope = notes::Scope::Goal { project: 1, goal: 1 };
    let Held::ReadLoad { load, range } = read_start(&mut core, &limits, 10, scoped(scope.clone(), Some(b"warm"), None))
    else {
        panic!("notes load")
    };
    assert_eq!(range.scope, reads::Scope::ScopedNotes { scope: scope.clone(), description: Some(Box::new(*b"warm")) });
    let rows = Box::new([
        note_row(1, notes::Scope::Project { project: 1 }, b"warm project"),
        note_row(2, scope.clone(), b"different description"),
    ]);
    let after = reads::row_key(&rows[1]).unwrap();
    let reads::Answer::Page(first) =
        loaded_answer(&mut core, &limits, load, reads::Loaded::Page { rows, next: Some(after.clone()) }).unwrap()
    else {
        panic!("bounded notes page")
    };
    assert!(first.rows.is_empty());
    assert_eq!(first.next, Some(reads::Cursor::Store { scope: range.scope, after: Some(after.clone()) }));
    core.reclaim();
    for changed in [
        scoped(scope.clone(), Some(b"other"), first.next.clone()),
        scoped(notes::Scope::Project { project: 1 }, Some(b"warm"), first.next.clone()),
    ] {
        let Held::Read { answer, .. } = read_start(&mut core, &limits, 10, changed) else {
            panic!("mismatched cursor refused")
        };
        assert_eq!(answer, reads::Answer::None(reads::Why::Unknown));
    }
    let Held::ReadLoad { load, range } =
        read_start(&mut core, &limits, 10, scoped(scope.clone(), Some(b"warm"), first.next))
    else {
        panic!("next page")
    };
    assert_eq!(range.after, Some(after));
    let reads::Answer::Page(last) = loaded_answer(
        &mut core,
        &limits,
        load,
        reads::Loaded::Page { rows: Box::new([note_row(3, scope, b"slow to warm up")]), next: None },
    )
    .unwrap() else {
        panic!("last page")
    };
    assert_eq!(last.rows.len(), 1);
    let reads::PageRow::Note(note) = last.rows.get(0).unwrap() else { panic!("note document") };
    assert_eq!((note.number, note.revision, note.body.bytes.as_ref()), (3, 7, b"whole note body".as_slice()));
    assert_eq!(last.next, None);
}

#[test]
fn resource_note_scope_preserves_literal_segments_and_exact_or_open_end() {
    let (mut core, limits) = watch_core();
    let scope = notes::Scope::Resources {
        project: 1,
        connector: 1,
        pattern: notes::Pattern {
            segments: Box::new([Box::<[u8]>::from(b"a".as_slice())]),
            last: notes::Last::Exact(Box::new(*b"b")),
        },
    };
    let open = notes::Scope::Resources {
        project: 1,
        connector: 1,
        pattern: notes::Pattern {
            segments: Box::new([Box::<[u8]>::from(b"a".as_slice())]),
            last: notes::Last::Open(Box::new(*b"b")),
        },
    };
    let Held::ReadLoad { load, .. } = read_start(&mut core, &limits, 10, scoped(scope.clone(), None, None)) else {
        panic!("resource scope")
    };
    let reads::Answer::Page(page) = loaded_answer(
        &mut core,
        &limits,
        load,
        reads::Loaded::Page {
            rows: Box::new([note_row(1, open, b"description"), note_row(2, scope, b"description")]),
            next: None,
        },
    )
    .unwrap() else {
        panic!("resource notes")
    };
    assert_eq!(page.rows.len(), 1);
    let reads::PageRow::Note(note) = page.rows.get(0).unwrap() else { panic!("exact scope note") };
    assert_eq!(note.number, 2);
}

#[test]
fn scoped_note_reads_bound_the_query_before_retaining_it_and_recheck_the_role() {
    let (mut core, limits) = watch_core();
    let too_long = vec![0; usize::try_from(limits.notes.description_bytes).unwrap() + 1];
    let too_many = notes::Scope::Resources {
        project: 1,
        connector: 1,
        pattern: notes::Pattern {
            segments: vec![Box::<[u8]>::from([]); usize::try_from(limits.notes.pattern_bytes).unwrap() + 1]
                .into_boxed_slice(),
            last: notes::Last::Exact(Box::new([])),
        },
    };
    for read in [scoped(notes::Scope::Project { project: 1 }, Some(&too_long), None), scoped(too_many, None, None)] {
        let Held::Read { answer, .. } = read_start(&mut core, &limits, 10, read) else {
            panic!("oversized query refused")
        };
        assert_eq!(answer, reads::Answer::None(reads::Why::Busy));
        assert!(core.reads.is_empty());
    }
    let Held::ReadLoad { load, .. } = read_start(&mut core, &limits, 10, scoped(notes::Scope::Deployment, None, None))
    else {
        panic!("deployment role admits")
    };
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    people::step(
        &mut core.people,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people },
        people::Event::Roles { project: 0, holdings: Box::new([]) },
        &mut out,
    );
    assert_eq!(
        loaded_answer(
            &mut core,
            &limits,
            load,
            reads::Loaded::Page { rows: Box::new([note_row(1, notes::Scope::Deployment, b"deployment")]), next: None }
        ),
        Some(reads::Answer::None(reads::Why::Unknown))
    );
}

#[test]
fn each_inbox_kind_selects_its_owner_objects_and_the_cursor_keeps_that_kind() {
    let (mut core, mut limits) = watch_core();
    limits.people.inbox_entries = 8;
    limits.documents.rows = 2;
    let reply_to = || ReplyTo::new(Token::new(1));
    send_task(&mut core, &limits, tasks::Event::Prepare { reply_to: reply_to(), task: 1 });
    send_task(
        &mut core,
        &limits,
        tasks::Event::Claim {
            reply_to: reply_to(),
            task: 1,
            attempt: 7,
            resumed: false,
            budget: 10,
            writes: Box::new([]),
        },
    );
    send_task(
        &mut core,
        &limits,
        tasks::Event::Turn {
            reply_to: reply_to(),
            task: 1,
            attempt: 7,
            turn: 1,
            read: None,
            offered: None,
            cumulative: 1,
        },
    );
    assert_eq!(core.tasks.task(1).unwrap().replies.len(), 1, "the reply has an owner-issued activity position");
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    let kinds = [
        people::EntryKind::Proposal { number: 1 },
        people::EntryKind::Escalation { revision: 1 },
        people::EntryKind::Question { message: 1 },
        people::EntryKind::PersonTask,
        people::EntryKind::Reply { message: 1 },
    ];
    people::step(
        &mut core.people,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people },
        people::Event::Waiting {
            task: 1,
            entries: kinds
                .into_iter()
                .enumerate()
                .map(|(i, kind)| people::Entry {
                    task: 1,
                    project: 1,
                    whom: people::Whom::Person(1),
                    kind,
                    at: Wall::from_nanos(u64::try_from(i).unwrap() + 1),
                })
                .collect(),
        },
        &mut out,
    );
    core.people.remember_result(
        &limits.people,
        1,
        people::ResultRef { task: 100, project: 1, at: Wall::from_nanos(6), position: 1 },
    );
    for (group, expected) in [
        (reads::Group::Proposals, InboxObject::Proposal { proposer: Proposer::Task(1), number: 1 }),
        (reads::Group::Escalations, InboxObject::Escalation { revision: 1 }),
        (reads::Group::Questions, InboxObject::Question { message: 1 }),
        (reads::Group::PersonTasks, InboxObject::PersonTask),
        (reads::Group::Results, InboxObject::Result { position: 1 }),
        (reads::Group::Replies, InboxObject::Reply { message: 1 }),
    ] {
        let mut from = None;
        let mut found = Vec::new();
        for _ in 0..8 {
            let held = read_start(
                &mut core,
                &limits,
                10,
                reads::Read::Inbox { group, filter: reads::Filter { project: Some(1), goal: None }, from },
            );
            let Held::Read { answer: reads::Answer::Page(page), .. } = held else { break };
            for row in &page.rows {
                let reads::PageRow::Inbox(entry) = row else { panic!("inbox row") };
                found.push(entry.object);
            }
            from = page.next;
            if matches!(from, Some(reads::Cursor::Store { .. }) | None) {
                break;
            }
        }
        assert_eq!(found, [expected], "{group:?} selects its exact owner kind");
    }
}

#[test]
fn stored_inbox_kind_filter_keeps_an_empty_page_cursor_and_refuses_a_different_kind() {
    let (mut core, limits) = watch_core();
    let mut owner = core.tasks.task(1).unwrap().clone();
    owner.number = 100;
    owner.requester = tasks::Party::Person(1);
    owner.phase = tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: Box::new([]) }));
    owner.result_position = 1;
    owner.ended_at = Some(Wall::from_nanos(10));
    let object = InboxObject::Result { position: 1 };
    let mut row = stored_row(Record::Tasks(tasks::Stored::Ended(Box::new(owner.clone()))), Some(owner));
    row.inbox = Some(Cursor::Inbox { task: 100, object, at: Some(Wall::from_nanos(10)) });
    let position = reads::row_key(&row).unwrap();
    let filter = reads::Filter { project: Some(1), goal: None };
    let scope = reads::Scope::Inbox { group: reads::Group::Replies, filter };
    let Held::ReadLoad { load, .. } = read_start(
        &mut core,
        &limits,
        10,
        reads::Read::Inbox {
            group: reads::Group::Replies,
            filter,
            from: Some(reads::Cursor::Store { scope, after: None }),
        },
    ) else {
        panic!("stored replies")
    };
    let reads::Answer::Page(first) = loaded_answer(
        &mut core,
        &limits,
        load,
        reads::Loaded::Page { rows: Box::new([row]), next: Some(position.clone()) },
    )
    .unwrap() else {
        panic!("filtered page")
    };
    assert!(first.rows.is_empty());
    core.reclaim();
    let Held::Read { answer, .. } = read_start(
        &mut core,
        &limits,
        10,
        reads::Read::Inbox { group: reads::Group::Results, filter, from: first.next.clone() },
    ) else {
        panic!("different query refused")
    };
    assert_eq!(answer, reads::Answer::None(reads::Why::Unknown));
    let Held::ReadLoad { range, .. } = read_start(
        &mut core,
        &limits,
        10,
        reads::Read::Inbox { group: reads::Group::Replies, filter, from: first.next },
    ) else {
        panic!("same query continues")
    };
    assert_eq!(range.after, Some(position));
}
