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

#[test]
fn a_partys_account_sign_in_is_made_with_an_issued_client_id() {
    let outcome = world::run(1, Story::SignIn);
    let owner = world::owner(&outcome);
    assert!(matches!(
        owner.account_sign_in_ended(),
        [accounts::Ended { end: accounts::End::Made { declined: false, .. }, .. }]
    ));
    let record = oauth::decode_record(
        owner.last_kept().expect("kept account sign-in record"),
        &fixtures::limits().client.document,
    )
    .expect("readable account sign-in record");
    assert_eq!(record.issued_client_id.as_deref(), Some(b"issued-client".as_slice()));
    assert_eq!(record.subject.as_deref(), Some(b"subject".as_slice()));
    assert!(
        matches!(owner.account_answers(), [accounts::Answer::Started {address, ..}] if address.as_ref() == owner.visit().expect("account sign-in visit"))
    );
}
#[test]
fn an_account_visit_address_carries_the_installation_id_and_the_name_hint() {
    let outcome = world::run(1, Story::SignIn);
    let url = world::owner(&outcome).visit().expect("account sign-in visit");
    assert!(url.windows(b"ext_agent_host_id=urn%3Auuid%3A".len()).any(|w| w == b"ext_agent_host_id=urn%3Auuid%3A"));
    assert!(url.windows(b"agent_name_hint=jig-test".len()).any(|w| w == b"agent_name_hint=jig-test"));
}
#[test]
fn an_account_sign_in_on_a_varying_port_is_made() {
    let outcome = world::run(2, Story::AccountSignInVaryingPort);
    let owner = world::owner(&outcome);
    assert!(matches!(owner.account_sign_in_ended(), [accounts::Ended { end: accounts::End::Made { .. }, .. }]));
    assert!(!owner.visit().expect("account sign-in visit").windows(b"31234".len()).any(|w| w == b"31234"));
    assert!(!owner.visit().expect("account sign-in visit").windows(b"%3A0%2F".len()).any(|w| w == b"%3A0%2F"));
}
#[test]
fn an_account_sign_out_is_confirmed_when_the_issuer_revoked_and_the_record_is_erased() {
    let outcome = world::run(3, Story::AccountSignOut { confirmed: true });
    let owner = world::owner(&outcome);
    assert!(owner.account_record_erased() && owner.account_removed());
    assert_eq!(
        owner.account_answers().last(),
        Some(&accounts::Answer::SignedOut { confirmed: true }),
        "issuer statuses: {:?}",
        world::issuer_saw(&outcome)
            .iter()
            .filter_map(|event| match event {
                skein_fake_peers::oauth::Observation::Answered { status, .. } => Some(*status),
                skein_fake_peers::oauth::Observation::Accepted { .. }
                | skein_fake_peers::oauth::Observation::Authorize { .. }
                | skein_fake_peers::oauth::Observation::Post { .. }
                | skein_fake_peers::oauth::Observation::Document { .. }
                | skein_fake_peers::oauth::Observation::Revoke { .. }
                | skein_fake_peers::oauth::Observation::Redirect { .. }
                | skein_fake_peers::oauth::Observation::Closed { .. } => None,
            })
            .collect::<Vec<_>>()
    );
    assert!(
        world::issuer_saw(&outcome)
            .iter()
            .any(|event| matches!(event, skein_fake_peers::oauth::Observation::Revoke { .. }))
    );
}
#[test]
fn an_account_sign_out_the_issuer_did_not_confirm_is_answered_unconfirmed() {
    let outcome = world::run(4, Story::AccountSignOut { confirmed: false });
    let owner = world::owner(&outcome);
    assert!(owner.account_record_erased() && owner.account_removed());
    assert!(matches!(owner.account_answers().last(), Some(accounts::Answer::SignedOut { confirmed: false })));
    assert!(
        world::issuer_saw(&outcome)
            .iter()
            .any(|event| matches!(event, skein_fake_peers::oauth::Observation::Revoke { .. }))
    );
    assert!(
        world::issuer_saw(&outcome)
            .iter()
            .any(|event| matches!(event, skein_fake_peers::oauth::Observation::Answered { status: 400, .. }))
    );
}
#[test]
fn a_missing_scope_ends_the_account_sign_in_refused_naming_it() {
    let outcome = world::run(5, Story::AccountSignInMissingScope);
    let owner = world::owner(&outcome);
    assert!(matches!(
        owner.account_sign_in_ended(),
        [accounts::Ended { end: accounts::End::Failed { failure: accounts::Failure::MissingScope { scope: 1 } }, .. }]
    ));
    assert!(owner.account_record_erased() && owner.account_removed());
}
#[test]
fn a_full_driver_refuses_a_new_account_sign_in_later() {
    let outcome = world::run(6, Story::AccountSignInFull);
    let owner = world::owner(&outcome);
    assert!(matches!(
        owner.account_sign_in_ended(),
        [accounts::Ended { end: accounts::End::Failed { failure: accounts::Failure::Later { after: None } }, .. }]
    ));
    assert_eq!(owner.account_answers(), [accounts::Answer::Refused { refusal: accounts::Refusal::NotStarted }]);
    assert!(owner.visit().is_none());
}
