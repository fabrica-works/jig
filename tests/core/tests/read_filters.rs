use jig_core_world::stories::reads;

#[test]
fn scoped_description_search_pages_durable_notes_after_restart() {
    reads::scoped_description_search_pages_durable_notes_after_restart();
}

#[test]
fn result_and_reply_filters_page_their_own_retained_updates() {
    reads::result_and_reply_filters_page_their_own_retained_updates();
}
