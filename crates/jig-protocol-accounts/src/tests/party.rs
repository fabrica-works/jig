use crate::{
    party,
    translate::{self, Down, Up},
};
use jig_domain_accounts as child;
use crate::party::Registration;
use skein_lib::{List, Time, bytes};
use skein_oauth_accounts as driver;

pub fn registration() -> Registration {
    Registration {
        issuer: bytes::copy_of(b"http://127.0.0.1:31000"),
        client: bytes::copy_of(b"client"),
        scopes: Box::new([bytes::copy_of(b"openid"), bytes::copy_of(b"read")]),
        name_hint: Some(bytes::copy_of(b"jig")),
        authorization: bytes::copy_of(b"http://127.0.0.1:31000/authorize"),
        token: bytes::copy_of(b"http://127.0.0.1:31000/token"),
        revocation: Some(bytes::copy_of(b"http://127.0.0.1:31000/revoke")),
        keys: bytes::copy_of(b"http://127.0.0.1:31000/keys"),
        endpoint_transport: driver::Transport::Plaintext,
        must_grant: Box::new([bytes::copy_of(b"openid"), bytes::copy_of(b"read")]),
        resource: Some(bytes::copy_of(b"api")),
        redirect_path: bytes::copy_of(b"callback"),
        redirect_port: None,
    }
}
fn account(port: Option<u16>) -> driver::Account {
    let mut r = registration();
    r.redirect_port = port;
    party::party_account(
        &r,
        driver::Endpoint { address: "127.0.0.1:31000".parse().unwrap(), transport: driver::Transport::Plaintext },
        List::with_capacity(0),
        &[5; 16],
    )
}
#[test]
fn party_account_sign_in_carries_the_installation_id_and_the_name_hint() {
    let driver::Account::SignIn { registration: r, keeper, shared, origins, .. } = account(None) else {
        panic!("account sign-in")
    };
    assert_eq!(&*r.authorization_url, b"http://127.0.0.1:31000/authorize");
    assert_eq!(&*r.token_endpoint, b"http://127.0.0.1:31000/token");
    assert_eq!(&*r.redirect_uri, b"http://127.0.0.1:0/callback");
    assert_eq!(&*r.scope, b"openid read");
    assert!(r.client_secret.is_none() && !r.pkce_for_confidential && r.metadata_claim.is_none());
    assert!(matches!(keeper, driver::Keeper::Owner { kept: None }) && !shared && origins.is_empty());
    let terms = r.terms.unwrap();
    assert_eq!(&*terms.parameters[0].name, crate::installation::NAME);
    assert_eq!(terms.parameters[0].value, crate::installation::urn(&[5; 16]));
    assert_eq!(
        (&*terms.parameters[1].name, &*terms.parameters[1].value),
        (b"agent_name_hint".as_slice(), b"jig".as_slice())
    );
    assert!(terms.parameters.iter().all(|p| p.sent == skein_oauth::Sent::EverySignIn));
    assert_eq!(terms.resource.as_deref(), Some(b"api".as_slice()));
    assert_eq!(terms.revocation_endpoint.as_deref(), Some(b"http://127.0.0.1:31000/revoke".as_slice()));
}
#[test]
fn party_account_sign_in_requires_the_issued_client_id_and_the_must_grant_scopes() {
    let driver::Account::SignIn { registration: r, .. } = account(None) else { panic!("account sign-in") };
    let terms = r.terms.unwrap();
    assert!(terms.issued_client_id && terms.varying_port);
    assert_eq!(terms.required_scopes.as_deref(), Some(b"openid read".as_slice()));
    assert_eq!(
        terms.openid.unwrap(),
        skein_oauth::OpenId {
            issuer: bytes::copy_of(b"http://127.0.0.1:31000"),
            key_set: Some(bytes::copy_of(b"http://127.0.0.1:31000/keys"))
        }
    );
}
#[test]
fn a_fixed_account_sign_in_port_is_not_a_varying_port() {
    let driver::Account::SignIn { registration: r, .. } = account(Some(31234)) else { panic!("account sign-in") };
    assert!(!r.terms.unwrap().varying_port);
    assert_eq!(&*r.redirect_uri, b"http://127.0.0.1:31234/callback");
}
#[test]
fn an_account_visit_is_the_childs_visited_event() {
    let url = b"http://issuer.test/visit";
    assert_eq!(
        translate::up(driver::Event::Visit { account: 55, url: bytes::copy_of(url) }, Time::ZERO, b""),
        Up::Visited { source: 55, address: bytes::copy_of(url) }
    );
}
#[test]
fn an_account_visit_longer_than_the_address_bound_fails_the_account_sign_in_misconfigured() {
    assert_eq!(
        translate::up(
            driver::Event::Visit { account: 55, url: bytes::zeroed(child::ADDRESS as usize + 1) },
            Time::ZERO,
            b""
        ),
        Up::SignInFailed { source: 55, failure: child::Failure::Misconfigured }
    );
}
#[test]
fn an_account_signed_in_without_a_subject_fails_misconfigured() {
    assert_eq!(
        translate::up(
            driver::Event::SignedIn { account: 55, generation: 1, missing: Box::default(), subject: None },
            Time::ZERO,
            b""
        ),
        Up::SignInFailed { source: 55, failure: child::Failure::Misconfigured }
    );
}
#[test]
fn an_account_signed_in_with_a_missing_scope_fails_naming_its_index() {
    assert_eq!(
        translate::up(
            driver::Event::SignedIn {
                account: 55,
                generation: 1,
                missing: bytes::copy_of(b"write read"),
                subject: Some(bytes::copy_of(b"subject"))
            },
            Time::ZERO,
            b"openid read write"
        ),
        Up::SignInFailed { source: 55, failure: child::Failure::MissingScope { scope: 1 } }
    );
}
#[test]
fn an_account_sign_out_is_down_and_its_revoked_is_up_as_confirmed() {
    assert_eq!(translate::down_signing(child::Signing::SignOut { source: 55 }), Down::SignOut { source: 55 });
    assert_eq!(translate::down_signing(child::Signing::Replaced { source: 55, by: 56 }), Down::SignOut { source: 55 });
    for confirmed in [false, true] {
        assert_eq!(
            translate::up(driver::Event::SignedOut { account: 55, revoked: confirmed }, Time::ZERO, b""),
            Up::SignedOut { source: 55, confirmed }
        );
    }
}
#[test]
fn a_full_account_add_fails_the_account_sign_in_later() {
    assert_eq!(
        translate::up(
            driver::Event::Refused { account: 55, asked: driver::Asked::Add, why: driver::Refusal::Full { bound: 2 } },
            Time::ZERO,
            b""
        ),
        Up::SignInFailed { source: 55, failure: child::Failure::Later { after: None } }
    );
}
#[test]
fn a_removed_account_frees_its_number() {
    let mut records = crate::records::Records::new(1);
    records.admit(55).unwrap();
    assert!(records.admit(56).is_err());
    assert_eq!(translate::up(driver::Event::Removed { account: 55 }, Time::ZERO, b""), Up::Removed { source: 55 });
    records.remove(55);
    records.admit(56).unwrap();
}
