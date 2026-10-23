//! The client and actual engine share one translation boundary
//! (client/domain/testing.md, 5). The front and real stories are added after
//! the complete document carriers; no scripted truth stands in for core facts.
pub mod engine;
mod fixture;
pub mod front;
pub mod random;
pub use front::Joint;
mod offered_packets;
pub mod packets;
pub mod read_packets;
pub mod referee;
pub mod requests;
pub mod session;
pub mod transport;
pub mod truth;
pub mod wire;

#[cfg(test)]
mod account_moves;
#[cfg(test)]
mod stories;

#[cfg(test)]
fn take_signed_in(
    client: crate::harness::ClientId,
    mut callbacks: Vec<(crate::harness::ClientId, jig_client_test_domain::Event)>,
) -> Vec<(crate::harness::ClientId, jig_client_test_domain::Event)> {
    let signed_in = |value: &(crate::harness::ClientId, jig_client_test_domain::Event)| {
        matches!(value, (recipient, jig_client_test_domain::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::SignedIn }
        }) if *recipient == client)
    };
    let at = callbacks.iter().position(signed_in).expect("the real sign-in tells the public client");
    callbacks.remove(at);
    assert!(!callbacks.iter().any(signed_in), "one authentication notification");
    callbacks
}
