//! Offers and the authenticated entrance share their current authority facts.
use super::*;
use crate::acts as offers;
use crate::acts::{Act, Admission, Beyond, Bounds};

fn policy_holder(numbers: &[u32]) -> offers::Recipient {
    let mut roles = List::with_capacity(u32::try_from(numbers.len()).unwrap());
    for number in numbers {
        roles.push(*number).unwrap();
    }
    offers::Recipient::Roles { project: 1, roles }
}

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
    assert_eq!(beyond, Beyond::Proposed { to: policy_holder(&[0]) });
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
fn a_policy_proposal_names_every_eligible_role_without_choosing_one_holder() {
    let (mut core, mut limits) = core();
    limits.authority.roles = 3;
    let mut policy = core.authority.policy(1).unwrap().clone();
    let mut second = policy.roles[0].clone();
    second.number = 4;
    let mut third = second.clone();
    third.number = 5;
    third.decides = authority::Proposals(16);
    policy.roles = Box::new([policy.roles[0].clone(), second, third]);
    core.authority = authority::Domain::new(core.authority.rules().clone(), limits.authority).unwrap();
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut core.authority, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    let Admission::Bounded { beyond, .. } =
        offers::admit(&core, &limits, 1, Act::SetGoal, &Subject::Project { project: 1 })
    else {
        panic!("bounded goal")
    };
    assert_eq!(beyond, Beyond::Proposed { to: policy_holder(&[0, 4]) });
    assert!(!core.goal_start(1, 1, people::Role::Owner, 1, 501, None, limits.tasks.tree_tasks).unwrap().direct);
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
    assert_eq!(beyond, Beyond::Proposed { to: policy_holder(&[0]) });
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
        people::Ask::SetGoal {
            project: 1,
            spec: b"goal".as_slice().into(),
            charter: 1,
            budget: 1,
            priority: 1,
            deadline: None,
        },
        people::Ask::SetPool { project: 1, person: 1, budget: 1 },
        people::Ask::SetRoles { project: 1, holdings: Box::new([]) },
        people::Ask::MakeService { project: 1, name: b"service".as_slice().into(), role: people::Role::Member },
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
            let mut replied = false;
            let mut saved = 0;
            for row in &out {
                match row {
                    Request::Held(held) => {
                        if let Held::PeopleReply { reply, .. } = &**held {
                            replied |= *reply == people::Reply::Outcome(outcome);
                        }
                    }
                    Request::Write(Write::Save(Record::People(people::Stored::Answer { outcome: value, .. }))) => {
                        assert_eq!(*value, outcome);
                        saved += 1;
                    }
                    Request::Write(Write::Save(Record::Tasks(_))) => panic!("refused act wrote a task"),
                    Request::Write(_) | Request::Ask { .. } | Request::Now(_) | Request::Decided => {}
                }
            }
            assert!(replied, "index {index}, replay {replay}: {out:?}");
            assert_eq!(saved, usize::from(!replay));
            core.reclaim();
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
    let mut sent = false;
    for row in &out {
        if let tasks::Request::Sent { .. } = row {
            sent = true;
        }
    }
    assert!(sent, "{out:?}");
    let word = core.tasks.task(2).unwrap().inbox.last().unwrap();
    assert_eq!(word.from, tasks::Party::Person(1));
    assert_eq!(word.words.as_ref(), b"maintainer word");
    assert_eq!(word.kind, tasks::MessageKind::Words);
}

#[test]
fn offered_person_handoffs_and_answers_reach_the_child_and_change_the_offer() {
    let (mut core, limits) = make_tree();
    let mut row = new_row(&core, 5, tasks::Funder::Period { project: 1, period: 1 }, None);
    row.executor = tasks::Executor::Person(tasks::PersonAddress::Role(0));
    send_task(
        &mut core,
        &limits,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(80)),
            creator: tasks::Party::Person(1),
            batch: Box::new([row]),
        },
    );
    let subject = Subject::PersonTask { task: 5 };
    assert_eq!(offers::admit(&core, &limits, 1, Act::Take, &subject), Admission::Allowed);
    let event = core
        .person_task(Token::new(81), 1, 1, Some(people::Role::Owner), people::Ask::TakePerson { project: 1, task: 5 })
        .unwrap();
    send_task(&mut core, &limits, event);
    assert_eq!(core.person_tasks.remove(&Token::new(81)), Some(PersonTaskRoute::Take(5)));
    assert_eq!(core.tasks.task(5).unwrap().taken_by, Some(1));
    assert_eq!(offers::admit(&core, &limits, 1, Act::Take, &subject), Admission::Refused(people::Refusal::NotWaiting));
    assert_eq!(offers::admit(&core, &limits, 1, Act::HandBack, &subject), Admission::Allowed);
    let event = core
        .person_task(
            Token::new(82),
            1,
            1,
            Some(people::Role::Owner),
            people::Ask::HandBackPerson { project: 1, task: 5 },
        )
        .unwrap();
    send_task(&mut core, &limits, event);
    assert_eq!(core.person_tasks.remove(&Token::new(82)), Some(PersonTaskRoute::HandBack(5)));
    assert_eq!(core.tasks.task(5).unwrap().taken_by, None);
    let why = people::Refusal::NotWaiting;
    assert_eq!(offers::admit(&core, &limits, 1, Act::Answer, &subject), Admission::Refused(why));
    let answer = people::Ask::AnswerPerson {
        project: 1,
        task: 5,
        revision: core.tasks.task(5).unwrap().revisions.person,
        result: people::PersonResult::Report { words: b"done".as_slice().into() },
    };
    assert_eq!(core.person_task(Token::new(83), 1, 1, Some(people::Role::Owner), answer.clone()), Err(why));
    let event = core
        .person_task(Token::new(84), 1, 1, Some(people::Role::Owner), people::Ask::TakePerson { project: 1, task: 5 })
        .unwrap();
    send_task(&mut core, &limits, event);
    assert_eq!(core.person_tasks.remove(&Token::new(84)), Some(PersonTaskRoute::Take(5)));
    assert_eq!(offers::admit(&core, &limits, 1, Act::Answer, &subject), Admission::Allowed);
    let event = core.person_task(Token::new(85), 1, 1, Some(people::Role::Owner), answer).unwrap();
    send_task(&mut core, &limits, event);
    assert_eq!(core.person_tasks.remove(&Token::new(85)), Some(PersonTaskRoute::Answer(5)));
    let actor = core.tasks.task(5).unwrap().person_answer.unwrap();
    assert_eq!(actor.by, 1);
    assert_eq!(
        offers::admit(&core, &limits, 1, Act::Answer, &subject),
        Admission::Refused(people::Refusal::Ended { how: people::Ending::Decided { by: 1, at: actor.at } })
    );
}

#[test]
fn task_controls_owner_changes_priorities_and_note_scopes_share_their_judges() {
    let (mut core, limits) = make_tree();
    for (act, ask, number) in [
        (Act::Stop, people::Ask::Stop { project: 1, task: 2 }, 2),
        (Act::Cancel, people::Ask::Cancel { project: 1, task: 2, reason: b"done".as_slice().into() }, 2),
        (
            Act::Release,
            people::Ask::Release {
                project: 1,
                task: 3,
                revision: core.tasks.task(3).unwrap().revisions.hold,
                again: None,
            },
            3,
        ),
    ] {
        assert_eq!(offers::admit(&core, &limits, 1, act, &Subject::Task { task: number }), Admission::Allowed);
        assert_eq!(
            core.control_admit(&ask, 1, 1, Some(people::Role::Owner), limits.tasks.depth, limits.tasks.result_bytes),
            Ok(number)
        );
    }
    let why = people::Refusal::Authority { lacking: people::Lack::Tasks { left: 3 }, proposable: None };
    assert_eq!(offers::admit(&core, &limits, 1, Act::TakeOver, &Subject::Task { task: 2 }), Admission::Refused(why));
    assert_eq!(
        core.move_admit(1, Some(people::Role::Owner), 1, 2, limits.tasks.tree_tasks, limits.tasks.depth),
        Err(why)
    );
    assert_eq!(offers::admit(&core, &limits, 1, Act::Prioritise, &Subject::Goals { project: 1 }), Admission::Allowed);
    let event = core
        .prioritise(Token::new(90), 1, 1, Some(people::Role::Owner), Box::new([(1, 2)]), limits.tasks.tasks)
        .unwrap();
    let tasks::Event::Prioritise { project, by, goals, .. } = event else { panic!("one admitted priority request") };
    assert_eq!((project, by, goals.as_ref()), (1, tasks::Party::Person(1), &[(1, 2)][..]));
    for act in [Act::Adopt, Act::ChangePolicy, Act::SetRoles, Act::MakeService, Act::SetPool] {
        assert_eq!(offers::admit(&core, &limits, 1, act, &Subject::Project { project: 1 }), Admission::Allowed);
        assert_eq!(core.role_allowed(1, 1, limits.tasks.tree_tasks), Ok(()));
    }
    let why = people::Refusal::Authority { lacking: people::Lack::Shape, proposable: None };
    for act in [Act::EditNote, Act::DeleteNote] {
        assert_eq!(offers::note(&core, 1, 1, &people::NoteScope::Project, act), Err(why));
    }
    let mut rules = core.authority.rules().clone();
    rules.ceiling.notes = authority::Scopes(2);
    let mut policy = core.authority.policy(1).unwrap().clone();
    policy.ceiling.notes = authority::Scopes(2);
    policy.roles[0].authority.notes = authority::Scopes(2);
    core.authority = authority::Domain::new(rules, limits.authority).unwrap();
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut core.authority, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    for act in [Act::EditNote, Act::DeleteNote] {
        assert_eq!(offers::note(&core, 1, 1, &people::NoteScope::Project, act), Ok(()));
    }
    holding(&mut core, &limits, people::Role::Observer);
    let why = people::Refusal::Role { held: Some(people::Role::Observer) };
    for act in [Act::EditNote, Act::DeleteNote] {
        assert_eq!(offers::note(&core, 1, 1, &people::NoteScope::Project, act), Err(why));
    }
}

fn sign_in(core: &mut Core, limits: &Limits) {
    let identity = core.people.identity(1).unwrap().clone();
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    people::step(
        &mut core.people,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people },
        people::Event::SignedIn {
            reply_to: ReplyTo::new(Token::new(100)),
            person: 1,
            sign_in: 9,
            identity,
            kind: people::Kind::Person,
        },
        &mut out,
    );
    assert_eq!(core.people.person(9, Time::ZERO, Wall::EPOCH), Some(1));
}

fn ask_now(core: &mut Core, limits: &Limits, key: u8, ask: people::Ask) -> people::Outcome {
    let Requests::Out(out) = step(
        core,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
        Event::People(people::Event::Ask { reply_to: ReplyTo::new(Token::new(101)), sign_in: 9, key: [key; 16], ask }),
    );
    for row in &out {
        if let Request::Held(held) = row
            && let Held::PeopleReply { reply: people::Reply::Outcome(outcome), .. } = &**held
        {
            return *outcome;
        }
    }
    panic!("one terminal keyed outcome: {out:?}");
}

#[test]
fn offered_proposal_decisions_are_made_at_the_named_holder_and_another_holder_is_refused() {
    for (act, choice, expected) in [
        (Act::Accept, people::ProposalDecision::Accept, people::ProposalChoice::Accepted),
        (
            Act::Reject,
            people::ProposalDecision::Reject { reason: b"no".as_slice().into() },
            people::ProposalChoice::Rejected,
        ),
        (Act::PassUp, people::ProposalDecision::Pass, people::ProposalChoice::Passed),
    ] {
        let (mut core, limits) = make_tree();
        sign_in(&mut core, &limits);
        let mut next = new_row(&core, 5, tasks::Funder::Task(1), None);
        next.authority.delegation.tasks = 1;
        next.authority.delegation.depth = 1;
        send_task(
            &mut core,
            &limits,
            tasks::Event::Propose {
                reply_to: ReplyTo::new(Token::new(102)),
                proposal: tasks::Proposal {
                    number: 10,
                    proposer: 2,
                    project: 1,
                    action: tasks::ProposalAction::Batch(Box::new([next])),
                    reason: b"room".as_slice().into(),
                    as_holder: true,
                    state: tasks::ProposalState::Pending {
                        holder: tasks::ProposalHolder::Person(1),
                        since: Wall::EPOCH,
                    },
                },
            },
        );
        let object = Subject::Proposal { proposer: Proposer::Task(2), number: 10 };
        assert_eq!(offers::admit(&core, &limits, 1, act, &object), Admission::Allowed);
        assert_eq!(offers::admit(&core, &limits, 2, act, &object), Admission::Refused(people::Refusal::NotWaiting));
        assert_eq!(
            ask_now(
                &mut core,
                &limits,
                1,
                people::Ask::DecideProposal { project: 1, proposer: 2, proposal: 10, decision: choice }
            ),
            people::Outcome::ProposalDecided { proposer: 2, proposal: 10, by: 1, at: Wall::EPOCH, choice: expected }
        );
    }
}

#[test]
fn leaving_an_escalation_held_closing_a_chat_and_marking_it_seen_are_admitted() {
    let (mut core, limits) = tree_core();
    sign_in(&mut core, &limits);
    let chat = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, None);
    send_task(
        &mut core,
        &limits,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(110)),
            creator: tasks::Party::Person(1),
            batch: Box::new([chat]),
        },
    );
    let task = Subject::Task { task: 1 };
    assert_eq!(offers::admit(&core, &limits, 1, Act::MarkSeen, &task), Admission::Allowed);
    assert_eq!(
        ask_now(&mut core, &limits, 1, people::Ask::MarkSeen { seen: people::Seen::Chat { task: 1, through: 0 } }),
        people::Outcome::MarkedSeen { seen: people::Seen::Chat { task: 1, through: 0 } }
    );
    core.reclaim();
    assert_eq!(offers::admit(&core, &limits, 1, Act::Close, &task), Admission::Allowed);
    assert_eq!(offers::close(&core, 1, core.tasks.task(1).unwrap()), Ok(()));
    send_task(&mut core, &limits, tasks::Event::Hold { task: 1, why: tasks::Hold::Uncertain { entry: 1 } });
    let Admission::Bounded { .. } = offers::admit(&core, &limits, 1, Act::AmendRelease, &task) else {
        panic!("amend and release remains bounded with an explicit uncertainty choice")
    };
    let revision = core.tasks.escalation(1).unwrap().escalation.revision();
    send_task(
        &mut core,
        &limits,
        tasks::Event::RoutedEscalation { task: 1, revision, holder: tasks::EscalationHolder::Person(1), entry: 1 },
    );
    assert_eq!(offers::admit(&core, &limits, 1, Act::LeaveHeld, &Subject::Escalation { task: 1 }), Admission::Allowed);
    assert_eq!(offers::escalation(&core, &limits, 2, Act::LeaveHeld, 1), Err(people::Refusal::Unknown));
    assert_eq!(
        ask_now(
            &mut core,
            &limits,
            2,
            people::Ask::DecideEscalation {
                project: 1,
                task: 1,
                revision,
                decision: people::EscalationDecision::Reject { reason: b"leave held".as_slice().into() },
            }
        ),
        people::Outcome::EscalationDecided {
            task: 1,
            revision,
            by: 1,
            at: Wall::EPOCH,
            choice: people::EscalationChoice::Rejected
        }
    );
    core.reclaim();
    assert_eq!(offers::admit(&core, &limits, 1, Act::Close, &task), Admission::Allowed);
    assert_eq!(
        ask_now(&mut core, &limits, 3, people::Ask::Close { project: 1, chat: 1, disposals: List::with_capacity(0) }),
        people::Outcome::Closing { chat: 1, cancelled: 0, kept: 0 }
    );
}
