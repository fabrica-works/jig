//! The real deployment's project names reach the public application-held client.
use super::*;
use jig_client::{Asker, Client, Handle, Readable, Readout, Told};
use skein_lib::{List, Queue};

const PROJECT: u32 = 2;
const ME: Handle = Handle { asker: Asker::JIG, number: 1 };
const PAGE: Handle = Handle { asker: Asker::JIG, number: 2 };
// Each public request and its reply settles in fewer than sixteen synchronous passes.
const PASSES: u32 = 16;

fn exchange(session: &mut Session, client: &mut Client, now: Time, wall: Wall, event: jig_client::Event) -> Vec<Told> {
    let mut out = Queue::with_capacity(jig_client::max_out(&session.limits.client));
    client.step(now, wall, event, &mut out);
    let mut told = Vec::new();
    for _ in 0..PASSES {
        while let Some(request) = out.pop() {
            if let jig_client::Request::Told { told: value } = request {
                told.push(value);
            } else {
                session.send(ClientId(0), now, wall, Request::Client { request }).expect("public request");
            }
        }
        let callbacks = session.poll(now, wall).expect("real engine replies");
        if callbacks.is_empty() {
            return told;
        }
        for (owner, callback) in callbacks {
            session.received(owner, now, wall, &callback);
            let Event::Client { event } = callback else { panic!("client callback") };
            client.step(now, wall, event, &mut out);
        }
    }
    panic!("public exchange did not settle");
}

#[test]
fn the_frame_names_each_project() {
    let (mut configuration, limits) = jig_core_world::fixtures::two_projects(8455);
    configuration.core.seed_projects[0].name = Box::from("Tally");
    // The shared deployment fixture leaves project 2 unnamed: its seed is Project 2.
    let (mut session, now, wall) = session_with_limits(configuration, limits, Peers::new(Vec::new(), Vec::new()));
    session.bind(ClientId(0), 1, identity(), now, wall);
    session.poll(now, wall).expect("signed-in owner");
    let mut client = Client::unlinked(&session.limits.client, 8455).expect("public client");
    exchange(&mut session, &mut client, now, wall, jig_client::Event::StartUnlinked { kept: List::with_capacity(0) });
    exchange(&mut session, &mut client, now, wall, jig_client::Event::Up { up: jig_client::Up::SignedIn });
    let client::ReadValue::Me(me) = read(&mut session, &mut client, now, wall, ME, client::Read::Me) else {
        panic!("public me document")
    };
    let names: Vec<_> = me.projects.iter().map(|row| (row.project, row.name.bytes())).collect();
    assert_eq!(names, [(1, b"Tally".as_slice()), (PROJECT, b"Project 2".as_slice())]);
    let client::ReadValue::OwnerProject(project) =
        read(&mut session, &mut client, now, wall, PAGE, client::Read::Project { project: PROJECT })
    else {
        panic!("public project document")
    };
    assert_eq!((project.number, project.name.bytes()), (PROJECT, b"Project 2".as_slice()));
}

fn read(
    session: &mut Session,
    client: &mut Client,
    now: Time,
    wall: Wall,
    handle: Handle,
    what: client::Read,
) -> client::ReadValue {
    let output = exchange(session, client, now, wall, jig_client::Event::Read { handle, read: Readable::Jig(what) });
    output
        .into_iter()
        .find_map(|value| {
            let Told::Page { handle: delivered, page: Readout::Jig(client::ReadResult::Value(value)) } = value else {
                return None;
            };
            (delivered == handle).then_some(value)
        })
        .expect("one public document")
}
