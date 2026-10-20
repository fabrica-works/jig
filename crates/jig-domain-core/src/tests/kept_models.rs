//! A sender names no model: the core fills each authority's models from the task it acts for
//! (domain/authority.md, sections 3 and 5).
use super::*;

const TASK: u64 = 10;
const MODELS: tasks::Models = tasks::Models(0b101);

fn core_with_task() -> Core {
    let (mut core, limits) = tree_core();
    let mut row = new_row(&core, TASK, tasks::Funder::Period { project: 1, period: 1 }, None);
    row.authority.models = MODELS;
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([row]));
    core
}

#[test]
fn proposed_members_amendments_and_widenings_take_the_task_s_models() {
    let core = core_with_task();
    let mut member = new_row(&core, 11, tasks::Funder::Task(TASK), None);
    member.authority.models = tasks::Models::NONE;
    let mut batch = tasks::ProposalAction::Batch(Box::new([member]));
    core.keep_proposal_models(TASK, &mut batch);
    let tasks::ProposalAction::Batch(members) = batch else { panic!("batch") };
    assert_eq!(members.first().unwrap().authority.models, MODELS);

    let mut authority = members.first().unwrap().authority.clone();
    authority.models = tasks::Models::NONE;
    let mut widen = tasks::ProposalAction::Widen { task: TASK, authority: authority.clone() };
    core.keep_proposal_models(TASK, &mut widen);
    let tasks::ProposalAction::Widen { authority: widened, .. } = widen else { panic!("widen") };
    assert_eq!(widened.models, MODELS);

    let mut amendment = fixtures::unchanged(b"narrow");
    amendment.authority = Some(authority);
    core.keep_models(TASK, &mut amendment);
    assert_eq!(amendment.authority.unwrap().models, MODELS);
}

#[test]
fn an_unknown_task_keeps_no_model() {
    let core = core_with_task();
    assert_eq!(core.kept_models(99), tasks::Models::NONE);
}
