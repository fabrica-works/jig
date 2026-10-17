use jig_client_protocol_world::fixtures;
const CHAT_LIST_WAIT: skein_lib::Duration = skein_lib::Duration::from_secs(1);
#[test]
fn a_person_s_chats_list_live_then_ended_by_last_activity() {
    use jig_client_protocol_world::stories as bytes;
    use skein_fake_person::{Find, Step};
    let (mut story, client) = bytes::ready(fixtures::SEED);
    bytes::start_chat(&mut story, client, b"Second chat");
    bytes::end_chat(&mut story, client, b"Hello");
    story.person(
        client,
        vec![
            Step::Go { address: bytes::conversations() },
            Step::See { find: Find::text(b"Second chat"), within: CHAT_LIST_WAIT },
            Step::Press { find: Find::named(skein_ui::Role::Button, b"Read earlier") },
            Step::See { find: Find::text(b"Hello"), within: CHAT_LIST_WAIT },
        ],
    );
    bytes::finish(&mut story);
    assert_eq!(story.front().task_queries_read(), 1);
    let tasks: Vec<_> = story
        .tree(client)
        .nodes()
        .iter()
        .filter_map(|node| node.key.filter(|key| key.kind == 20).map(|key| key.a))
        .collect();
    assert_eq!(tasks, vec![2, 1]);
}
