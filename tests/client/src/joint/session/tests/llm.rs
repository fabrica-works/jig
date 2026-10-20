//! Choices cross the joint transport through the real root and its durable task documents.
use super::*;
use jig_client::contract as c;
use jig_domain_authority as authority;
use jig_fake_workers::{Script, Worker};
use skein_lib::{List, Queue};

fn configuration() -> root::Config {
    let mut config = jig_core_world::world::config(8455);
    let mut alternate = config.core.settings.models[0].clone();
    alternate.name = Box::from(b"alternate".as_slice());
    let mut blocked = alternate.clone();
    blocked.name = Box::from(b"blocked".as_slice());
    config.core.settings.charters[0].policy.alternatives = Box::new([alternate.clone(), blocked.clone()]);
    config.core.settings.models = Box::new([config.core.settings.models[0].clone(), alternate, blocked]);
    config.core.settings.llm_providers.providers[0].models = Box::new([
        Box::from(b"scripted".as_slice()),
        Box::from(b"alternate".as_slice()),
        Box::from(b"blocked".as_slice()),
    ]);
    config
}

fn install_policy(rules: authority::Rules, policy: authority::Policy, limits: authority::Limits) -> authority::Domain {
    let mut domain = authority::Domain::new(rules, limits).expect("bounded authority policy");
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut out);
    domain
}

fn ready(config: root::Config) -> (Session, Time, Wall) {
    let worker = Worker::new(1, 1, vec![Box::new([Script::Wait])]);
    let (mut session, now, wall) = configured_session(config, Peers::new(vec![worker], Vec::new()));
    session.bind(ClientId(0), 1, identity(), now, wall);
    session.poll(now, wall).expect("genuine authentication");
    (session, now, wall)
}

fn down(value: jig_client::Down) -> Request {
    Request::Client { request: jig_client::Request::Down { down: value } }
}

fn read(session: &mut Session, now: Time, wall: Wall, what: c::Read) -> c::ReadValue {
    session
        .send(
            ClientId(0),
            now,
            wall,
            down(jig_client::Down::Read { read: Token::new(91), what: jig_client::Readable::Jig(what) }),
        )
        .expect("whole read request");
    let callbacks = session.poll(now, wall).expect("whole read answer");
    let [
        (
            ClientId(0),
            Event::Client {
                event:
                    jig_client::Event::Up {
                        up:
                            jig_client::Up::ReadDone {
                                result: jig_client::Readout::Jig(c::ReadResult::Value(value)), ..
                            },
                    },
            },
        ),
    ] = callbacks.as_slice()
    else {
        panic!("one document: {callbacks:?}")
    };
    value.clone()
}

fn me(session: &mut Session, now: Time, wall: Wall) -> Box<c::Me> {
    let c::ReadValue::Me(value) = read(session, now, wall, c::Read::Me) else { panic!("Me document") };
    value
}

fn task(session: &mut Session, now: Time, wall: Wall, task: u64) -> Box<c::Task> {
    let c::ReadValue::Task(value) = read(session, now, wall, c::Read::Task { task }) else { panic!("Task document") };
    value
}

fn text(value: &[u8]) -> c::Text {
    c::Text::new(value.into(), 4096).expect("fixture name")
}

fn choice(account: Option<u32>) -> c::LlmChoice {
    c::LlmChoice {
        account,
        main: c::ModelEffort { model: text(b"scripted"), effort: Some(text(b"high")) },
        subagents: List::with_capacity(0),
    }
}

fn ask(session: &mut Session, now: Time, wall: Wall, key: u8, ask: c::Ask) -> c::Answer {
    session
        .send(
            ClientId(0),
            now,
            wall,
            down(jig_client::Down::Send {
                attempt: Token::new(u64::from(key)),
                key: [key; 16],
                request: jig_client::Ask::Jig(Box::new(ask)),
                value: None,
            }),
        )
        .expect("whole keyed request");
    let callbacks = session.poll(now, wall).expect("durable keyed answer");
    let [
        (
            ClientId(0),
            Event::Client {
                event:
                    jig_client::Event::Up {
                        up: jig_client::Up::Answer { answer: Some(jig_client::Answered::Jig(answer)), .. },
                    },
            },
        ),
    ] = callbacks.as_slice()
    else {
        panic!("one final answer: {callbacks:?}")
    };
    answer.clone()
}

fn creating(me: &c::Me, choice: c::LlmChoice) -> c::Ask {
    let mut request = jig_client::shapes::chat(me, 1, text(b"a chosen run")).expect("chat template");
    let c::Ask::Create { tasks, .. } = &mut request else { panic!("create recipe") };
    tasks.get_mut(0).expect("one member").llm = Some(choice);
    request
}

fn created(answer: &c::Answer) -> u64 {
    let c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::Created { first, .. })) = answer else {
        panic!("created: {answer:?}")
    };
    *first
}

fn account(session: &Session) -> u32 {
    session
        .driver
        .world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Accounts(jig_domain_accounts::Stored::Account { number, .. })) => {
                Some(*number)
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("configured deployment account")
}

#[test]
fn chosen_effort_reaches_the_task() {
    let (mut session, now, wall) = ready(configuration());
    let me = me(&mut session, now, wall);
    let chosen = choice(None);
    let number = created(&ask(&mut session, now, wall, 41, creating(&me, chosen.clone())));
    assert_eq!(task(&mut session, now, wall, number).llm, Some(chosen));
}

#[test]
fn named_account_returns_its_kept_pin() {
    let (mut session, now, wall) = ready(configuration());
    let me = me(&mut session, now, wall);
    let account = account(&session);
    let chosen = choice(Some(account));
    let number = created(&ask(&mut session, now, wall, 41, creating(&me, chosen.clone())));
    let actual = task(&mut session, now, wall, number);
    assert_eq!(actual.kept_account, Some(account));
}

#[test]
fn model_amendment_replaces_the_choice() {
    let (mut session, now, wall) = ready(configuration());
    let me = me(&mut session, now, wall);
    let account = account(&session);
    let mut chosen = choice(Some(account));
    chosen.subagents = crate::test::list([c::ModelEffort { model: text(b"blocked"), effort: Some(text(b"low")) }]);
    let number = created(&ask(&mut session, now, wall, 41, creating(&me, chosen)));
    let before = task(&mut session, now, wall, number);
    let mut next = before.llm.clone().expect("current choice");
    next.main.model = text(b"alternate");
    ask(
        &mut session,
        now,
        wall,
        42,
        c::Ask::Amend {
            project: 1,
            task: number,
            revision: Some(before.spec_revision),
            change: Box::new(c::Amendment {
                addressed: None,
                spec: None,
                wake: None,
                dependencies: None,
                authority: None,
                instructions: None,
                tracked: None,
                llm: Some(next.clone()),
            }),
            reason: text(b"change model"),
        },
    );
    let actual = task(&mut session, now, wall, number).llm.expect("amended choice");
    assert_eq!(actual, next);
}

#[test]
fn authority_refuses_the_chosen_model() {
    let (mut session, now, wall) = ready(configuration());
    let me = me(&mut session, now, wall);
    let mut request = creating(&me, choice(None));
    let c::Ask::Create { tasks, .. } = &mut request else { panic!("create recipe") };
    tasks.get_mut(0).expect("one member").authority.models =
        Some(c::AuthorityModels::Named(crate::test::list([Box::from(b"alternate".as_slice())])));
    assert_eq!(
        ask(&mut session, now, wall, 41, request),
        c::Answer::Refused {
            refusal: c::Refusal::Llm { model: text(b"scripted"), why: c::LlmWhy::Authority },
            at: None,
        }
    );
}

#[test]
fn me_lists_allowed_models_in_configuration_order() {
    let mut config = configuration();
    let mut rules = config.core.authority.rules().clone();
    rules.ceiling.models = authority::Models::Named(0b011);
    let mut policy = config.core.authority.policy(1).expect("project policy").clone();
    policy.ceiling.models = authority::Models::Named(0b011);
    for role in &mut policy.roles {
        role.authority.models = authority::Models::Named(0b011);
    }
    config.core.authority = install_policy(rules, policy, jig_core_world::world::limits().core.authority);
    let (mut session, now, wall) = ready(config);
    let me = me(&mut session, now, wall);
    let models = &me.projects.get(0).expect("project membership").models;
    assert_eq!(
        models.iter().map(|model| model.name.bytes()).collect::<Vec<_>>(),
        [b"scripted".as_slice(), b"alternate".as_slice()]
    );
}

#[test]
fn an_unconfigured_model_is_refused() {
    let (mut session, now, wall) = ready(configuration());
    let me = me(&mut session, now, wall);
    let mut chosen = choice(None);
    chosen.main.model = text(b"removed");
    let answer = ask(&mut session, now, wall, 41, creating(&me, chosen));
    assert_eq!(
        answer,
        c::Answer::Refused {
            refusal: c::Refusal::Llm { model: text(b"removed"), why: c::LlmWhy::Unconfigured },
            at: None
        }
    );
}

#[test]
fn an_unconfigured_effort_is_refused_at_its_operation() {
    let (mut session, now, wall) = ready(configuration());
    let mut chosen = choice(None);
    chosen.main.effort = Some(text(b"absent"));
    let answer = ask(
        &mut session,
        now,
        wall,
        41,
        c::Ask::Compound {
            project: 1,
            operations: crate::test::list([
                c::Operation::Message { project: 1, task: 1, words: text(b"hello") },
                c::Operation::Amend {
                    project: 1,
                    task: 1,
                    revision: None,
                    change: Box::new(c::Amendment {
                        addressed: None,
                        spec: None,
                        wake: None,
                        dependencies: None,
                        authority: None,
                        instructions: None,
                        tracked: None,
                        llm: Some(chosen),
                    }),
                    reason: text(b"change effort"),
                },
            ]),
        },
    );
    assert_eq!(
        answer,
        c::Answer::Refused {
            refusal: c::Refusal::Llm { model: text(b"scripted"), why: c::LlmWhy::Effort },
            at: Some(1)
        }
    );
}

#[test]
fn task_authority_keeps_the_named_model() {
    let (mut session, now, wall) = ready(configuration());
    let me = me(&mut session, now, wall);
    let mut chosen = choice(None);
    chosen.main.model = text(b"alternate");
    let mut request = creating(&me, chosen);
    let c::Ask::Create { tasks, .. } = &mut request else { panic!("create recipe") };
    let names = c::AuthorityModels::Named(crate::test::list([Box::from(b"alternate".as_slice())]));
    tasks.get_mut(0).expect("one member").authority.models = Some(names.clone());
    let number = created(&ask(&mut session, now, wall, 41, request));
    assert_eq!(task(&mut session, now, wall, number).authority.authority.models, Some(names));
}

#[test]
fn authority_admits_a_named_model() {
    let mut config = configuration();
    let rules = config.core.authority.rules().clone();
    let policy = config.core.authority.policy(1).expect("project policy").clone();
    let mut limits = *config.core.authority.limits();
    limits.batch = 2;
    config.core.authority = install_policy(rules, policy, limits);
    let (mut session, now, wall) = ready(config);
    let me = me(&mut session, now, wall);
    let mut chosen = choice(None);
    chosen.main.model = text(b"blocked");
    let mut request = creating(&me, chosen);
    let c::Ask::Create { tasks, .. } = &mut request else { panic!("create recipe") };
    let mut first = tasks.get(0).expect("one member").clone();
    first.authority.models = Some(c::AuthorityModels::Named(crate::test::list([Box::from(b"blocked".as_slice())])));
    first.authority.delegation.tasks = 0;
    first.authority.delegation.depth = 0;
    first.authority.delegation.kinds = List::with_capacity(0);
    first.budget = c::NewBudget::Named;
    first.authority.spend = 10;
    let mut second = first.clone();
    second.authority.models = Some(c::AuthorityModels::Named(crate::test::list([Box::from(b"alternate".as_slice())])));
    second.llm.as_mut().expect("chosen model").main.model = text(b"alternate");
    *tasks = crate::test::list([first, second]);
    let answer = ask(&mut session, now, wall, 41, request);
    let c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::Created { first, last, .. })) = answer else {
        panic!("admitted named models: {answer:?}")
    };
    assert_eq!(last.checked_sub(first), Some(1));
}

#[test]
fn question_offers_keep_the_named_model() {
    use crate::joint::wire;
    use jig_domain_core::{acts, documents, reads};

    let (mut session, _, _) = ready(configuration());
    let mut source = wire::documents::tests::question_source();
    let mut authority = wire::task::tests::source().authority;
    authority.models = jig_domain_tasks::Models::Named(0b010);
    source.acts = crate::test::list([acts::Offered {
        effect: None,
        act: acts::Act::Amend,
        admission: acts::Admission::Bounded {
            within: acts::Bounds::Amend {
                budget_left: 0,
                grants: List::with_capacity(0),
                implications: List::with_capacity(0),
                ceiling: Box::new(authority.clone()),
                authority: Box::new(authority),
            },
            beyond: acts::Beyond::Refused(people::Refusal::KeyConflict),
        },
    }]);
    let what = c::Read::Question { task: source.task.number, message: source.message };
    let to =
        session.driver.transport.reply_to(Reply::Read { client: ClientId(0), read: Token::new(91), what }).into_token();
    let answer = reads::Answer::Document(Box::new(documents::Document::Question(source)));
    let Incoming::Callback {
        event:
            Event::Client {
                event:
                    jig_client::Event::Up {
                        up:
                            jig_client::Up::ReadDone {
                                result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Question(value))),
                                ..
                            },
                    },
            },
        ..
    } = read_packets::read_with_codec(
        &mut session.driver.transport,
        to,
        &answer,
        &session.limits.client.domain,
        &session.codec,
    )
    .expect("whole original Question")
    else {
        panic!("Question callback")
    };
    let c::Admission::Bounded { within: c::Bounds::Amend { authority, .. }, .. } =
        &value.acts.get(0).expect("offered amendment").admission
    else {
        panic!("amendment authority")
    };
    assert_eq!(
        authority.authority.models,
        Some(c::AuthorityModels::Named(crate::test::list([Box::from(b"alternate".as_slice()),])))
    );
}
