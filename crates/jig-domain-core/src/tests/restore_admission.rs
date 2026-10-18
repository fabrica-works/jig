//! Restoration precedes deployment allocation and transient admission
//! (`domain/engine.md`, 5 and 6; `client/domain/contract.md`, 3.1 and 4).
const PARTY: u64 = 1;
const OTHER_TASK: u64 = 3;
use super::*;

const UTF8_IDENTITY_WORDS: (&[u8], &[u8]) = (b"~\0\x01", b"\0^!");

fn env() -> Env<Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }
}

fn identity() -> people::Identity {
    people::Identity {
        key: people::IdentityKey { provider: 0, subject: Box::new([0, 255, 128, 7]) },
        login: Box::from(UTF8_IDENTITY_WORDS.0),
        name: Box::from(UTF8_IDENTITY_WORDS.1),
    }
}

fn signin(core: &mut Core, env: &Env<Limits>, token: u64) {
    let Requests::Out(mut out) =
        step(core, env, Event::SignIn { reply_to: ReplyTo::new(Token::new(token)), identity: identity() });
    assert_eq!(out.len(), 2, "one answer and the completed decision, with no write");
    let Some(Request::Held(reply)) = out.pop() else { panic!("held sign-in answer") };
    let Held::PeopleReply { to, reply } = *reply else { panic!("original people reply") };
    assert_eq!(to.into_token(), Token::new(token));
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

fn core_keys() -> [CoreKey; 7] {
    [
        CoreKey::Projection(ProjectionKey::Goal(0)),
        CoreKey::RunProof(0),
        CoreKey::Call(CallKey { task: 0, attempt: 0, completion: 0, position: 0 }),
        CoreKey::Turn { task: 0, attempt: 0, turn: 0 },
        CoreKey::Terminal { task: 0, attempt: 0 },
        CoreKey::ProposalDecision { proposer: Proposer::Task(0), proposal: 0 },
        CoreKey::EscalationDecision { task: 0, revision: 0 },
    ]
}

#[test]
fn the_deployment_key_sorts_before_every_other_core_key() {
    for key in core_keys() {
        assert!(CoreKey::Deployment < key);
    }
}

#[test]
fn restore_ranges_admit_no_turn_terminal_history_or_note() {
    let mut history = List::with_capacity(9);
    for key in core_keys().into_iter().skip(3) {
        history.push(Key::Core(key)).expect("history fixture capacity");
    }
    for key in [
        Key::Tasks(tasks::Key::Ended(0)),
        Key::Tasks(tasks::Key::History { task: 0, revision: 0 }),
        Key::Tasks(tasks::Key::Milestone { task: 0, position: 0 }),
        Key::Tasks(tasks::Key::Activity { task: 0, position: 0 }),
        Key::Notes(notes::Key::Entry { name: 0 }),
    ] {
        history.push(key).expect("history fixture capacity");
    }
    for key in &history {
        for range in RestoreRange::LIVE {
            assert!(!range.admits(key), "{range:?} excludes {key:?}");
        }
    }
}

#[test]
fn restore_ranges_admit_each_live_family_once() {
    let mut live = List::with_capacity(17);
    for key in core_keys().into_iter().take(3) {
        live.push(Key::Core(key)).expect("live fixture capacity");
    }
    for key in [
        Key::Core(CoreKey::Deployment),
        Key::Tasks(tasks::Key::Live(0)),
        Key::Tasks(tasks::Key::Stub(0)),
        Key::Tasks(tasks::Key::Ledger(tasks::Funder::Task(0))),
        Key::Tasks(tasks::Key::Writer(0)),
        Key::Tasks(tasks::Key::Pool(0)),
        Key::Tasks(tasks::Key::PersonProposal(0)),
        Key::People(people::Key::Person(0)),
        Key::People(people::Key::ReadResults { party: 0 }),
        Key::People(people::Key::ReadConversation { party: 0, task: 0 }),
        Key::People(people::Key::SignIn(0)),
        Key::People(people::Key::Roles(0)),
        Key::People(people::Key::Policy(0)),
        Key::People(people::Key::Answer(people::RequestKey { person: 0, key: [0; 16] })),
    ] {
        live.push(key).expect("live fixture capacity");
    }
    for key in &live {
        let mut count = 0;
        for range in RestoreRange::LIVE {
            count += u32::from(range.admits(key));
        }
        assert_eq!(count, 1, "{key:?}");
    }
}

fn record_key_rows() -> [(CoreRecord, CoreKey); 8] {
    [
        (CoreRecord::Deployment(Core::new(config(50), &limits()).counters.deployment()), CoreKey::Deployment),
        (
            CoreRecord::Projection(ProjectionRecord::Goal {
                goal: ProjectionGoal {
                    number: 7,
                    project: 1,
                    priority: 1,
                    words: Box::new([]),
                    phase: tasks::Phase::Waiting,
                },
                pending: Box::new([]),
                changed: false,
                fed_at: None,
            }),
            CoreKey::Projection(ProjectionKey::Goal(7)),
        ),
        (
            CoreRecord::Call(CallRecord {
                key: CallKey { task: 11, attempt: 13, completion: 17, position: 19 },
                part: CallPart::Controlled,
                settled: None,
            }),
            CoreKey::Call(CallKey { task: 11, attempt: 13, completion: 17, position: 19 }),
        ),
        (
            CoreRecord::RunProof(RunProof::claimed(23, 29, 29, None, jig_domain_fleet::HostKind::Worker)),
            CoreKey::RunProof(23),
        ),
        (
            CoreRecord::Turn(TurnRecord {
                opens_window: false,
                task: 31,
                attempt: 37,
                turn: 41,
                spent: 0,
                read: None,
                at: Wall::EPOCH,
                transcript: Box::new([]),
            }),
            CoreKey::Turn { task: 31, attempt: 37, turn: 41 },
        ),
        (
            CoreRecord::Terminal(TerminalRecord { task: 43, attempt: 47, cumulative: 0, end: tasks::End::Parked }),
            CoreKey::Terminal { task: 43, attempt: 47 },
        ),
        (
            CoreRecord::ProposalDecision(ProposalDecisionRecord {
                project: 1,
                proposer: Proposer::Task(53),
                proposal: 59,
                kind: tasks::ProposalKind::Batch,
                by: 1,
                at: Wall::EPOCH,
                choice: people::ProposalChoice::Accepted,
                created: None,
            }),
            CoreKey::ProposalDecision { proposer: Proposer::Task(53), proposal: 59 },
        ),
        (
            CoreRecord::EscalationDecision(EscalationDecisionRecord {
                project: 1,
                requester: 1,
                task: 61,
                revision: 67,
                by: 1,
                at: Wall::EPOCH,
                decision: people::EscalationDecision::Release,
            }),
            CoreKey::EscalationDecision { task: 61, revision: 67 },
        ),
    ]
}

#[test]
fn a_records_key_is_the_key_its_row_is_saved_under() {
    for (row, key) in record_key_rows() {
        assert_eq!(row.key(), key, "the row owns its address");
        assert_eq!(Record::Core(row).key(), Key::Core(key), "the child tag wraps the same address");
    }
}

fn census_fixture() -> (Core, Env<Limits>) {
    let mut env = env();
    env.limits.people.inbox_entries = 8;
    let mut configuration = config(79);
    configuration.owners = Box::new([]);
    let mut core = Core::new(configuration, &env.limits);
    assert_eq!(core.restart_begin(), RestartRequest::Step(RestartStep::LoadCore));
    let mut header = core.counters.deployment();
    header.tasks = 20;
    header.people = 7;
    header.messages = 5;
    header.runs = 1;
    core.counters = Counters::new(header);
    restore_census_people(&mut core, &env);
    assert_eq!(core.restart_done(RestartStep::LoadCore), RestartRequest::Step(RestartStep::LoadInbox));
    (core, env)
}
#[test]
fn the_census_lists_each_party_after_its_read_position() {
    let (mut core, env) = census_fixture();
    assert_eq!(
        core.census(&env.limits).results.as_ref(),
        [ReadResults { party: 1, after: 3 }, ReadResults { party: 7, after: 0 }]
    );
}
fn ended_census_chat() -> tasks::Stored {
    let (live, _) = watch_core();
    let mut row = live.tasks.task(1).expect("fixture chat").clone();
    row.number = 2;
    row.root = 2;
    row.depth = 0;
    row.result_position = 5;
    row.last_reply = 1;
    row.activity = 1;
    row.attempt = 1;
    row.turn = 1;
    row.replies =
        Box::new([tasks::ReplyHint { number: 1, activity: 1, attempt: 1, turn: 1, at: Wall::from_nanos(50) }]);
    row.ended_at = Some(Wall::from_nanos(100));
    row.phase = tasks::Phase::Ended(tasks::Ending::Cancelled {
        by: tasks::Party::Person(1),
        at: Wall::from_nanos(100),
        reason: Box::new([1]),
        result: None,
    });
    tasks::Stored::Ended(Box::new(row))
}
#[test]
fn an_ended_chat_handed_twice_to_the_census_changes_nothing() {
    let (mut core, env) = census_fixture();
    let _census = core.census(&env.limits);
    let chat = ended_census_chat();
    assert!(core.restore_inbox_row(&env.limits, &Record::Tasks(chat.clone())));
    let before = core.people.counts(&env.limits.people, 1).expect("party counts");
    assert!(core.restore_inbox_row(&env.limits, &Record::Tasks(chat.clone())));
    assert_eq!(core.people.counts(&env.limits.people, 1), Some(before));
    assert_eq!(core.people.cached_results(1).expect("party results").len(), 1);
    let _next = core.restart_done(RestartStep::LoadInbox);
    assert!(core.census_progress.is_empty());
}

fn restore_census_people(core: &mut Core, env: &Env<Limits>) {
    let people_env = Env { now: env.now, wall: env.wall, limits: env.limits.people };
    let mut out = Queue::with_capacity(people::max_out(&env.limits.people));
    for party in [1, 7] {
        let mut who = identity();
        who.key.subject = Box::new([u8::try_from(party).expect("fixture party")]);
        people::step(
            &mut core.people,
            &people_env,
            people::Event::Restore { record: people::Stored::Person { number: party, identity: who } },
            &mut out,
        );
    }
    for record in [
        people::Stored::ReadResults { party: 1, through: 3 },
        people::Stored::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
    ] {
        people::step(&mut core.people, &people_env, people::Event::Restore { record }, &mut out);
    }
    people::step(&mut core.people, &people_env, people::Event::Restored, &mut out);
    assert!(out.is_empty());
}

#[test]
fn a_census_position_followed_by_another_task_is_refused() {
    let (mut core, env) = census_fixture();
    let _census = core.census(&env.limits);
    assert!(core.restore_inbox_row(
        &env.limits,
        &Record::People(people::Stored::EndedRead { task: OTHER_TASK, party: PARTY, through: 0 })
    ));
    assert!(!core.restore_inbox_row(&env.limits, &Record::Tasks(ended_census_chat())));
}
