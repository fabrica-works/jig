//! One bounded chat deployment and independent provider script (channel.md, 12).
use crate::world::{Script, Settings};
use jig_test_service::{Files, Seeds};
use skein_lib::{Duration, List, Secret};

pub const SEED: u64 = 7;
pub(crate) const NOTES_LOAD_ROWS: u32 = 4;
pub const TOKEN: &[u8] = b"test-token";
const DIGEST: &[u8] = b"6fb46f7a92742970166379ed5195e79c4493a7cc5664280c039cfd4095ba5faf";
const REPORT: &[u8] = br#"{"form":"report","text":"Hello","fields":{}}"#;
const ACCOUNT: &[u8] = br#"{"access_token":"test-token","expires_at":1767225720,"account_id":"acc"}"#;

#[must_use]
pub fn calm(seed: u64) -> Settings {
    Settings { seed, config: skein_sim::Config::calm() }
}
#[must_use]
pub fn one_chat(words: &[u8]) -> Script {
    Script { words: words.into(), result: REPORT.into() }
}

const ENGINE: &[u8] = br#"{
"limits": {"profile": "small"},
"llm": {
"endpoints": [{
"name": "fake",
"number": 1,
"dialect": 0,
"account": 1,
"provider": "fake-provider",
"address": {"socket": "127.0.0.1:7101"},
"transport": "plaintext",
"headers": [],
"connect_ms": 10000,
"handshake_ms": 10000
}],
"models": [{
"name": "fake",
"endpoint": "fake",
"window": 1048576,
"output": 512,
"prices": {"input": 1, "cached": 1, "output": 1, "unit": 1000000}
}],
"accounts": [{
"number": 1,
"source": {"handed_in": {"file": "accounts/chat.json"}}
}],
"records": "accounts/records"
},
"hosts": {
"workers": [
{"name": "worker-a", "digest": "secrets/worker-a.digest"}
],
"listener": [{"socket": "127.0.0.1:7200"}]
},
"charters": [{
"number": 1, "instructions": "charters/chat.txt", "model": "fake", "alternatives": [],
"hosts": "workers", "waiting_ms": 10000, "resume": true,
"turns": 4, "time_ms": 30000, "reserve_turns": 1, "reserve_time_ms": 5000, "spend": 20,
"inspect": false, "modify": false, "shell": false, "agents": false, "call_timeout_ms": 10000
}],
"chats": {"charter": 1, "close_grace_ms": 5000},
"clients": {
"listener": {"socket": "127.0.0.1:7100"},
"heartbeat_ms": 10000,
"link": true
},
"deployment": {"directory": "state", "seed": 7},
"owners": [{"provider": 1, "subject": "owner-a"}],
"providers": [{"number": 1, "kind": "link"}, {"number": 2, "kind": "deployment"}],
"connectors": [1,2],
"memory_bytes": 1152921504606846976,
"rules": {
"ceiling": {"tools": 0, "grants": [],
"delegation": {"kinds": [{"kind": "charter", "number": 1}], "tasks": 8, "depth": 8},
"budget": {"spend": 100, "deadline_unix_seconds": null}, "notes": 0, "note_resources": []},
"period_spend": 100, "minimum_run_spend": 1, "maximum_run_spend": 100,
"implies": [], "requirements": []
},
"projects": [{"number": 1, "roles": [{"connector": 1, "permission": 1, "role": 0}],
"policy": {"escalation_role": 0, "period_spend": 100, "projections": [], "requirements": [],
"ceiling": {"tools": 0, "grants": [],
"delegation": {"kinds": [{"kind": "charter", "number": 1}], "tasks": 8, "depth": 8},
"budget": {"spend": 100, "deadline_unix_seconds": null}, "notes": 0, "note_resources": []},
"roles": [{"number": 0, "period_spend": 100, "requests": 511, "decides": 31,
"authority": {"tools": 0, "grants": [],
"delegation": {"kinds": [{"kind": "charter", "number": 1}], "tasks": 8, "depth": 8},
"budget": {"spend": 100, "deadline_unix_seconds": null}, "notes": 0, "note_resources": []}}]
}}]
}"#;

pub const WORKER_DOCUMENT: &[u8] = br#"{
"workers":[{
"engine":{"socket":"127.0.0.1:7200"},"name":"worker-a","secret":"worker.secret","slots":1,
"graces":{"grace_ms":10000,"agent_stop_ms":5000,"delivery_ms":1000,"save_ms":1000},
"agent":{"program":"agent-program","configuration":"agent.json","arguments":["agent"]},
"limits":{"profile":"small","overrides":{"hub.slots":1,"hub.run_calls":1}},
"memory_bytes":1152921504606846976
}],"application":{"workers":[{"root":"agent-root"}]}}
"#;

pub(crate) fn engine_document() -> Vec<u8> {
    let mut document = b"{\"engine\":".to_vec();
    document.extend_from_slice(ENGINE);
    document.extend_from_slice(
        br#", "application":{"credential":"app.secret","connectors":[
        {"prefix":["one"],"resources":[]},{"prefix":["two"],"resources":[]}]},
        "service":{"components":["accounts","workers"]}}"#,
    );
    document
}
pub(crate) fn files(worker: bool) -> Files {
    let inputs: &[(&[u8], &[u8])] = if worker {
        &[(b"worker.secret", b"worker-secret"), (b"agent.json", b"{}")]
    } else {
        &[
            (b"charters/chat.txt", b"Reply to the person."),
            (b"secrets/worker-a.digest", DIGEST),
            (b"accounts/chat.json", ACCOUNT),
            (b"app.secret", b"application-token"),
        ]
    };
    let mut entries = List::with_capacity(u32::try_from(inputs.len()).expect("fixture files"));
    for (name, bytes) in inputs {
        entries.push(((*name).into(), Secret::new((*bytes).into()))).expect("fixture files");
    }
    Files { entries }
}
pub(crate) fn seeds() -> Seeds {
    Seeds { deployment: [7; 16], seed: SEED, crypto: Secret::new(Box::new([7; 32])) }
}
pub(crate) fn endpoints() -> smith_protocol_channel::Endpoints {
    let mut entries = List::with_capacity(1);
    entries
        .push(smith_protocol_channel::Endpoint { name: b"fake".as_slice().into(), number: 0, dialect: 0, account: 1 })
        .expect("one endpoint");
    smith_protocol_channel::Endpoints::new(entries)
}
pub(crate) fn agent() -> jig_test_service::Agent {
    use skein_lib::Duration;
    let agent_limits = smith_agent_process_world::limits();
    let host_limits = smith_host_domain::Limits {
        agents: 1,
        directories: 1,
        conflicts: 0,
        path_bytes: 64,
        name_bytes: 64,
        accounts: 2,
        charter_bytes: 65_536,
        transcript_bytes: 262_144,
        answered_bytes: 65_536,
        message_bytes: 65_536,
        messages: 8,
        calls: 1,
        call_bytes: 65_536,
        answer_bytes: smith_host_domain::Delivered::worst_case(),
        turns: 8,
        turn_bytes: 65_536,
        unacknowledged_bytes: smith_domain::max_turn_bytes(&agent_limits.domain).expect("one concrete agent turn"),
        fact_bytes: 65_536,
        outcome_bytes: 4096,
        detail_bytes: 64,
        spawn_timeout: Duration::from_secs(1),
        no_progress: Duration::from_secs(120),
        silence: Duration::from_secs(60),
        long_span: Duration::from_secs(60),
        long_open: 1,
        wall_time: Duration::from_secs(300),
        cancel_grace: Duration::from_secs(5),
        exit_grace: Duration::from_secs(5),
        teardown: Duration::from_secs(1),
        kill_after: Duration::from_secs(2),
        facts: 1,
    };
    let channel = agent_limits.channel;
    let mut limits = smith_host_protocol::Limits {
        bodies: channel.bodies,
        charter: channel.charter,
        transcript: channel.transcript,
        endpoints: channel.endpoints,
        channel: channel.channel,
        start_bytes: channel.start_bytes,
        calls: host_limits.calls,
    };
    limits.channel.output_bytes = smith_host_protocol::output_bytes(&limits).expect("host frames");
    jig_test_service::Agent {
        endpoints: endpoints(),
        limits,
        host_limits,
        bounds: jig_domain_host::Bounds {
            models: Box::new([jig_domain_host::ModelBound {
                name: b"fake".as_slice().into(),
                window: 1_048_576,
                output: 512,
            }]),
            turns: 8,
            spend: 100,
            time: Duration::from_secs(300),
            waiting: Duration::from_secs(60),
            checks: Duration::from_secs(60),
            message_bytes: 65_536,
            inbox: 8,
            unacknowledged: 8,
        },
    }
}

pub(crate) fn agent_configuration() -> smith_agent_service::Config {
    let mut config = smith_agent_process_world::configuration();
    config.limits.llm.accounts = 2;
    config.channel_endpoints = endpoints();
    let (destinations, _) = config.llm_endpoints.into_parts();
    let destination = destinations.into_boxed().into_vec().pop().expect("one provider destination");
    config.llm_endpoints = smith_protocol_llm::Endpoints::new(
        Box::new([smith_protocol_llm::ConfiguredEndpoint {
            name: smith_domain::llm::Endpoint(0),
            destination,
            account: 1,
            connect: Duration::from_secs(1),
            handshake: Duration::from_secs(1),
            reasoning_effort: None,
            identity: smith_protocol_llm::IdentityProfile::Plain,
            models: Box::default(),
        }]),
        1,
        2,
    )
    .expect("provider account");
    config
}
