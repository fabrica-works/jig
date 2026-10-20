use jig_domain_accounts as accounts;
use jig_protocol_accounts_world::{
    fixtures::ACCOUNT_CHAT,
    owner::{Heard, Trace},
    world::{self, Story},
};
use skein_lib::Duration;
#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

fn heard(story: Story) -> Vec<Heard> {
    let outcome = world::run(1, story);
    world::owner(&outcome).child_heard().to_vec()
}
#[test]
fn a_rate_limited_refresh_fails_later_with_the_issuers_delay() {
    assert_eq!(
        heard(Story::RateLimitedRefresh),
        [
            Heard::Granted { account: ACCOUNT_CHAT, generation: 1 },
            Heard::Failed {
                account: ACCOUNT_CHAT,
                failure: accounts::Failure::Later { after: Some(Duration::from_secs(7)) }
            },
        ]
    );
}
#[test]
fn a_refused_refresh_token_fails_as_sign_in_again() {
    let mut outcome = world::run(1, Story::RefusedRefresh);
    let owner = world::owner(&outcome);
    assert_eq!(
        owner.child_heard(),
        [
            Heard::Granted { account: ACCOUNT_CHAT, generation: 1 },
            Heard::Failed { account: ACCOUNT_CHAT, failure: accounts::Failure::SignInAgain },
        ]
    );
    assert_revoked(owner);
    // Snapshot the actual store through Get, rather than the historical last_kept capture.
    let snapshot = world::owner_mut(&mut outcome).read_record();
    assert!(snapshot.is_none());
    let restarted = world::run_from(2, Story::NoRecord, snapshot.as_deref());
    assert_eq!(
        world::owner(&restarted).child_heard(),
        [Heard::Failed { account: ACCOUNT_CHAT, failure: accounts::Failure::SignInAgain },]
    );
    assert!(world::issuer_saw(&restarted).is_empty(), "a restart cannot resend the dead refresh token");
}
#[test]
fn a_rejected_client_fails_as_misconfigured() {
    assert_eq!(
        heard(Story::RejectedClient),
        [
            Heard::Granted { account: ACCOUNT_CHAT, generation: 1 },
            Heard::Failed { account: ACCOUNT_CHAT, failure: accounts::Failure::Misconfigured },
        ]
    );
}
#[test]
fn a_damaged_stored_record_fails_its_source_as_unloaded() {
    assert_eq!(
        heard(Story::DamagedRecord),
        [Heard::Failed {
            account: ACCOUNT_CHAT,
            failure: accounts::Failure::Unloaded { at: accounts::Place::File, why: accounts::Unread::Unreadable },
        }]
    );
}

#[test]
fn a_close_during_an_exchange_drains_it() {
    let outcome = world::run(1, Story::ClosedExchange);
    let owner = world::owner(&outcome);
    assert_eq!(owner.child_heard(), [Heard::Granted { account: ACCOUNT_CHAT, generation: 1 }]);
    assert!(owner.closed());
    let trace = owner.trace();
    let kept = trace.iter().position(|entry| *entry == Trace::Kept { account: ACCOUNT_CHAT, generation: 2 });
    let closed = trace.iter().position(|entry| *entry == Trace::Closed);
    assert!(kept.is_some() && kept < closed, "the exchange's renewed record was kept before Closed");
}

fn assert_revoked(owner: &jig_protocol_accounts_world::owner::Owner) {
    assert!(owner.account_record_erased());
    assert!(owner.table_value(ACCOUNT_CHAT, 1).is_none());
    assert!(!owner.account_removed());
    assert!(owner.account_answers().is_empty(), "revocation is never a sign-out answer");
    let fact = owner
        .facts()
        .iter()
        .rev()
        .find(|fact| fact.source == ACCOUNT_CHAT && fact.state == accounts::State::NeedsSignIn)
        .expect("the source remains visible as needs sign-in before teardown");
    assert!(!fact.usable);
    assert!(fact.attention);
    assert_eq!(fact.generation, None);
}
#[test]
fn a_refused_grant_erases_its_record_without_signing_out() {
    let mut outcome = world::run(1, Story::RefusedGrant);
    assert_revoked(world::owner(&outcome));
    assert!(world::owner_mut(&mut outcome).read_record().is_none());
}
