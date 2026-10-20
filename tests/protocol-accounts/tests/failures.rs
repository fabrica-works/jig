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
    assert_eq!(
        heard(Story::RefusedRefresh),
        [
            Heard::Granted { account: ACCOUNT_CHAT, generation: 1 },
            Heard::Failed { account: ACCOUNT_CHAT, failure: accounts::Failure::SignInAgain },
        ]
    );
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
