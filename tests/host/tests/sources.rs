//! Source scheduling uses typed fakes here; command I/O belongs to channel.
use jig_domain_host::{
    self as host,
    sources::{Answer, Operation},
};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

struct World {
    host: host::Domain,
    env: Env<host::Limits>,
    out: Queue<host::Request>,
}
impl World {
    fn new() -> Self {
        let mut limits = jig_host_world::Settings::calm(1).host;
        limits.program_sources = 2;
        limits.slots = 0;
        Self {
            host: host::Domain::new(&limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            out: Queue::with_capacity(host::max_out(&limits)),
        }
    }
    fn step(&mut self, event: host::Event) -> Vec<host::Request> {
        host::step(&mut self.host, &self.env, event, &mut self.out);
        let mut out = Vec::new();
        while let Some(request) = self.out.pop() {
            out.push(request);
        }
        out
    }
    fn sources(&mut self, names: &[&[u8]]) -> Vec<host::Request> {
        self.step(host::Event::Down(host::LinkDown::Sources {
            sources: names.iter().map(|name| name_of(name)).collect(),
        }))
    }
    fn checked(&mut self, owner: Token, state: host::LoginState) -> Vec<host::Request> {
        self.step(host::Event::SourceAnswered { owner, answer: Answer::Checked(state) })
    }
}
fn name_of(name: &[u8]) -> host::SourceName {
    host::SourceName::new(name).expect("fixture source name")
}
fn command(requests: &[host::Request], name: &[u8], operation: Operation) -> Token {
    let [host::Request::Source { owner, name: actual, operation: actual_op, .. }] = requests else {
        panic!("{requests:?}")
    };
    assert_eq!(*actual, name_of(name));
    assert_eq!(*actual_op, operation);
    *owner
}
fn snapshot(request: &host::Request, names: &[&[u8]]) {
    let host::Request::Up(host::LinkUp::Logins { logins }) = request else { panic!("{request:?}") };
    assert_eq!(
        logins.iter().map(|login| login.name).collect::<Vec<_>>(),
        names.iter().map(|name| name_of(name)).collect::<Vec<_>>()
    );
}
#[test]
fn replacement_installs_added_names_and_preserves_kept_sign_outs_while_busy() {
    let mut world = World::new();
    let old = command(&world.sources(&[b"a", b"b"]), b"a", Operation::Check);
    assert!(world.step(host::Event::Down(host::LinkDown::SignOut { name: name_of(b"b") })).is_empty());
    assert!(world.sources(&[b"b", b"c"]).is_empty());
    assert!(world.step(host::Event::Down(host::LinkDown::Check { name: name_of(b"c") })).is_empty());
    assert!(world.step(host::Event::Down(host::LinkDown::SignOut { name: name_of(b"c") })).is_empty());
    let b = command(&world.checked(old, host::LoginState::Usable), b"b", Operation::SignOut);
    let answer =
        world.step(host::Event::SourceAnswered { owner: b, answer: Answer::SignedOut(host::SignOutResult::SignedOut) });
    assert!(matches!(
        answer[0],
        host::Request::Up(host::LinkUp::SignedOut { result: host::SignOutResult::SignedOut, .. })
    ));
    let c = command(&answer[1..], b"c", Operation::SignOut);
    let answer =
        world.step(host::Event::SourceAnswered { owner: c, answer: Answer::SignedOut(host::SignOutResult::SignedOut) });
    let b = command(&answer[1..], b"b", Operation::Check);
    let c = command(&world.checked(b, host::LoginState::NotSignedIn), b"c", Operation::Check);
    snapshot(&world.checked(c, host::LoginState::NotSignedIn)[0], &[b"b", b"c"]);
}
#[test]
fn replacement_answers_unknown_only_for_a_removed_queued_sign_out() {
    let mut world = World::new();
    world.sources(&[b"a", b"b"]);
    world.step(host::Event::Down(host::LinkDown::SignOut { name: name_of(b"b") }));
    assert_eq!(
        world.sources(&[b"a", b"c"]),
        vec![host::Request::Up(host::LinkUp::SignedOut {
            name: name_of(b"b"),
            result: host::SignOutResult::Failed { reason: host::SourceFailure::Unknown, detail: Box::new([]) },
        })]
    );
}
#[test]
fn overlapping_checks_coalesce_and_an_unchanged_explicit_check_answers() {
    let mut world = World::new();
    let a = command(&world.sources(&[b"a"]), b"a", Operation::Check);
    for _ in 0..2 {
        assert!(world.step(host::Event::Down(host::LinkDown::Check { name: name_of(b"a") })).is_empty());
    }
    let fresh = command(&world.checked(a, host::LoginState::NotSignedIn), b"a", Operation::Check);
    snapshot(&world.checked(fresh, host::LoginState::NotSignedIn)[0], &[b"a"]);
    world.env.now = Time::ZERO.saturating_add(Duration::from_secs(60));
    host::resume(&mut world.host, &world.env, &mut world.out);
    let periodic = world.out.pop().expect("interval check");
    let periodic = command(&[periodic], b"a", Operation::Check);
    assert!(world.checked(periodic, host::LoginState::NotSignedIn).is_empty());
    world.env.now = Time::ZERO.saturating_add(Duration::from_secs(119));
    let explicit =
        command(&world.step(host::Event::Down(host::LinkDown::Check { name: name_of(b"a") })), b"a", Operation::Check);
    world.env.now = Time::ZERO.saturating_add(Duration::from_secs(120));
    host::resume(&mut world.host, &world.env, &mut world.out);
    assert!(world.out.is_empty(), "the active check also covers the expired interval");
    let answered = world.checked(explicit, host::LoginState::NotSignedIn);
    assert_eq!(answered.len(), 1);
    snapshot(&answered[0], &[b"a"]);
    assert!(world.host.sources_settled());
}
#[test]
fn empty_sources_still_publish_an_inventory_and_unknown_commands_end() {
    let mut world = World::new();
    snapshot(&world.sources(&[])[0], &[]);
    assert_eq!(
        world.step(host::Event::Down(host::LinkDown::Check { name: name_of(b"unknown") })),
        vec![host::Request::SourceFault]
    );
    let answer = world.step(host::Event::Down(host::LinkDown::SignOut { name: name_of(b"unknown") }));
    assert_eq!(
        answer,
        vec![host::Request::Up(host::LinkUp::SignedOut {
            name: name_of(b"unknown"),
            result: host::SignOutResult::Failed { reason: host::SourceFailure::Unknown, detail: Box::new([]) }
        })]
    );
}
#[test]
fn a_lost_sign_out_is_settled_without_replaying_its_reply_on_redial() {
    let mut world = World::new();
    let a = command(&world.sources(&[b"a"]), b"a", Operation::Check);
    world.checked(a, host::LoginState::Usable);
    let sign_out = command(
        &world.step(host::Event::Down(host::LinkDown::SignOut { name: name_of(b"a") })),
        b"a",
        Operation::SignOut,
    );
    assert!(world.step(host::Event::SourceLost).is_empty());
    assert!(world.sources(&[b"a"]).is_empty());
    assert!(world.step(host::Event::Down(host::LinkDown::Check { name: name_of(b"a") })).is_empty());
    let next = world.step(host::Event::SourceAnswered {
        owner: sign_out,
        answer: Answer::SignedOut(host::SignOutResult::SignedOut),
    });
    let check = command(&next, b"a", Operation::Check);
    assert!(check != sign_out);
    snapshot(&world.checked(check, host::LoginState::NotSignedIn)[0], &[b"a"]);
    assert!(
        world
            .step(host::Event::SourceAnswered {
                owner: sign_out,
                answer: Answer::SignedOut(host::SignOutResult::SignedOut)
            })
            .is_empty()
    );
}
#[test]
fn shutdown_waits_for_the_command_and_answers_it_once() {
    let mut world = World::new();
    let a = command(&world.sources(&[b"a"]), b"a", Operation::Check);
    world.checked(a, host::LoginState::Usable);
    let sign_out = command(
        &world.step(host::Event::Down(host::LinkDown::SignOut { name: name_of(b"a") })),
        b"a",
        Operation::SignOut,
    );
    world.step(host::Event::CancelAll { reason: host::Reason::Shutdown });
    assert!(!world.host.sources_settled());
    let answer = world.step(host::Event::SourceAnswered {
        owner: sign_out,
        answer: Answer::SignedOut(host::SignOutResult::SignedOut),
    });
    assert_eq!(
        answer,
        vec![host::Request::Up(host::LinkUp::SignedOut {
            name: name_of(b"a"),
            result: host::SignOutResult::SignedOut
        })]
    );
    for _ in 0..2 {
        assert_eq!(
            world.step(host::Event::Down(host::LinkDown::SignOut { name: name_of(b"a") })),
            vec![host::Request::Up(host::LinkUp::SignedOut {
                name: name_of(b"a"),
                result: host::SignOutResult::Failed { reason: host::SourceFailure::Closed, detail: Box::new([]) }
            })]
        );
    }
    assert!(world.host.sources_settled());
    assert_eq!(world.host.next_deadline(), None);
}

#[test]
fn shutdown_answers_closed_only_for_sign_outs_that_never_started() {
    let mut world = World::new();
    let check = command(&world.sources(&[b"a"]), b"a", Operation::Check);
    assert!(world.step(host::Event::Down(host::LinkDown::SignOut { name: name_of(b"a") })).is_empty());
    assert_eq!(
        world.step(host::Event::CancelAll { reason: host::Reason::Shutdown }),
        vec![host::Request::Up(host::LinkUp::SignedOut {
            name: name_of(b"a"),
            result: host::SignOutResult::Failed { reason: host::SourceFailure::Closed, detail: Box::new([]) }
        })]
    );
    assert!(world.checked(check, host::LoginState::Usable).is_empty());
    assert!(world.host.sources_settled());
}
