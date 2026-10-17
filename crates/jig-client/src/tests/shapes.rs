use super::fixtures::*;
use crate::{contract::*, shapes::*};
use skein_lib::{List, Wall};
const CHAT_TASK: u64 = 41;
const CANCELLED: u64 = 43;
const MOVED: u64 = 47;
const DEADLINE: Wall = Wall::from_nanos(100);
const NOW: Wall = Wall::from_nanos(10);
const SPAN_DEADLINE: Wall = Wall::from_nanos(5_000_000_010);
fn words() -> Text {
    Text::new(b"Hello".as_slice().into(), 32).unwrap()
}
fn draft() -> GoalDraft {
    GoalDraft {
        words: words(),
        executor: OwnerExecutor::Agent { charter: 1 },
        budget: 150,
        deadline: None,
        priority: Some(7),
    }
}
#[test]
fn chat_fills_the_chat_template_into_one_as_allowed_task() {
    let me = me();
    let Ask::Create { project, tasks } = chat(&me, PROJECT, words(), NOW).unwrap() else { panic!("create") };
    assert_eq!(project, PROJECT);
    assert_eq!(tasks.len(), 1);
    let task = tasks.get(0).unwrap();
    assert_eq!(task.executor, OwnerExecutor::Agent { charter: 1 });
    assert_eq!(
        task.spec,
        Specification { words: words(), parameters: List::with_capacity(0), inputs: List::with_capacity(0) }
    );
    assert_eq!(task.contract, NewContract::Report { words: 32 });
    assert_eq!(task.budget, NewBudget::AsAllowed { cap: None });
    let mut expected = creating().within.authority.whole().unwrap();
    expected.spend = 0;
    expected.deadline = None;
    assert_eq!(task.authority, expected);
    assert_eq!(task.wake, None);
    assert_eq!(task.dependencies.len(), 0);
    assert_eq!(task.named.len(), 0);
    assert_eq!(task.tracked, None);
    assert_eq!(task.label, 1);
    assert_eq!(task.flags, TaskFlags { conversation: true, listed: false, retention: Retention::Kept });
}
#[test]
fn close_chat_cancels_the_chat_last_with_its_template_s_grace() {
    let delegates = [Delegate { task: CANCELLED, fate: Fate::Cancel }, Delegate { task: MOVED, fate: Fate::Move }];
    let Ask::Compound { project, operations } = close_chat(&me(), PROJECT, CHAT_TASK, &delegates, words()).unwrap()
    else {
        panic!("compound")
    };
    assert_eq!(project, PROJECT);
    assert_eq!(
        operations.as_slice(),
        &[
            Operation::Cancel { project: PROJECT, task: CANCELLED, reason: words(), grace: None },
            Operation::Move { project: PROJECT, task: MOVED, reason: words() },
            Operation::Cancel { project: PROJECT, task: CHAT_TASK, reason: words(), grace: Some(SHAPE_GRACE) },
        ]
    );
}
#[test]
fn fall_puts_a_draft_without_a_deadline_beyond_a_bound_that_has_one() {
    let mut creating = creating();
    creating.within.latest = Some(DEADLINE);
    let mut draft = draft();
    draft.budget = 10;
    assert_eq!(fall(&creating, &draft), Fall::Proposable(Recipient::Role { project: PROJECT, role: OWNER_ROLE }));
    creating.within.maximum_deadline = Some(DEADLINE);
    assert_eq!(fall(&creating, &draft), Fall::NotOffered(NotOffered::Ceiling));
}
#[test]
fn fall_gives_not_offered_for_an_executor_the_bounds_do_not_list() {
    let mut draft = draft();
    draft.executor = OwnerExecutor::Agent { charter: 2 };
    assert_eq!(fall(&creating(), &draft), Fall::NotOffered(NotOffered::Executor));
}
#[test]
fn set_goal_proposes_to_the_holder_the_bounds_name() {
    let Judged::Propose { ask, to } = set_goal(&me(), PROJECT, draft(), words(), NOW) else { panic!("proposed") };
    assert_eq!(to, Recipient::Role { project: PROJECT, role: OWNER_ROLE });
    let Ask::Propose { project, action: ProposeAction::Batch { tasks }, reason } = ask else { panic!("batch") };
    assert_eq!((project, reason), (PROJECT, words()));
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks.get(0).unwrap().tracked, Some(7));
    assert_eq!(tasks.get(0).unwrap().authority.spend, 150);
}
#[derive(Clone, Copy, Debug)]
enum Unfillable {
    Approval,
    CutVerdict,
    ZeroDepth,
    Executor,
    Budget,
    Deadline,
}
const UNFILLABLE: [Unfillable; 6] = [
    Unfillable::Approval,
    Unfillable::CutVerdict,
    Unfillable::ZeroDepth,
    Unfillable::Executor,
    Unfillable::Budget,
    Unfillable::Deadline,
];
#[test]
fn a_template_shapes_cannot_fill_is_not_offered() {
    for case in UNFILLABLE {
        let mut me = me();
        let template = me.projects.get_mut(0).unwrap().templates.get_mut(0).unwrap();
        match case {
            Unfillable::Approval => template.contract = facts_task().result_contract,
            Unfillable::CutVerdict => {
                template.contract = TaskContractDocument::Verdict { choices: List::with_capacity(0), cut: true }
            }
            Unfillable::ZeroDepth => template.shallower = 0,
            Unfillable::Executor => template.executor = TemplateExecutor::Chosen,
            Unfillable::Budget => template.budget = TemplateBudget::Given,
            Unfillable::Deadline => template.deadline = TemplateDeadline::Given,
        }
        assert_eq!(chat(&me, PROJECT, words(), NOW), Err(NotOffered::Template), "{case:?}");
    }
}
#[test]
fn a_template_span_and_narrowing_fill_the_task() {
    let mut me = me();
    let template = me.projects.get_mut(0).unwrap().templates.get_mut(0).unwrap();
    template.shallower = 3;
    template.deadline = TemplateDeadline::Span(SHAPE_GRACE);
    let Ask::Create { tasks, .. } = chat(&me, PROJECT, words(), NOW).unwrap() else { panic!("create") };
    let task = tasks.get(0).unwrap();
    assert_eq!(task.authority.delegation.depth, 2);
    assert_eq!(task.authority.deadline, Some(SPAN_DEADLINE));
}
