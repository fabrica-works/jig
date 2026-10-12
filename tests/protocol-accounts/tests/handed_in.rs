use jig_domain_accounts as accounts;
use jig_protocol_accounts_world::{
    fixtures::ACCOUNT_HANDED,
    owner::Heard,
    world::{self, Story},
};
use skein_lib::Duration;
#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

fn expiring_story() -> Story {
    Story::HandedIn { lapses_in: Duration::from_secs(60) }
}
#[test]
fn a_handed_in_account_expires_then_fails_as_sign_in_again() {
    let outcome = world::run(1, expiring_story());
    assert_eq!(
        world::owner(&outcome).child_heard(),
        [
            Heard::Granted { account: ACCOUNT_HANDED, generation: 1 },
            Heard::Expiring { account: ACCOUNT_HANDED, generation: 1 },
            Heard::Failed { account: ACCOUNT_HANDED, failure: accounts::Failure::SignInAgain },
        ]
    );
}
#[test]
fn a_rejected_handed_in_token_fails_as_sign_in_again() {
    let outcome = world::run(1, Story::HandedInRejected);
    assert_eq!(
        world::owner(&outcome).child_heard(),
        [
            Heard::Granted { account: ACCOUNT_HANDED, generation: 1 },
            Heard::Failed { account: ACCOUNT_HANDED, failure: accounts::Failure::SignInAgain },
        ]
    );
}
#[test]
fn the_driver_never_writes_a_handed_in_accounts_file() {
    let outcome = world::run(1, expiring_story());
    assert_eq!(outcome.machine.writes, 0);
}
