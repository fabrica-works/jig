use jig_client_protocol_world::{fixtures, native};
#[test]
fn a_native_client_signs_in_by_the_service_s_link_and_starts_a_chat() {
    let outcome = native::run(fixtures::SEED, native::Script::start_chat(b"Hello"));
    assert_eq!(outcome.signed_in, 1);
    assert_eq!(outcome.chats_made, 1);
    assert!(outcome.drained);
}
