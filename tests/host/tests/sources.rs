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
fn replacement_waits_for_the_old_whole_inventory_and_delayed_command() {
    let mut world = World::new();
    let a = command(&world.sources(&[b"a", b"b"]), b"a", Operation::Check);
    assert!(world.sources(&[b"c"]).is_empty());
    let b = command(&world.checked(a, host::LoginState::Usable), b"b", Operation::Check);
    let requests = world.checked(b, host::LoginState::NotSignedIn);
    assert_eq!(requests.len(), 2);
    snapshot(&requests[0], &[b"a", b"b"]);
    let c = command(&requests[1..], b"c", Operation::Check);
    snapshot(&world.checked(c, host::LoginState::NotSignedIn)[0], &[b"c"]);
    assert!(world.host.sources_settled());
}
#[test]
fn overlapping_checks_coalesce_and_an_unchanged_explicit_check_answers() {
    let mut world = World::new();
    let a = command(&world.sources(&[b"a"]), b"a", Operation::Check);
    for _ in 0..2 {
        assert!(world.step(host::Event::Down(host::LinkDown::Check { name: name_of(b"a") })).is_empty());
    }
    assert_eq!(world.checked(a, host::LoginState::NotSignedIn).len(), 1);
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
            result: host::SignOutResult::Failed { reason: host::SourceFailure::Closed, detail: Box::new([]) }
        })]
    );
    assert!(world.host.sources_settled());
    assert_eq!(world.host.next_deadline(), None);
}
