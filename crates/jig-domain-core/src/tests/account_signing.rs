//! A party's sign-in and sign-out, admitted by people, reach the accounts' component as the core's
//! root outputs, and the component's answers reach the accounts child and the party's outcome
//! (`domain/people.md`, 5.1; `domain/accounts.md`, 3.3 and section 5).
use super::*;

const PARTY: accounts::Holder = accounts::Holder::Party(1);
const SIGN_IN: u64 = 9;

/// A provider offering a sign-in, room for two runtime accounts, and person 1
/// signed in.
fn signing_core() -> (Core, Env<Limits>) {
    let mut limits = limits();
    limits.accounts.accounts = 2;
    let mut config = config(73);
    config.worker_names = Box::new([]);
    let offers = accounts::Offers { sign_in: true, key: true, programs: Box::new([]) };
    config.settings.llm_providers = accounts::Providers {
        providers: Box::new([accounts::Provider {
            name: b"Fake".as_slice().into(),
            signed_in_plan: true,
            manage_usage: Some(b"https://usage.example/".as_slice().into()),
            number: 3,
            models: Box::new([]),
            offers,
        }]),
    };
    let mut core = Core::new(config, &limits).expect("fixture core configuration");
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
    Event::People(people::Event::Ask { models: Box::new([]), reply_to, sign_in: SIGN_IN, key: [key; 16], ask })
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
fn a_party_s_account_sign_in_is_answered_at_its_visit_and_sign_out_at_its_end() {
    let (mut core, env) = signing_core();
    let sign_in = people::Ask::AccountSignIn { holder: people::AccountHolder::Own, provider: 3 };
    let (started, answered) = stepped(&mut core, &env, ask(5, sign_in));
    let Some(accounts::Request::Signing(accounts::Signing::SignIn { source, provider: 3 })) = started else {
        panic!("the component starts the provider's sign-in: {started:?}");
    };
    assert_eq!(answered, None, "the account sign-in waits for its visit");
    let address: Box<[u8]> = b"https://issuer.test/visit".as_slice().into();
    let visited = accounts::Event::Visited { source, address: address.clone() };
    assert_eq!(
        stepped(&mut core, &env, Event::Account(visited)),
        (None, Some(people::Outcome::AccountSignInStarted { source, address })),
        "the account sign-in is answered at its visit"
    );
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
        (None, Some(people::Outcome::AccountSignedOut { confirmed: true, failed_hosts: Box::new([]) }))
    );
}

#[test]
fn a_value_is_in_no_committed_row() {
    const VALUE: &[u8] = b"raw-llm-key-value-never-committed";
    let (mut core, env) = signing_core();
    // This fixture's provider now offers both sign-in and LLM keys.
    let ask = people::Ask::AccountChange {
        holder: people::AccountHolder::Own,
        change: people::AccountChange::AddKey { provider: 3, join: None, digest: [7; 32] },
    };
    let Requests::Out(mut out) = step(
        &mut core,
        &env,
        Event::Key {
            reply_to: ReplyTo::new(Token::new(90)),
            sign_in: SIGN_IN,
            key: [90; 16],
            ask: Box::new(ask.clone()),
            carried: Carried { number: 1_u64 << 63_u32, digest: [7; 32] },
        },
    );
    let mut source = None;
    while let Some(request) = out.pop() {
        if let Request::Now(now) = request
            && let Now::Account(accounts::Request::Signing(accounts::Signing::Keep { source: number, carried, .. })) =
                *now
        {
            assert_eq!(carried, 1_u64 << 63_u32);
            source = Some(number);
        }
    }
    assert_eq!(core.keys.len(), 1);
    let source = source.expect("the component keeps the LLM key");
    // A duplicate in flight releases only its own carried value and joins the
    // first request's answer; the original value remains owned by the keep.
    let Requests::Out(mut duplicate) = step(
        &mut core,
        &env,
        Event::Key {
            reply_to: ReplyTo::new(Token::new(91)),
            sign_in: SIGN_IN,
            key: [90; 16],
            ask: Box::new(ask.clone()),
            carried: Carried { number: (1_u64 << 63_u32) + 1, digest: [7; 32] },
        },
    );
    let mut unused = 0_u32;
    while let Some(request) = duplicate.pop() {
        if let Request::Held(held) = request {
            let Held::CarriedUnused { number } = *held else { panic!("only the duplicate LLM key value is unused") };
            assert_eq!(number, (1_u64 << 63_u32) + 1);
            unused += 1;
        }
    }
    assert_eq!(unused, 1);
    assert_eq!(core.keys.len(), 1);
    let Requests::Out(mut out) = step(
        &mut core,
        &env,
        Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
            account: source,
            generation: 1,
            valid: accounts::KEY_VALID,
            form: accounts::Form::Credential(accounts::Scheme::Key),
        })),
    );
    let mut saved = 0_u32;
    while let Some(request) = out.pop() {
        if let Request::Write(Write::Save(record)) = request {
            match record {
                Record::People(people::Stored::Answer { key, ask: retained, outcome, .. }) => {
                    saved += 1;
                    assert_eq!(*retained, ask);
                    assert_eq!(outcome, people::Outcome::AccountKeyKept { source });
                    // The answer row's byte fields are the request key and the LLM key's
                    // digest. The raw value never crosses the core's input boundary.
                    let people::Ask::AccountChange { change: people::AccountChange::AddKey { digest, .. }, .. } =
                        *retained
                    else {
                        panic!("the retained LLM key ask")
                    };
                    for bytes in [key.key.as_slice(), digest.as_slice()] {
                        for window in bytes.windows(VALUE.len()) {
                            assert_ne!(window, VALUE);
                        }
                    }
                }
                Record::Accounts(accounts::Stored::Account { subject, display, .. }) => {
                    for bytes in [subject.as_ref(), display.as_ref()] {
                        for window in bytes.windows(VALUE.len()) {
                            assert_ne!(window, VALUE);
                        }
                    }
                }
                Record::Accounts(accounts::Stored::Source { name, .. } | accounts::Stored::Issued { name, .. }) => {
                    for window in name.windows(VALUE.len()) {
                        assert_ne!(window, VALUE);
                    }
                }
                Record::Accounts(accounts::Stored::Next { .. }) | Record::Core(CoreRecord::Deployment(_)) => {}
                other @ (Record::Core(_) | Record::Tasks(_) | Record::People(_) | Record::Notes(_)) => {
                    panic!("unexpected saved row: {other:?}")
                }
            }
        }
    }
    assert_eq!(saved, 1);
    assert!(core.keys.is_empty() && core.key_input.is_none());
}

#[test]
fn program_sign_out_routes_the_offline_member_result() {
    let (mut core, env) = signing_core();
    let mut providers = core.settings.llm_providers.clone();
    providers.providers[0].offers.programs = Box::new([b"claude_code".as_slice().into()]);
    core.accounts = accounts::Domain::new(&env.limits.accounts, providers);
    assert!(core.fleet.configure_workers(&env.limits.fleet, Box::new([b"offline".as_slice().into()])));
    for record in [
        accounts::Stored::Issued { number: 21, holder: PARTY, name: b"personal".as_slice().into(), signed_out: false },
        accounts::Stored::Account {
            number: 20,
            provider: 3,
            holder: PARTY,
            place: 0,
            uses: None,
            subject: Box::new([]),
            display: Box::new([]),
        },
        accounts::Stored::Source {
            account: 20,
            number: 21,
            place: 0,
            kind: accounts::Kind::Program,
            name: b"personal".as_slice().into(),
            held: None,
        },
    ] {
        drop(step(&mut core, &env, Event::Account(accounts::Event::Restore { record })));
    }
    drop(step(&mut core, &env, Event::Account(accounts::Event::Want)));
    assert_eq!(
        stepped(
            &mut core,
            &env,
            ask(77, people::Ask::AccountSignOut { holder: people::AccountHolder::Own, source: 21 })
        ),
        (
            None,
            Some(people::Outcome::AccountSignedOut {
                confirmed: false,
                failed_hosts: Box::new([people::AccountHostFailure {
                    name: b"offline".as_slice().into(),
                    why: people::AccountHostWhy::NotInContact
                }])
            })
        )
    );
}
