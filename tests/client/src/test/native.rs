//! Native transport input shared by the scripted and joint fixtures.
pub(crate) fn up(up: jig_client::Up) -> jig_client_test_domain::Event {
    jig_client_test_domain::Event::Client { event: jig_client::Event::Up { up } }
}
