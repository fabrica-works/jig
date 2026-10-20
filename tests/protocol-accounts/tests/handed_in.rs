use jig_domain_accounts as accounts;
use jig_protocol_accounts_world::{
    fixtures::ACCOUNT_HANDED,
    owner::{Heard, Trace},
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
fn a_handed_in_record_is_kept_once_and_not_kept_again_at_the_next_start() {
    let kept_only = |trace: &[Trace]| -> Vec<Trace> {
        trace.iter().filter(|entry| matches!(entry, Trace::Kept { .. })).copied().collect()
    };
    let first = world::run(1, expiring_story());
    let owner = world::owner(&first);
    assert_eq!(kept_only(owner.trace()), [Trace::Kept { account: ACCOUNT_HANDED, generation: 1 }]);
    let held = owner.last_kept().expect("the store holds the handed-in record").to_vec();
    // The next start reads the record the first left in the store.
    let second = world::run_from(1, expiring_story(), Some(&held));
    let again = world::owner(&second);
    assert!(again.trace().contains(&Trace::Loaded { account: ACCOUNT_HANDED, generation: 1 }));
    assert_eq!(kept_only(again.trace()), [], "a record the store holds is not kept again");
    assert_eq!(again.child_heard(), owner.child_heard());
    assert_eq!(again.lent(), owner.lent(), "its generation names the same value at both starts");
}
