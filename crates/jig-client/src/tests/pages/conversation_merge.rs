//! Conversation read/watch agreement through the public boundary.
use super::*;
use crate::{objects::Source, windows::Edge};
use skein_lib::Token;

fn rows(values: &[ConversationLine]) -> List<ConversationLine> {
    let mut result = List::with_capacity(u32::try_from(values.len()).unwrap());
    for value in values {
        result.push(value.clone()).unwrap();
    }
    result
}
fn watch_rows(domain: &mut Domain, watch: Token, values: &[ConversationLine]) {
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
    );
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
    watch_rows(&mut domain, watch, &[old.clone()]);
    let mut current = old.clone();
    current.first_words = text(b"\xff\x00whole");
    current.seen = 0;
    current.last_reply = u64::MAX;
    current.changed = skein_lib::Wall::EPOCH;
    current.budget.left = u64::MAX;
    watch_rows(&mut domain, watch, &[current.clone()]);
    assert_eq!(reply(&domain, 8).first_words.bytes(), b"\xff\x00whole");
    let opaque = cursor(b"\xff\x00opaque-row");
    read_rows(&mut domain, first, &[old], None, Some(opaque.clone()), true);
    assert_eq!(domain.conversation_rows().rows.as_slice(), &[current]);
    assert_eq!(domain.conversation_rows().edges.after, Edge::More(opaque));
    assert!(domain.conversation_rows().cut);
    assert_eq!(reply(&domain, 8).first_words.bytes(), b"\xff\x00whole");
    assert_eq!(reply(&domain, 8).first_unread, 1);
    let id = domain.objects().find(ObjectKey::Task(8)).unwrap();
    assert_eq!(domain.objects().get(id).unwrap().sources.len(), 1);
    assert!(domain.objects().get(id).unwrap().sources.contains(&Source::Watch(skein_lib::Id::from_token(watch))));
}
