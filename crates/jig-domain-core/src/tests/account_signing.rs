//! A party's sign-in and sign-out, admitted by people, reach the accounts' component as the core's
//! root outputs, and the component's answers reach the accounts child and the party's outcome
//! (`domain/people.md`, 5.1; `domain/accounts.md`, 3.3 and section 5).
use super::*;

const PARTY: accounts::Holder = accounts::Holder::Party(1);
const SIGN_IN: u64 = 9;

/// A provider offering a sign-in, room for one account beside the configured one, and person 1
/// signed in.
fn signing_core() -> (Core, Env<Limits>) {
    let mut limits = limits();
    limits.accounts.accounts = 2;
    let mut config = config(73);
    let offers = accounts::Offers { sign_in: true, key: false, programs: Box::new([]) };
    config.settings.llm_providers =
        accounts::Providers { providers: Box::new([accounts::Provider { number: 3, models: Box::new([]), offers }]) };
    let mut core = Core::new(config, &limits);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    signed_in(&mut core, &env);
    drop(step(&mut core, &env, Event::Account(accounts::Event::Want)));
    (core, env)
}
fn signed_in(core: &mut Core, env: &Env<Limits>) {
    let identity = people::Identity {
        key: people::IdentityKey { provider: 0, subject: Box::new([1]) },
        login: b"party".as_slice().into(),
        name: b"A Party".as_slice().into(),
    };
    let env = Env { now: env.now, wall: env.wall, limits: env.limits.people };
    let mut out = Queue::with_capacity(people::max_out(&env.limits));
    let person = people::Stored::Person { number: 1, identity: identity.clone() };
    people::step(&mut core.people, &env, people::Event::Restore { record: person }, &mut out);
    people::step(&mut core.people, &env, people::Event::Restored, &mut out);
    let reply_to = ReplyTo::new(Token::new(100));
    let kind = people::Kind::Person;
    people::step(
        &mut core.people,
        &env,
        people::Event::SignedIn { reply_to, person: 1, sign_in: SIGN_IN, identity, kind },
        &mut out,
    );
    assert_eq!(core.people.person(SIGN_IN, env.now, env.wall), Some(1));
}
fn ask(key: u8, ask: people::Ask) -> Event {
    let reply_to = ReplyTo::new(Token::new(u64::from(key)));
    Event::People(people::Event::Ask { reply_to, sign_in: SIGN_IN, key: [key; 16], ask })
}
/// The one ask, if any, the core makes of the accounts' component for one event, and the party's
/// outcome, if one is answered.
fn stepped(core: &mut Core, env: &Env<Limits>, event: Event) -> (Option<accounts::Request>, Option<people::Outcome>) {
    let Requests::Out(mut out) = step(core, env, event);
    let (mut asked, mut answered) = (None, None);
    while let Some(request) = out.pop() {
        match request {
            Request::Now(now) => {
                let Now::Account(request) = *now else { panic!("only the accounts' component is asked") };
                assert!(asked.replace(request).is_none(), "one ask: {asked:?}");
            }
            Request::Held(held) => {
                let Held::PeopleReply { reply: people::Reply::Outcome(outcome), .. } = *held else {
                    panic!("only the party's outcome is held: {held:?}");
                };
                assert!(answered.replace(outcome).is_none(), "one outcome: {answered:?}");
            }
            Request::Write(_) | Request::Decided => {}
            Request::Ask { .. } => panic!("only the accounts' component is asked: {request:?}"),
        }
    }
    (asked, answered)
}

#[test]
fn a_party_s_sign_in_and_sign_out_ask_the_accounts_component() {
    let (mut core, env) = signing_core();
    let sign_in = people::Ask::AccountSignIn { holder: people::AccountHolder::Own, provider: 3 };
    let (started, answered) = stepped(&mut core, &env, ask(5, sign_in));
    let Some(accounts::Request::Signing(accounts::Signing::SignIn { source, provider: 3 })) = started else {
        panic!("the component starts the provider's sign-in: {started:?}");
    };
    assert_eq!(answered, Some(people::Outcome::AccountSignInStarted { source }), "the party is answered at its start");
    let signed_in = accounts::Event::SignedIn {
        source,
        subject: Box::from(&b"party"[..]),
        display: Box::from(&b"Party"[..]),
        declined: false,
    };
    assert_eq!(
        stepped(&mut core, &env, Event::Account(signed_in)),
        (Some(accounts::Request::Component(accounts::ToComponent::Want { account: source })), None),
        "its answer reaches the child, which wants the new source"
    );
    let ended = core.accounts.pop_ended().expect("the sign-in ended");
    assert_eq!(ended.source, source);
    let accounts::End::Made { account, declined: false } = ended.end else { panic!("made: {ended:?}") };
    assert_eq!(core.accounts.account(account).expect("made").holder(), PARTY);
    let sign_out = people::Ask::AccountSignOut { holder: people::AccountHolder::Own, source };
    assert_eq!(
        stepped(&mut core, &env, ask(6, sign_out)),
        (Some(accounts::Request::Signing(accounts::Signing::SignOut { source })), None),
        "the party is answered once the component confirms"
    );
    assert!(core.accounts.source(source).is_none(), "the source is gone before the component confirms");
    let confirmed = Event::Account(accounts::Event::SignedOut { source, confirmed: true });
    assert_eq!(
        stepped(&mut core, &env, confirmed),
        (None, Some(people::Outcome::AccountSignedOut { confirmed: true }))
    );
}
