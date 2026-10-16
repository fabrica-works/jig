//! Read positions move atomically with their conversation.
use super::*;

const FIRST_SIGN_IN: u64 = 10;
const SECOND_SIGN_IN: u64 = 20;
const MARK_CHAT: u8 = 1;

fn moving_chat() -> (Test, Vec<Request>) {
    let mut test = Test::new(LIMITS);
    test.signin(1, FIRST_SIGN_IN, identity(0, 1));
    test.signin(2, SECOND_SIGN_IN, identity(0, 2));
    test.send(Event::Roles {
        project: 1,
        holdings: Box::new([Holding { person: 1, role: Role::Member }, Holding { person: 2, role: Role::Member }]),
    });
    test.send(Event::ChatState {
        party: 1,
        project: 1,
        task: 9,
        last_reply: 3,
        live: true,
        seen: None,
        replies: Box::new([]),
    });
    let request =
        route(&test.request(FIRST_SIGN_IN, MARK_CHAT, Ask::MarkSeen { seen: Seen::Chat { task: 9, through: 3 } }));
    test.send(Event::ChatSeen { request, project: 1, last_reply: 3 });
    let moved = test.send(Event::MoveChat { task: 9, project: 1, from: 1, to: 2 });
    (test, moved)
}
#[test]
fn a_moved_chats_old_requester_loses_its_read_position() {
    let (test, out) = moving_chat();
    assert_eq!(test.d.chat_seen(1, 9), Some(0));
    assert_eq!(out[0], Request::Erase { key: Key::ReadChat { party: 1, task: 9 } });
}
#[test]
fn a_moved_chats_new_requester_starts_with_nothing_read() {
    let (test, out) = moving_chat();
    assert_eq!(test.d.chat_seen(2, 9), Some(0));
    assert_eq!(out[1], Request::Save { record: Stored::ReadChat { party: 2, task: 9, through: 0 } });
}
