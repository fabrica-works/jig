//! One admitted deployment and one answer, shared by this world's stories.
use crate::{Script, world::Settings};
use jig_channel_world::fixtures as channel;

pub const SEED: u64 = 7;
pub const ANSWER: &[u8] = b"World answer";

#[must_use]
pub fn calm(seed: u64) -> Settings {
    Settings { seed, config: skein_sim::Config::calm() }
}
#[must_use]
pub fn one_person_chat(words: &[u8]) -> Box<[Script]> {
    Box::new([Script::answered_chat(words, ANSWER)])
}
pub(crate) fn document() -> Vec<u8> {
    let engine = String::from_utf8(channel::engine_document())
        .expect("engine JSON")
        .replace(r#""components":["accounts","workers"]"#, r#""components":["accounts","workers","clients"]"#)
        .replace(r#""application":{"credential""#, r#""application":{"workers":[{"root":"agent-root"}],"credential""#);
    let worker = std::str::from_utf8(channel::WORKER_DOCUMENT).expect("worker JSON");
    let end = worker.find("],\"application\"").expect("worker array").checked_add(1).expect("array end");
    let worker = worker.get(1..end).expect("worker member");
    format!("{},{} }}", engine.trim_end().strip_suffix('}').expect("whole engine document"), worker)
        .replace(r#""socket": "127.0.0.1:7200""#, r#""in_process":"workers""#)
        .replace(r#""socket":"127.0.0.1:7200""#, r#""in_process":"workers""#)
        .into_bytes()
}
pub(crate) fn files() -> jig_test_service::Files {
    let mut engine = channel::files(false);
    let mut entries = skein_lib::List::with_capacity(6);
    for entry in engine.entries.into_boxed() {
        entries.push(entry).expect("engine files");
    }
    for entry in channel::files(true).entries.into_boxed() {
        entries.push(entry).expect("worker files");
    }
    engine.entries = entries;
    engine
}
pub(crate) fn provider() -> skein_fake_peers::llm::Peer {
    let mut script = channel::one_chat(b"Hello");
    script.result = br#"{"form":"report","text":"World answer","fields":{}}"#.as_slice().into();
    smith_agent_process_world::fake::configured(
        Box::new([skein_fake_llm_domain::api::Script { cue: Box::default(), turns: channel::provider_turns(&script) }]),
        skein_llm::Credential { access_token: channel::TOKEN.into(), account_id: b"acc".as_slice().into() },
        skein_lib::Duration::ZERO,
    )
}
pub(crate) fn client_settings() -> jig_shell_native::Settings {
    jig_shell_native::Settings {
        engine: skein_io::Addr::Ip((std::net::Ipv4Addr::LOCALHOST, 7100).into()),
        directory: b"private".as_slice().into(),
        limits: jig_client_protocol_world::fixtures::connections_limits(),
        io: skein_io::Limits {
            sockets: 32,
            names: 1,
            refusals: 8,
            intake: 4096,
            receive: 1024,
            output: 4096,
            sends: 8,
            accepts: 4,
            backlog: 8,
            close_timeout: skein_lib::Duration::from_secs(5),
            retry: skein_lib::Duration::from_millis(1),
        },
    }
}
