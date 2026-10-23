//! One person's durable sign-in, with independent peers kept alive across both binaries.
use crate::{live_sign_in, support};
use jig_client::{Asker, Handle, Key, contract as c};
use jig_client_protocol_world::native::{Received, Script};
use jig_protocol_accounts_world::{fixtures, world::Story};
use skein_fake_peers::{llm, oauth};
use skein_io::kernel::Signal;
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{Referee, real};
use std::{path::Path, sync::mpsc};
use support::Process;

const ASK: Asker = Asker(88);
const SIGN_IN: Key = Key { asker: ASK, number: 1 };
const SIGN_OUT: Key = Key { asker: ASK, number: 2 };
const ACCOUNTS: Handle = Handle { asker: ASK, number: 1 };
const CHAT: Handle = Handle { asker: ASK, number: 2 };
const PROVIDER: u32 = 1;
// A fresh client must not repeat the first client's durable request keys.
const RESTARTED_CLIENT_SEED: u64 = 8;
const STAGE_WAIT: Duration = Duration::from_secs(20);
const SETTLE_WAIT: Duration = Duration::from_secs(30);
const PEER_WAIT: Duration = Duration::from_secs(60);

#[derive(Debug)]
enum Phase {
    Admission,
    Address,
    Usable,
    Answer,
    SignOut,
    Closing,
    Ended,
}
struct Judge {
    phase: Phase,
    restored: Option<c::OwnerAccountLine>,
    source: Option<u32>,
    accounts: Option<c::OwnerAccounts>,
    received: usize,
    chat: Option<u64>,
    confirmed: bool,
    deadline: Time,
    run: String,
}
impl Judge {
    fn receive(&mut self, native: &jig_client_protocol_world::native::NativeClient) -> Option<c::Answer> {
        let mut answer = None;
        for received in &native.received[self.received..] {
            match received {
                Received::Delivery(c::Delivery::OwnerAccounts(document)) => {
                    self.accounts = Some(document.as_ref().clone());
                }
                Received::Delivery(c::Delivery::Task(document)) if Some(document.chip.task) == self.chat => {
                    let line = live_sign_in::run_line(document);
                    if line != self.run {
                        eprintln!("Restart chat: {line}");
                        self.run = line;
                    }
                    assert!(!live_sign_in::failed(document), "restart chat can't answer: {}", self.run);
                }
                Received::Answer { answer: value, .. } => answer = Some(value.clone()),
                Received::SignedIn { .. } | Received::Sent { .. } | Received::Delivery(_) | Received::Read(_) => {}
            }
        }
        self.received = native.received.len();
        answer
    }
}
impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        let Process::Person(person) = &mut procs[1] else { panic!("native person") };
        let Some(native) = &mut person.client else { return };
        let answer = self.receive(native);
        match self.phase {
            Phase::Admission if native.signed_in == 1 && native.account_sign_in_ready() => {
                watch_accounts(native);
                if self.restored.is_some() {
                    self.phase = Phase::Usable;
                } else {
                    native.ask_account_sign_in(
                        SIGN_IN,
                        c::Ask::SignIn { holder: c::AccountHolder::Own, provider: PROVIDER },
                    );
                    self.phase = Phase::Address;
                }
            }
            Phase::Address => {
                if let Some(answer) = answer {
                    let c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountSignInStarted { source, address })) =
                        answer
                    else {
                        panic!("sign-in: {answer:?}")
                    };
                    self.source = Some(source);
                    let Process::Visitor(visitor) = &mut procs[2] else { panic!("visitor") };
                    visitor.hand(&address);
                    self.phase = Phase::Usable;
                }
            }
            Phase::Usable => {
                let Some(document) = &self.accounts else { return };
                let source = self.source.expect("source from sign-in or previous binary");
                let Some(account) = document.accounts.iter().find(|a| {
                    a.sources.iter().any(|s| s.number == source && s.usable && s.state == c::OwnerAccountState::Granted)
                }) else {
                    return;
                };
                if let Some(previous) = &self.restored {
                    assert_eq!(account.number, previous.number, "same account after restart");
                    assert_eq!(account.sources, previous.sources, "same usable source after restart");
                    eprintln!("Restored sign-in source={source}, account={}: usable", account.number);
                    native.account_request(jig_client::Event::Unwatch { handle: ACCOUNTS });
                    native.hold_after_script();
                    native.resume_after_account_setup();
                    self.phase = Phase::Answer;
                } else {
                    eprintln!("Signed in source={source}, account={}: usable", account.number);
                    native.finish_account_sign_in();
                    self.phase = Phase::Closing;
                }
            }
            Phase::Answer => {
                if let Some(c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::Created { first, .. }))) = answer {
                    self.chat = Some(first);
                    native.account_request(jig_client::Event::Watch {
                        handle: CHAT,
                        subject: jig_client::Watchable::Jig(c::Subject::Task { task: first }),
                    });
                }
                if native.script_finished() {
                    native.account_request(jig_client::Event::Unwatch { handle: CHAT });
                    watch_accounts(native);
                    native.hold_for_account_sign_in();
                    native.ask_account_sign_in(
                        SIGN_OUT,
                        c::Ask::SignOut { holder: c::AccountHolder::Own, source: self.source.expect("source") },
                    );
                    self.phase = Phase::SignOut;
                }
            }
            Phase::SignOut => {
                if let Some(answer) = answer {
                    assert_eq!(
                        answer,
                        c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountSignedOut {
                            confirmed: true,
                            failed_hosts: Box::new([])
                        }))
                    );
                    self.confirmed = true;
                }
                if self.confirmed && self.accounts.as_ref().is_some_and(|a| a.accounts.is_empty()) {
                    native.account_request(jig_client::Event::Unwatch { handle: ACCOUNTS });
                    native.finish_account_sign_in();
                    self.phase = Phase::Closing;
                }
            }
            Phase::Closing => {
                if native.is_empty() {
                    let Process::Binary(binary) = &mut procs[0] else { panic!("binary") };
                    binary.signal(Signal::Kill);
                    self.phase = Phase::Ended;
                }
            }
            Phase::Admission | Phase::Ended => {}
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Process]) {
        let Process::Binary(binary) = &procs[0] else { panic!("binary") };
        assert!(
            binary.exit_status().is_none() || matches!(self.phase, Phase::Ended),
            "binary failed: {}",
            String::from_utf8_lossy(binary.stderr())
        );
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.passed()).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= self.deadline && !self.passed())
            .then(|| format!("restart sign-in phase {:?}: {}", self.phase, self.run))
    }
    fn passed(&self) -> bool {
        matches!(self.phase, Phase::Ended)
    }
}

fn stage(directory: &Path, restored: Option<c::OwnerAccountLine>) -> c::OwnerAccountLine {
    let clock = Clock::new();
    let seed = if restored.is_some() { RESTARTED_CLIENT_SEED } else { support::CLIENT_SEED };
    let source = restored.as_ref().map(|a| a.sources.get(0).expect("signed-in source").number);
    let mut world = real::World::new(Judge {
        phase: Phase::Admission,
        restored,
        source,
        accounts: None,
        received: 0,
        chat: None,
        confirmed: false,
        deadline: clock.now().now.saturating_add(STAGE_WAIT),
        run: String::new(),
    });
    let _client_directory = live_sign_in::spawn_client(
        &mut world,
        directory,
        Script::answered_report(live_sign_in::WORDS.as_bytes(), support::ANSWER, STAGE_WAIT),
        seed,
    );
    world.spawn(|| Process::Visitor(Box::default()));
    let outcome = world.run(&clock, SETTLE_WAIT);
    assert!(outcome.procs.iter().all(Host::is_empty), "both binaries' io settles");
    let Process::Person(person) = &outcome.procs[1] else { panic!("native person") };
    person
        .client
        .as_ref()
        .expect("native client")
        .received
        .iter()
        .rev()
        .find_map(|r| match r {
            Received::Delivery(c::Delivery::OwnerAccounts(document)) if !document.accounts.is_empty() => {
                Some(document.accounts.get(0).expect("signed-in account").clone())
            }
            Received::Delivery(_)
            | Received::Answer { .. }
            | Received::SignedIn { .. }
            | Received::Sent { .. }
            | Received::Read(_) => None,
        })
        .expect("usable signed-in account")
}

const PEER_POLL: Duration = Duration::from_millis(10);
struct Peers {
    stop: mpsc::Receiver<()>,
    ended: bool,
    deadline: Time,
    wake: Time,
}
impl Referee<Process> for Peers {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        self.wake = now.saturating_add(PEER_POLL);
        if !self.ended && !matches!(self.stop.try_recv(), Err(mpsc::TryRecvError::Empty)) {
            for proc in procs {
                match proc {
                    Process::Issuer(p) => p.shutdown(),
                    Process::Peer(p) => p.shutdown(),
                    Process::Binary(_) | Process::Person(_) | Process::Visitor(_) => {
                        unreachable!("only independent peers")
                    }
                }
            }
            self.ended = true;
        }
    }
    fn observe(&mut self, _now: Time, _procs: &[Process]) {}
    fn next_deadline(&self) -> Option<Time> {
        Some(self.deadline.min(self.wake))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= self.deadline).then(|| "independent peers did not settle".into())
    }
    fn passed(&self) -> bool {
        self.ended
    }
}

/// Keep one issuer and LLM alive while a story kills and starts its application binaries.
pub(super) fn with_peers(
    plans: Vec<skein_fake_oauth::Plan>,
    story: impl FnOnce(),
    observe: impl FnOnce(&oauth::Peer, &llm::Peer) + Send,
) {
    std::thread::scope(|scope| {
        let (stop, receiver) = mpsc::channel();
        let (ready, listening) = mpsc::channel();
        let peers = scope.spawn(move || {
            let clock = Clock::new();
            let mut world = real::World::new(Peers {
                stop: receiver,
                ended: false,
                deadline: clock.now().now.saturating_add(PEER_WAIT),
                wake: clock.now().now,
            });
            world.spawn(|| Process::Issuer(Box::new(support::issuer_with_plans(plans))));
            world.spawn(|| Process::Peer(Box::new(live_sign_in::fake_peer(live_sign_in::Peers::Fake))));
            ready.send(()).expect("peer world ready");
            let outcome = world.run(&clock, PEER_WAIT);
            assert!(outcome.procs.iter().all(Host::is_empty), "independent peers' io settles");
            let Process::Issuer(issuer) = &outcome.procs[0] else { panic!("issuer") };
            let Process::Peer(peer) = &outcome.procs[1] else { panic!("LLM") };
            observe(issuer, peer);
        });
        listening.recv().expect("peers constructed");
        story();
        drop(stop);
        peers.join().expect("peer world");
    });
}

pub fn run(directory: &Path) {
    live_sign_in::configuration_at(directory, "127.0.0.1:34443", live_sign_in::Peers::Fake);
    let mut plans = fixtures::issuer_plans(Story::AccountSignInVaryingPort);
    let mut first = plans.remove(0);
    let skein_fake_oauth::Body::Token(token) = &mut first.body else { panic!("token plan") };
    token.access_token = fixtures::TOKEN_FIRST.into();
    // Sign-in leaves refresh token `next`; the independent issuer checks it on refresh.
    plans.insert(0, fixtures::rotate());
    plans.insert(0, first);
    with_peers(
        plans,
        || {
            let account = stage(directory, None);
            stage(directory, Some(account));
        },
        |issuer, peer| {
            let posts = issuer.observations().iter().filter(|o| matches!(o, oauth::Observation::Post { .. })).count();
            let revokes =
                issuer.observations().iter().filter(|o| matches!(o, oauth::Observation::Revoke { .. })).count();
            let credentials: Vec<_> = peer
                .observations()
                .iter()
                .filter_map(|o| if let llm::Observation::Credential { index, .. } = o { Some(*index) } else { None })
                .collect();
            let answered =
                peer.observations().iter().any(|o| matches!(o, llm::Observation::Answered { result: Ok(()), .. }));
            assert_eq!(posts, 2, "code grant and one validated refresh grant");
            assert_eq!(revokes, 1, "issuer revoked the sign-in");
            assert_eq!(credentials, vec![None, Some(0)], "first token refused, second accepted");
            assert!(answered, "new grant answered");
        },
    );
}

fn watch_accounts(native: &mut jig_client_protocol_world::native::NativeClient) {
    native.account_request(jig_client::Event::Watch {
        handle: ACCOUNTS,
        subject: jig_client::Watchable::Jig(c::Subject::Accounts),
    });
}
