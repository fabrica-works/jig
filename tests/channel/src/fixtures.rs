//! One bounded chat deployment and independent provider script (channel.md, 12).
use crate::world::{Script, Settings};
use jig_test_service::{Files, Seeds};
use skein_lib::{Duration, List, Secret};

pub const SEED: u64 = 7;
pub const TOKEN: &[u8] = b"test-token";
const DIGEST: &[u8] = b"6fb46f7a92742970166379ed5195e79c4493a7cc5664280c039cfd4095ba5faf";
const REPORT: &[u8] = br#"{"form":"report","text":"Hello","fields":{}}"#;
const ACCOUNT: &[u8] = br#"{"access_token":"test-token","expires_at":1767229200,"account_id":"acc"}"#;

#[must_use]
pub fn calm(seed: u64) -> Settings {
    Settings { seed, config: skein_sim::Config::calm() }
}
#[must_use]
pub fn one_chat(words: &[u8]) -> Script {
    Script {
        words: words.into(),
        result: REPORT.into(),
        leading_turns: 0,
        latency: Duration::ZERO,
        story: crate::stories::Story::Chat,
        recorded: None,
    }
}

const ENGINE: &[u8] = br#"{
"limits": {"profile": "small", "overrides": {
"tasks.result_bytes":32,"people.amendment_bytes":64,"core.policy_bytes":256,"tasks.spec_bytes":128,"tasks.message_bytes":512,"tasks.hold_bytes":64,"tasks.authority_bytes":64,"people.words":512,"core.call_answer_bytes":2048,"notes.recalled":1,
"people.operations":2,"tasks.tasks":2,"tasks.funders":2,"tasks.project_tasks":1,"tasks.tree_tasks":1,"tasks.project_goals":1,
"tasks.batch":1,"tasks.dependencies":1,"tasks.delegates":1,"tasks.charters":1,
"people.people":1,"people.projects":1,"people.sign_ins":1,"people.holdings":1,
"people.inbox_tasks":4,"people.pending":1,"people.waiters":1,
"fleet.attempts":1,"fleet.calls":1,"fleet.turns":8,
"brief.briefs":1,"brief.sections":1,
"notes.entries_per_scope":1,"views.runs":1,"views.watchers":2,"views.watchers_per_party":2,
"core.load_slots":1,"core.load_bytes":8192,"core.resume_bytes":4096,"core.run_bytes":4096,"views.text_bytes":1024
,"core.templates":2,"documents.rows":1,"documents.lines":1,"documents.chips":1,"documents.text_bytes":128
}},
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
"templates": [
{"number": 1, "name": "chat", "executor": {"charter": 1}, "contract": {"report": {"words": 32}}, "shallower": 1, "budget": {"as_allowed": null}, "deadline": "none", "tracked": null, "conversation": true, "listed": false, "retention": "kept", "label": 1, "grace_ms": 5000},
{"number": 2, "name": "goal", "executor": "chosen", "contract": {"report": {"words": 32}}, "shallower": 1, "budget": "given", "deadline": "given", "tracked": {"priority": 1}, "conversation": false, "listed": true, "retention": "kept", "label": 2, "grace_ms": null}
],
"clients": {
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
"policy": {"templates": [], "escalation_role": 0, "period_spend": 100, "projections": [], "requirements": [],
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
"limits":{"profile":"small","overrides":{"hub.slots":1,"hub.run_calls":1,"tasks.message_bytes":512,"people.words":512,"core.call_answer_bytes":2048}},
"memory_bytes":1152921504606846976
}],"application":{"workers":[{"root":"agent-root"}]}}
"#;

#[must_use]
pub fn engine_document() -> Vec<u8> {
    let mut document = b"{\"engine\":".to_vec();
    document.extend_from_slice(ENGINE);
    document.extend_from_slice(
        br#", "application":{"server":{"listener":{"socket":"127.0.0.1:7100"}},"credential":"app.secret","connectors":[
        {"prefix":["one"],"resources":[]},{"prefix":["two"],"resources":[]}]},
        "service":{"components":["accounts","workers"]}}"#,
    );
    document
}
#[must_use]
pub fn files(worker: bool) -> Files {
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
#[must_use]
pub fn seeds() -> Seeds {
    Seeds { deployment: [7; 16], seed: SEED, crypto: Secret::new(Box::new([7; 32])) }
}
pub(crate) fn endpoints() -> smith_protocol_channel::Endpoints {
    let mut entries = List::with_capacity(1);
    entries
        .push(smith_protocol_channel::Endpoint { name: b"fake".as_slice().into(), number: 0, dialect: 0, account: 1 })
        .expect("one endpoint");
    smith_protocol_channel::Endpoints::new(entries)
}
#[must_use]
pub fn agent() -> jig_test_service::Agent {
    use skein_lib::Duration;
    let agent_limits = smith_agent_process_world::limits();
    let host_limits = smith_host_domain::Limits {
        agents: 1,
        directories: 1,
        conflicts: 0,
        path_bytes: 64,
        name_bytes: 64,
        accounts: 2,
        charter_bytes: 4096,
        transcript_bytes: 16_384,
        answered_bytes: 65_536,
        message_bytes: 65_536,
        messages: 8,
        calls: 1,
        call_bytes: 65_536,
        answer_bytes: smith_host_domain::Delivered::worst_case(),
        turns: 8,
        turn_bytes: 8192,
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

#[must_use]
pub fn agent_configuration() -> smith_agent_service::Config {
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
            identity: smith_protocol_llm::IdentityProfile::Plain,
            models: Box::default(),
        }]),
        1,
        2,
    )
    .expect("provider account");
    config
}

pub(crate) fn peer_bounds() -> (skein_channel::Schema, skein_channel::Limits) {
    let profile = store_limits();
    let sizes = profile.workers.sizes;
    let codec = jig_protocol_worker_wire::sizes::limits(&sizes).expect("small wire");
    let cap = jig_protocol_worker_wire::sizes::worker_cap(&sizes).expect("peer output").cap;
    let limits = skein_channel::Limits {
        chunk: 16_384,
        credential: 4096,
        skip: 16_384,
        output_bytes: cap.bytes,
        output_frames: cap.frames,
        kinds: 15,
    };
    (jig_protocol_worker_wire::schema(&codec).expect("small schema"), limits)
}

pub(crate) fn engine_call_document() -> Vec<u8> {
    String::from_utf8(engine_document())
        .expect("fixture UTF-8")
        .replace("\"tools\": 0", "\"tools\": 256")
        .replace("\"notes\": 0", "\"notes\": 1")
        .into_bytes()
}

pub const TASK: u64 = 1;
pub const ATTEMPT: u64 = 1;
#[must_use]
pub fn referee() -> crate::referee::Rules {
    crate::referee::Rules::default()
}

pub(crate) fn engine_window_document() -> Vec<u8> {
    String::from_utf8(engine_document()).expect("fixture UTF-8").replace("\"turns\": 4", "\"turns\": 8").into_bytes()
}

pub(crate) fn peer_hello() -> skein_channel::Frame {
    use jig_protocol_worker_wire as wire;
    let codec = &wire::CEILINGS;
    let mut models = List::with_capacity(1);
    models
        .push(
            wire::ModelBound::new(
                codec,
                wire::ModelBoundParts { name: b"fake".as_slice().into(), window: 1_048_576, output: 512 },
            )
            .expect("peer model"),
        )
        .expect("one model");
    let bounds = wire::Bounds::new(
        codec,
        wire::BoundsParts {
            models,
            turns: 8,
            spend: 100,
            time: Duration::from_secs(300),
            waiting: Duration::from_secs(60),
            checks: Duration::from_secs(60),
            message_bytes: 65_536,
            inbox: 8,
            unacknowledged: 8,
        },
    )
    .expect("peer bounds");
    let hello = wire::Hello::new(
        codec,
        wire::HelloParts {
            slots: 1,
            stop_bound: Duration::from_secs(15),
            workstreams: List::with_capacity(0),
            hosting: List::with_capacity(0),
            bounds,
        },
    )
    .expect("peer hello");
    let mut body = skein_lib::Writer::new(hello.measure() as usize);
    hello.encode(&mut body).expect("peer hello");
    let mut frame = skein_channel::frame_writer(0x0100, hello.measure()).expect("hello frame");
    frame.put(&body.finish()).expect("hello body");
    frame.finish().expect("hello frame")
}

#[must_use]
pub fn provider_turns(script: &Script) -> Box<[skein_fake_llm_domain::api::Turn]> {
    use skein_fake_llm_domain::api::{Finish, Line, Turn};
    if matches!(script.story, crate::stories::Story::Window) {
        return vec![
            Turn {
                lines: Box::new([Line::Text { text: b"Progress".as_slice().into() }]),
                finish: Finish::Stop,
                tokens: 1,
            },
            Turn {
                lines: Box::new([Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() }]),
                finish: Finish::ToolCalls,
                tokens: 1,
            },
            Turn {
                lines: Box::new([Line::Text { text: b"Current summary".as_slice().into() }]),
                finish: Finish::Stop,
                tokens: 1,
            },
            Turn {
                lines: Box::new([Line::Call { name: b"finish".as_slice().into(), arguments: script.result.clone() }]),
                finish: Finish::ToolCalls,
                tokens: 1,
            },
        ]
        .into_boxed_slice();
    }
    let mut turns = Vec::new();
    for number in 0..script.leading_turns {
        turns.push(Turn {
            lines: Box::new([if matches!(script.story, crate::stories::Story::Call) {
                Line::Call {
                    name: b"recall".as_slice().into(),
                    arguments: br#"{"kind":"name","name":1}"#.as_slice().into(),
                }
            } else if matches!(script.story, crate::stories::Story::Wait | crate::stories::Story::InvalidTranscript) {
                Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() }
            } else {
                Line::Text { text: format!("Progress {number}").into_bytes().into() }
            }]),
            finish: if matches!(
                script.story,
                crate::stories::Story::Call | crate::stories::Story::Wait | crate::stories::Story::InvalidTranscript
            ) {
                Finish::ToolCalls
            } else {
                Finish::Stop
            },
            tokens: 1,
        });
    }
    turns.push(Turn {
        lines: Box::new([Line::Call { name: b"finish".as_slice().into(), arguments: script.result.clone() }]),
        finish: Finish::ToolCalls,
        tokens: 1,
    });
    turns.into_boxed_slice()
}

pub(crate) fn party_script(script: &Script) -> Box<[jig_fake_parties::Act]> {
    if matches!(
        script.story,
        crate::stories::Story::Stalled | crate::stories::Story::Quiet | crate::stories::Story::Replaced
    ) {
        return Box::default();
    }
    let mut acts = vec![
        jig_fake_parties::Act::SignIn { provider: 1, subject: b"owner-a".as_slice().into() },
        jig_fake_parties::Act::Request {
            key: [1; 16],
            ask: jig_fake_parties::Ask::Chat { words: script.words.clone() },
        },
    ];
    if matches!(script.story, crate::stories::Story::Inbox) {
        acts.push(jig_fake_parties::Act::Request {
            key: [2; 16],
            ask: jig_fake_parties::Ask::Say {
                task: jig_fake_parties::Task::Last,
                words: b"From the person".as_slice().into(),
                release: None,
            },
        });
    }
    acts.into_boxed_slice()
}

pub(crate) fn engine_message_document() -> Vec<u8> {
    engine_document()
}

/// Match the fixture's final admitted core bounds, parsed once per test process.
#[must_use]
pub fn store_limits() -> jig_test_service::limits::Limits {
    static LIMITS: std::sync::OnceLock<jig_test_service::limits::Limits> = std::sync::OnceLock::new();
    *LIMITS.get_or_init(|| {
        let bounds = jig_protocol_config::DocumentLimits {
            bytes: u32::try_from(ENGINE.len()).expect("fixture document size"),
            tokens: u32::try_from(ENGINE.len()).expect("fixture document size"),
            depth: 32,
        };
        let document = jig_protocol_config::document(ENGINE, &bounds).expect("fixture document");
        let engine = jig_protocol_config::engine(&document, 0).expect("fixture engine");
        jig_test_service::limits::for_profile(jig_protocol_config::Profile::Small, &engine.limits)
    })
}

#[must_use]
pub fn people_member(words: &[u8], spend: u64) -> jig_domain_people::Member {
    use jig_domain_people as people;
    people::Member {
        deadline: jig_domain_people::Deadline::Named,
        executor: people::TaskExecutor::Agent { charter: 1 },
        words: words.into(),
        parameters: Box::new([]),
        inputs: Box::new([]),
        budget: people::Budget::Named,
        named: Box::new([]),
        contract: people::Contract::Report { words: 16 },
        authority: people::Authority {
            tools: 0,
            grants: Box::new([]),
            delegation: people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
            spend,
            deadline: None,
            notes: 0,
            note_resources: Box::new([]),
        },
        wake: people::WakePolicy {
            words: people::WakeRule::Immediate,
            notices: people::WakeRule::Never,
            news: people::WakeRule::Never,
            results: people::ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        },
        dependencies: Box::new([]),
        tracked: None,
        label: 0,
        flags: people::Flags::PLAIN,
    }
}

#[must_use]
pub fn chat(store: &crate::world::Store, project: u32, words: &[u8]) -> jig_domain_people::Ask {
    use jig_domain_people as people;
    let mut member = people_member(words, 0);
    let Some(jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Policy { value, .. }))) =
        store.rows.get(&jig_test_domain::Key::Core(jig_domain_core::Key::People(people::Key::Policy(project))))
    else {
        panic!("durable configured policy")
    };
    member.authority = value.roles.iter().find(|role| role.number == 0).expect("fixture role").authority.clone();
    member.authority.spend = 0;
    member.authority.delegation.tasks =
        member.authority.delegation.tasks.min(store_limits().root.core.tasks.tree_tasks).saturating_sub(1);
    member.authority.delegation.depth = member.authority.delegation.depth.saturating_sub(1);
    member.budget = people::Budget::AsAllowed { cap: None };
    member.contract = people::Contract::Report { words: 32 };
    member.flags = people::Flags { conversation: true, listed: false, retention: people::Retention::Kept };
    member.label = jig_client_test_records::CHAT;
    people::Ask::Create { project, batch: Box::new([member]) }
}
