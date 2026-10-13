use super::super::*;
use super::fixtures;
#[test]
fn stopping_keeps_the_printed_link_live_and_unspent() {
    let env = fixtures::env();
    let mut out = Queue::with_capacity(8);
    let mut component = Component::new(fixtures::linked_settings(), &env, &mut out).expect("start");
    let Some(Request::Io(skein_io::Request::Listen { .. })) = out.pop() else { panic!("listen") };
    let Some(Request::Link { .. }) = out.pop() else { panic!("link") };
    let secret = component.link.as_ref().expect("live link").secret.clone();
    let until = component.link.as_ref().expect("live link").until;
    let socket = Token::new(99);
    component.down(
        &env,
        Event::Io(skein_io::Event::Accepted { owner: Token::new(0), socket, peer: fixtures::settings().listener }),
        &mut out,
    );
    let Some(Request::Io(skein_io::Request::Bind { owner, .. })) = out.pop() else { panic!("bind") };
    let record =
        w::LinkSignIn::new(&env.limits.wire, w::LinkSignInParts { secret: secret.expose().into() }).expect("link");
    let mut body = skein_lib::Writer::new(usize::try_from(record.measure()).expect("size"));
    record.encode(&mut body).expect("body");
    let connection = component.connections.get_mut(&owner).expect("connection");
    connection.route = Route::Link;
    connection.exchange = Some(Token::new(2));
    connection.body.append(&body.finish()).expect("body fits");
    component.down(&env, Event::Stop, &mut out);
    let mut connection = component.connections.remove(&owner).expect("connection");
    component.body(&env, &mut connection, &mut out);
    assert_eq!(
        w::SignInAnswer::decode(&env.limits.wire, &mut skein_lib::Reader::new(&connection.response)),
        Ok(w::SignInAnswer::NotReady)
    );
    let live = component.link.as_ref().expect("still live");
    assert_eq!(live.secret, secret);
    assert_eq!(live.until, until);
    assert!(out.is_empty(), "no core event, keep, or replacement link");
}
