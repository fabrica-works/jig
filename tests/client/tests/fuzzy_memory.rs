//! The complete ordered page sweep retains the original heap and scope checks.
#[path = "support/memory.rs"]
mod fixture;
#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;
#[test]
fn full_documents_tabs_and_all_page_kinds_fit_the_client_and_view_worst_cases() {
    fixture::exercise(None);
}
