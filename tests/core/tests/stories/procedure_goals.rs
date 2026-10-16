//! A person's procedure goal retains its executor through the durable request boundary.
use super::*;

fn goal(budget: u64) -> people::Ask {
    people::Ask::SetGoal {
        project: 1,
        spec: b"procedure goal".as_slice().into(),
        executor: people::GoalExecutor::Procedure { connector: 1, code: 1 },
        budget,
        priority: 7,
        deadline: Some(skein_lib::Wall::from_nanos(60_000_000_000)),
    }
}

fn raise_pool_ceiling(world: &mut World, seed: u64) {
    let (configuration, _) = fixture(seed, false);
    let role = &configuration.core.authority.policy(1).expect("configured goal policy").roles[0];
    let authority = &role.authority;
    ask(
        world,
        92,
        people::Ask::ChangePolicy {
            project: 1,
            change: people::PolicyChange::Role(people::PolicyRole {
                number: role.number,
                authority: people::Authority {
                    tools: authority.tools.0,
                    grants: authority
                        .grants
                        .iter()
                        .map(|grant| people::Grant {
                            connector: grant.connector,
                            kind: grant.kind,
                            pattern: people::Pattern {
                                segments: grant.pattern.segments.clone(),
                                last: match &grant.pattern.last {
                                    authority::Last::Exact(value) => people::Last::Exact(value.clone()),
                                    authority::Last::Open(value) => people::Last::Open(value.clone()),
                                },
                            },
                        })
                        .collect(),
                    delegation: people::Delegation {
                        kinds: authority
                            .delegation
                            .kinds
                            .iter()
                            .map(|kind| match kind {
                                authority::Executor::Charter(code) => people::Executor::Charter(*code),
                                authority::Executor::Procedure(code) => people::Executor::Procedure(*code),
                                authority::Executor::Role(role) => people::Executor::Role(*role),
                            })
                            .collect(),
                        tasks: authority.delegation.tasks,
                        depth: authority.delegation.depth,
                    },
                    spend: authority.budget.spend,
                    deadline: authority.budget.deadline,
                    notes: authority.notes.0,
                    note_resources: Box::new([]),
                },
                period_spend: 1000,
                requests: role.requests.0,
                decides: role.decides.0,
            }),
        },
    );
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::PolicyChanged { project: 1 }))
    );
}

#[test]
fn a_procedure_goal_commits_before_delivery_and_replays_its_exact_owner_facts() {
    let mut world = jig_core_world::stories::offers::world(215, None);
    world.hold_procedures = true;
    let assignments = world.assignments.len();
    let replies = world.people_answers.len();
    world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
    ask(&mut world, 90, goal(100));
    assert_eq!(world.people_answers.len(), replies);
    assert!(world.procedure_inputs.is_empty(), "no procedure receives an uncommitted goal");
    world.release_commits();
    let Some(people::Reply::Outcome(people::Outcome::GoalStarted { task })) = world.people_answers.last().copied()
    else {
        panic!("procedure goal: {:?}", world.people_answers)
    };
    let row = live_row(&world, task).clone();
    assert_eq!(row.executor, tasks::Executor::Procedure { connector: 1, code: 1 });
    assert_eq!(row.spec.words.as_ref(), b"procedure goal");
    assert_eq!(row.tracked, Some(7));
    assert_eq!(row.numbers.budget, 100);
    assert_eq!(row.authority.tools, tasks::Tools(0));
    assert_eq!(row.authority.budget.deadline, Some(skein_lib::Wall::from_nanos(60_000_000_000)));
    assert_eq!(world.assignments.len(), assignments, "procedure goal allocates no agent run");
    assert!(
        world
            .procedure_inputs
            .iter()
            .any(|(_, connector, code, context)| *connector == 1 && *code == 1 && context.task == task)
    );
    let inputs = world.procedure_inputs.len();
    ask(&mut world, 90, goal(100));
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::GoalStarted { task })));
    assert_eq!(world.procedure_inputs.len(), inputs, "same keyed creation does not run twice");
    world.restart_with(jig_core_world::stories::offers::world_configuration(215));
    ask(&mut world, 90, goal(100));
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::GoalStarted { task })));
    assert_eq!(live_row(&world, task).executor, row.executor);
    assert_eq!(live_row(&world, task).tracked, row.tracked);
    jig_core_world::watches::open(&mut world, 93, core::documents::Watched::Task { task });
    let core::documents::Document::Task(document) = jig_core_world::watches::latest(&world, 93) else {
        panic!("actual task document")
    };
    assert_eq!(document.executor, row.executor);
    assert_eq!(document.project, 1);
    assert_eq!(world.offers.as_ref().expect("offer referee").checked, 1);
}

#[test]
fn a_procedure_goal_proposal_restores_and_accepts_the_original_executor() {
    let mut world = jig_core_world::stories::offers::world(216, None);
    world.hold_procedures = true;
    ask(&mut world, 90, jig_core_world::fixtures::propose_goal(goal(300)));
    let Some(people::Reply::Outcome(people::Outcome::Proposed { proposal, to: 0 })) =
        world.people_answers.last().copied()
    else {
        panic!("procedure proposal: {:?}", world.people_answers)
    };
    let key = root::Key::Core(core::Key::Tasks(tasks::Key::PersonProposal(proposal)));
    let Some(root::Record::Core(core::Record::Tasks(tasks::Stored::PersonProposal(row)))) = world.store.rows.get(&key)
    else {
        panic!("retained proposal")
    };
    let row = row.clone();
    assert_eq!(
        row.batch().first().expect("proposed member").executor,
        tasks::Executor::Procedure { connector: 1, code: 1 }
    );
    assert_eq!(row.batch().first().expect("proposed member").tracked, Some(7));
    assert_eq!(row.batch().first().expect("proposed member").authority.tools, tasks::Tools(0));
    assert!(world.procedure_inputs.is_empty());
    world.restart_with(jig_core_world::stories::offers::world_configuration(216));
    assert_eq!(
        world.store.rows.get(&key),
        Some(&root::Record::Core(core::Record::Tasks(tasks::Stored::PersonProposal(row.clone()))))
    );
    ask(&mut world, 90, jig_core_world::fixtures::propose_goal(goal(300)));
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Proposed { proposal, to: 0 }))
    );
    ask(
        &mut world,
        91,
        people::Ask::DecideProposal {
            project: 1,
            proposer: people::Proposer::Party(row.proposer),
            proposal,
            decision: people::ProposalDecision::Accept,
        },
    );
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Authority {
            lacking: people::Lack::Budget { needs: 300, left: 200 },
            proposable: Some(people::Accepter::Role(0))
        })))
    );
    assert!(world.procedure_inputs.is_empty(), "current funding is rechecked before acceptance");
    raise_pool_ceiling(&mut world, 216);
    ask(&mut world, 93, people::Ask::SetPool { project: 1, person: row.proposer, budget: 800 });
    assert!(matches!(world.people_answers.last(), Some(people::Reply::Outcome(people::Outcome::PoolSet { .. }))));
    ask(
        &mut world,
        94,
        people::Ask::DecideProposal {
            project: 1,
            proposer: people::Proposer::Party(row.proposer),
            proposal,
            decision: people::ProposalDecision::Accept,
        },
    );
    assert!(
        matches!(
            world.people_answers.last(),
            Some(people::Reply::Outcome(people::Outcome::ProposalDecided {
                choice: people::ProposalChoice::Accepted,
                ..
            }))
        ),
        "acceptance: {:?}",
        world.people_answers
    );
    let made = live_row(&world, row.batch().first().expect("proposed member").number);
    assert_eq!(made.executor, row.batch().first().expect("proposed member").executor);
    assert_eq!(made.tracked, row.batch().first().expect("proposed member").tracked);
    assert_eq!(made.spec, row.batch().first().expect("proposed member").spec);
    assert_eq!(made.authority, row.batch().first().expect("proposed member").authority);
    assert_eq!(made.numbers.budget, 300);
    assert!(world.procedure_inputs.iter().any(|(_, connector, code, context)| *connector == 1
        && *code == 1
        && context.task == row.batch().first().expect("proposed member").number));
}
