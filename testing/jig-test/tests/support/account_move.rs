//! The process-tier move in protocol/service.md, 8: two deployments, one issuer.
use crate::{live_sign_in, restarted_sign_in, support};
use jig_client::{Asker, Handle, Key, accounts, contract as c};
use jig_client_protocol_world::native::{NativeClient, Received, Script};
use jig_protocol_accounts_world::{fixtures, world::Story};
use skein_fake_peers::{llm, oauth};
use skein_io::kernel::Signal;
use skein_lib::{Duration, Secret, Time};
use skein_shell::{Clock, Host};
use skein_world::{Referee, real};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    net::TcpStream,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};
use support::Process;

const ASKER: Asker = Asker(99);
const ACCOUNTS: Handle = Handle { asker: ASKER, number: 1 };
const CHAT: Handle = Handle { asker: ASKER, number: 2 };
const WAIT: Duration = Duration::from_secs(20);
const SETTLE: Duration = Duration::from_secs(30);
// Distinct rotations make the file's token stale as soon as B refreshes it.
const FILE_REFRESH: &[u8] = b"move-file-refresh";
const B_REFRESH: &[u8] = b"move-b-refresh";
const A: usize = 0;
const B: usize = 1;
const BINARIES: [usize; 2] = [0, 2];
const PEOPLE: [usize; 2] = [1, 3];
const VISITOR: usize = 4;

fn key(number: u64) -> Key {
    Key { asker: ASKER, number }
}
fn client(procs: &mut [Process], deployment: usize) -> &mut NativeClient {
    let Process::Person(person) = &mut procs[PEOPLE[deployment]] else { panic!("native person") };
    person.client.as_mut().expect("native client")
}
fn watch(native: &mut NativeClient) {
    native.account_request(jig_client::Event::Watch {
        handle: ACCOUNTS,
        subject: jig_client::Watchable::Jig(c::Subject::Accounts),
    });
}

#[derive(Debug)]
enum Phase {
    Admission,
    Address,
    SignedIn,
    MovingOut,
    Written,
    MovingIn,
    Usable { account: u32 },
    Answer,
    Restored,
    Closing,
    Ended,
}
struct Judge {
    phase: Phase,
    file: PathBuf,
    restored: Option<c::OwnerAccountLine>,
    account: Option<c::OwnerAccountLine>,
    source: Option<u32>,
    documents: [Option<c::OwnerAccounts>; 2],
    received: [usize; 2],
    written: bool,
    chat: Option<u64>,
    run: String,
    deadline: Time,
}
impl Judge {
    fn receive(&mut self, native: &NativeClient, deployment: usize) -> Option<c::Answer> {
        let mut answer = None;
        for received in &native.received[self.received[deployment]..] {
            match received {
                Received::Delivery(c::Delivery::OwnerAccounts(document)) => {
                    self.documents[deployment] = Some(document.as_ref().clone());
                }
                Received::Delivery(c::Delivery::Task(document)) if Some(document.chip.task) == self.chat => {
                    self.run = live_sign_in::run_line(document);
                    assert!(!live_sign_in::failed(document), "moved account's run can't answer: {}", self.run);
                }
                Received::Answer { answer: value, .. } => {
                    assert!(answer.is_none(), "one outstanding account request per client");
                    answer = Some(value.clone());
                }
                Received::SignedIn { .. } | Received::Sent { .. } | Received::Delivery(_) | Received::Read(_) => {}
            }
        }
        self.received[deployment] = native.received.len();
        answer
    }
    fn finish(&mut self, procs: &mut [Process]) {
        for deployment in [A, B] {
            let native = client(procs, deployment);
            native.account_request(jig_client::Event::Unwatch { handle: ACCOUNTS });
            native.finish_account_sign_in();
        }
        self.phase = Phase::Closing;
    }
    fn usable(&self, deployment: usize) -> Option<&c::OwnerAccountLine> {
        self.documents[deployment].as_ref()?.accounts.iter().find(|account| {
            account.plan
                && account.sources.iter().any(|s| {
                    s.kind == c::OwnerAccountKind::SignIn && s.usable && s.state == c::OwnerAccountState::Granted
                })
        })
    }
    fn admit(&mut self, procs: &mut [Process]) {
        if ![A, B].into_iter().all(|d| {
            let native = client(procs, d);
            native.signed_in == 1 && native.account_sign_in_ready()
        }) {
            return;
        }
        let addresses: Vec<_> = PEOPLE
            .iter()
            .map(|index| {
                let Process::Person(person) = &procs[*index] else { panic!("native person") };
                std::str::from_utf8(support::printed_link(&person.bytes).expect("printed link"))
                    .expect("URL")
                    .split('/')
                    .nth(2)
                    .expect("host")
                    .to_owned()
            })
            .collect();
        assert_ne!(addresses[A], addresses[B], "deployments listen on their own ports");
        for deployment in [A, B] {
            watch(client(procs, deployment));
        }
        if self.restored.is_some() {
            self.phase = Phase::Restored;
        } else {
            client(procs, A).ask_account_sign_in(key(1), c::Ask::SignIn { holder: c::AccountHolder::Own, provider: 1 });
            self.phase = Phase::Address;
        }
    }
    fn depart(&mut self, procs: &mut [Process], answers: &[Option<c::Answer>; 2]) {
        match self.phase {
            Phase::Admission => self.admit(procs),
            Phase::Address => {
                if let Some(answer) = &answers[A] {
                    let c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountSignInStarted { source, address })) =
                        answer
                    else {
                        panic!("A's sign-in: {answer:?}")
                    };
                    self.source = Some(*source);
                    let Process::Visitor(visitor) = &mut procs[VISITOR] else { panic!("visitor") };
                    visitor.hand(address);
                    self.phase = Phase::SignedIn;
                }
            }
            Phase::SignedIn => {
                if let Some(account) = self.usable(A).cloned() {
                    assert!(account.sources.iter().any(|source| Some(source.number) == self.source));
                    client(procs, A).account_request(accounts::move_out(
                        key(2),
                        [2; 16],
                        c::AccountHolder::Own,
                        account.number,
                    ));
                    self.account = Some(account);
                    self.phase = Phase::MovingOut;
                }
            }
            Phase::MovingOut => {
                if let Some(answer) = &answers[A] {
                    let c::Answer::Done(c::Outcome::Made { what: c::Made::MovingOut { account, number, record } }) =
                        answer
                    else {
                        panic!("move out: {answer:?}")
                    };
                    assert_eq!(*account, self.account.as_ref().expect("A's account").number);
                    assert!(!record.is_empty(), "move out carries the record");
                    assert!(
                        skein_lib::bytes::find(record.expose(), FILE_REFRESH).is_some(),
                        "fixture refresh token in file"
                    );
                    let mut file = OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .mode(0o600)
                        .open(&self.file)
                        .expect("new private move file");
                    file.write_all(record.expose()).expect("whole move file written");
                    file.sync_all().expect("move file synced before Written");
                    assert_eq!(file.metadata().expect("move metadata").permissions().mode() & 0o777, 0o600);
                    client(procs, A).account_request(accounts::written(
                        key(3),
                        [3; 16],
                        c::AccountHolder::Own,
                        *number,
                    ));
                    self.phase = Phase::Written;
                }
            }
            Phase::Written => {
                if let Some(answer) = &answers[A] {
                    assert_eq!(*answer, c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountChanged)));
                    self.written = true;
                }
                if self.written && self.documents[A].as_ref().is_some_and(|d| d.accounts.is_empty()) {
                    let file = Secret::new(fs::read(&self.file).expect("shell reads move file").into_boxed_slice());
                    client(procs, B).account_request(accounts::move_in(key(4), [4; 16], c::AccountHolder::Own, file));
                    self.phase = Phase::MovingIn;
                }
            }
            Phase::MovingIn
            | Phase::Usable { .. }
            | Phase::Answer
            | Phase::Restored
            | Phase::Closing
            | Phase::Ended => {
                unreachable!("departure phases only")
            }
        }
    }
}
impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        let mut answers = [None, None];
        for deployment in [A, B] {
            let Process::Person(person) = &procs[PEOPLE[deployment]] else { panic!("native person") };
            let Some(native) = &person.client else { return };
            answers[deployment] = self.receive(native, deployment);
        }
        match self.phase {
            Phase::Admission | Phase::Address | Phase::SignedIn | Phase::MovingOut | Phase::Written => {
                self.depart(procs, &answers);
            }
            Phase::MovingIn => {
                if let Some(answer) = &answers[B] {
                    let c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountMovedIn { account, renewed: false })) =
                        answer
                    else {
                        panic!("move in: {answer:?}")
                    };
                    self.account = None;
                    fs::remove_file(&self.file).expect("shell removes answered move file");
                    self.phase = Phase::Usable { account: *account };
                }
            }
            Phase::Usable { account: moved } => {
                if let Some(account) = self.usable(B).cloned() {
                    assert_eq!(account.number, moved, "B's moved-in account");
                    self.account = Some(account);
                    let native = client(procs, B);
                    native.hold_after_script();
                    native.resume_after_account_setup();
                    self.phase = Phase::Answer;
                }
            }
            Phase::Answer => {
                if let Some(answer) = &answers[B] {
                    let c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::Created { first, .. })) = answer else {
                        panic!("moved account's chat: {answer:?}")
                    };
                    self.chat = Some(*first);
                    client(procs, B).account_request(jig_client::Event::Watch {
                        handle: CHAT,
                        subject: jig_client::Watchable::Jig(c::Subject::Task { task: *first }),
                    });
                }
                if client(procs, B).script_finished() {
                    client(procs, B).account_request(jig_client::Event::Unwatch { handle: CHAT });
                    self.account = Some(self.usable(B).expect("B still usable after answer").clone());
                    self.finish(procs);
                }
            }
            Phase::Restored => {
                if let (Some(a), Some(b)) = (&self.documents[A], &self.documents[B]) {
                    assert!(a.accounts.is_empty(), "restarted A holds no account");
                    assert_eq!(b.accounts.len(), 1, "restarted B holds the moved account");
                    let previous = self.restored.as_ref().expect("previous B account");
                    let account = self.usable(B).expect("restarted B's account is usable");
                    assert_eq!(account.number, previous.number, "B kept the account across restart");
                    assert_eq!(account.holder, previous.holder, "B kept its holder");
                    assert_eq!(account.sources, previous.sources, "B kept its usable sources");
                    self.account = Some(account.clone());
                    self.finish(procs);
                }
            }
            Phase::Closing => {
                if [A, B].into_iter().all(|d| client(procs, d).is_empty()) {
                    for index in BINARIES {
                        let Process::Binary(binary) = &mut procs[index] else { panic!("binary") };
                        binary.signal(Signal::Kill);
                    }
                    self.phase = Phase::Ended;
                }
            }
            Phase::Ended => {}
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Process]) {
        for index in BINARIES {
            let Process::Binary(binary) = &procs[index] else { panic!("binary") };
            assert!(
                binary.exit_status().is_none() || matches!(self.phase, Phase::Ended),
                "binary failed: {}",
                String::from_utf8_lossy(binary.stderr())
            );
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.passed()).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= self.deadline && !self.passed()).then(|| format!("account move phase {:?}: {}", self.phase, self.run))
    }
    fn passed(&self) -> bool {
        matches!(self.phase, Phase::Ended)
    }
}

fn stage(a: &Path, b: &Path, file: &Path, restored: Option<c::OwnerAccountLine>) -> c::OwnerAccountLine {
    let clock = Clock::new();
    // Fresh request keys for the client after restart, as in restarted_sign_in.
    let seed = if restored.is_some() { 8 } else { support::CLIENT_SEED };
    let mut world = real::World::new(Judge {
        phase: Phase::Admission,
        file: file.into(),
        restored,
        account: None,
        source: None,
        documents: [None, None],
        received: [0, 0],
        written: false,
        chat: None,
        run: String::new(),
        deadline: clock.now().now.saturating_add(WAIT),
    });
    let _a_client = live_sign_in::spawn_client(&mut world, a, Script::account_sign_in(), seed);
    let _b_client = live_sign_in::spawn_client(
        &mut world,
        b,
        Script::answered_report(live_sign_in::WORDS.as_bytes(), support::ANSWER, WAIT),
        seed,
    );
    world.spawn(|| Process::Visitor(Box::default()));
    let outcome = world.run(&clock, SETTLE);
    assert!(outcome.procs.iter().all(Host::is_empty), "both deployments' io settles");
    let Process::Person(person) = &outcome.procs[PEOPLE[B]] else { panic!("B's native person") };
    person
        .client
        .as_ref()
        .expect("B's client")
        .received
        .iter()
        .rev()
        .find_map(|received| {
            if let Received::Delivery(c::Delivery::OwnerAccounts(document)) = received {
                document.accounts.get(0).cloned()
            } else {
                None
            }
        })
        .expect("B's moved account")
}

/// A direct replay of the known fake token, against the same issuer that rotated it.
fn refuse_file_token() {
    let body = format!(
        "grant_type=refresh_token&client_id=client&refresh_token={}",
        std::str::from_utf8(FILE_REFRESH).expect("fixture token")
    );
    let mut socket = TcpStream::connect("127.0.0.1:31000").expect("same fake issuer");
    socket.set_read_timeout(Some(std::time::Duration::from_secs(5))).expect("probe timeout");
    socket.write_all(format!(
        "POST /token HTTP/1.1\r\nHost: 127.0.0.1:31000\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()
    ).as_bytes()).expect("stale refresh sent");
    let mut response = Vec::new();
    socket.take(4096).read_to_end(&mut response).expect("issuer refusal");
    assert!(response.starts_with(b"HTTP/1.1 400 "), "stale refresh refused");
    let body = response.split(|byte| *byte == b'\n').next_back().expect("refusal body");
    let error: serde_json::Value = serde_json::from_slice(body).expect("issuer error JSON");
    assert_eq!(error["error"], "invalid_grant", "the file's refresh was rotated");
}

pub fn run(directory: &Path) {
    let a = directory.join("a");
    let b = directory.join("b");
    for deployment in [&a, &b] {
        fs::create_dir(deployment).expect("separate deployment directory");
        live_sign_in::configuration_at(deployment, "127.0.0.1:34443", live_sign_in::Peers::Fake);
    }
    let mut first = fixtures::issuer_plans(Story::AccountSignInVaryingPort).remove(0);
    let skein_fake_oauth::Body::Token(token) = &mut first.body else { panic!("sign-in token plan") };
    token.access_token = fixtures::TOKEN_FIRST.into();
    token.refresh_token = Some(FILE_REFRESH.into());
    let mut rotated = fixtures::rotate();
    let skein_fake_oauth::Body::Token(token) = &mut rotated.body else { panic!("refresh token plan") };
    token.refresh_token = Some(B_REFRESH.into());
    // A spare successful plan proves stale-token refusal is issuer validation, not a scripted error.
    restarted_sign_in::with_peers(
        vec![first, rotated, fixtures::rotate()],
        || {
            let file = directory.join("account.move");
            let account = stage(&a, &b, &file, None);
            refuse_file_token();
            stage(&a, &b, &file, Some(account));
            assert!(!file.exists(), "the application removed its imported file");
        },
        |issuer, peer| {
            let posts = issuer.observations().iter().filter(|o| matches!(o, oauth::Observation::Post { .. })).count();
            let revokes =
                issuer.observations().iter().filter(|o| matches!(o, oauth::Observation::Revoke { .. })).count();
            let refused = issuer
                .observations()
                .iter()
                .filter(|o| matches!(o, oauth::Observation::Answered { status: 400, .. }))
                .count();
            assert_eq!(posts, 3, "code grant, B's refresh and the explicit stale-token replay");
            let granted = issuer
                .observations()
                .iter()
                .filter(|o| matches!(o, oauth::Observation::Answered { status: 200, .. }))
                .count();
            assert_eq!(granted, 2, "only A's code grant and B's refresh succeed");
            assert_eq!(revokes, 0, "moving out does not revoke the transferred account");
            assert_eq!(refused, 1, "issuer observed the stale file token's refusal");
            let credentials: Vec<_> = peer
                .observations()
                .iter()
                .filter_map(|o| if let llm::Observation::Credential { index, .. } = o { Some(*index) } else { None })
                .collect();
            assert_eq!(credentials, vec![Some(0)], "B's first run uses the refreshed grant");
            assert!(
                peer.observations().iter().any(|o| matches!(o, llm::Observation::Answered { result: Ok(()), .. })),
                "refreshed grant answered"
            );
        },
    );
}
