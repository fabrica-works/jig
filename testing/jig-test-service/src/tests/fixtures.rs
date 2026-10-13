use crate::{End, Files, PartyOutput, Seeds, Service, StoreAnswer, StoreRequest, iterate};
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{List, ReplyTo, Secret, Time, Token, Wall};
use skein_lib::{Queue, Writer};

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
"ceiling": {"tools": 1023, "grants": [],
"delegation": {"kinds": [{"kind": "charter", "number": 1}], "tasks": 8, "depth": 8},
"budget": {"spend": 100, "deadline_unix_seconds": null}, "notes": 7, "note_resources": []},
"period_spend": 100, "minimum_run_spend": 1, "maximum_run_spend": 100,
"implies": [], "requirements": []
},
"projects": [{"number": 1, "roles": [{"connector": 1, "permission": 1, "role": 0}],
"policy": {"escalation_role": 0, "period_spend": 100, "projections": [], "requirements": [],
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
pub fn started(document: &[u8]) -> Service {
    crate::startup(document, &files(document), None, &seeds(), None).expect("fixture startup")
}
pub fn turn(service: &mut Service) {
    iterate(service, Time::from_nanos(1), Wall::from_nanos(1));
}
pub fn problem(document: &[u8]) -> jig_protocol_config::Problem {
    let found = crate::startup(document, &Files { entries: List::with_capacity(0) }, None, &seeds(), None)
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
        for _ in 0..engine.store_requests().len() {
            match engine.store_requests().pop().expect("store request") {
                StoreRequest::Commit { number, .. } => {
                    self.commits.push(number);
                    if commit {
                        engine.store_answers().push(StoreAnswer::Committed { number, fill: 0 });
                    }
                }
                StoreRequest::Restart(step) => {
                    engine.store_answers().push(StoreAnswer::Restored { step, rows: Box::new([]) });
                }
                StoreRequest::Load(_) | StoreRequest::Read(_) => panic!("fixture has no stored history"),
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
                let _number = self.commits.pop();
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
        match self.commits.pop() {
            Some(number) => number,
            None => panic!("a commit left this iteration; last reply {:?}, sign in {:?}", self.reply, self.sign_in),
        }
    }
    pub fn commit(&mut self, number: u64) {
        engine(&mut self.service).store_answers().push(StoreAnswer::Committed { number, fill: 0 });
        self.turn(false);
    }
    pub fn fail(&mut self, number: u64) {
        engine(&mut self.service).store_answers().push(StoreAnswer::Failed { number });
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
            ask: people::Ask::StartChat { project: 1, words: words.into() },
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
