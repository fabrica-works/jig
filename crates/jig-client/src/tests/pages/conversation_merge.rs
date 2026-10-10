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
    drive(
        domain,
        Time::ZERO,
        Event::Watch {
            watch,
            event: WatchEvent::Delivered(Delivery::Conversations(Box::new(Conversations {
                rows: rows(values),
                more: true,
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
