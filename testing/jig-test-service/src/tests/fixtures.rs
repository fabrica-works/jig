pub(crate) mod family;
use crate::{End, Files, PartyOutput, Seeds, Service, iterate};
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{List, ReplyTo, Secret, Time, Token, Wall};
use skein_lib::{Queue, Writer};

const ENGINE: &[u8] = br#"{
"limits": {"profile": "small", "overrides": {"tasks.result_bytes":32,"people.amendment_bytes":64,"core.policy_bytes":256,"tasks.spec_bytes":128,"tasks.message_bytes":512,"tasks.hold_bytes":64,"tasks.authority_bytes":64,"people.words":512,"core.call_answer_bytes":2048,"notes.recalled":1}},
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
"name": "small",
"endpoint": "fake",
"window": 8192,
"output": 1024,
"prices": {"input": 1, "cached": 1, "output": 1, "unit": 1000}
}],
"accounts": [{
"number": 1,
"source": {"handed_in": {"file": "accounts/chat.json"}}
}],
"records": "accounts/records"
},
"hosts": {
"workers": [
{"name": "worker-a", "digest": "secrets/worker-a.digest"},
{"name": "worker-b", "digest": "secrets/worker-b.digest"}
],
"listener": [{"in_process": "workers"}]
},
"charters": [{
"number": 1, "instructions": "charters/chat.txt", "model": "small", "alternatives": [],
"hosts": "workers", "waiting_ms": 10000, "resume": true,
"turns": 4, "time_ms": 30000, "reserve_turns": 1, "reserve_time_ms": 5000, "spend": 20,
"inspect": true, "modify": false, "shell": false, "agents": false, "call_timeout_ms": 10000
}],
"templates": [
{"number": 1, "name": "chat", "executor": {"charter": 1}, "contract": {"report": {"words": 32}}, "shallower": 1, "budget": {"as_allowed": null}, "deadline": "none", "tracked": null, "conversation": true, "listed": false, "retention": "kept", "label": 1, "grace_ms": 5000},
{"number": 2, "name": "goal", "executor": "chosen", "contract": {"report": {"words": 32}}, "shallower": 1, "budget": "given", "deadline": "given", "tracked": {"priority": 1}, "conversation": false, "listed": true, "retention": "kept", "label": 2, "grace_ms": null}
],
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
"ceiling": {"tools": 1023, "grants": [],
"delegation": {"kinds": [{"kind": "charter", "number": 1}], "tasks": 8, "depth": 8},
"budget": {"spend": 100, "deadline_unix_seconds": null}, "notes": 7, "note_resources": []},
"period_spend": 100, "minimum_run_spend": 1, "maximum_run_spend": 100,
"implies": [], "requirements": []
},
"projects": [{"number": 1, "roles": [{"connector": 1, "permission": 1, "role": 0}],
"policy": {"templates": [], "escalation_role": 0, "period_spend": 100, "projections": [], "requirements": [],
"ceiling": {"tools": 1023, "grants": [],
"delegation": {"kinds": [{"kind": "charter", "number": 1}], "tasks": 8, "depth": 8},
"budget": {"spend": 100, "deadline_unix_seconds": null}, "notes": 7, "note_resources": []},
"roles": [{"number": 0, "period_spend": 100, "requests": 511, "decides": 31,
"authority": {"tools": 1023, "grants": [],
"delegation": {"kinds": [{"kind": "charter", "number": 1}], "tasks": 8, "depth": 8},
"budget": {"spend": 100, "deadline_unix_seconds": null}, "notes": 7, "note_resources": []}}]
}}]
}"#;

pub fn engine_document() -> Box<[u8]> {
    join(&[
        b"{\"engine\":",
        ENGINE,
        br#", "application":{"credential":"app.secret","connectors":[
    {"prefix":["one"],"resources":[]},{"prefix":["two"],"resources":[]}]},"service":{"components":[]}}"#,
    ])
}
pub fn join(parts: &[&[u8]]) -> Box<[u8]> {
    let mut size = 0_usize;
    for part in parts {
        size = size.checked_add(part.len()).expect("fixture size");
    }
    let mut writer = Writer::new(size);
    for part in parts {
        writer.put(part).expect("fixture size");
    }
    writer.finish()
}
pub fn with(document: &[u8], from: &[u8], to: &[u8]) -> Box<[u8]> {
    let at = skein_lib::bytes::find(document, from).expect("fixture replacement");
    join(&[&document[..at], to, &document[at.checked_add(from.len()).expect("fixture range")..]])
}
pub fn decimal(mut value: u64) -> Box<[u8]> {
    let mut digits = List::with_capacity(20);
    for _ in 0_u32..20 {
        digits
            .push(
                u8::try_from(value.checked_rem(10).expect("base ten"))
                    .expect("digit")
                    .checked_add(b'0')
                    .expect("digit"),
            )
            .expect("decimal length");
        value = value.checked_div(10).expect("base ten");
        if value == 0 {
            break;
        }
    }
    let mut writer = Writer::new(usize::try_from(digits.len()).expect("decimal length"));
    for digit in digits.iter().rev() {
        writer.put(&[*digit]).expect("decimal length");
    }
    writer.finish()
}

pub const NAMES: [&[u8]; 5] = [
    b"charters/chat.txt",
    b"secrets/worker-a.digest",
    b"secrets/worker-b.digest",
    b"accounts/chat.json",
    b"app.secret",
];
pub fn files(_document: &[u8]) -> Files {
    let mut entries = List::with_capacity(5);
    for name in NAMES {
        let contents = match name {
            b"accounts/chat.json" => ACCOUNT,
            b"secrets/worker-a.digest" => DIGEST_A,
            b"secrets/worker-b.digest" => DIGEST_B,
            _ => b"fixture contents",
        };
        entries.push((name.into(), Secret::new(contents.into()))).expect("named files");
    }
    Files { entries }
}
pub fn seeds() -> Seeds {
    Seeds { deployment: [7; 16], seed: 7, crypto: Secret::new(Box::new([7; 32])) }
}
pub fn journal(limits: &jig_test_domain::Limits) -> skein_lib::JournalLimits {
    let room = core::room_max(&limits.core).expect("final core limits");
    let connector = jig_test_connector::MAX_OUT * 2;
    skein_lib::JournalLimits {
        writes: room.writes.checked_add(connector).expect("journal writes").max(2000),
        held: room
            .held
            .checked_add(connector)
            .expect("journal held")
            .max(2000)
            .checked_mul(limits.journal.commits)
            .expect("journal commits"),
        ..limits.journal
    }
}
pub fn started(document: &[u8]) -> Service {
    let mut parsed = crate::startup::parse(document).expect("fixture parsed");
    parsed.limits.root.journal = journal(&parsed.limits.root);
    let (mut engine, io, bytes, _) =
        crate::startup::engine_startup(parsed, &files(document), None, &seeds(), None, false).expect("fixture startup");
    memory_store(&mut engine);
    Service::new(Some(engine), List::with_capacity(0), io, bytes, engine_queue())
}
fn engine_queue() -> u32 {
    jig_test_domain::max_out(&crate::limits::profile_limits(jig_protocol_config::Profile::Small).root)
        .checked_mul(2)
        .expect("engine queue")
}
pub fn memory_store(engine: &mut crate::Engine) {
    let limits = crate::store::limits(
        &engine.env.limits,
        &crate::limits::profile_limits(jig_protocol_config::Profile::Small).store,
    )
    .expect("final admitted fixture store");
    engine.store.component = jig_protocol_store::Store::over(
        &limits,
        jig_test_store::application(&engine.env.limits.connector),
        jig_protocol_store::Stores {
            store: skein_kv::Store::memory(&limits.store),
            secrets: skein_kv::Store::memory(&limits.secrets),
        },
    );
    engine.store.ask(jig_protocol_store::Request::Open {
        owner: OWNER,
        root: jig_protocol_store::Root { token: OWNER, fd: skein_io::kernel::Fd::new(1) },
    });
}
pub fn turn(service: &mut Service) {
    iterate(service, Time::from_nanos(1), Wall::from_nanos(1));
}
pub fn problem(document: &[u8]) -> jig_protocol_config::Problem {
    let found = crate::startup(document, &Files { entries: List::with_capacity(0) }, None, &seeds(), None, None)
        .err()
        .expect("startup refused");
    found.problems.get(0).expect("one startup problem").clone()
}
pub const OWNER: Token = Token::new(1);

pub struct Driven {
    pub service: Service,
    pub commits: Queue<u64>,
    pub released: usize,
    pub busy: Queue<Box<jig_test_domain::Event>>,
    pub reply: Option<people::Reply>,
    pub sign_in: Token,
    pub held: alloc::collections::BTreeMap<u64, jig_protocol_store::Event>,
}
impl Driven {
    pub fn started(document: &[u8]) -> Self {
        let mut driven = Self {
            service: started(document),
            commits: Queue::with_capacity(16),
            released: 0,
            busy: Queue::with_capacity(16),
            reply: None,
            sign_in: OWNER,
            held: alloc::collections::BTreeMap::new(),
        };
        for _ in 0_u32..64 {
            driven.turn(true);
            if engine(&mut driven.service).domain.ready() {
                break;
            }
        }
        assert!(engine(&mut driven.service).domain.ready(), "restart completed before sign-in");
        driven.party(core::Event::Period { project: 1, period: 1, budget: 100, next: None });
        driven.commit_all();
        driven.party(core::Event::SignIn {
            reply_to: ReplyTo::new(Token::new(10)),
            identity: people::Identity {
                key: people::IdentityKey { provider: 1, subject: b"owner-a".as_slice().into() },
                login: b"owner".as_slice().into(),
                name: b"Owner".as_slice().into(),
            },
        });
        driven.commit_all();
        let Some(people::Reply::SignedIn { .. }) = driven.reply else {
            panic!("sign-in completed: {:?}", driven.reply);
        };
        driven.released = 0;
        driven
    }
    pub fn party(&mut self, event: core::Event) {
        engine(&mut self.service).parties_in().push(event);
        self.turn(false);
    }
    pub fn turn(&mut self, commit: bool) {
        turn(&mut self.service);
        let engine = engine(&mut self.service);
        for _ in 0..engine.store.above.len() {
            let event = engine.store.above.pop().expect("store terminal");
            if let jig_protocol_store::Event::Committed { number, .. } = &event {
                self.commits.push(*number);
                self.held.insert(*number, event);
            } else {
                engine.store.above.push(event);
            }
        }
        for _ in 0..engine.parties_out().len() {
            let output = engine.parties_out().pop().expect("party output");
            match output {
                PartyOutput::Held(jig_test_domain::Delivery::Core(core::Held::PeopleReply { reply, .. })) => {
                    if let people::Reply::SignedIn { sign_in, .. } = reply {
                        self.sign_in = Token::new(sign_in);
                    }
                    self.reply = Some(reply);
                }
                PartyOutput::Busy(event) => self.busy.push(event),
                PartyOutput::Held(_) | PartyOutput::Now(_) => {}
            }
            self.released = self.released.checked_add(1).expect("released count");
        }
        if commit {
            for _ in 0..self.commits.len() {
                let number = self.commits.pop().expect("held commits");
                if let Some(event) = self.held.remove(&number) {
                    engine.store.above.push(event);
                }
            }
        }
    }
    pub fn commit_all(&mut self) {
        for _ in 0..self.commits.len() {
            let number = self.commits.pop().expect("commit count");
            self.commit(number);
        }
        for _ in 0_u32..4 {
            self.turn(true);
        }
    }
    pub fn commit_asked(&mut self) -> u64 {
        self.commits.pop().expect("a commit left this iteration")
    }
    pub fn terminal(&mut self, number: u64) -> jig_protocol_store::Event {
        self.held.remove(&number).expect("real commit terminal held by the fixture")
    }
    pub fn commit(&mut self, number: u64) {
        let event = self.terminal(number);
        engine(&mut self.service).store.above.push(event);
        self.turn(false);
    }
    pub fn fail(&mut self, number: u64) {
        self.held.remove(&number);
        engine(&mut self.service).store.above.push(jig_protocol_store::Event::Failed { number });
        self.turn(false);
    }
    pub fn released_to_parties(&self) -> usize {
        self.released
    }
    pub fn end(&self) -> Option<&End> {
        self.service.end()
    }
    pub fn chat(&mut self, number: u64, words: &[u8]) {
        self.party(core::Event::People(people::Event::Ask {
            reply_to: ReplyTo::new(Token::new(number)),
            sign_in: self.sign_in.raw(),
            key: u128::from(number).to_be_bytes(),
            ask: chat(1, words),
        }));
    }
}

pub fn engine(service: &mut Service) -> &mut crate::Engine {
    service.engine().expect("fixture engine")
}

pub const ACCOUNT: &[u8] = br#"{"access_token":"test-token","expires_at":120,"account_id":"acc"}"#;
pub const TOKEN: &[u8] = b"test-token";
pub const SIGNED_SOURCE: &[u8] = br#"{"sign_in":{"client":"client","authorization":"http://127.0.0.1:7101/authorize","token":"http://127.0.0.1:7101/token","endpoint":{"socket":"127.0.0.1:7101"},"transport":"plaintext","redirect":"http://localhost:7123/callback","scope":"read"}}"#;
pub const DIGEST_A: &[u8] = b"8766b9cb08e6040b704f1e3ee1e186efccf2635b1d2634d6525333007e6aeae1";
pub const DIGEST_B: &[u8] = b"ff492ef788c89b555e6f738b33d2422f57dbb6656af2402155672c5f123a90af";

pub fn accounts_document() -> Box<[u8]> {
    with(&engine_document(), br#""components":[]"#, br#""components":["accounts"]"#)
}
pub fn workers_document() -> Box<[u8]> {
    with(&engine_document(), br#""components":[]"#, br#""components":["accounts","workers"]"#)
}
pub fn config() -> jig_protocol_config::Engine {
    config_document(ENGINE)
}
pub fn signed_config() -> jig_protocol_config::Engine {
    config_document(&with(ENGINE, br#"{"handed_in": {"file": "accounts/chat.json"}}"#, SIGNED_SOURCE))
}
fn config_document(input: &[u8]) -> jig_protocol_config::Engine {
    let bounds = jig_protocol_config::DocumentLimits { bytes: 8192, tokens: 4096, depth: 32 };
    let document = jig_protocol_config::document(input, &bounds).expect("fixture JSON");
    let mut config = jig_protocol_config::engine(&document, 0).expect("fixture config");
    for charter in &mut config.charters {
        charter.policy.instructions = b"fixture contents".as_slice().into();
    }
    config
}
pub fn values() -> crate::values::Values {
    let mut accounts = jig_protocol_accounts::table::Table::new(2, 32);
    accounts
        .granted(
            1,
            1,
            jig_protocol_accounts::table::Value {
                token: Secret::new(TOKEN.into()),
                account_id: Secret::new(b"acc".as_slice().into()),
            },
            Time::from_nanos(120_000_000_000),
        )
        .expect("fixture value");
    let mut application = skein_lib::Map::with_capacity(1);
    application.insert(1, Secret::new(b"application-token".as_slice().into())).expect("application value");
    crate::values::Values { accounts, application }
}
pub fn workers(config: &jig_protocol_config::Engine) -> crate::workers::Workers {
    let limits = crate::limits::for_profile(jig_protocol_config::Profile::Small, &config.limits).workers;
    crate::workers::Workers::new(config, &files(&[]), seeds().deployment, &limits, 64).expect("fixture workers")
}
pub fn assignment(config: &jig_protocol_config::Engine) -> jig_test_domain::Assignment {
    use jig_domain_accounts::{Grant, GrantTable};
    use jig_domain_brief::{Core, GatherPlaced};
    use skein_lib::Duration;
    jig_test_domain::Assignment {
        conversation: 3,
        task: 2,
        attempt: 3,
        charter: 1,
        run: Box::new(core::RunCharter {
            policy: config.charters[0].policy.clone(),
            contract: jig_domain_tasks::Contract::Report { words: 64 },
            authority: jig_domain_tasks::Authority {
                tools: jig_domain_tasks::Tools(1023),
                grants: Box::default(),
                delegation: jig_domain_tasks::Delegation { kinds: Box::default(), tasks: 0, depth: 0 },
                budget: jig_domain_tasks::Budget { spend: 20, deadline: None },
                notes: jig_domain_tasks::Scopes(0),
                note_resources: Box::default(),
            },
            budget: 20,
        }),
        brief: Box::new([
            GatherPlaced::Core { kind: Core::Task, text: b"Make a report".as_slice().into() },
            GatherPlaced::Core { kind: Core::Results, text: b"Prior result".as_slice().into() },
        ]),
        messages: Box::default(),
        transcript: Box::default(),
        answered: Box::default(),
        grants: Box::new([
            Grant { table: GrantTable::Accounts, account: 1, generation: 1, valid: Duration::from_secs(30) },
            Grant { table: GrantTable::Application, account: 1, generation: 1, valid: Duration::from_secs(30) },
        ]),
    }
}
pub fn endpoints() -> smith_protocol_channel::Endpoints {
    let mut entries = List::with_capacity(1);
    entries
        .push(smith_protocol_channel::Endpoint { name: b"fake".as_slice().into(), number: 1, dialect: 0, account: 1 })
        .expect("endpoint");
    smith_protocol_channel::Endpoints::new(entries)
}
pub fn call(input: &[u8]) -> jig_domain_fleet::Call {
    let name = smith_channel::v2::CallName::new(
        &smith_channel::v2::CEILINGS,
        smith_channel::v2::CallNameParts { activation: 3, completion: 7, position: 2 },
    )
    .expect("call name");
    let mut writer = Writer::new(usize::try_from(name.measure()).expect("call name length"));
    name.encode(&mut writer).expect("call name size");
    jig_domain_fleet::Call {
        name: writer.finish(),
        tool: b"cancel".as_slice().into(),
        writes: true,
        input: input.into(),
        deadline: skein_lib::Duration::from_secs(1),
    }
}

pub const WORKER_DOCUMENT: &[u8] = br#"{
"workers":[{
"engine":{"socket":"127.0.0.1:7200"},"name":"worker-a","secret":"worker.secret","slots":1,
"graces":{"grace_ms":10000,"agent_stop_ms":5000,"delivery_ms":1000,"save_ms":1000},
"agent":{"program":"agent-program","configuration":"agent.json","arguments":["agent"]},
"limits":{"profile":"small","overrides":{"hub.slots":1,"hub.run_calls":1,"hub.turns":1}},
"memory_bytes":1152921504606846976
}],"application":{"workers":[{"root":"agent-root"}]}}
"#;
pub fn worker_files() -> Files {
    let mut entries = List::with_capacity(2);
    entries
        .push((b"worker.secret".as_slice().into(), Secret::new(b"worker-secret".as_slice().into())))
        .expect("secret file");
    entries.push((b"agent.json".as_slice().into(), Secret::new(b"{}".as_slice().into()))).expect("agent configuration");
    Files { entries }
}
pub fn agent() -> crate::Agent {
    use skein_lib::Duration;
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
        messages: 2,
        calls: 1,
        call_bytes: 65_536,
        answer_bytes: smith_host_domain::Delivered::worst_case(),
        turns: 1,
        turn_bytes: 65_536,
        unacknowledged_bytes: 65_536,
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
    let mut limits = smith_host_protocol::Limits {
        bodies: smith_channel::Limits {
            turn_body: 65_536,
            accepted_result: 4096,
            host_ask_input: 4096,
            message_text: 65_536,
            host_reply_text: 65_536,
            start_charter: 65_536,
            start_transcript: 262_144,
            start_answered: 4,
            start_grants: 2,
            start_messages: 2,
            ..smith_channel::CEILINGS
        },
        charter: smith_charter::CEILINGS,
        transcript: smith_transcript::CEILINGS,
        endpoints: 1,
        channel: skein_channel::Limits {
            chunk: 64,
            credential: 1,
            skip: 1_048_576,
            output_bytes: 1_048_576,
            output_frames: 8,
            kinds: 32,
        },
        start_bytes: 1_048_576,
        calls: 1,
    };
    limits.channel.output_bytes = smith_host_protocol::output_bytes(&limits).expect("host frames");
    crate::Agent {
        endpoints: endpoints(),
        limits,
        host_limits,
        bounds: jig_domain_host::Bounds {
            models: Box::new([jig_domain_host::ModelBound {
                name: b"small".as_slice().into(),
                window: 8192,
                output: 1024,
            }]),
            turns: 8,
            spend: 100,
            time: Duration::from_secs(300),
            waiting: Duration::from_secs(60),
            checks: Duration::from_secs(60),
            message_bytes: 65_536,
            inbox: 2,
            unacknowledged: 1,
        },
    }
}
pub fn worker_runtime() -> crate::WorkerRuntime {
    let mut roots = List::with_capacity(1);
    roots.push(skein_io::kernel::Fd::new(9)).expect("worker root");
    crate::WorkerRuntime { roots }
}
pub fn worker_service() -> Service {
    crate::startup(WORKER_DOCUMENT, &worker_files(), Some(&agent()), &seeds(), None, Some(worker_runtime()))
        .expect("worker service")
}
pub fn worker_assignment() -> jig_domain_host::LinkDown {
    let names =
        [jig_protocol_charter::EndpointName { number: 1, name: b"fake".as_slice().into(), account: 1, dialect: 0 }];
    let mut assignment =
        crate::tools::charter(assignment(&config()), &names, &seeds().deployment).expect("charter assignment");
    assignment.assignment.identity = [3; 16];
    assignment.assignment.run = Token::new(7);
    assignment.assignment.attempt = Token::new(3);
    assignment.assignment.grants = Box::new([jig_domain_host::Grant {
        table: jig_domain_host::GrantTable::Accounts,
        account: 1,
        generation: 1,
        valid: skein_lib::Duration::from_secs(30),
    }]);
    jig_domain_host::LinkDown::Assign { assignment }
}
pub fn concrete_turn() -> smith_host_domain::Turn {
    use smith_domain_session::{llm, record};
    let value = record::Turn {
        model: b"model".as_slice().into(),
        effort: None,
        version: record::VERSION,
        endpoint: llm::Endpoint(1),
        dialect: 0,
        sequence: 1,
        kind: record::Kind::First { affinity: llm::Affinity { key: [3; 16], thread: 1 } },
        usage: llm::Usage {
            input_tokens: Some(1),
            output_tokens: Some(1),
            cache_read_tokens: Some(0),
            cache_write_tokens: Some(0),
            reasoning_tokens: Some(0),
        },
        spent: 2,
        messages: Box::default(),
    };
    smith_host_domain::Turn {
        opens_window: false,
        number: 1,
        spent: 2,
        read: None,
        body: smith_host_domain::TurnValue::new(value, 65_536).expect("typed turn"),
    }
}

pub fn two_workers() -> Service {
    let start =
        skein_lib::bytes::find(WORKER_DOCUMENT, b"[{\n").expect("worker start").checked_add(1).expect("member start");
    let end = skein_lib::bytes::find(WORKER_DOCUMENT, b"],\"application\"").expect("worker end");
    let member = &WORKER_DOCUMENT[start..end];
    let second = with(member, b"worker-a", b"worker-b");
    let document = join(&[
        br#"{"workers":["#,
        member,
        b",",
        &second,
        br#"],"application":{"workers":[{"root":"agent-a"},{"root":"agent-b"}]}}"#,
    ]);
    let mut roots = List::with_capacity(2);
    roots.push(skein_io::kernel::Fd::new(9)).expect("first root");
    roots.push(skein_io::kernel::Fd::new(10)).expect("second root");
    crate::startup(&document, &worker_files(), Some(&agent()), &seeds(), None, Some(crate::WorkerRuntime { roots }))
        .expect("two worker layers")
}

pub fn combined_document() -> Box<[u8]> {
    let at = skein_lib::bytes::find(WORKER_DOCUMENT, b"],\"application\"")
        .expect("worker array end")
        .checked_add(1)
        .expect("array end");
    let workers = &WORKER_DOCUMENT[1..at];
    let document = with(&engine_document(), b"\"service\":", &join(&[workers, b",\"service\":"]));
    let document = with(
        &document,
        br#""application":{"credential":"#,
        br#""application":{"workers":[{"root":"agent-root"}],"credential":"#,
    );
    with(&document, br#""socket":"127.0.0.1:7200""#, br#""in_process":"workers""#)
}

pub fn combined_files(document: &[u8]) -> Files {
    let mut entries = List::with_capacity(7);
    for file in files(document).entries.into_boxed() {
        entries.push(file).expect("engine files");
    }
    for file in worker_files().entries.into_boxed() {
        entries.push(file).expect("worker files");
    }
    Files { entries }
}

pub fn service_bytes(document: &[u8]) -> u64 {
    let mut parsed = crate::startup::parse(document).expect("fixture parsed");
    let store = crate::store::limits(&parsed.limits.root, &parsed.limits.store).expect("fixture admitted");
    parsed.limits.store = store;
    crate::limits::worst_case(&parsed.limits)
        .expect("root bounds")
        .checked_add(crate::workers::configured_bytes(&parsed.engine).expect("configured bounds"))
        .expect("service total")
}

pub fn people_member(words: &[u8], spend: u64) -> jig_domain_people::Member {
    use jig_domain_people as people;
    people::Member {
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

pub fn chat(project: u32, words: &[u8]) -> jig_domain_people::Ask {
    use jig_domain_people as people;
    let mut member = people_member(words, 0);
    member.budget = people::Budget::AsAllowed { cap: None };
    member.authority.tools = 1023;
    member.authority.notes = 7;
    member.contract = people::Contract::Report { words: 32 };
    member.flags = people::Flags { conversation: true, listed: false, retention: people::Retention::Kept };
    member.label = jig_client::shapes::CHAT;
    people::Ask::Create { project, batch: Box::new([member]) }
}
