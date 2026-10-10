//! Conversation read/watch agreement through the public boundary.
use super::*;
use crate::{
    action::{Act, End},
    objects::Source,
    windows::{ConversationWindow, Edge, Edges},
};
use skein_lib::Token;

fn rows(values: &[ConversationLine]) -> List<ConversationLine> {
    let mut result = List::with_capacity(u32::try_from(values.len()).unwrap());
    for value in values {
        result.push(value.clone()).unwrap();
    }
    result
}
fn watch_rows(domain: &mut Domain, watch: Token, values: &[ConversationLine]) -> Box<[Request]> {
    watch_owner(domain, watch, values, true, cursor(b"fixture-watch-after"))
}
fn watch_owner(
    domain: &mut Domain,
    watch: Token,
    values: &[ConversationLine],
    more: bool,
    after: Cursor,
) -> Box<[Request]> {
    drive(
        domain,
        Time::ZERO,
        Event::Watch {
            watch,
            event: WatchEvent::Delivered(Delivery::Conversations(Box::new(Conversations {
                rows: rows(values),
                more,
                after,
            }))),
        },
    )
}
fn read_rows(
    domain: &mut Domain,
    read: Token,
    values: &[ConversationLine],
    before: Option<Cursor>,
    after: Option<Cursor>,
    cut: bool,
) {
    drive(
        domain,
        Time::ZERO,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Conversations(Box::new(ConversationPage {
                rows: rows(values),
                before,
                after,
                cut,
            }))),
        },
    );
}
fn cursor(value: &[u8]) -> Cursor {
    Cursor { value: text(value) }
}
fn reply(domain: &Domain, task: u64) -> &crate::objects::result::Reply {
    domain.objects().get(domain.objects().find(ObjectKey::Task(task)).unwrap()).unwrap().reply.as_ref().unwrap()
}
#[test]
fn live_watch_growth_keeps_a_continuation_to_known_dropped_past() {
    let (mut domain, watch, first) = live_conversation_page(7107);
    let eight = list_entry(8, Phase::Idle, 10);
    let ended = list_entry(9, Phase::Done, 9);
    watch_rows(&mut domain, watch, core::slice::from_ref(&eight));
    // This owner cursor starts AFTER task 9; retaining it would skip the row
    // which the later live replacement drops. It is not a reverse boundary.
    let after_nine = cursor(b"owner-issued-after-nine");
    read_rows(&mut domain, first, &[eight.clone(), ended], None, Some(after_nine), true);
    let ten = list_entry(10, Phase::Idle, 11);
    let restart = cursor(b"\xffowner-store-start");
    drive(
        &mut domain,
        Time::ZERO,
        Event::Watch {
            watch,
            event: WatchEvent::Delivered(Delivery::Conversations(Box::new(Conversations {
                rows: rows(&[ten.clone(), eight.clone()]),
                more: false,
                after: restart.clone(),
            }))),
        },
    );
    assert_eq!(domain.conversation_rows().rows.as_slice(), &[ten, eight]);
    assert!(domain.conversation_rows().edges.reaches(End::Before));
    assert!(domain.conversation_rows().cut);
    assert!(
        !domain.conversation_rows().edges.reaches(End::After),
        "known stored task 9 was dropped; the list still needs an owner continuation before that task"
    );
    assert_eq!(domain.conversation_rows().edges.after, Edge::More(restart.clone()));
    let read = more(&mut domain, End::After, &restart);
    let ended = list_entry(9, Phase::Done, 9);
    let prefix = cursor(b"\x00owner-live-tail");
    read_rows(&mut domain, read, core::slice::from_ref(&ended), Some(prefix.clone()), None, false);
    assert_eq!(domain.conversation_rows().rows.get(1), Some(&ended));
    assert_eq!(domain.conversation_rows().edges.before, Edge::More(prefix));
    assert!(domain.conversation_rows().edges.reaches(End::After));
}
#[test]
fn a_stale_read_preserves_one_current_conversation_and_its_reply() {
    let (mut domain, watch, first) = live_conversation_page(7101);
    let old = list_entry(8, Phase::Idle, 99);
    watch_rows(&mut domain, watch, core::slice::from_ref(&old));
    let mut current = old.clone();
    current.first_words = DocumentText { bytes: text(b"\xff\x00whole"), cut: true };
    current.seen = 0;
    current.last_reply = u64::MAX;
    current.changed = None;
    current.runtime = None;
    current.unread = None;
    current.cached_replies = u32::MAX;
    current.budget.left = u64::MAX;
    watch_rows(&mut domain, watch, core::slice::from_ref(&current));
    assert_eq!(reply(&domain, 8).first_words.bytes(), b"\xff\x00whole");
    let opaque = cursor(b"\xff\x00opaque-row");
    read_rows(&mut domain, first, &[old], None, Some(opaque.clone()), true);
    assert_eq!(domain.conversation_rows().rows.as_slice(), &[current]);
    assert_eq!(domain.conversation_rows().edges.after, Edge::More(opaque));
    assert!(domain.conversation_rows().cut);
    assert_eq!(reply(&domain, 8).first_words.bytes(), b"\xff\x00whole");
    assert_eq!(reply(&domain, 8).first_unread, 1);
    assert_eq!(reply(&domain, 8).first_words, DocumentText { bytes: text(b"\xff\x00whole"), cut: true });
    let id = domain.objects().find(ObjectKey::Task(8)).unwrap();
    assert_eq!(domain.objects().get(id).unwrap().sources.len(), 1);
    assert!(domain.objects().get(id).unwrap().sources.contains(&Source::Watch(skein_lib::Id::from_token(watch))));
}

#[test]
fn overlap_is_deduplicated_before_both_bounded_slides() {
    for end in [End::After, End::Before] {
        let mut window = ConversationWindow::new(2);
        let eight = list_entry(8, Phase::Idle, 10);
        let nine = list_entry(9, Phase::Done, 9);
        let stale = list_entry(8, Phase::Done, 99);
        let ten = list_entry(10, Phase::Done, 8);
        assert_eq!(
            window.slide(
                rows(&[eight.clone(), nine.clone()]),
                Edges { before: Edge::End, after: Edge::More(cursor(b"old")) },
                End::After
            ),
            0
        );
        let incoming = Edges { before: Edge::More(cursor(b"\xffback")), after: Edge::More(cursor(b"\x00next")) };
        assert_eq!(window.slide(rows(&[stale, ten.clone()]), incoming.clone(), end), 1);
        match end {
            End::After => assert_eq!(window.rows.as_slice(), &[nine, ten]),
            End::Before => assert_eq!(window.rows.as_slice(), &[ten, eight]),
        }
        assert_eq!(window.rows.len(), window.rows.capacity());
        assert_eq!(window.edges, incoming);
    }
    let mut window = ConversationWindow::new(2);
    let eight = list_entry(8, Phase::Idle, 10);
    assert_eq!(
        window.slide(rows(&[eight.clone(), eight.clone()]), Edges { before: Edge::End, after: Edge::End }, End::After),
        0
    );
    assert_eq!(window.rows.as_slice(), &[eight]);
}
fn more(domain: &mut Domain, end: End, value: &Cursor) -> Token {
    domain.reclaim();
    let out = drive(
        domain,
        Time::ZERO,
        Event::Act { act: Act::More { window: Token::new(crate::pages::conversations::CONVERSATIONS_WINDOW), end } },
    );
    read_named(&out, &Read::Conversations { after: Some(value.clone()) })
}
#[test]
fn opposite_edge_reads_keep_their_opaque_cursor_and_drop_the_other_end() {
    let (mut domain, watch, first) = live_conversation_page(7102);
    let eight = list_entry(8, Phase::Idle, 10);
    let nine = list_entry(9, Phase::Done, 9);
    let ten = list_entry(10, Phase::Done, 8);
    watch_rows(&mut domain, watch, core::slice::from_ref(&eight));
    let older = cursor(b"\xff\x00older");
    read_rows(&mut domain, first, &[eight.clone(), nine.clone()], None, Some(older.clone()), true);
    let read = more(&mut domain, End::After, &older);
    let back = cursor(b"\x00\xffreverse");
    read_rows(&mut domain, read, &[nine.clone(), ten.clone()], Some(back.clone()), None, false);
    assert_eq!(domain.conversation_rows().rows.as_slice(), &[nine.clone(), ten.clone()]);
    assert_eq!(domain.conversation_rows().edges.before, Edge::More(back.clone()));
    assert_eq!(domain.conversation_rows().edges.after, Edge::End);
    assert!(domain.objects().find(ObjectKey::Task(8)).is_none());
    // Away from the live end, a delivery does not replace the displayed past.
    let mut ignored = nine.clone();
    ignored.project = 2;
    ignored.first_words = DocumentText::complete(text(b"ignored live source"));
    ignored.seen = ignored.last_reply;
    watch_rows(&mut domain, watch, &[eight.clone(), ignored]);
    assert_eq!(domain.conversation_rows().rows.as_slice(), &[nine.clone(), ten]);
    assert_eq!(reply(&domain, 9).first_words.bytes(), b"help checkout");
    assert_eq!(reply(&domain, 9).first_unread, 3);
    let object = domain.objects().get(domain.objects().find(ObjectKey::Task(9)).unwrap()).unwrap();
    assert_eq!(object.project, Some(1));
    assert_eq!(object.card, crate::objects::Card::Open);
    let object = domain.objects().get(domain.objects().find(ObjectKey::Task(10)).unwrap()).unwrap();
    assert_eq!(object.card, crate::objects::Card::Open);
    let read = more(&mut domain, End::Before, &back);
    let forward = cursor(b"\xffforward");
    read_rows(&mut domain, read, &[eight.clone(), nine.clone()], None, Some(forward.clone()), false);
    assert_eq!(domain.conversation_rows().rows.as_slice(), &[eight, nine]);
    assert_eq!(domain.conversation_rows().edges.before, Edge::End);
    assert_eq!(domain.conversation_rows().edges.after, Edge::More(forward));
    assert!(domain.conversation_rows().cut);
    assert!(domain.objects().find(ObjectKey::Task(10)).is_none());
    assert_eq!(reply(&domain, 8).first_unread, 3);
}
#[test]
fn a_live_replacement_keeps_read_rows_and_clears_its_own_unread_receipt() {
    let (mut domain, watch, first) = live_conversation_page(7103);
    let eight = list_entry(8, Phase::Idle, 10);
    let ended = list_entry(9, Phase::Done, 9);
    let observed = watch_rows(&mut domain, watch, core::slice::from_ref(&eight));
    let task_read = read_named(&observed, &Read::Task { task: 8 });
    assert_ne!(task_read, first, "the object document has an independent terminal obligation");
    let after = cursor(b"past");
    read_rows(&mut domain, first, &[eight.clone(), ended.clone()], None, Some(after.clone()), true);
    let mut seen = eight;
    seen.seen = seen.last_reply;
    seen.unread = Some(Count { value: 0, limited: false });
    watch_rows(&mut domain, watch, core::slice::from_ref(&seen));
    assert_eq!(domain.conversation_rows().rows.as_slice(), &[seen, ended.clone()]);
    let id = domain.objects().find(ObjectKey::Task(8)).unwrap();
    assert!(domain.objects().get(id).unwrap().reply.is_none());
    assert_eq!(domain.conversation_rows().edges.after, Edge::More(after.clone()));
    assert!(domain.conversation_rows().cut);
    watch_rows(&mut domain, watch, &[]);
    assert_eq!(domain.conversation_rows().rows.as_slice(), &[ended]);
    assert_eq!(domain.conversation_rows().edges.after, Edge::More(after));
    let removed = domain.objects().get(domain.objects().find(ObjectKey::Task(8)).unwrap()).unwrap();
    assert!(removed.sources.is_empty());
    assert_eq!(
        removed.card,
        crate::objects::Card::Leaving {
            how: crate::objects::Left::Looking,
            until: Time::ZERO.saturating_add(LIMITS.linger)
        }
    );
    assert_eq!(reply(&domain, 9).first_unread, 3);
}
#[test]
fn old_page_terminals_and_sibling_clients_cannot_replace_current_conversations() {
    let (mut domain, old_watch, old_read) = live_conversation_page(7104);
    let (mut sibling, sibling_watch, sibling_read) = live_conversation_page(7104);
    let sibling_row = list_entry(80, Phase::Idle, 8);
    watch_rows(&mut sibling, sibling_watch, core::slice::from_ref(&sibling_row));
    read_rows(&mut sibling, sibling_read, core::slice::from_ref(&sibling_row), None, None, false);
    drive(&mut domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
    domain.reclaim();
    let entered =
        drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Conversations { project: Some(2) }) });
    let watch = open_token(&entered);
    let read = read_named(&entered, &Read::Conversations { after: None });
    let mut current = list_entry(8, Phase::Idle, 1);
    current.project = 2;
    current.first_words = DocumentText { bytes: text(b"\xffcurrent"), cut: true };
    watch_rows(&mut domain, watch, core::slice::from_ref(&current));
    watch_rows(&mut domain, old_watch, &[list_entry(90, Phase::Idle, 99)]);
    read_rows(&mut domain, old_read, &[list_entry(91, Phase::Idle, 99)], None, Some(cursor(b"stale")), true);
    read_rows(&mut domain, read, &[list_entry(8, Phase::Idle, 99)], None, None, false);
    assert_eq!(domain.conversation_rows().rows.as_slice(), &[current]);
    assert!(!domain.conversation_rows().cut);
    assert_eq!(reply(&domain, 8).first_words.bytes(), b"\xffcurrent");
    assert!(domain.objects().find(ObjectKey::Task(90)).is_none());
    assert!(domain.objects().find(ObjectKey::Task(91)).is_none());
    assert_eq!(sibling.conversation_rows().rows.as_slice(), &[sibling_row]);
    assert_eq!(reply(&sibling, 80).first_words.bytes(), b"help checkout");
}

#[test]
fn a_full_live_window_keeps_both_winners_when_a_read_disagrees_on_every_key() {
    let (mut domain, watch, first) = live_conversation_page(7105);
    let mut eight = list_entry(8, Phase::Running, 8);
    eight.first_words = DocumentText { bytes: text(b"\x00\xffeight"), cut: true };
    eight.seen = 0;
    eight.last_reply = u64::MAX;
    let mut nine = list_entry(9, Phase::Idle, 9);
    nine.first_words = DocumentText::complete(text(b"nine"));
    nine.project = 2;
    nine.seen = 10;
    nine.last_reply = 20;
    watch_rows(&mut domain, watch, &[eight.clone(), nine.clone()]);
    let stale_eight = list_entry(8, Phase::Done, 99);
    let stale_nine = list_entry(9, Phase::Done, 99);
    let opaque = cursor(b"\xff\x00same-keys");
    read_rows(&mut domain, first, &[stale_eight, stale_nine], None, Some(opaque.clone()), true);
    assert_eq!(domain.conversation_rows().rows.as_slice(), &[nine, eight]);
    assert_eq!(domain.conversation_rows().rows.len(), LIMITS.rows);
    assert!(domain.conversation_rows().edges.reaches(End::Before));
    assert_eq!(domain.conversation_rows().edges.after, Edge::More(opaque));
    assert_eq!(reply(&domain, 8).first_unread, 1);
    assert_eq!(reply(&domain, 8).first_words.bytes(), b"\x00\xffeight");
    assert_eq!(reply(&domain, 9).first_unread, 11);
    assert_eq!(reply(&domain, 9).first_words.bytes(), b"nine");
    for (task, project) in [(8, 1), (9, 2)] {
        let object = domain.objects().get(domain.objects().find(ObjectKey::Task(task)).unwrap()).unwrap();
        assert_eq!(object.sources.len(), 1);
        assert_eq!(object.project, Some(project));
    }
}

const THREE: crate::Limits = crate::Limits { rows: 3, objects: 3, reads: 8, ..LIMITS };
fn step_three(domain: &mut Domain, event: Event) -> Box<[Request]> {
    let env = skein_lib::Env { now: Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: THREE };
    let mut out = skein_lib::Queue::with_capacity(crate::max_out(&THREE));
    crate::step(domain, &env, event, &mut out);
    let mut requests = List::with_capacity(crate::max_out(&THREE));
    while let Some(request) = out.pop() {
        requests.push(request).unwrap();
    }
    requests.into_boxed()
}
fn three() -> (Domain, Token, Token) {
    let mut domain = Domain::new(&THREE, 7110);
    let begin = step_three(
        &mut domain,
        Event::Start {
            address: Address::Conversations { project: None },
            kept: List::with_capacity(0),
            offset: crate::boundary::Offset { minutes: 60 },
        },
    );
    let frame_watch = open_token(&begin);
    step_three(&mut domain, Event::Hello { versions: VersionRange { first: 1, last: 1 } });
    let entered = step_three(
        &mut domain,
        Event::Watch { watch: frame_watch, event: WatchEvent::Delivered(Delivery::Frame(Box::new(frame(7)))) },
    );
    let watch = open_token(&entered);
    let read = read_named(&entered, &Read::Conversations { after: None });
    (domain, watch, read)
}
fn watch_three(domain: &mut Domain, watch: Token, values: &[ConversationLine], after: &Cursor) {
    step_three(
        domain,
        Event::Watch {
            watch,
            event: WatchEvent::Delivered(Delivery::Conversations(Box::new(Conversations {
                rows: rows(values),
                more: false,
                after: after.clone(),
            }))),
        },
    );
}
fn page_three(
    domain: &mut Domain,
    read: Token,
    values: &[ConversationLine],
    before: Option<Cursor>,
    after: Option<Cursor>,
) {
    step_three(
        domain,
        Event::Read {
            read,
            result: ReadResult::Value(ReadValue::Conversations(Box::new(ConversationPage {
                rows: rows(values),
                before,
                after,
                cut: false,
            }))),
        },
    );
}
fn more_three(domain: &mut Domain, end: End, cursor: &Cursor) -> Token {
    domain.reclaim();
    let out = step_three(
        domain,
        Event::Act { act: Act::More { window: Token::new(crate::pages::conversations::CONVERSATIONS_WINDOW), end } },
    );
    read_named(&out, &Read::Conversations { after: Some(cursor.clone()) })
}
fn past(task: u64) -> ConversationLine {
    let mut row = list_entry(task, Phase::Done, 0);
    row.changed = None;
    row.runtime = None;
    row
}
#[test]
fn owner_restart_order_keeps_current_overlap_whole_and_recovers_both_displaced_ends() {
    let (mut domain, watch, first) = three();
    let eight = list_entry(8, Phase::Idle, 10);
    let mut thirteen = past(13);
    thirteen.first_words = DocumentText { bytes: text(b"\xff\x00current-thirteen"), cut: true };
    thirteen.seen = 7;
    thirteen.last_reply = 9;
    thirteen.cached_replies = u32::MAX;
    let twelve = past(12);
    let restart = cursor(b"\xffowner-store-start");
    watch_three(&mut domain, watch, core::slice::from_ref(&eight), &restart);
    let next = cursor(b"owner-after-twelve");
    page_three(&mut domain, first, &[eight.clone(), thirteen.clone(), twelve.clone()], None, Some(next));
    let ten = list_entry(10, Phase::Idle, 11);
    watch_three(&mut domain, watch, &[ten.clone(), eight.clone()], &restart);
    assert_eq!(domain.conversation_rows().rows.as_slice(), &[ten.clone(), eight.clone(), thirteen.clone()]);
    assert_eq!(domain.conversation_rows().edges.after, Edge::More(restart.clone()));
    let read = more_three(&mut domain, End::After, &restart);
    let fourteen = past(14);
    let prefix = cursor(b"\x00owner-live-tail");
    page_three(&mut domain, read, &[fourteen.clone(), past(13), twelve.clone()], Some(prefix.clone()), None);
    assert_eq!(domain.conversation_rows().rows.as_slice(), &[fourteen.clone(), thirteen.clone(), twelve]);
    assert_eq!(domain.conversation_rows().rows.len(), THREE.rows);
    assert_eq!(domain.conversation_rows().edges.before, Edge::More(prefix.clone()));
    assert!(domain.conversation_rows().edges.reaches(End::After));
    assert_eq!(reply(&domain, 13).first_words, thirteen.first_words);
    assert_eq!(reply(&domain, 13).first_unread, 8);
    let object = domain.objects().get(domain.objects().find(ObjectKey::Task(13)).unwrap()).unwrap();
    assert_eq!(object.sources.len(), 1);
    let read = more_three(&mut domain, End::Before, &prefix);
    let live_next = cursor(b"owner-after-live-tail");
    page_three(&mut domain, read, &[eight.clone(), ten.clone()], None, Some(live_next.clone()));
    assert_eq!(domain.conversation_rows().rows.as_slice(), &[ten, eight, fourteen]);
    assert!(domain.conversation_rows().edges.reaches(End::Before));
    assert_eq!(domain.conversation_rows().edges.after, Edge::More(live_next));
}
#[test]
fn repeated_owner_replays_recover_multiple_past_pages_and_new_source_prefixes() {
    let (mut domain, watch, first) = three();
    let eight = list_entry(8, Phase::Idle, 10);
    let restart = cursor(b"\xffstore-owner");
    watch_three(&mut domain, watch, core::slice::from_ref(&eight), &restart);
    page_three(&mut domain, first, &[eight.clone(), past(13), past(12)], None, Some(cursor(b"after12")));
    for task in [10, 11] {
        let current = list_entry(task, Phase::Idle, task + 10);
        watch_three(&mut domain, watch, &[current.clone(), eight.clone()], &restart);
        let read = more_three(&mut domain, End::After, &restart);
        let prefix = cursor(b"live-prefix");
        let next = cursor(b"after12");
        page_three(&mut domain, read, &[past(14), past(13), past(12)], Some(prefix.clone()), Some(next.clone()));
        assert_eq!(domain.conversation_rows().rows.as_slice(), &[past(14), past(13), past(12)]);
        let read = more_three(&mut domain, End::After, &next);
        let back = cursor(b"before11");
        page_three(&mut domain, read, &[past(11), past(9)], Some(back.clone()), None);
        assert_eq!(domain.conversation_rows().rows.as_slice(), &[past(12), past(11), past(9)]);
        assert_eq!(domain.conversation_rows().edges.before, Edge::More(back.clone()));
        let read = more_three(&mut domain, End::Before, &back);
        page_three(&mut domain, read, &[past(14), past(13), past(12)], Some(prefix.clone()), Some(next));
        assert_eq!(domain.conversation_rows().rows.as_slice(), &[past(14), past(13), past(12)]);
        let read = more_three(&mut domain, End::Before, &prefix);
        page_three(&mut domain, read, &[eight.clone(), current], None, Some(restart.clone()));
        assert!(domain.conversation_rows().edges.reaches(End::Before));
        assert_eq!(domain.conversation_rows().edges.after, Edge::More(restart.clone()));
    }
}
#[test]
fn empty_owner_pages_preserve_the_held_opposite_edge_and_whole_rows() {
    let (mut domain, _watch, first) = live_conversation_page(7112);
    let eight = list_entry(8, Phase::Idle, 10);
    let held_before = cursor(b"\xffheld-before");
    let next = cursor(b"\x00next");
    read_rows(&mut domain, first, &[eight, past(13)], Some(held_before.clone()), Some(next.clone()), true);
    let before = domain.conversation_rows().rows.clone();
    let read = more(&mut domain, End::After, &next);
    read_rows(&mut domain, read, &[], None, None, false);
    assert_eq!(domain.conversation_rows().rows, before);
    assert_eq!(domain.conversation_rows().edges.before, Edge::More(held_before.clone()));
    assert!(domain.conversation_rows().edges.reaches(End::After));
    assert!(domain.conversation_rows().cut);
    let read = more(&mut domain, End::Before, &held_before);
    read_rows(&mut domain, read, &[], None, None, false);
    assert_eq!(domain.conversation_rows().rows, before);
    assert!(domain.conversation_rows().edges.reaches(End::Before));
    assert!(domain.conversation_rows().edges.reaches(End::After));
    assert!(domain.conversation_rows().cut);
}
#[test]
fn an_owner_replay_joins_once_and_full_admission_keeps_the_current_cursor() {
    let (mut domain, watch, first) = live_conversation_page(7113);
    let eight = list_entry(8, Phase::Idle, 10);
    let restart = cursor(b"source-restart");
    watch_owner(&mut domain, watch, core::slice::from_ref(&eight), false, restart.clone());
    read_rows(&mut domain, first, &[eight.clone(), past(13)], None, Some(cursor(b"after13")), false);
    let ten = list_entry(10, Phase::Idle, 11);
    watch_owner(&mut domain, watch, &[ten, eight.clone()], false, restart.clone());
    let first = more(&mut domain, End::After, &restart);
    let joined = drive(
        &mut domain,
        Time::ZERO,
        Event::Act {
            act: Act::More { window: Token::new(crate::pages::conversations::CONVERSATIONS_WINDOW), end: End::After },
        },
    );
    for request in &joined {
        if let Request::Read { what, .. } = request {
            assert_ne!(*what, Read::Conversations { after: Some(restart.clone()) });
        }
    }
    assert_eq!(domain.conversation_rows().edges.after, Edge::More(restart));
    assert_ne!(first, Token::new(0));

    // Earlier page reads still owe terminals after navigation. Two displayed
    // object reads fill the remaining slots; the new owner cursor stays whole.
    let (mut domain, old_watch, old_read) = live_conversation_page(7114);
    let history = drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 42, section: crate::address::TaskSection::History }) },
    );
    let history_read = read_named(&history, &Read::History { task: 42, subtree: false, after: None });
    drive(&mut domain, Time::ZERO, Event::Ended { watch: old_watch, end: WatchEnd::Closed });
    domain.reclaim();
    let entered =
        drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Conversations { project: None }) });
    let watch = open_token(&entered);
    let first = read_named(&entered, &Read::Conversations { after: None });
    assert_ne!(old_read, first);
    assert_ne!(history_read, first);
    let eight = list_entry(8, Phase::Idle, 10);
    watch_owner(&mut domain, watch, core::slice::from_ref(&eight), false, cursor(b"original-source"));
    read_rows(&mut domain, first, &[eight, past(13)], None, Some(cursor(b"after13")), false);
    let next = cursor(b"new-source-restart");
    watch_owner(
        &mut domain,
        watch,
        &[list_entry(11, Phase::Idle, 12), list_entry(8, Phase::Idle, 10)],
        false,
        next.clone(),
    );
    let out = drive(
        &mut domain,
        Time::ZERO,
        Event::Act {
            act: Act::More { window: Token::new(crate::pages::conversations::CONVERSATIONS_WINDOW), end: End::After },
        },
    );
    for request in &out {
        if let Request::Read { what, .. } = request {
            assert_ne!(*what, Read::Conversations { after: Some(next.clone()) });
        }
    }
    assert_eq!(domain.conversation_rows().edges.after, Edge::More(next));
    let Page::Work { refused, .. } = domain.page() else {
        panic!("the conversation page remains open");
    };
    assert_eq!(*refused, Some(Limit::Reads));
}

#[test]
fn a_short_owner_restart_places_its_prefix_before_the_retained_past() {
    let (mut domain, watch, first) = three();
    let eight = list_entry(8, Phase::Idle, 10);
    let restart = cursor(b"owner-start");
    watch_three(&mut domain, watch, core::slice::from_ref(&eight), &restart);
    page_three(&mut domain, first, &[eight.clone(), past(13), past(12)], None, Some(cursor(b"after12")));
    watch_three(&mut domain, watch, &[list_entry(10, Phase::Idle, 11), eight.clone()], &restart);
    let read = more_three(&mut domain, End::After, &restart);
    let next = cursor(b"owner-after14");
    page_three(&mut domain, read, &[past(14)], Some(cursor(b"owner-live-tail")), Some(next.clone()));
    assert_eq!(domain.conversation_rows().rows.as_slice(), &[eight, past(14), past(13)]);
    assert_eq!(domain.conversation_rows().edges.after, Edge::More(next));
}
