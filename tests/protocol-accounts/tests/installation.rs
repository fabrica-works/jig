//! The installation's identifier on the sign-in's wire (llm.md 5.4;
//! providers.md 2.2): the authorization request carries
//! `ext_agent_host_id` with the identifier the store keeps, at a first
//! start and at a later one that restores it instead of its own draw.
use jig_protocol_accounts::installation;
use jig_protocol_accounts_world::world::{self, Installation, Story};
use skein_fake_peers::oauth as peer;

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

const FIRST: [u8; 16] = [5; 16];
const LATER: [u8; 16] = [6; 16];

/// The parameter as the authorization URL's query carries it, form-encoded.
fn parameter(id: &[u8; 16]) -> Vec<u8> {
    let urn = std::str::from_utf8(&installation::urn(id)).expect("ascii").replace(':', "%3A");
    let mut parameter = installation::NAME.to_vec();
    parameter.push(b'=');
    parameter.extend_from_slice(urn.as_bytes());
    parameter
}

/// The authorization request's target the issuer saw, which it answered
/// with its redirect.
fn authorized(outcome: &skein_world::Outcome<world::Process>) -> Vec<u8> {
    let saw = world::issuer_saw(outcome);
    let at = saw
        .iter()
        .position(|observation| matches!(observation, peer::Observation::Authorize { .. }))
        .expect("the issuer saw the authorization request");
    assert!(
        saw[at..].iter().any(|observation| matches!(observation, peer::Observation::Redirect { .. })),
        "the issuer approved it"
    );
    assert_eq!(world::visitor(outcome).status(), Some(302), "the visit was answered with the redirect");
    let peer::Observation::Authorize { target, .. } = &saw[at] else { unreachable!("found above") };
    target.to_vec()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}

#[test]
fn a_first_sign_in_sends_the_identifier_its_start_drew_and_kept() {
    let outcome = world::start(1, Story::SignIn, None, Installation { drawn: FIRST, kept: None });
    assert_eq!(world::owner(&outcome).installation(), FIRST, "an empty store keeps this start's draw");
    let target = authorized(&outcome);
    assert!(contains(&target, &parameter(&FIRST)), "{}", String::from_utf8_lossy(&target));
}

#[test]
fn a_later_sign_in_sends_the_kept_identifier_not_its_own_draw() {
    let outcome = world::start(2, Story::SignIn, None, Installation { drawn: LATER, kept: Some(FIRST) });
    assert_eq!(world::owner(&outcome).installation(), FIRST, "a later start restores the kept identifier");
    let target = authorized(&outcome);
    assert!(contains(&target, &parameter(&FIRST)), "{}", String::from_utf8_lossy(&target));
    assert!(!contains(&target, &parameter(&LATER)), "this start's own draw is never sent");
}
