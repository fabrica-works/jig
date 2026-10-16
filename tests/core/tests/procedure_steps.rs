use jig_core_world::effects::World;
use jig_core_world::fixtures::{live_row, member};
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

const CONNECTOR: u16 = 1;
const PROCEDURE_CODE: u32 = 1;
const DELEGATE_CALL: u64 = 9001;
const AMEND_CALL: u64 = 9002;
const COMPLETION: u32 = 1;
const MOVED_COMPLETION: u32 = 2;

fn procedure_world(seed: u64) -> (World, u64) {
    let (configuration, limits) = jig_core_world::faults::plan_fixture(seed);
    let mut world = World::configured(seed, false, configuration, limits);
    world.hold_procedures = true;
    let root_run = world.assignments[0];
    let procedure =
        member(&world, tasks::Executor::Procedure { connector: CONNECTOR, code: PROCEDURE_CODE }, b"procedure");
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(DELEGATE_CALL)),
        key: core::CallKey { task: root_run.0, attempt: root_run.1, completion: COMPLETION, position: 0 },
        batch: Box::new([procedure]),
        stubs: Box::new([]),
    }));
    let core::CallPart::Delegated(numbers) = &world.answers.last().expect("procedure made").2 else {
        panic!("procedure refused")
    };
    let task = numbers[0];
    (world, task)
}

fn decide(world: &mut World, task: u64, decision: core::ProcedureStep) {
    let (step, number, code, _) = world.procedure_inputs.last().expect("procedure input");
    world.send(root::Event::Core(core::Event::ProcedureStep {
        task,
        step: *step,
        connector: *number,
        code: u32::from(*code),
        decision,
    }));
}

fn make_delegate(world: &mut World, procedure: u64) -> u64 {
    let mut child = member(world, tasks::Executor::Agent { charter: 1 }, b"delegate");
    child.authority.budget.spend = 5;
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    decide(
        world,
        procedure,
        core::ProcedureStep { batch: Box::new([child]), ..core::ProcedureStep::new(core::StepEnd::Wait) },
    );
    live_row(world, procedure).delegates[0]
}

#[test]
fn a_procedure_hears_the_first_task_its_step_made_with_its_revision() {
    let (mut world, procedure) = procedure_world(461);
    let child = make_delegate(&mut world, procedure);
    let record = connector::Record::ProcedureDelegateOutcome {
        task: procedure,
        step: 1,
        child: Some(child),
        revisions: Box::new([(child, 0)]),
    };
    assert_eq!(
        world.store.rows.get(&root::Key::Connector { number: CONNECTOR, key: record.key() }),
        Some(&root::Record::Connector { number: CONNECTOR, record })
    );
}

#[test]
fn a_step_naming_a_moved_revision_is_refused_and_told_what_it_is_now() {
    let (mut world, procedure) = procedure_world(462);
    let child = make_delegate(&mut world, procedure);
    let run = world.assignments[0];
    let amendment = tasks::Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        release: None,
        spec: Some(tasks::Spec { words: b"moved".as_slice().into(), parameters: Box::new([]), inputs: Box::new([]) }),
        wake: None,
        dependencies: None,
        authority: None,
        reason: b"changed".as_slice().into(),
    };
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(AMEND_CALL)),
        key: core::CallKey { task: run.0, attempt: run.1, completion: MOVED_COMPLETION, position: 0 },
        action: core::NamedAction::Amend { target: child, revision: 0, amendment: amendment.clone() },
    }));
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::WakeProcedure { task: procedure })));
    let before = live_row(&world, child).clone();
    decide(
        &mut world,
        procedure,
        core::ProcedureStep {
            amendments: Box::new([core::StepAmendment { task: child, revision: 0, amendment }]),
            ..core::ProcedureStep::new(core::StepEnd::Wait)
        },
    );
    assert_eq!(live_row(&world, child), &before);
    let record = connector::Record::ProcedureDelegateOutcome {
        task: procedure,
        step: 2,
        child: None,
        revisions: Box::new([(child, 1)]),
    };
    assert_eq!(
        world.store.rows.get(&root::Key::Connector { number: CONNECTOR, key: record.key() }),
        Some(&root::Record::Connector { number: CONNECTOR, record })
    );
    assert_eq!(before.revisions.spec, 1);
}
