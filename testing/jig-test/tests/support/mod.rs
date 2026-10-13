//! Binary, native person and independent provider on Skein's real ring.
use jig_client_protocol_world::native::{NativeClient, Script};
use skein_io::kernel::{Complete, Done, Fd, Op, Signal, Submit};
use skein_lib::{Duration, Queue, Time, Token, Wall};
use skein_shell::{Clock, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Command, Mode, Streams},
    real,
};
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

pub const ANSWER: &[u8] = b"Process answer";
const PIPE: Token = Token::new(u64::MAX);
const CLOSE: Token = Token::new(u64::MAX - 1);

pub fn scratch() -> tempfile::TempDir {
    tempfile::tempdir().expect("scratch directory")
}
fn private(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).expect("fixture file");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private fixture");
}
pub fn agent_configuration(directory: &Path) {
    let value = serde_json::json!({
        "profile":{"name":"standard","declared":{"memory":2_147_483_648_u64,"tool_payload":512,"read_window":256},
            "policy":{"max_turns":5,"group_stop":10}},
        "endpoints":[{"name":"fake","number":0,"dialect":0,"account":1,"provider":"codex",
            "address":"127.0.0.1:34443","transport":"tls","server_name":"skein.test","trust_der":directory.join("root.der"),"identity":"plain","connect":1000,"handshake":1000,
            "models":[{"name":"fake","window":1_048_576,"output":1024,"reasoning_item":64,"head":60000,"idle":30000}]}],
        "environment":[]});
    private(&directory.join("agent.json"), &serde_json::to_vec(&value).expect("agent JSON"));
}
pub fn configuration(directory: &Path) {
    for name in ["accounts", "accounts/records", "secrets", "charters", "state", "agent-root"] {
        fs::create_dir(directory.join(name)).expect("deployment directory");
    }
    for (name, bytes) in jig_channel_world::fixtures::files(false).entries.into_boxed() {
        private(&directory.join(std::str::from_utf8(&name).expect("fixture path")), bytes.expose());
    }
    private(&directory.join("worker.secret"), b"worker-secret");
    private(
        &directory.join("accounts/chat.json"),
        br#"{"access_token":"token","expires_at":4102444800,"account_id":"acc"}"#,
    );
    private(&directory.join("root.der"), skein_tls_world::pki::ROOT);
    agent_configuration(directory);
    let mut value: serde_json::Value =
        serde_json::from_slice(&jig_channel_world::fixtures::engine_document()).expect("engine JSON");
    let worker: serde_json::Value =
        serde_json::from_slice(jig_channel_world::fixtures::WORKER_DOCUMENT).expect("worker JSON");
    value["workers"] = worker["workers"].clone();
    value["workers"][0]["engine"] = serde_json::json!({"in_process":"workers"});
    value["workers"][0]["agent"]["program"] = env!("CARGO_BIN_EXE_jig-test").into();
    value["workers"][0]["agent"]["configuration"] = directory.join("agent.json").to_str().expect("agent path").into();
    value["workers"][0]["agent"]["arguments"] = serde_json::json!(["agent", directory.join("agent.json")]);
    value["application"]["workers"] = worker["application"]["workers"].clone();
    value["service"]["components"] = serde_json::json!(["accounts", "workers", "clients"]);
    value["engine"]["hosts"]["listener"] = serde_json::json!([{"in_process":"workers"}]);
    value["engine"]["clients"]["listener"] = serde_json::json!({"socket":"127.0.0.1:0"});
    value["engine"]["llm"]["endpoints"][0]["address"] = serde_json::json!({"socket":"127.0.0.1:34443"});
    value["engine"]["limits"]["overrides"]["core.load_slots"] = 8.into();
    value["engine"]["limits"]["overrides"]["people.sign_ins"] = 2.into();
    private(&directory.join("config.json"), &serde_json::to_vec(&value).expect("deployment JSON"));
}
pub fn start(directory: &Path) -> Binary {
    Binary::start(
        Command {
            program: env!("CARGO_BIN_EXE_jig-test").into(),
            arguments: vec!["serve".into(), "config.json".into()],
            directory: directory.into(),
            environment: std::env::vars_os().filter(|(name, _)| name == "PATH").collect(),
        },
        Mode::Pipes,
        65536,
    )
    .expect("jig-test starts")
}
pub fn printed_link(bytes: &[u8]) -> Option<&[u8]> {
    let end = bytes.iter().position(|byte| *byte == b'\n')?;
    Some(&bytes[..end])
}
fn client_settings(address: std::net::SocketAddr) -> jig_shell_native::Settings {
    jig_shell_native::Settings {
        engine: skein_io::Addr::Ip(address),
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
            close_timeout: Duration::from_secs(5),
            retry: Duration::from_millis(1),
        },
    }
}
struct Person {
    client: Option<NativeClient>,
    root: Option<Fd>,
    script: Option<Script>,
    input: Option<Fd>,
    output: Option<Fd>,
    reading: bool,
    closing: u32,
    bytes: Vec<u8>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl Person {
    fn submit(&mut self, token: Token, kind: Op) {
        self.submissions.push(Submit { op: token, kind });
    }
    fn close(&mut self, fd: Fd) {
        self.closing += 1;
        self.submit(CLOSE, Op::Close { fd });
    }
    fn completed(&mut self, complete: Complete) {
        if complete.op == CLOSE {
            self.closing -= 1;
            complete.result.expect("pipe closed");
        } else if complete.op == PIPE {
            self.reading = false;
            let Op::PipeRead { buf, .. } = complete.kind else { panic!("stdout read") };
            let Done::Count(count) = complete.result.expect("stdout read completes") else { panic!("read count") };
            self.bytes.extend_from_slice(&buf[..count as usize]);
            assert!(self.bytes.len() <= 4096, "bounded printed links");
            if count == 0 {
                let fd = self.output.take().expect("stdout");
                self.close(fd);
            }
        } else {
            self.client.as_mut().expect("native client").completions().push(complete);
        }
    }
}
impl Host for Person {
    fn iterate(&mut self, now: Time, wall: Wall) {
        while let Some(complete) = self.completions.pop() {
            self.completed(complete);
        }
        if let Some(fd) = self.input.take() {
            self.close(fd);
        }
        if self.client.is_none()
            && let Some(link) = printed_link(&self.bytes)
        {
            let text = std::str::from_utf8(link).expect("printed URL");
            let address = text
                .strip_prefix("http://")
                .expect("loopback URL")
                .split('/')
                .next()
                .expect("URL address")
                .parse()
                .expect("socket address");
            let mut client = NativeClient::new(
                client_settings(address),
                self.root.take().expect("client root"),
                skein_shell::effective_user(),
                7,
                self.script.take().expect("person script"),
            );
            client.sign_in(link).expect("printed sign-in link");
            self.client = Some(client);
        }
        if !self.reading
            && let Some(fd) = self.output
        {
            self.reading = true;
            self.submit(PIPE, Op::PipeRead { fd, buf: vec![0; 1024].into_boxed_slice() });
        }
        if let Some(client) = &mut self.client {
            client.iterate(now, wall);
            while let Some(submit) = client.submissions().pop() {
                self.submissions.push(submit);
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        !self.completions.is_empty()
            || self.input.is_some()
            || (!self.reading && self.output.is_some())
            || self.client.as_ref().is_some_and(|client| client.work_pending(now))
    }
    fn next_deadline(&self) -> Option<Time> {
        self.client.as_ref().and_then(Host::next_deadline)
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.client.as_ref().and_then(Host::next_policy_deadline)
    }
    fn is_empty(&self) -> bool {
        self.output.is_none() && self.closing == 0 && self.client.as_ref().is_some_and(Host::is_empty)
    }
    fn worst_case(&self) -> u64 {
        64 * 1024 * 1024
    }
    fn operations(&self) -> u32 {
        512
    }
}
enum Process {
    Binary(Box<Binary>),
    Person(Box<Person>),
    Peer(Box<skein_fake_peers::llm::Peer>),
}
impl Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Binary(x) => x.as_ref(),
            Self::Person(x) => x.as_ref(),
            Self::Peer(x) => x.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Binary(x) => x.as_mut(),
            Self::Person(x) => x.as_mut(),
            Self::Peer(x) => x.as_mut(),
        }
    }
}
impl Host for Process {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host_mut().iterate(now, wall);
    }
    fn drain(&mut self) {
        self.host_mut().drain();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.host().next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }
    fn worst_case(&self) -> u64 {
        self.host().worst_case()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
}
struct Judge {
    ended: bool,
    deadline: Time,
}
impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        let Process::Person(person) = &procs[1] else { panic!("native person") };
        if !self.ended && person.client.as_ref().is_some_and(Host::is_empty) {
            let Process::Binary(binary) = &mut procs[0] else { panic!("binary") };
            binary.signal(Signal::Kill);
            for proc in procs {
                if let Process::Peer(peer) = proc {
                    peer.shutdown();
                }
            }
            self.ended = true;
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Process]) {
        let Process::Binary(binary) = &procs[0] else { panic!("binary") };
        assert!(
            binary.exit_status().is_none() || self.ended,
            "jig-test failed: {}",
            String::from_utf8_lossy(binary.stderr())
        );
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.ended).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.ended && now >= self.deadline).then(|| "native person's script did not finish".into())
    }
    fn passed(&self) -> bool {
        self.ended
    }
}
pub fn run(directory: &Path, script: Script, peer: Option<skein_fake_peers::llm::Peer>) -> (u32, u32) {
    let seconds = if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("live") { 280 } else { 60 };
    let clock = Clock::new();
    let mut binary = start(directory);
    let streams = binary.take_streams().expect("binary pipes");
    let fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("pipes mode") };
    let client_directory = tempfile::tempdir_in(directory).expect("fresh native client directory");
    let root = skein_shell::open_root(client_directory.path()).expect("client root");
    let mut world = real::World::new(Judge {
        ended: false,
        deadline: clock.now().now.saturating_add(Duration::from_secs(seconds - 5)),
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds.into_iter().chain([root]).collect(), || {
        Process::Person(Box::new(Person {
            client: None,
            root: Some(root),
            script: Some(script),
            input: Some(input),
            output: Some(output),
            reading: false,
            closing: 0,
            bytes: Vec::new(),
            completions: Queue::with_capacity(512),
            submissions: Queue::with_capacity(512),
        }))
    });
    if let Some(peer) = peer {
        world.spawn(|| Process::Peer(Box::new(peer)));
    }
    let outcome = world.run(&clock, Duration::from_secs(seconds));
    assert!(outcome.procs.iter().all(Host::is_empty), "all real io settles");
    let Process::Person(person) = &outcome.procs[1] else { panic!("native person") };
    let client = person.client.as_ref().expect("signed-in client");
    (client.signed_in, client.chats_made)
}
