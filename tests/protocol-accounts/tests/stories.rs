use jig_domain_accounts as accounts;
use jig_protocol_accounts_world::{
    fixtures::{self, ACCOUNT_CHAT, TOKEN_SECOND},
    owner::{Heard, Trace},
    world::{self, Story},
};
use skein_oauth as oauth;
#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[test]
fn a_wanted_account_is_granted_and_its_value_lands_in_the_table() {
    let outcome = world::run(1, Story::Granted);
    let owner = world::owner(&outcome);
    assert_eq!(owner.child_heard(), [Heard::Granted { account: ACCOUNT_CHAT, generation: 1 }]);
    assert_eq!(owner.table_value(ACCOUNT_CHAT, 1), Some(b"access-first".as_slice()));
    assert!(owner.became_usable());
}
#[test]
fn a_rejected_generation_is_refreshed_and_the_newer_one_lent() {
    let outcome = world::run(1, Story::Refreshed);
    let owner = world::owner(&outcome);
    assert_eq!(
        owner.child_heard(),
        [
            Heard::Granted { account: ACCOUNT_CHAT, generation: 1 },
            Heard::Granted { account: ACCOUNT_CHAT, generation: 2 }
        ]
    );
    assert_eq!(owner.table_value(ACCOUNT_CHAT, 2), Some(b"access-second".as_slice()));
    let trace = owner.trace();
    let kept = trace.iter().position(|entry| *entry == Trace::Kept { account: ACCOUNT_CHAT, generation: 2 });
    let granted =
        trace.iter().position(|entry| *entry == Trace::Heard(Heard::Granted { account: ACCOUNT_CHAT, generation: 2 }));
    assert!(kept.is_some() && kept < granted, "the renewed record was kept before its token was lent");
    let stored = owner.last_kept().expect("the store holds the renewed record");
    let record = oauth::decode_record(stored, &fixtures::limits().client.document).expect("the stored record reads");
    assert_eq!((record.generation, &*record.access_token), (2, TOKEN_SECOND));
}
#[test]
fn a_close_releases_and_ends_with_closed() {
    let outcome = world::run(1, Story::Closed);
    let owner = world::owner(&outcome);
    assert!(owner.trace().contains(&Trace::Requested(accounts::ToComponent::Release { account: ACCOUNT_CHAT })));
    assert_eq!(owner.trace().last(), Some(&Trace::Closed));
    assert!(owner.closed());
}
#[test]
fn a_value_never_reaches_the_childs_facts() {
    let outcome = world::run(1, Story::Granted);
    let owner = world::owner(&outcome);
    let facts = format!("{:?}", owner.facts());
    let trace = format!("{:?}", owner.trace());
    for value in ["access-first", "seed"] {
        assert!(!facts.contains(value));
        assert!(!trace.contains(value));
    }
}
