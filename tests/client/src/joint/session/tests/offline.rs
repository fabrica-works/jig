use super::*;

#[test]
fn an_open_on_the_lost_connection_ends_at_the_native_transport_boundary() {
    let (mut session, now, wall) = session();
    let client = ClientId(0);
    session.bind(client, 1, identity(), now, wall);
    session.poll(now, wall).expect("signed in");
    session.offline_until = Some(now.saturating_add(Duration::from_secs(1)));
    let watch = Token::new(9);
    session.send(client, now, wall, frame(watch)).expect("lost transport accepts the operation");
    let (owner, event) = session.callbacks.pop().expect("native watch terminal");
    assert_eq!(owner, client);
    let Event::Client { event: jig_client::Event::Up { up } } = event else {
        panic!("native transport terminal");
    };
    assert_eq!(up, jig_client::Up::WatchEnded { watch, end: client::WatchEnd::Dropped });
    assert!(session.parked.is_empty());
}

#[test]
fn a_reload_bootstrap_survives_the_engine_restart_before_its_first_poll() {
    let (mut session, now, wall) = session();
    let client = ClientId(0);
    session.bind(client, 1, identity(), now, wall);
    session.poll(now, wall).expect("signed in");
    session.reload(client, now, wall);
    let until = now.saturating_add(Duration::from_secs(1));
    session.restart(jig_core_world::world::config(8455), now, wall, until).expect("restart");
    let callbacks = session.poll(now, wall).expect("new epoch bootstrap");
    assert!(callbacks.iter().any(|(owner, event)| *owner == client
        && matches!(event, Event::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedIn } })));
}
