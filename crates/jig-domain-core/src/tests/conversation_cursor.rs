//! Conversation owner continuations through admitted reads and held deliveries.
use super::*;

fn continuation(core: &Core, limits: &Limits, after: Option<Cursor>) -> reads::Cursor {
    let document =
        make(core, limits, 1, &Subject::Conversations { party: 1 }, &Window { rows: limits.documents.rows, after })
            .expect("bounded owner document");
    let Document::Conversations(value) = document else { panic!("conversation document") };
    reads::conversations_cursor(&value)
}

fn retained(core: &Core, number: u64) -> reads::Row {
    let mut owner = core.tasks.task(1).expect("fixture owner").clone();
    owner.number = number;
    owner.phase = tasks::Phase::Ended(tasks::Ending::Cancelled {
        by: tasks::Party::Person(1),
        at: Wall::from_nanos(number),
        reason: b"kept reason".as_slice().into(),
        result: None,
    });
    owner.ended_at = Some(Wall::from_nanos(number));
    stored_row(
        Record::Tasks(tasks::Stored::ConversationNewest(tasks::Conversation { party: 1, task: number })),
        Some(owner),
    )
}

fn load(core: &mut Core, limits: &Limits, from: reads::Cursor) -> (Token, reads::Range) {
    let held = read_start(core, limits, 10, reads::Read::Conversations { from: Some(from) });
    let Held::ReadLoad { load, range } = held else { panic!("owner range load") };
    assert_eq!(range.party, 1);
    assert_eq!(range.scope, reads::Scope::Conversations);
    assert!(range.backwards);
    assert_eq!(range.rows, limits.documents.rows);
    assert_eq!(range.bytes, limits.load_bytes);
    (load, *range)
}

fn numbers(page: &reads::Page) -> Box<[u64]> {
    let mut values = List::with_capacity(page.rows.len());
    for row in &page.rows {
        let number = match row {
            reads::PageRow::Conversation(value) => value.chip.number,
            reads::PageRow::EndedDelegate(_)
            | reads::PageRow::History(_)
            | reads::PageRow::Milestone(_)
            | reads::PageRow::Activity(_)
            | reads::PageRow::Inbox(_)
            | reads::PageRow::Goal(_)
            | reads::PageRow::Note(_)
            | reads::PageRow::Spend(_) => panic!("only conversation rows"),
        };
        values.push(number).expect("one number per admitted row");
    }
    values.into_boxed()
}

fn retained_rows(core: &Core, ids: &[u64]) -> Box<[reads::Row]> {
    let mut rows = List::with_capacity(u32::try_from(ids.len()).expect("fixture row bound"));
    for number in ids {
        rows.push(retained(core, *number)).expect("bounded fixture rows");
    }
    rows.into_boxed()
}

#[test]
fn conversation_owner_continuations_match_live_reads_without_a_fabricated_more_flag() {
    let (mut core, mut limits) = watch_core();
    make_task(&mut core, &limits, 2);
    for rows in [1, 2] {
        limits.documents.rows = rows;
        let expected = continuation(&core, &limits, None);
        let Held::Read { answer: reads::Answer::Page(page), .. } =
            read_start(&mut core, &limits, 10, reads::Read::Conversations { from: None })
        else {
            panic!("live page")
        };
        assert_eq!(page.next, Some(expected.clone()));
        assert!(!page.cut);
        assert_eq!(numbers(&page).as_ref(), if rows == 1 { &[1][..] } else { &[1, 2][..] });
        if rows == 1 {
            assert_eq!(
                expected,
                reads::Cursor::Live {
                    scope: reads::Scope::Conversations,
                    after: Cursor::Number { number: 1 },
                    store_after: None
                }
            );
        } else {
            assert_eq!(expected, reads::Cursor::Store { scope: reads::Scope::Conversations, after: None });
        }
    }
}

#[test]
fn an_empty_live_conversation_window_still_issues_its_owned_store_start() {
    let (mut core, limits) = watch_core();
    let after = Cursor::Number { number: u64::MAX };
    let expected = continuation(&core, &limits, Some(after));
    assert_eq!(expected, reads::Cursor::Store { scope: reads::Scope::Conversations, after: None });
    let Held::Read { answer: reads::Answer::Page(page), .. } = read_start(
        &mut core,
        &limits,
        10,
        reads::Read::Conversations {
            from: Some(reads::Cursor::Live { scope: reads::Scope::Conversations, after, store_after: None }),
        },
    ) else {
        panic!("empty live page")
    };
    assert!(page.rows.is_empty());
    assert_eq!(page.next, Some(expected));
    assert!(!page.cut);
}

#[test]
#[expect(clippy::manual_map, reason = "the strict subset uses an exhaustive match rather than a closure")]
fn a_conversation_store_start_replays_multiple_pages_after_repeated_live_growth() {
    let (mut core, mut limits) = watch_core();
    limits.documents.rows = 2;
    let start = continuation(&core, &limits, None);
    for growth in [false, true] {
        if growth {
            make_task(&mut core, &limits, 2);
        }
        assert_eq!(continuation(&core, &limits, None), start);
        let mut from = start.clone();
        let pages = if growth {
            [(&[10, 9][..], true), (&[8, 7][..], true), (&[6, 5][..], false)]
        } else {
            [(&[9, 8][..], true), (&[7, 6][..], true), (&[5][..], false)]
        };
        for (ids, more) in pages {
            core.reclaim();
            let (token, range) = load(&mut core, &limits, from.clone());
            assert_eq!(
                range.after,
                match &from {
                    reads::Cursor::Store { after, .. } => after.clone(),
                    reads::Cursor::Live { .. }
                    | reads::Cursor::ConversationsBefore { .. }
                    | reads::Cursor::ConversationsLiveBefore { .. }
                    | reads::Cursor::ConversationsStart => panic!("stored range"),
                }
            );
            let values = retained_rows(&core, ids);
            let next = if more {
                Some(reads::row_key(values.last().expect("nonempty page")).expect("owner key"))
            } else {
                None
            };
            let reads::Answer::Page(page) =
                loaded_answer(&mut core, &limits, token, reads::Loaded::Page { rows: values, next: next.clone() })
                    .expect("terminal")
            else {
                panic!("retained page")
            };
            assert_eq!(numbers(&page).as_ref(), ids);
            assert!(!page.cut);
            assert!(loaded_answer(&mut core, &limits, token, reads::Loaded::Gone).is_none());
            assert_eq!(
                page.next,
                match next {
                    Some(after) =>
                        Some(reads::Cursor::Store { scope: reads::Scope::Conversations, after: Some(after) }),
                    None => None,
                }
            );
            if let Some(next) = page.next {
                from = next;
            }
        }
    }
}

#[test]
fn conversation_continuations_preserve_filtered_last_examined_positions_and_visibility() {
    let (mut core, mut limits) = watch_core();
    limits.documents.rows = 2;
    let start = continuation(&core, &limits, None);
    let mut invisible = retained(&core, 20);
    let owner = invisible.owner.as_mut().expect("owner");
    owner.project = 99;
    let mut foreign = retained(&core, 19);
    let owner = foreign.owner.as_mut().expect("owner");
    owner.requester = tasks::Party::Person(2);
    let next = reads::row_key(&foreign).expect("last examined owner");
    let (token, _) = load(&mut core, &limits, start);
    let reads::Answer::Page(page) = loaded_answer(
        &mut core,
        &limits,
        token,
        reads::Loaded::Page { rows: Box::new([invisible, foreign]), next: Some(next.clone()) },
    )
    .expect("one terminal") else {
        panic!("filtered page")
    };
    assert!(page.rows.is_empty());
    assert_eq!(page.next, Some(reads::Cursor::Store { scope: reads::Scope::Conversations, after: Some(next.clone()) }));
    core.reclaim();
    let (token, range) = load(&mut core, &limits, page.next.expect("empty page progresses"));
    assert_eq!(range.after, Some(next));
    let rows = Box::new([retained(&core, 18)]);
    let reads::Answer::Page(last) =
        loaded_answer(&mut core, &limits, token, reads::Loaded::Page { rows, next: None }).expect("one terminal")
    else {
        panic!("visible page")
    };
    assert_eq!(numbers(&last).as_ref(), [18]);
    assert_eq!(last.next, None);
}

#[test]
fn conversation_owner_continuations_keep_row_byte_and_last_examined_admission() {
    let (mut core, mut limits) = watch_core();
    let start = continuation(&core, &limits, None);
    limits.documents.rows = 1;
    let (token, _) = load(&mut core, &limits, start.clone());
    let rows = Box::new([retained(&core, 9), retained(&core, 8)]);
    assert_eq!(
        loaded_answer(&mut core, &limits, token, reads::Loaded::Page { rows, next: None }),
        Some(reads::Answer::None(reads::Why::Busy))
    );
    core.reclaim();
    let row = retained(&core, 9);
    limits.load_bytes = row.owned_bytes().expect("bounded row").checked_sub(1).expect("row has owned bytes");
    let (token, _) = load(&mut core, &limits, start.clone());
    assert_eq!(
        loaded_answer(&mut core, &limits, token, reads::Loaded::Page { rows: Box::new([row]), next: None }),
        Some(reads::Answer::None(reads::Why::Busy))
    );
    core.reclaim();
    limits.load_bytes = 1_000_000;
    let (token, _) = load(&mut core, &limits, start);
    let wrong = reads::row_key(&retained(&core, 10)).expect("wrong last examined owner");
    let rows = Box::new([retained(&core, 9)]);
    assert_eq!(
        loaded_answer(&mut core, &limits, token, reads::Loaded::Page { rows, next: Some(wrong) }),
        Some(reads::Answer::None(reads::Why::Unknown))
    );
}

#[test]
fn conversation_owner_continuations_are_authenticated_and_cannot_be_reused_for_another_query() {
    let (mut core, limits) = watch_core();
    let start = continuation(&core, &limits, None);
    let Held::Read { answer, .. } =
        read_start(&mut core, &limits, 10, reads::Read::Goals { project: 1, from: Some(start.clone()) })
    else {
        panic!("scope refusal")
    };
    assert_eq!(answer, reads::Answer::None(reads::Why::Unknown));
    let Held::Read { answer, .. } =
        read_start(&mut core, &limits, 999, reads::Read::Conversations { from: Some(start.clone()) })
    else {
        panic!("sign-in refusal")
    };
    assert_eq!(answer, reads::Answer::None(reads::Why::SignedOut));
    let (token, _) = load(&mut core, &limits, start);
    restore_document_party(&mut core, &limits, &[]);
    let rows = Box::new([retained(&core, 9)]);
    assert_eq!(
        loaded_answer(&mut core, &limits, token, reads::Loaded::Page { rows, next: None }),
        Some(reads::Answer::None(reads::Why::SignedOut))
    );
}

fn live_page(core: &mut Core, limits: &Limits, from: Option<reads::Cursor>) -> reads::Page {
    let Held::Read { answer: reads::Answer::Page(page), .. } =
        read_start(core, limits, 10, reads::Read::Conversations { from })
    else {
        panic!("held live page")
    };
    *page
}

fn five_live() -> (Core, Limits) {
    let (mut core, mut limits) = tree_core();
    for number in 1..=5 {
        make_task(&mut core, &limits, number);
    }
    let identity = core.people.identity(1).expect("fixture identity").clone();
    core.people = people::Domain::new(&limits.people, Box::new([]), core.settings.deployment_provider);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    for record in [
        people::Stored::Person { number: 1, identity },
        people::Stored::SignIn { number: 10, person: 1, expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()) },
        people::Stored::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
    ] {
        people::step(&mut core.people, &env, people::Event::Restore { record }, &mut out);
        while out.pop().is_some() {}
    }
    people::step(&mut core.people, &env, people::Event::Restored, &mut out);
    limits.documents.rows = 2;
    (core, limits)
}

#[test]
fn conversation_owner_before_cursors_recover_first_repeated_and_multiple_live_slices() {
    let (mut core, limits) = five_live();
    for _ in 0_u32..2 {
        let first = live_page(&mut core, &limits, None);
        assert_eq!(numbers(&first).as_ref(), [1, 2]);
        assert_eq!(first.before, None);
        let middle = live_page(&mut core, &limits, first.next.clone());
        assert_eq!(numbers(&middle).as_ref(), [3, 4]);
        assert_eq!(
            middle.before,
            Some(reads::Cursor::ConversationsLiveBefore { before: Some(Cursor::Number { number: 3 }) })
        );
        let last = live_page(&mut core, &limits, middle.next.clone());
        assert_eq!(numbers(&last).as_ref(), [5]);
        assert_eq!(last.next, Some(reads::Cursor::Store { scope: reads::Scope::Conversations, after: None }));
        let previous = live_page(&mut core, &limits, last.before);
        assert_eq!(numbers(&previous).as_ref(), [3, 4]);
        assert_eq!(previous.before, middle.before);
        assert_eq!(previous.next, middle.next);
        let start = live_page(&mut core, &limits, previous.before);
        assert_eq!(numbers(&start).as_ref(), [1, 2]);
        assert_eq!(start.before, None);
        assert_eq!(start.next, first.next);
        let repeated = live_page(&mut core, &limits, start.next);
        assert_eq!(repeated, middle);
        assert_eq!(live_page(&mut core, &limits, Some(reads::Cursor::ConversationsStart)), first);
    }
}

fn backward_page(core: &mut Core, limits: &Limits, from: reads::Cursor, ids: &[u64], more: bool) -> reads::Page {
    core.reclaim();
    let Held::ReadLoad { load, range } =
        read_start(core, limits, 10, reads::Read::Conversations { from: Some(from.clone()) })
    else {
        panic!("opposite stored range")
    };
    assert_eq!(range.scope, reads::Scope::Conversations);
    assert!(!range.backwards);
    let reads::Cursor::ConversationsBefore { before } = from else { panic!("owner before cursor") };
    assert_eq!(range.after, Some(before));
    assert_eq!(range.rows, limits.documents.rows);
    assert_eq!(range.bytes, limits.load_bytes);
    let rows = oldest_rows(core, ids);
    let next = if more { Some(reads::row_key(rows.last().expect("examined row")).expect("owner key")) } else { None };
    let reads::Answer::Page(page) =
        loaded_answer(core, limits, load, reads::Loaded::Page { rows, next }).expect("one terminal")
    else {
        panic!("opposite stored page")
    };
    assert!(!page.cut);
    *page
}

#[test]
fn conversation_owner_opposite_store_pages_preserve_order_and_bridge_back_to_all_live_rows() {
    let (mut core, limits) = five_live();
    let before = reads::Cursor::ConversationsBefore {
        before: reads::row_key(&oldest(&core, 5)).expect("current page first examined owner"),
    };
    let middle = backward_page(&mut core, &limits, before, &[6, 7], true);
    assert_eq!(numbers(&middle).as_ref(), [7, 6]);
    assert_eq!(
        middle.next,
        Some(reads::Cursor::Store {
            scope: reads::Scope::Conversations,
            after: Some(reads::row_key(&retained(&core, 6)).expect("displayed tail"))
        })
    );
    let head = backward_page(&mut core, &limits, middle.before.expect("earlier stored rows"), &[8, 9], false);
    assert_eq!(numbers(&head).as_ref(), [9, 8]);
    assert_eq!(head.before, Some(reads::Cursor::ConversationsLiveBefore { before: None }));
    let tail = live_page(&mut core, &limits, head.before);
    assert_eq!(numbers(&tail).as_ref(), [4, 5]);
    assert_eq!(tail.next, Some(reads::Cursor::Store { scope: reads::Scope::Conversations, after: None }));
    let previous = live_page(&mut core, &limits, tail.before);
    assert_eq!(numbers(&previous).as_ref(), [2, 3]);
    let first = live_page(&mut core, &limits, previous.before);
    assert_eq!(numbers(&first).as_ref(), [1]);
    assert_eq!(first.before, None);
    assert_eq!(numbers(&live_page(&mut core, &limits, first.next)).as_ref(), [2, 3]);
}

#[test]
fn conversation_owner_opposite_cursors_refuse_foreign_queries_and_keep_empty_prefix_edges() {
    let (mut core, limits) = watch_core();
    for from in [
        reads::Cursor::ConversationsStart,
        reads::Cursor::ConversationsLiveBefore { before: None },
        reads::Cursor::ConversationsBefore { before: reads::row_key(&oldest(&core, 9)).expect("owner key") },
    ] {
        let Held::Read { answer, .. } =
            read_start(&mut core, &limits, 10, reads::Read::Goals { project: 1, from: Some(from) })
        else {
            panic!("wrong scope")
        };
        assert_eq!(answer, reads::Answer::None(reads::Why::Unknown));
    }
    let before = reads::Cursor::ConversationsLiveBefore { before: Some(Cursor::Number { number: 1 }) };
    let first = live_page(&mut core, &limits, Some(before));
    assert!(first.rows.is_empty());
    assert_eq!(first.before, None);
    assert_eq!(first.next, Some(reads::Cursor::ConversationsStart));
    assert_eq!(numbers(&live_page(&mut core, &limits, first.next)).as_ref(), [1]);
}

fn reverse_load(core: &mut Core, limits: &Limits, number: u64) -> Token {
    core.reclaim();
    let before = reads::row_key(&oldest(core, number)).expect("owner boundary");
    let Held::ReadLoad { load, range } = read_start(
        core,
        limits,
        10,
        reads::Read::Conversations { from: Some(reads::Cursor::ConversationsBefore { before: before.clone() }) },
    ) else {
        panic!("reverse owner load")
    };
    assert_eq!(range.scope, reads::Scope::Conversations);
    assert_eq!(range.after, Some(before));
    assert!(!range.backwards);
    assert_eq!(range.rows, limits.documents.rows);
    assert_eq!(range.bytes, limits.load_bytes);
    load
}

#[test]
fn conversation_owner_reverse_filtered_pages_use_examined_positions_in_both_directions() {
    let (mut core, mut limits) = watch_core();
    limits.documents.rows = 2;
    let token = reverse_load(&mut core, &limits, 5);
    let mut first = oldest(&core, 6);
    let owner = first.owner.as_mut().expect("owner");
    owner.project = 99;
    let mut last = oldest(&core, 7);
    let owner = last.owner.as_mut().expect("owner");
    owner.requester = tasks::Party::Person(2);
    let head = reads::row_key(&last).expect("last examined");
    let reads::Answer::Page(page) = loaded_answer(
        &mut core,
        &limits,
        token,
        reads::Loaded::Page { rows: Box::new([first, last]), next: Some(head.clone()) },
    )
    .expect("one terminal") else {
        panic!("whole filtered page")
    };
    assert!(page.rows.is_empty());
    assert!(!page.cut);
    assert_eq!(page.before, Some(reads::Cursor::ConversationsBefore { before: head }));
    assert_eq!(
        page.next,
        Some(reads::Cursor::Store {
            scope: reads::Scope::Conversations,
            after: Some(reads::Position::Owner(Key::Tasks(tasks::Key::ConversationNewest {
                party: 1,
                task: tasks::Newest::of(6)
            })))
        })
    );
    let previous = backward_page(&mut core, &limits, page.before.expect("filtered prefix progresses"), &[8, 9], false);
    assert_eq!(numbers(&previous).as_ref(), [9, 8]);
    assert_eq!(previous.before, Some(reads::Cursor::ConversationsLiveBefore { before: None }));
}

#[test]
fn conversation_owner_reverse_pages_refuse_row_byte_and_last_examined_overruns_whole() {
    let (mut core, mut limits) = watch_core();
    limits.documents.rows = 1;
    let token = reverse_load(&mut core, &limits, 5);
    let rows = oldest_rows(&core, &[6, 7]);
    assert_eq!(
        loaded_answer(&mut core, &limits, token, reads::Loaded::Page { rows, next: None }),
        Some(reads::Answer::None(reads::Why::Busy))
    );
    let row = oldest(&core, 6);
    limits.load_bytes = row.owned_bytes().expect("whole owned row").checked_sub(1).expect("nonempty payload");
    let token = reverse_load(&mut core, &limits, 5);
    assert_eq!(
        loaded_answer(&mut core, &limits, token, reads::Loaded::Page { rows: Box::new([row]), next: None }),
        Some(reads::Answer::None(reads::Why::Busy))
    );
    limits.load_bytes = 1_000_000;
    let token = reverse_load(&mut core, &limits, 5);
    let wrong = reads::row_key(&oldest(&core, 5)).expect("wrong last examined");
    let rows = oldest_rows(&core, &[6]);
    assert_eq!(
        loaded_answer(&mut core, &limits, token, reads::Loaded::Page { rows, next: Some(wrong) }),
        Some(reads::Answer::None(reads::Why::Unknown))
    );
    let token = reverse_load(&mut core, &limits, 5);
    let rows = oldest_rows(&core, &[6]);
    restore_document_party(&mut core, &limits, &[]);
    assert_eq!(
        loaded_answer(&mut core, &limits, token, reads::Loaded::Page { rows, next: None }),
        Some(reads::Answer::None(reads::Why::SignedOut))
    );
}

fn oldest(core: &Core, number: u64) -> reads::Row {
    let mut row = retained(core, number);
    row.value = Record::Tasks(tasks::Stored::ConversationOldest(tasks::Conversation { party: 1, task: number }));
    row
}
fn oldest_rows(core: &Core, ids: &[u64]) -> Box<[reads::Row]> {
    let mut rows = List::with_capacity(u32::try_from(ids.len()).expect("fixture bound"));
    for number in ids {
        rows.push(oldest(core, *number)).expect("fixture bound");
    }
    rows.into_boxed()
}

#[test]
fn a_before_cursor_continues_in_the_oldest_family() {
    let (mut core, mut limits) = watch_core();
    limits.documents.rows = 1;
    let (load, _) = load(
        &mut core,
        &limits,
        reads::Cursor::Store {
            scope: reads::Scope::Conversations,
            after: Some(reads::Position::Owner(Key::Tasks(tasks::Key::ConversationNewest {
                party: 1,
                task: tasks::Newest::of(10),
            }))),
        },
    );
    let rows = retained_rows(&core, &[9]);
    let reads::Answer::Page(page) =
        loaded_answer(&mut core, &limits, load, reads::Loaded::Page { rows, next: None }).expect("page")
    else {
        panic!("page")
    };
    let before = page.before.expect("before cursor");
    assert_eq!(
        before,
        reads::Cursor::ConversationsBefore {
            before: reads::Position::Owner(Key::Tasks(tasks::Key::ConversationOldest { party: 1, task: 9 }))
        }
    );
    let page = backward_page(&mut core, &limits, before, &[10], false);
    assert_eq!(numbers(&page).as_ref(), [10]);
}
