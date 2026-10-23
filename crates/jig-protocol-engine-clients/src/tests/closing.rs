use super::super::*;
use super::fixtures;
#[test]
fn stopping_keeps_the_printed_link_live_and_unspent() {
    let env = fixtures::env();
    let mut out = Queue::with_capacity(8);
    let mut component = Component::new(fixtures::linked_settings(), &env, &mut out).expect("start");
    let Some(Request::Link { .. }) = out.pop() else { panic!("link") };
    let secret = component.link.as_ref().expect("live link").secret.clone();
    let until = component.link.as_ref().expect("live link").until;
    let owner = Token::new(99);
    let record =
        w::LinkSignIn::new(&env.limits.wire, w::LinkSignInParts { secret: secret.expose().into() }).expect("link");
    let mut body = skein_lib::Writer::new(usize::try_from(record.measure()).expect("size"));
    record.encode(&mut body).expect("body");
    component.down(&env, Event::Stop, &mut out);
    component.down(
        &env,
        Event::Transport(crate::FromTransport::Exchange {
            exchange: owner,
            credential: None,
            message: crate::Message::SignInLink(body.finish()),
        }),
        &mut out,
    );
    let connection = component.connections.get(&owner).expect("exchange");
    let Some(crate::Reply::Record(answer)) = &connection.response else { panic!("answer") };
    assert_eq!(
        w::SignInAnswer::decode(&env.limits.wire, &mut skein_lib::Reader::new(answer)),
        Ok(w::SignInAnswer::NotReady)
    );
    let live = component.link.as_ref().expect("still live");
    assert_eq!(live.secret, secret);
    assert_eq!(live.until, until);
    assert!(out.is_empty(), "no core event, keep, or replacement link");
}

#[test]
fn a_mint_past_the_mints_in_flight_is_refused_busy() {
    const FIRST: Token = Token::new(100);
    const NEXT: Token = Token::new(101);
    let mut env = fixtures::env();
    env.limits.mints = 1;
    env.limits.providers = 1;
    let mut settings = fixtures::linked_settings();
    let identity = settings.link.clone().expect("identity");
    settings.providers = skein_lib::List::with_capacity(1);
    settings.providers.push(identity.key.provider).expect("provider");
    let mut out = Queue::with_capacity(8);
    let mut component = Component::new(settings, &env, &mut out).expect("start");
    out = Queue::with_capacity(8);
    component.down(&env, Event::Mint { exchange: FIRST, identity: identity.clone() }, &mut out);
    let Some(Request::Core(_)) = out.pop() else { panic!("first mint admitted") };
    component.down(&env, Event::Mint { exchange: NEXT, identity }, &mut out);
    let Some(Request::MintRefused { exchange, why }) = out.pop() else { panic!("busy mint") };
    assert_eq!((exchange, why), (NEXT, crate::MintRefusal::Busy));
    assert!(out.is_empty());
}

#[test]
fn close_and_abort_drop_a_links_reservation_even_before_its_core_answer() {
    for abort in [false, true] {
        let env = fixtures::env();
        let mut out = Queue::with_capacity(8);
        let mut component = Component::new(fixtures::linked_settings(), &env, &mut out).unwrap();
        out.pop().expect("printed link");
        let link = component.link.take().expect("open link");
        let exchange = Token::new(1);
        component.spending = Some((exchange, link));
        component.pending.exchanges.insert(exchange, Waiting::Core { held: None }).unwrap();
        component.down(&env, if abort { Event::Abort } else { Event::Close }, &mut out);
        assert!(component.link.is_none());
        assert!(component.spending.is_none());
        component.down(
            &env,
            Event::Reply { to: ReplyTo::new(exchange), reply: people::Reply::Again(people::Again::Busy) },
            &mut out,
        );
        assert!(component.link.is_none(), "late retry answer cannot reopen a closing link");
        assert!(component.spending.is_none());
        assert!(out.is_empty(), "no new link, credential or core request");
    }
}
