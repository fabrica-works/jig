//! Offers and the authenticated entrance share their current authority facts.
use super::*;
use crate::acts as offers;
use crate::acts::{Act, Admission, Beyond, Bounds};

fn holding(core: &mut Core, limits: &Limits, role: people::Role) {
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    people::step(
        &mut core.people,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people },
        people::Event::Roles { project: 1, holdings: Box::new([people::Holding { person: 1, role }]) },
        &mut out,
    );
}

fn policy(core: &mut Core, accept: bool, period: u64) {
    let mut value = core.authority.policy(1).unwrap().clone();
    value.roles[0].decides = if accept { authority::Proposals::ALL } else { authority::Proposals(16) };
    value.roles[0].period_spend = period;
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut core.authority, authority::Event::Policy { project: 1, policy: value }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Changed { project: 1 }));
}

#[test]
fn a_goal_within_its_offer_is_direct_and_a_supported_goal_beyond_it_is_proposed() {
    let (core, limits) = core();
    let Admission::Bounded { within: Bounds::Goal { budget, pool_left, charters, .. }, beyond } =
        offers::admit(&core, &limits, 1, Act::SetGoal, &Subject::Project { project: 1 })
    else {
        panic!("bounded goal")
    };
    assert_eq!((budget, pool_left), (500, 500));
    assert_eq!(charters.as_slice(), &[1]);
    assert_eq!(beyond, Beyond::Proposed { to: offers::Recipient::Role { project: 1, role: 0 } });
    assert!(core.goal_start(1, 1, people::Role::Owner, 1, budget, None, limits.tasks.tree_tasks).unwrap().direct);
    assert!(!core.goal_start(1, 1, people::Role::Owner, 1, budget + 1, None, limits.tasks.tree_tasks).unwrap().direct);
    assert_eq!(
        core.goal_start(1, 1, people::Role::Owner, 2, budget, None, limits.tasks.tree_tasks).unwrap_err(),
        people::Refusal::NotOffered
    );
}

#[test]
fn a_goal_without_an_accepting_role_reports_the_same_refusal_in_its_bound_and_admission() {
    let (mut core, limits) = core();
    policy(&mut core, false, 500);
    let Admission::Bounded { beyond: Beyond::Refused(why), .. } =
        offers::admit(&core, &limits, 1, Act::SetGoal, &Subject::Project { project: 1 })
    else {
        panic!("unproposable bound")
    };
    assert_eq!(why, people::Refusal::Authority { lacking: people::Lack::Approver, proposable: Some(false) });
    assert_eq!(core.goal_start(1, 1, people::Role::Owner, 1, 501, None, limits.tasks.tree_tasks).unwrap_err(), why);
    assert!(core.goal_start(1, 1, people::Role::Owner, 1, 100, None, limits.tasks.tree_tasks).unwrap().direct);
}

#[test]
fn an_amendment_uses_live_pool_and_remaining_task_facts_and_preserves_narrowing() {
    let (mut core, limits) = core();
    policy(&mut core, true, 1000);
    make_task(&mut core, &limits, 1);
    send_task(
        &mut core,
        &limits,
        tasks::Event::CarvePool {
            reply_to: ReplyTo::new(Token::new(2)),
            project: 1,
            person: 1,
            period: 1,
            budget: 600,
        },
    );
    let Admission::Bounded { within: Bounds::Amend { budget_left, authority, ceiling, .. }, beyond } =
        offers::admit(&core, &limits, 1, Act::Amend, &Subject::Task { task: 1 })
    else {
        panic!("bounded amendment")
    };
    assert_eq!(budget_left, 400);
    assert_eq!(authority.ceiling.spend, 500);
    assert_eq!(ceiling.ceiling.spend, 600);
    assert_eq!(authority.tasks, core.tasks.delegation(1).unwrap().tasks_left);
    assert_eq!(beyond, Beyond::Proposed { to: offers::Recipient::Role { project: 1, role: 0 } });
    let mut after = core.tasks.task(1).unwrap().authority.clone();
    after.budget.spend = 550;
    let mut change = tasks::Amendment {
        spec: None,
        wake: None,
        dependencies: None,
        authority: Some(after),
        release: None,
        reason: Box::new([]),
    };
    assert_eq!(
        core.amend_admit(1, Some(people::Role::Owner), 1, 1, &change, limits.tasks.depth, &limits.tasks),
        Ok((false, true))
    );
    policy(&mut core, false, 1000);
    let Admission::Bounded { beyond: Beyond::Refused(why), .. } =
        offers::admit(&core, &limits, 1, Act::Amend, &Subject::Task { task: 1 })
    else {
        panic!("unproposable amendment")
    };
    assert_eq!(
        core.amend_admit(1, Some(people::Role::Owner), 1, 1, &change, limits.tasks.depth, &limits.tasks),
        Err(why)
    );
    change.authority.as_mut().unwrap().budget.spend = 90;
    assert_eq!(
        core.amend_admit(1, Some(people::Role::Owner), 1, 1, &change, limits.tasks.depth, &limits.tasks),
        Ok((true, false))
    );
}

#[test]
fn an_observers_core_role_refusals_match_their_offers() {
    let (mut core, limits) = core();
    holding(&mut core, &limits, people::Role::Observer);
    let project = Subject::Project { project: 1 };
    let expected = people::Refusal::Role { held: Some(people::Role::Observer) };
    for act in
        [Act::SetGoal, Act::Prioritise, Act::Adopt, Act::ChangePolicy, Act::SetRoles, Act::MakeService, Act::SetPool]
    {
        assert_eq!(offers::admit(&core, &limits, 1, act, &project), Admission::Refused(expected));
    }
    core.people = people::Domain::new(&limits.people, Box::new([]), 2);
    let mut child = Queue::with_capacity(people::max_out(&limits.people));
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    people::step(
        &mut core.people,
        &env,
        people::Event::Restore {
            record: people::Stored::Person {
                number: 1,
                identity: people::Identity {
                    key: people::IdentityKey { provider: 0, subject: Box::new([1]) },
                    login: b"owner".as_slice().into(),
                    name: b"A Person".as_slice().into(),
                },
            },
        },
        &mut child,
    );
    people::step(
        &mut core.people,
        &env,
        people::Event::Restore {
            record: people::Stored::Roles {
                project: 1,
                holdings: Box::new([people::Holding { person: 1, role: people::Role::Observer }]),
            },
        },
        &mut child,
    );
    people::step(
        &mut core.people,
        &env,
        people::Event::Restore {
            record: people::Stored::SignIn {
                number: 9,
                person: 1,
                expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()),
            },
        },
        &mut child,
    );
    people::step(
        &mut core.people,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people },
        people::Event::Restored,
        &mut child,
    );
    for (index, ask) in [
        people::Ask::SetGoal { project: 1, spec: Box::new([]), charter: 1, budget: 1, priority: 1, deadline: None },
        people::Ask::SetPool { project: 1, person: 1, budget: 1 },
        people::Ask::SetRoles { project: 1, holdings: Box::new([]) },
        people::Ask::MakeService { project: 1, name: Box::new([]), role: people::Role::Member },
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(offers::request(&core, &limits, 1, 1, Some(people::Role::Observer), &ask), Err(expected));
        let key = [u8::try_from(index).unwrap(); 16];
        let outcome = people::Outcome::Refused(expected);
        for replay in [false, true] {
            let Requests::Out(out) = step(
                &mut core,
                &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
                Event::People(people::Event::Ask {
                    reply_to: ReplyTo::new(Token::new(77)),
                    sign_in: 9,
                    key,
                    ask: ask.clone(),
                }),
            );
            assert!(
                out.iter().any(|row| if let Request::Held(held) = row {
                    if let Held::PeopleReply { reply, .. } = &**held {
                        *reply == people::Reply::Outcome(outcome)
                    } else {
                        false
                    }
                } else {
                    false
                }),
                "{out:?}"
            );
            assert_eq!(
                out.iter()
                    .filter(|row| {
                        if let Request::Write(Write::Save(Record::People(people::Stored::Answer {
                            outcome: saved,
                            ..
                        }))) = row
                        {
                            *saved == outcome
                        } else {
                            false
                        }
                    })
                    .count(),
                usize::from(!replay)
            );
            assert!(!out.iter().any(|row| if let Request::Write(Write::Save(Record::Tasks(_))) = row {
                true
            } else {
                false
            }));
        }
    }
}

#[test]
fn a_maintainer_word_uses_the_trusted_entrance_and_keeps_its_actor() {
    let (mut core, limits) = make_tree();
    holding(&mut core, &limits, people::Role::Maintainer);
    assert_eq!(offers::admit(&core, &limits, 1, Act::Write, &Subject::Task { task: 2 }), Admission::Allowed);
    let event = core
        .person_message(
            &limits,
            PersonMessage {
                request: Token::new(77),
                person: 1,
                project: 1,
                task: 2,
                question: None,
                release: false,
                words: b"maintainer word".as_slice().into(),
                at: Wall::EPOCH,
            },
        )
        .unwrap();
    let mut out = Queue::with_capacity(tasks::max_out(&limits.tasks));
    tasks::step(&mut core.tasks, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.tasks }, event, &mut out);
    assert!(out.iter().any(|row| if let tasks::Request::Sent { .. } = row { true } else { false }), "{out:?}");
    let word = core.tasks.task(2).unwrap().inbox.last().unwrap();
    assert_eq!(word.from, tasks::Party::Person(1));
    assert_eq!(word.words.as_ref(), b"maintainer word");
    assert_eq!(word.kind, tasks::MessageKind::Words);
}
