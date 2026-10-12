//! Restoration precedes deployment allocation and transient admission
//! (`domain/engine.md`, 5 and 6; `client/domain/contract.md`, 3.1 and 4).
use super::*;

fn env() -> Env<Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }
}

fn identity() -> people::Identity {
    people::Identity {
        key: people::IdentityKey { provider: 0, subject: Box::new([0, 255, 128, 7]) },
        login: Box::new([255, 0, 1]),
        name: Box::new([0, 128, 254]),
    }
}

fn signin(core: &mut Core, env: &Env<Limits>, token: u64) {
    let Requests::Out(mut out) =
        step(core, env, Event::SignIn { reply_to: ReplyTo::new(Token::new(token)), identity: identity() });
    assert_eq!(out.len(), 2, "one answer and the completed decision, with no write");
    let Some(Request::Held(reply)) = out.pop() else { panic!("held sign-in answer") };
    let Held::PeopleReply { to, sign_in, reply } = *reply else { panic!("original people reply") };
    assert_eq!(to.into_token(), Token::new(token));
    assert_eq!(sign_in, None, "refusal has no allocated sign-in candidate");
    assert_eq!(reply, people::Reply::Again(people::Again::NotReady));
    let Some(Request::Decided) = out.pop() else { panic!("completed sign-in decision") };
    assert!(out.is_empty());
}

#[test]
fn an_empty_load_commits_the_original_bootstrap_only_after_loading_completes() {
    let env = env();
    let mut core = Core::new(config(47), &env.limits);
    let mut expected = core.counters.deployment();
    assert!(core.counters.dirty(), "ordinary first start still needs its header");
    assert_eq!(core.restart_begin(), RestartRequest::Step(RestartStep::LoadCore));
    assert!(!core.counters.dirty(), "no header commit while its durable source is unknown");
    assert_eq!(core.restart_begin(), RestartRequest::Idle);
    assert_eq!(core.counters.deployment(), expected);
    for token in [7101, 7102] {
        signin(&mut core, &env, token);
        assert!(!core.counters.dirty());
        assert_eq!(core.counters.deployment(), expected);
    }
    assert_eq!(core.restart_done(RestartStep::LoadCore), RestartRequest::Step(RestartStep::LoadInbox));
    assert!(core.counters.dirty());
    expected.commits = 1;
    assert_eq!(core.counters.next_commit(), expected, "the first real header keeps the supplied deployment identity");
    assert!(!core.counters.dirty());
}

#[test]
fn a_restored_header_preserves_every_counter_and_refused_signins_allocate_nothing() {
    let env = env();
    let mut core = Core::new(config(48), &env.limits);
    let mut expected = core.counters.deployment();
    expected.id = [239; 16];
    expected.tasks = 11;
    expected.people = 12;
    expected.sign_ins = 13;
    expected.messages = 14;
    expected.runs = 15;
    expected.calls = 16;
    expected.connector_rows = 17;
    expected.applications = 18;
    expected.commits = 19;
    assert_eq!(core.restart_begin(), RestartRequest::Step(RestartStep::LoadCore));
    assert_eq!(core.restore_core(CoreRecord::Deployment(expected), &env.limits), Restored::Deployment { commits: 19 });
    for token in [7103, 7104] {
        signin(&mut core, &env, token);
        assert_eq!(core.counters.deployment(), expected);
        assert!(!core.counters.dirty());
    }
    assert_eq!(core.restart_done(RestartStep::LoadCore), RestartRequest::Step(RestartStep::LoadInbox));
    signin(&mut core, &env, 7105);
    assert_eq!(core.counters.deployment(), expected);
    assert!(!core.counters.dirty(), "restoring an existing deployment never bootstrap-writes it");
    assert_eq!(fresh(&mut core.counters, Family::Task), Some(12));
    expected.tasks = 12;
    expected.commits = 20;
    assert_eq!(core.counters.next_commit(), expected, "the first new allocation advances the actual durable header");
}

#[test]
fn an_unknown_header_cannot_allocate_any_family_or_become_dirty_on_time() {
    let mut env = env();
    let mut core = Core::new(config(49), &env.limits);
    let expected = core.counters.deployment();
    assert_eq!(core.restart_begin(), RestartRequest::Step(RestartStep::LoadCore));
    for family in [
        Family::Task,
        Family::Person,
        Family::SignIn,
        Family::Message,
        Family::Run,
        Family::Call,
        Family::ConnectorRow,
        Family::Application,
    ] {
        assert_eq!(fresh(&mut core.counters, family), None);
        assert_eq!(core.counters.deployment(), expected);
        assert!(!core.counters.dirty());
    }
    env.now = Time::ZERO.saturating_add(Duration::from_secs(75));
    env.wall = Wall::from_nanos(Duration::from_secs(65).as_nanos());
    let Requests::Out(mut out) = fire(&mut core, &env, Timer::People);
    let Some(Request::Decided) = out.pop() else { panic!("completed timer decision") };
    assert!(out.is_empty());
    assert_eq!(core.counters.deployment(), expected);
    assert!(!core.counters.dirty());
}
