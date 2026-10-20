//! An agent task's LLM choice: kept with the task, moved by Amend, given to a delegate, and
//! refused naming the model and why (domain/tasks.md, sections 3, 4 and 7; domain/authority.md,
//! section 8.1).
use jig_domain_tasks::{
    Amendment, Authority, Event, Executor, LlmChoice, LlmFault, ModelEffort, Models, Party, PersonAddress, Problem,
    Refusal,
};
use jig_tasks_world::{LIMITS, Reply, World, authority, domain, task};

const PERSON: Party = Party::Person(1);

fn effort(model: u8, effort: Option<u8>) -> ModelEffort {
    ModelEffort { model, effort }
}

fn choice(main: ModelEffort, subagents: &[ModelEffort]) -> LlmChoice {
    LlmChoice { account: None, main, subagents: subagents.into() }
}

fn allowing(models: u64) -> Authority {
    Authority { models: Models(models), ..authority() }
}

fn chosen(number: u64, models: u64, llm: LlmChoice) -> jig_domain_tasks::New {
    let mut new = task(number, &[]);
    new.authority = allowing(models);
    new.llm = Some(llm);
    new
}

fn amend(world: &mut World, task: u64, authority: Option<Authority>, llm: Option<LlmChoice>) -> Reply {
    let reply_to = world.to();
    let message = world.record(task).last_message + 1;
    world.send(Event::Amend {
        funder: None,
        reply_to,
        by: PERSON,
        task,
        message,
        stop_run: false,
        amendment: Amendment {
            tracked: None,
            addressed: None,
            instructions: None,
            release: None,
            spec: None,
            wake: None,
            dependencies: None,
            authority,
            llm,
            reason: b"another model".as_slice().into(),
        },
    });
    world.replies.last_key_value().expect("amendment answered").1.clone()
}

fn refused(reply: &Reply, model: u8, why: LlmFault) {
    assert!(
        matches!(reply, Reply::Refused(Problem { why: Refusal::Llm { model: m, why: w }, .. }) if *m == model && *w == why),
        "expected {model} {why:?}, got {reply:?}"
    );
}

#[test]
fn a_choice_is_kept_through_a_restart_amended_and_given_to_a_delegate() {
    let mut world = World::new(71, LIMITS);
    let first = choice(effort(0, Some(1)), &[effort(1, None), effort(2, Some(0))]);
    assert_eq!(world.make(PERSON, vec![chosen(1, 0b111, first.clone())]), Reply::Made(vec![1]));
    assert_eq!(world.record(1).llm, Some(first));
    world.restart();

    let second = choice(effort(2, Some(0)), &[]);
    assert!(!matches!(amend(&mut world, 1, None, Some(second.clone())), Reply::Refused(_)));
    assert_eq!(world.record(1).llm, Some(second.clone()));
    assert!(!matches!(amend(&mut world, 1, None, None), Reply::Refused(_)), "no choice leaves it unchanged");
    assert_eq!(world.record(1).llm, Some(second.clone()));

    let mut child = chosen(2, 0b010, choice(effort(1, None), &[]));
    child.funder = jig_domain_tasks::Funder::Task(1);
    assert_eq!(world.make(Party::Task(1), vec![child]), Reply::Made(vec![2]));
    assert_eq!(world.record(2).llm, Some(choice(effort(1, None), &[])));
    world.restart();
    assert_eq!(world.record(1).llm, Some(second));
}

#[test]
fn a_choice_is_refused_naming_its_first_failing_model_and_why() {
    let procedure = Executor::Procedure { connector: 1, code: 1 };
    let person = Executor::Person(PersonAddress::Person(9));
    let cases = [
        (Executor::Agent { charter: 1 }, 0b1111, choice(effort(9, None), &[]), 9, LlmFault::Unconfigured),
        (Executor::Agent { charter: 1 }, 0b1111, choice(effort(0, Some(2)), &[]), 0, LlmFault::Effort),
        (Executor::Agent { charter: 1 }, 0b1111, choice(effort(0, None), &[]), 0, LlmFault::Effort),
        (
            Executor::Agent { charter: 1 },
            0b1111,
            choice(effort(0, Some(0)), &[effort(1, Some(0))]),
            1,
            LlmFault::Effort,
        ),
        (Executor::Agent { charter: 1 }, 0b1111, choice(effort(1, None), &[effort(3, None)]), 3, LlmFault::Charter),
        (
            Executor::Agent { charter: 1 },
            0b0001,
            choice(effort(0, Some(0)), &[effort(1, None)]),
            1,
            LlmFault::Authority,
        ),
        (
            Executor::Agent { charter: 1 },
            0b1111,
            choice(effort(0, Some(0)), &[effort(1, None), effort(1, None)]),
            1,
            LlmFault::Repeated,
        ),
        (procedure, 0b1111, choice(effort(1, None), &[]), 1, LlmFault::Procedure),
        (person, 0b1111, choice(effort(1, None), &[]), 1, LlmFault::Procedure),
    ];
    for (executor, models, llm, model, why) in cases {
        let mut world = World::new(72, LIMITS);
        let before = world.records.clone();
        let mut new = chosen(1, models, llm);
        new.executor = executor;
        refused(&world.make(PERSON, vec![new]), model, why);
        assert_eq!(world.records, before, "{why:?} writes nothing");
    }
}

#[test]
fn an_amendment_s_choice_and_a_narrowed_authority_are_checked_against_each_other() {
    let mut world = World::new(73, LIMITS);
    let kept = choice(effort(0, Some(0)), &[effort(1, None)]);
    assert_eq!(world.make(PERSON, vec![chosen(1, 0b011, kept.clone())]), Reply::Made(vec![1]));
    refused(&amend(&mut world, 1, None, Some(choice(effort(2, Some(0)), &[]))), 2, LlmFault::Authority);
    refused(&amend(&mut world, 1, Some(allowing(0b001)), None), 1, LlmFault::Authority);
    assert_eq!(world.record(1).llm, Some(kept));
    assert_eq!(world.record(1).authority.models, Models(0b011));

    let mut procedure = task(2, &[]);
    procedure.executor = Executor::Procedure { connector: 1, code: 1 };
    assert_eq!(world.make(PERSON, vec![procedure]), Reply::Made(vec![2]));
    refused(&amend(&mut world, 2, None, Some(choice(effort(1, None), &[]))), 1, LlmFault::Procedure);
}

#[test]
fn a_charter_s_default_is_its_first_models_within_authority_with_their_first_efforts() {
    let domain = domain(&LIMITS, 74);
    assert_eq!(
        domain.default_llm(1, Models(0b111)),
        Some(choice(effort(0, Some(0)), &[effort(1, None), effort(2, Some(0))]))
    );
    assert_eq!(domain.default_llm(1, Models(0b1110)), Some(choice(effort(1, None), &[effort(2, Some(0))])));
    assert_eq!(domain.default_llm(2, Models(0b1000)), Some(choice(effort(3, None), &[])));
    assert_eq!(domain.default_llm(2, Models(0b0111)), None, "the authority allows none of the charter's");
    assert_eq!(domain.default_llm(9, Models(u64::MAX)), None, "the charter is not configured");
}
