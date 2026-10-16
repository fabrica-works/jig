use super::*;
use jig_client_protocol_http::Unlinked;
const EXCHANGE: u64 = 42;
use jig_client_protocol_link as link;
use skein_lib::{
    Queue, Secret, Time, Token, Writer,
    base64::{self, Alphabet},
    stream::Down,
};
use skein_world::stream::Wire;
const TOKEN: [u8; 40] = [7; 40];
fn limits() -> Limits {
    Limits {
        exchanges: 2,
        body_bytes: 128,
        deadline: skein_lib::Duration::from_secs(1),
        http: skein_http::client::Limits { head: 1024, headers: 8, request: 1024, read: 64, send: 64 },
    }
}
struct Peer {
    shell: Connections,
    wire: Wire,
    up: Queue<link::Event>,
    below: Queue<(Conn, Down)>,
    conn: Option<Conn>,
}
impl Peer {
    fn new() -> Self {
        Self {
            shell: Connections::new(&limits()),
            wire: Wire::new(1),
            up: Queue::with_capacity(256),
            below: Queue::with_capacity(256),
            conn: None,
        }
    }
    fn exchange(&mut self) {
        self.shell.down(
            Time::ZERO,
            link::Request::Exchange {
                exchange: Token::new(EXCHANGE),
                credential: Some(Secret::new(Box::new(TOKEN))),
                message: link::Message::Request(Box::from(b"ask".as_slice())),
            },
            &mut self.below,
        );
    }
    fn drive(&mut self) {
        self.shell.take(Time::ZERO, &mut self.up, &mut self.below);
        for _ in 0..4000 {
            while let Some((conn, down)) = self.below.pop() {
                self.conn = Some(conn);
                self.wire.take(down);
            }
            if let Some(up) = self.wire.answer() {
                self.shell.up(Time::ZERO, self.conn.expect("one test connection"), up, &mut self.up, &mut self.below);
            }
        }
    }
    fn respond(&mut self, response: &[u8]) {
        self.wire.write(response);
        self.drive();
    }
}
#[test]
fn an_exchange_carries_the_bearer_token() {
    let mut peer = Peer::new();
    peer.exchange();
    peer.respond(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok");
    let text = std::str::from_utf8(&peer.wire.received).expect("HTTP request");
    let mut writer = Writer::new(54);
    base64::encode(Alphabet::Url, &TOKEN, &mut writer).expect("bearer");
    let expected = format!("authorization: Bearer {}", std::str::from_utf8(&writer.finish()).expect("base64"));
    assert!(text.to_ascii_lowercase().contains(&expected.to_ascii_lowercase()), "{text}");
    assert!(text.to_ascii_lowercase().contains("content-type: application/octet-stream\r\n"));
    assert!(text.to_ascii_lowercase().contains("content-length: 3\r\n"));
    assert!(text.to_ascii_lowercase().contains("host: localhost\r\n"));
    assert!(
        matches!(peer.up.pop(), Some(link::Event::Reply { exchange, message: body }) if exchange == Token::new(EXCHANGE) && body.as_ref() == b"ok")
    );
}
#[test]
fn a_body_of_unknown_length_read_as_it_arrives_is_the_same_whatever_its_cuts() {
    let response = b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nwhole body";
    for size in [response.len(), 1] {
        let mut peer = Peer::new();
        peer.exchange();
        for piece in response.chunks(size) {
            peer.respond(piece);
        }
        peer.wire.eof = true;
        peer.drive();
        assert!(matches!(peer.up.pop(), Some(link::Event::Reply { exchange, message: body })
            if exchange == Token::new(EXCHANGE) && body.as_ref() == b"whole body"));
        assert!(peer.up.is_empty());
    }
}
#[test]
fn a_connection_cut_mid_body_fails_the_exchange() {
    let mut peer = Peer::new();
    peer.exchange();
    peer.respond(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nab");
    peer.wire.eof = true;
    peer.drive();
    assert!(matches!(peer.up.pop(), Some(link::Event::Lost { exchange }) if exchange == Token::new(EXCHANGE)));
    assert!(peer.up.is_empty());
}
#[test]
fn the_stream_s_bytes_reach_the_link_in_order() {
    let mut peer = Peer::new();
    peer.shell.down(Time::ZERO, link::Request::OpenStream { credential: None }, &mut peer.below);
    peer.respond(b"HTTP/1.1 200 OK\r\n\r\ndata: Zmlyc3Q=\n\ndata: c2Vjb25k\n\n");
    let mut bytes = Vec::new();
    while let Some(event) = peer.up.pop() {
        if let link::Event::Streamed { message: link::Streamed::Record(part) } = event {
            bytes.extend_from_slice(&part);
        }
    }
    assert_eq!(bytes, b"firstsecond");
    peer.shell.down(Time::ZERO, link::Request::CloseStream, &mut peer.below);
    peer.drive();
    assert!(peer.wire.finished);
}
#[test]
fn a_link_of_another_form_is_refused() {
    let mut shell = Connections::new(&limits());
    for (label, text, expected) in [
        ("scheme", b"https://127.0.0.1:8080/".as_slice(), Unlinked::NotHttp),
        ("secret", b"http://127.0.0.1:8080/jig/v1/sign-in/link".as_slice(), Unlinked::NoSecret),
        ("address", b"http://example/".as_slice(), Unlinked::Address),
        ("short secret", b"http://127.0.0.1:8080/jig/v1/sign-in/link?secret=AA".as_slice(), Unlinked::NoSecret),
    ] {
        assert_eq!(shell.printed(text), Err(expected), "{label}");
    }
}
#[derive(Clone, Copy)]
enum FileCase {
    KeepToken,
    ReloadToken,
    Store,
}
struct Disk {
    shell: Shell,
    case: FileCase,
    started: bool,
    closing: bool,
    writes: u32,
    failures: u32,
    sockets: u32,
    token: Option<Vec<u8>>,
}
impl Disk {
    fn new(root: skein_io::kernel::Fd, case: FileCase) -> Self {
        let mut shell = Shell::new(Settings {
            engine: skein_io::Addr::Ip((std::net::Ipv4Addr::LOCALHOST, 8080).into()),
            directory: b"private".as_slice().into(),
            limits: limits(),
            io: file_io_limits(),
        });
        shell.adopt(root, 1000, 7);
        Self { shell, case, started: false, closing: false, writes: 0, failures: 0, sockets: 0, token: None }
    }
    fn start(&mut self) {
        self.started = true;
        match self.case {
            FileCase::KeepToken => {
                self.shell.keep(Some(Secret::new(Box::new(TOKEN))));
            }
            FileCase::ReloadToken => {
                self.closing = true;
            }
            FileCase::Store => {
                self.shell.down(link::Request::Save {
                    record: link::Record { key: b"key".as_slice().into(), bytes: b"pending".as_slice().into() },
                });
                self.shell.down(link::Request::Exchange {
                    exchange: Token::new(EXCHANGE),
                    credential: Some(Secret::new(Box::new(TOKEN))),
                    message: link::Message::Request(b"ask".as_slice().into()),
                });
            }
        }
    }
}
impl skein_shell::Host for Disk {
    fn iterate(&mut self, now: Time, wall: skein_lib::Wall) {
        self.shell.iterate(now, wall);
        if !self.started
            && let Some((credential, _)) = self.shell.loaded()
        {
            self.token = credential.map(|token| token.expose().to_vec());
            self.start();
        }
        if self.shell.kept().is_some() {
            self.closing = true;
        }
        while let Some(event) = self.shell.events().pop() {
            match event {
                link::Event::Stored => self.writes += 1,
                link::Event::NotStored => {
                    self.failures += 1;
                    self.closing = true;
                }
                link::Event::Lost { .. } => self.closing = true,
                other @ (link::Event::SignIn { .. }
                | link::Event::SignOut
                | link::Event::Start { .. }
                | link::Event::Save { .. }
                | link::Event::Erase { .. }
                | link::Event::Send { .. }
                | link::Event::Read { .. }
                | link::Event::Reply { .. }
                | link::Event::Open { .. }
                | link::Event::Continue { .. }
                | link::Event::Close { .. }
                | link::Event::Streamed { .. }
                | link::Event::StreamGone) => panic!("file fixture event {other:?}"),
            }
        }
        for submit in self.shell.submissions().iter() {
            if matches!(submit.kind, skein_io::kernel::Op::Socket { .. }) {
                assert!(self.writes > 0, "store durable before connection");
                self.sockets += 1;
            }
        }
        if self.closing {
            self.shell.close();
        }
    }
    fn completions(&mut self) -> &mut Queue<skein_io::kernel::Complete> {
        self.shell.completions()
    }
    fn submissions(&mut self) -> &mut Queue<skein_io::kernel::Submit> {
        self.shell.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.shell.work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.shell.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.shell.is_empty()
    }
    fn worst_case(&self) -> u64 {
        self.shell.worst_case()
    }
    fn operations(&self) -> u32 {
        self.shell.operations()
    }
}
struct DiskMachine(skein_fake_machine::Machine);
impl skein_world::Machine for DiskMachine {
    fn step(&mut self, call: skein_sim::Call, answers: &mut Queue<skein_sim::Answer>) {
        skein_fake_machine::step(&mut self.0, call, answers);
    }
}
struct DiskJudge;
impl skein_world::Referee<Disk> for DiskJudge {
    fn act(&mut self, _now: Time, _procs: &mut [Disk]) {}
    fn observe(&mut self, _now: Time, _procs: &[Disk]) {}
    fn next_deadline(&self) -> Option<Time> {
        Some(Time::ZERO.saturating_add(skein_lib::Duration::from_secs(1)))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= Time::ZERO.saturating_add(skein_lib::Duration::from_secs(1)))
            .then(|| "file fixture did not drain".to_owned())
    }
    fn passed(&self) -> bool {
        true
    }
}
fn file_io_limits() -> skein_io::Limits {
    skein_io::Limits {
        sockets: 4,
        names: 1,
        refusals: 1,
        intake: 1024,
        receive: 64,
        output: 1024,
        sends: 2,
        accepts: 1,
        backlog: 2,
        close_timeout: skein_lib::Duration::from_secs(5),
        retry: skein_lib::Duration::from_millis(1),
    }
}
fn disk_world(
    machine: skein_fake_machine::Machine,
    root: skein_fake_machine::Opened,
    case: FileCase,
    fail: bool,
) -> skein_world::Outcome<Disk, DiskMachine> {
    let mut world = skein_world::World::new(7, skein_sim::Config::calm(), DiskJudge, skein_world::Memory::Unchecked)
        .with_machine(DiskMachine(machine));
    world.spawn_root(skein_sim::Handle::new(root.raw()), |root| Disk::new(root, case));
    world.run_with_faults(|_, _, submissions| {
        if fail && submissions.iter().any(|submit| matches!(submit.kind, skein_io::kernel::Op::Write { .. })) {
            Some(skein_sim::Faults { no_space: 1000, ..skein_sim::Faults::NONE })
        } else {
            None
        }
    })
}
fn fresh_disk(case: FileCase, fail: bool) -> skein_world::Outcome<Disk, DiskMachine> {
    let mut machine = skein_fake_machine::Machine::for_user(1000);
    let root = machine.lay(&[]);
    disk_world(machine, root, case, fail)
}
#[test]
fn the_token_is_kept_and_read_back_at_start() {
    let mut saved = fresh_disk(FileCase::KeepToken, false);
    let mut machine = std::mem::replace(&mut saved.machine.0, skein_fake_machine::Machine::new());
    let root = machine.reopen_root(0);
    let loaded = disk_world(machine, root, FileCase::ReloadToken, false);
    assert_eq!(loaded.procs[0].token.as_deref(), Some(TOKEN.as_slice()));
}
#[test]
fn the_small_store_is_written_before_the_step_s_exchanges() {
    let outcome = fresh_disk(FileCase::Store, false);
    assert_eq!(outcome.procs[0].writes, 1);
    assert_eq!(outcome.procs[0].sockets, 1);
    assert_eq!(outcome.procs[0].failures, 0);
}
#[test]
fn a_failed_store_write_withholds_the_exchanges() {
    let outcome = fresh_disk(FileCase::Store, true);
    assert_eq!(outcome.procs[0].failures, 1);
    assert_eq!(outcome.procs[0].writes, 0);
    assert_eq!(outcome.procs[0].sockets, 0);
}

#[test]
fn an_exchange_past_its_deadline_is_answered_lost() {
    let mut peer = Peer::new();
    peer.exchange();
    peer.drive();
    let until = Time::ZERO.saturating_add(limits().deadline);
    assert_eq!(peer.shell.next_deadline(), Some(until));
    peer.shell.take(until, &mut peer.up, &mut peer.below);
    assert!(matches!(peer.up.pop(), Some(link::Event::Lost { exchange }) if exchange == Token::new(EXCHANGE)));
    assert_eq!(peer.shell.next_deadline(), None);
    peer.respond(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok");
    assert!(peer.up.is_empty());
}
