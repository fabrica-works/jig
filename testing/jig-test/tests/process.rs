//! The shipped service and agent, killed and restarted over one durable store.
mod support;
use jig_client_protocol_world::native::Script;

#[test]
fn a_killed_jig_test_restarts_and_the_work_is_found() {
    let directory = support::scratch();
    support::configuration(directory.path());
    let mut script = jig_channel_world::fixtures::one_chat(b"Hello");
    script.result = br#"{"form":"report","text":"Process answer","fields":{}}"#.as_slice().into();
    let peer = smith_agent_process_world::fake::configured_transport(
        Box::new([skein_fake_llm_domain::api::Script {
            cue: Box::default(),
            turns: jig_channel_world::fixtures::provider_turns(&script),
        }]),
        skein_llm::Credential { access_token: b"token".as_slice().into(), account_id: b"acc".as_slice().into() },
        skein_lib::Duration::ZERO,
        skein_fake_peers::Transport::Tls,
    );
    assert_eq!(support::run(directory.path(), Script::answered_chat(b"Hello", support::ANSWER), Some(peer)), (1, 1));
    assert_eq!(support::run(directory.path(), Script::found_chat(support::ANSWER), None), (1, 0));
}
