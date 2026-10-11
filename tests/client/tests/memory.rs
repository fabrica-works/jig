use jig_client_world::{Settings, stories};
#[path = "support/memory.rs"]
mod fixture;
#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

#[test]
fn full_conversation_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(0));
}

#[test]
fn full_deployment_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(1));
}

#[test]
fn full_first_family_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(2));
}

#[test]
fn full_task_overview_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(3));
}

#[test]
fn full_task_activity_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(4));
}

#[test]
fn full_task_plan_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(5));
}

#[test]
fn full_task_history_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(6));
}

#[test]
fn full_task_budget_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(7));
}

#[test]
fn full_task_panels_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(8));
}

#[test]
fn full_project_overview_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(9));
}

#[test]
fn full_project_goals_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(10));
}

#[test]
fn full_project_tasks_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(11));
}

#[test]
fn full_project_notes_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(12));
}

#[test]
fn full_project_settings_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(13));
}

#[test]
fn full_project_resources_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(14));
}

#[test]
fn full_project_policy_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(15));
}

#[test]
fn full_project_roles_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(16));
}

#[test]
fn full_project_spend_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(17));
}

#[test]
fn full_project_recurring_documents_tabs_and_fields_fit_the_client_and_view_worst_cases() {
    fixture::exercise(Some(18));
}

/// A real client allocation crosses a deliberately small declared budget.
/// The same callback guard used by the full fixture must catch it.
#[test]
#[should_panic(expected = "bounded: client and view exceeded the heap envelope")]
fn a_client_that_exceeds_its_declared_heap_budget_is_caught() {
    let mut world = stories::configured(Settings { memory: true, heap_bytes_per_client: 1024, ..Settings::default() });
    world.client(7, stories::inbox());
}
