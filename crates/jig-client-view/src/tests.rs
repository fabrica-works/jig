mod binding;
mod owner;
mod phrases;
mod regions;

mod cards;
mod cards_objects;

mod cards_access;

mod pages;

mod proposed_task;
mod recurring;
mod work_pages;

mod turns;

mod page_capacity;

mod owner_wake;
mod refusal_facts;

mod owner_authority;
mod owner_budget;

mod frame_facts;
mod owner_outcome;
mod owner_result;
mod policy_edit;
mod runtime_facts;

mod owner_chip;

mod chip_facts;
mod owner_activity;
mod owner_amendment;
mod resource_facts;

mod owner_history;
mod page_loss;

mod history_pages;
mod owner_milestone;

mod owner_inbox;
mod owner_tree;
mod question_sender;

mod owner_person_task;
mod owner_proposal;

mod conversation_work;
mod historical_pages;
mod offered;
mod owner_deployment;
mod owner_project;

mod historical_objects;

mod task_proposals;

mod turn_read;

mod limits;

fn text_bytes(node: &skein_ui::Node) -> u32 {
    let mut bytes = 0_u32;
    if let Some(name) = &node.name {
        bytes = u32::try_from(name.len()).unwrap();
    }
    if let Some(value) = &node.value {
        bytes = bytes.checked_add(u32::try_from(value.text.len()).unwrap()).unwrap();
    }
    if let Some(external) = &node.external {
        bytes = bytes.checked_add(u32::try_from(external.len()).unwrap()).unwrap();
    }
    for run in &node.text {
        bytes = bytes.checked_add(u32::try_from(run.text.len()).unwrap()).unwrap();
    }
    bytes
}
