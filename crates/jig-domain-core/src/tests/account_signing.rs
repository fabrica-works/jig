//! A party's sign-in and sign-out reach the accounts' component as the core's root outputs, and
//! the component's answers reach the accounts child (`domain/accounts.md`, 3.3 and section 5).
use super::*;

const PARTY: accounts::Holder = accounts::Holder::Party(1);

/// A provider offering a sign-in, and room for one account beside the configured one.
fn signing_core() -> (Core, Env<Limits>) {
    let mut limits = limits();
    limits.accounts.accounts = 2;
    let mut config = config(73);
    let offers = accounts::Offers { sign_in: true, key: false, programs: Box::new([]) };
    config.settings.llm_providers =
        accounts::Providers { providers: Box::new([accounts::Provider { number: 3, models: Box::new([]), offers }]) };
    let mut core = Core::new(config, &limits);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    drop(step(&mut core, &env, Event::Account(accounts::Event::Want)));
    (core, env)
}
/// The one ask, if any, the core makes of the accounts' component for one event of the child's.
fn asked(core: &mut Core, env: &Env<Limits>, event: accounts::Event) -> Option<accounts::Request> {
    let Requests::Out(mut out) = step(core, env, Event::Account(event));
    let mut asked = None;
    while let Some(request) = out.pop() {
        match request {
            Request::Now(now) => {
                let Now::Account(request) = *now else { panic!("only the accounts' component is asked") };
                assert!(asked.replace(request).is_none(), "one ask: {asked:?}");
            }
            Request::Decided => {}
            Request::Write(_) | Request::Ask { .. } | Request::Held(_) => {
                panic!("only the accounts' component is asked: {request:?}")
            }
        }
    }
    asked
}

#[test]
fn a_party_s_sign_in_and_sign_out_ask_the_accounts_component() {
    let (mut core, env) = signing_core();
    let started =
        asked(&mut core, &env, accounts::Event::SignIn { request: Token::new(5), holder: PARTY, provider: 3 });
    let Some(accounts::Request::Signing(accounts::Signing::SignIn { source, provider: 3 })) = started else {
        panic!("the component starts the provider's sign-in: {started:?}");
    };
    let signed_in = accounts::Event::SignedIn {
        source,
        subject: Box::from(&b"party"[..]),
        display: Box::from(&b"Party"[..]),
        declined: false,
    };
    assert_eq!(
        asked(&mut core, &env, signed_in),
        Some(accounts::Request::Component(accounts::ToComponent::Want { account: source })),
        "its answer reaches the child, which wants the new source"
    );
    let ended = core.accounts.pop_ended().expect("the sign-in ended");
    assert_eq!((ended.request, ended.source), (Token::new(5), source));
    let accounts::End::Made { account, declined: false } = ended.end else { panic!("made: {ended:?}") };
    assert_eq!(core.accounts.account(account).expect("made").holder(), PARTY);
    let sign_out = accounts::Event::SignOut { request: Token::new(6), holder: PARTY, source };
    assert_eq!(
        asked(&mut core, &env, sign_out),
        Some(accounts::Request::Signing(accounts::Signing::SignOut { source }))
    );
    assert!(core.accounts.source(source).is_none(), "the source is gone before the component confirms");
    assert_eq!(asked(&mut core, &env, accounts::Event::SignedOut { source, confirmed: true }), None);
}
