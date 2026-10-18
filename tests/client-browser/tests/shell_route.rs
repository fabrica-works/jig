#[path = "../../../testing/jig-client-test-shell-web/src/mapping.rs"]
mod mapping;

#[test]
fn shell_sign_in_uses_the_server_s_mount() {
    let call =
        jig_client_protocol_http::exchange(&mapping::mapping(), None, jig_client_protocol_link::Message::SignInService);
    assert_eq!(call.path.as_ref(), b"/jig/v1/sign-in/deployment");
}
