//! Outbound protocol routing for the actual Front (testing.md, 5; contract.md, 3).
//! The source owns admission and durability; these routes retain shell names and
//! the actual authenticated session, without interpreting a business result.
use super::{
    engine::Driver,
    transport::{KeyedOwner, Reply, Watched},
    wire,
};
use crate::harness::ClientId;
use jig_client_test_domain::{Ask, JigOut, Request, first};
use jig_client_view::Limits;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_people::ApplicationScope;
use jig_test_connector::ApplicationRequest;
use jig_test_domain as root;
use sha2::{Digest, Sha256};
use skein_lib::{Time, Wall};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invalid {
    Session,
    Ask(wire::asks::Invalid),
    Read(wire::reads::Invalid),
    Watch(wire::watches::Invalid),
    Family(jig_client_test_domain::family::wire::Invalid),
    Continuation,
    Shell,
}

/// # Errors
/// Leaves an unauthenticated request with its caller, or rejects an unsupported
/// whole carrier, mismatched scope/position or a duty that belongs to the shell.
pub fn send(
    driver: &mut Driver,
    client: ClientId,
    now: Time,
    wall: Wall,
    request: &Request,
    limits: &Limits,
) -> Result<(), Invalid> {
    let sign_in = driver.transport.sign_in(client);
    let event = match request {
        Request::Send { attempt, key, ask } => {
            keyed(driver, client, sign_in.ok_or(Invalid::Session)?, *attempt, *key, ask)?
        }
        Request::Jig(value) => match value {
            JigOut::Read { read, what } => {
                let sign_in = sign_in.ok_or(Invalid::Session)?;
                let owner = wire::reads::read(what, limits.text).map_err(Invalid::Read)?;
                let reply = driver.transport.reply_to(Reply::Read { client, read: *read, what: what.clone() });
                root::Event::Core(core::Event::Read { reply, sign_in, read: owner })
            }
            JigOut::Open { watch, subject } => {
                let sign_in = sign_in.ok_or(Invalid::Session)?;
                let watcher = driver.transport.open(client, *watch, Watched::Jig(subject.clone()));
                match wire::watches::watch(watcher, sign_in, subject, limits.rows) {
                    Ok(event) => root::Event::Core(event),
                    Err(why) => {
                        driver.transport.end_watch(watcher);
                        return Err(Invalid::Watch(why));
                    }
                }
            }
            JigOut::Continue { watch, subject } => {
                let event =
                    driver.transport.continued(client, *watch, subject.clone()).map_err(|_| Invalid::Continuation)?;
                let Some(event) = event else { return Ok(()) };
                root::Event::Core(event)
            }
            JigOut::Close { watch } => {
                let Some(watcher) = driver.transport.watch_token(client, *watch) else { return Ok(()) };
                let terminals = driver.transport.close_watch(watcher);
                driver.send(now, wall, root::Event::Core(core::Event::Unwatch { watcher }));
                for event in terminals {
                    driver.send(now, wall, root::Event::Core(event));
                }
                return Ok(());
            }
            JigOut::Answered { .. } | JigOut::Unknown { .. } => return Err(Invalid::Shell),
        },
        Request::First(value) => match value {
            first::Out::Read { read, subject } => {
                let sign_in = sign_in.ok_or(Invalid::Session)?;
                let (scope, owner) =
                    jig_client_test_domain::family::wire::connector_subject(*subject).map_err(Invalid::Family)?;
                let reply =
                    driver.transport.reply_to(Reply::ApplicationRead { client, read: *read, subject: *subject });
                root::Event::ReadApplication {
                    reply,
                    sign_in,
                    scope: application_scope(scope),
                    number: 1,
                    subject: owner,
                    window: core::documents::Window { rows: limits.rows, after: None },
                }
            }
            first::Out::Open { watch, subject } => {
                let sign_in = sign_in.ok_or(Invalid::Session)?;
                let (scope, owner) =
                    jig_client_test_domain::family::wire::connector_subject(*subject).map_err(Invalid::Family)?;
                let scope = application_scope(scope);
                let watcher = driver.transport.open(client, *watch, Watched::Application { subject: *subject, scope });
                root::Event::WatchApplication {
                    watcher,
                    sign_in,
                    scope,
                    number: 1,
                    subject: owner,
                    window: core::documents::Window { rows: limits.rows, after: None },
                }
            }
            first::Out::Close { watch } => {
                let Some(watcher) = driver.transport.application_watch_token(client, *watch) else { return Ok(()) };
                let terminals = driver.transport.close_watch(watcher);
                driver.send(now, wall, root::Event::Core(core::Event::Unwatch { watcher }));
                for event in terminals {
                    driver.send(now, wall, root::Event::Core(event));
                }
                return Ok(());
            }
            first::Out::Fact { .. } => return Err(Invalid::Shell),
        },
        Request::Save { .. }
        | Request::Erase { .. }
        | Request::Show { .. }
        | Request::SignOut
        | Request::SignIn { .. }
        | Request::Fact { .. } => return Err(Invalid::Shell),
    };
    driver.send(now, wall, event);
    Ok(())
}

fn keyed(
    driver: &mut Driver,
    client: ClientId,
    sign_in: u64,
    attempt: skein_lib::Token,
    key: [u8; 16],
    ask: &Ask,
) -> Result<root::Event, Invalid> {
    Ok(match ask {
        Ask::Jig(value) => {
            let ask = wire::asks::ask(value).map_err(Invalid::Ask)?;
            let reply_to = driver.transport.reply_to(Reply::Keyed { client, attempt, key, owner: KeyedOwner::Jig });
            root::Event::Core(core::Event::People(people::Event::Ask { reply_to, sign_in, key, ask }))
        }
        Ask::First(value) => {
            let (project, value) =
                jig_client_test_domain::family::wire::connector_request(value).map_err(Invalid::Family)?;
            let number = 1;
            let digest = application_digest(number, project, &value);
            let reply_to = driver.transport.reply_to(Reply::Keyed {
                client,
                attempt,
                key,
                owner: KeyedOwner::Application { connector: number },
            });
            root::Event::Application {
                reply_to,
                sign_in,
                key,
                scope: ApplicationScope::Project(project),
                number,
                digest,
                value,
            }
        }
    })
}

fn application_scope(value: jig_test_connector::ClientScope) -> ApplicationScope {
    match value {
        jig_test_connector::ClientScope::Project(project) => ApplicationScope::Project(project),
        jig_test_connector::ClientScope::Deployment => ApplicationScope::Deployment,
        jig_test_connector::ClientScope::Own => ApplicationScope::Own,
    }
}

/// Canonical fixed-field application encoding, independent of Debug formatting.
/// The versioned domain, connector and project are part of the complete body.
#[must_use]
pub fn application_digest(connector: u16, project: u32, value: &ApplicationRequest) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"jig-test-application/request/1");
    digest.update(connector.to_be_bytes());
    digest.update(project.to_be_bytes());
    digest.update(value.object.to_be_bytes());
    digest.update(value.revision.to_be_bytes());
    digest.update(value.value.to_be_bytes());
    match value.release {
        None => digest.update([0]),
        Some(release) => {
            digest.update([1]);
            digest.update(release.task.to_be_bytes());
            digest.update(release.revision.to_be_bytes());
            digest.update([match release.again {
                None => 0,
                Some(false) => 1,
                Some(true) => 2,
            }]);
        }
    }
    digest.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::super::engine::Packet;
    use super::*;
    use jig_core_world::peers::Peers;
    use skein_lib::Token;

    fn driver() -> (Driver, Time, Wall) {
        let now = Time::from_nanos(17);
        let wall = Wall::from_nanos(1_000_000_000_000);
        let mut driver = Driver::new(
            jig_core_world::world::config(8455),
            jig_core_world::world::limits(),
            Peers::new(Vec::new(), Vec::new()),
            now,
            wall,
        );
        driver.transport.bind(
            ClientId(0),
            1,
            people::Identity {
                key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
                login: Box::from(b"login".as_slice()),
                name: Box::from(b"Person".as_slice()),
            },
        );
        (driver, now, wall)
    }

    #[test]
    fn real_request_dispatch_waits_for_the_supplied_session_without_consuming_native_names() {
        let (mut driver, now, wall) = driver();
        let value =
            Request::Jig(JigOut::Open { watch: Token::new(71), subject: jig_client_domain::contract::Subject::Frame });
        assert_eq!(
            send(&mut driver, ClientId(0), now, wall, &value, &crate::test::limits().jig),
            Err(Invalid::Session)
        );
        assert_eq!(driver.transport.watch_token(ClientId(0), Token::new(71)), None);
        assert!(driver.poll(now, wall).is_empty());
        assert_eq!(
            value,
            Request::Jig(JigOut::Open { watch: Token::new(71), subject: jig_client_domain::contract::Subject::Frame })
        );
    }

    #[test]
    fn real_keyed_request_keeps_its_original_attempt_and_key_behind_the_actual_commit() {
        let (mut driver, now, wall) = driver();
        let event = driver.transport.start_sign_in(ClientId(0));
        driver.send(now, wall, root::Event::Core(event));
        for packet in driver.poll(now, wall) {
            let Packet::Held { delivery, .. } = packet else { panic!("sign-in is committed") };
            let root::Delivery::Core(core::Held::PeopleReply { to, reply }) = *delivery else {
                panic!("actual sign-in answer")
            };
            assert_eq!(driver.transport.take_reply(to.into_token()), Some(Reply::SignIn { client: ClientId(0) }));
            let people::Reply::SignedIn { person, sign_in, .. } = reply else { panic!("actual authenticated person") };
            driver.transport.signed_in(ClientId(0), person, sign_in);
        }
        assert!(driver.transport.sign_in(ClientId(0)).is_some());
        let attempt = Token::new(u64::MAX);
        let key = [255; 16];
        let request = Request::Send {
            attempt,
            key,
            ask: Ask::Jig(Box::new(jig_client_domain::contract::Ask::Stop { project: 1, task: u64::MAX })),
        };
        driver.world.cut = Some(jig_core_world::effects::Cut::Submitted(driver.world.store.applied + 1));
        send(&mut driver, ClientId(0), now, wall, &request, &crate::test::limits().jig)
            .expect("whole original request is routed");
        assert!(driver.world.reached_cut.is_some());
        assert!(driver.withhold_completion().expect("actual decision becomes durable").is_none());
        assert!(driver.poll(now, wall).is_empty());
        driver.release_commits();
        let packets = driver.poll(now, wall);
        assert_eq!(packets.len(), 1);
        let Packet::Held { commit, delivery } = packets.into_iter().next().expect("original keyed answer") else {
            panic!("permanent decision is held")
        };
        assert!(commit > 0);
        let root::Delivery::Core(core::Held::PeopleReply { to, reply, .. }) = *delivery else {
            panic!("actual people terminal")
        };
        assert_eq!(
            driver.transport.take_reply(to.into_token()),
            Some(Reply::Keyed { client: ClientId(0), attempt, key, owner: KeyedOwner::Jig })
        );
        let translated = wire::answers::reply(reply);
        let (envelope, answer) = wire::answers::keyed(translated).expect("actual final body uses its keyed route");
        assert_eq!(envelope, skein_lib::RequestEnvelope::Final);
        assert!(answer.is_some());
        assert!(driver.poll(now, wall).is_empty());
    }

    #[test]
    fn application_digest_keeps_owner_scope_whole_width_and_absent_release_distinct_from_zero() {
        let value = ApplicationRequest { object: u64::MAX, revision: u64::MAX, value: 0, release: None };
        let original = application_digest(u16::MAX, u32::MAX, &value);
        assert_eq!(original, application_digest(u16::MAX, u32::MAX, &value));
        assert_ne!(original, application_digest(u16::MAX - 1, u32::MAX, &value));
        assert_ne!(original, application_digest(u16::MAX, u32::MAX - 1, &value));
        assert_ne!(
            original,
            application_digest(
                u16::MAX,
                u32::MAX,
                &ApplicationRequest {
                    release: Some(jig_test_connector::ApplicationRelease { task: 0, revision: 0, again: None }),
                    ..value
                }
            )
        );
        let released = ApplicationRequest {
            release: Some(jig_test_connector::ApplicationRelease { task: 0, revision: 0, again: None }),
            ..value
        };
        let released_digest = application_digest(u16::MAX, u32::MAX, &released);
        for (task, revision, again) in [(1, 0, None), (0, 1, None), (0, 0, Some(false)), (0, 0, Some(true))] {
            assert_ne!(
                released_digest,
                application_digest(
                    u16::MAX,
                    u32::MAX,
                    &ApplicationRequest {
                        release: Some(jig_test_connector::ApplicationRelease { task, revision, again }),
                        ..value
                    }
                )
            );
        }
        assert_ne!(original, application_digest(u16::MAX, u32::MAX, &ApplicationRequest { value: 1, ..value }));
        assert_ne!(original, application_digest(u16::MAX, u32::MAX, &ApplicationRequest { revision: 0, ..value }));
        assert_ne!(original, application_digest(u16::MAX, u32::MAX, &ApplicationRequest { object: 0, ..value }));
    }
}
