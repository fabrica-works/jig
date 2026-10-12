use super::*;

#[test]
fn marked_chats_lists_only_that_persons_positions() {
    let mut test = Test::new(LIMITS);
    test.member();
    test.signin(2, 20, identity(0, 2));
    for (sign_in, task, through) in [(10, 7, 3), (20, 8, 4), (10, 9, 5)] {
        test.send(Event::ChatState {
            party: if sign_in == 10 { 1 } else { 2 },
            project: 1,
            task,
            last_reply: through,
            live: false,
            replies: Box::new([]),
        });
        let seen = Seen::Chat { task, through };
        let request = route(&test.request(sign_in, u8::try_from(task).unwrap(), Ask::MarkSeen { seen }));
        test.send(Event::ChatSeen { request, project: 1, last_reply: through });
    }
    assert_eq!(test.d.marked_chats(1).into_boxed().as_ref(), &[(7, 3), (9, 5)]);
    assert_eq!(test.d.marked_chats(2).into_boxed().as_ref(), &[(8, 4)]);
    assert!(test.d.marked_chats(3).is_empty());
}
