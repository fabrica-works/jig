use jig_client::{Client, Event, Link, Request};
use jig_client_world::{Front, Test, harness::ClientId, joint::Joint};
use skein_lib::{Duration, List, Queue, Time, Wall};
const PARTY: u64 = 1;
const CLIENT: ClientId = ClientId(0);
const NOW: Time = Time::ZERO;
const WALL: Wall = Wall::from_nanos(1_000_000_000_000);
const FIRST_RETRY: Duration = Duration::from_millis(10);
const MOST_RETRY: Duration = Duration::from_millis(80);
fn front() -> Joint {
    let mut owner = jig_core_world::world::limits();
    owner.core.documents.rows = 2;
    owner.core.documents.lines = 2;
    owner.core.documents.chips = 2;
    owner.core.documents.text_bytes = 32;
    jig_client_world::joint::front::fixture(&owner)
}
fn limits() -> jig_client::Limits {
    let world = jig_client_world::test::limits();
    let l = world.jig;
    jig_client::Limits {
        domain: jig_client::DomainLimits {
            owners: &[],
            requests: world.client.domain.requests,
            ask_bytes: 16384,
            rows: l.rows,
            segments: l.segments,
            text: l.text,
            projects: l.projects,
            fields: l.fields,
            delegates: l.delegates,
            records: l.records,
            kept: l.kept,
            parts: l.parts,
            authority: 8,
            heartbeat: world.client.domain.heartbeat,
            handles: 8,
            turn_bytes: 32,
            reads: l.reads,
            watches: l.watches,
            document_bytes: l.document_bytes,
            backoff: jig_client::Backoff { first: FIRST_RETRY, most: MOST_RETRY },
        },
        exchanges: 8,
    }
}
fn drain(client: &mut Client, front: &mut Joint, out: &mut Queue<Request>) {
    while let Some(request) = out.pop() {
        match request {
            Request::Down { down } => front.down(CLIENT, NOW, WALL, down).expect("Jig transport operation"),
            Request::Told { .. } => {}
            Request::Exchange { .. }
            | Request::OpenStream { .. }
            | Request::CloseStream
            | Request::Save { .. }
            | Request::Erase { .. }
            | Request::SignOut => panic!("unlinked client emits domain operations"),
        }
    }
    client.reclaim();
}
#[test]
fn an_unlinked_client_sees_me_live_against_the_real_core() {
    let limits = limits();
    let mut client = Client::unlinked(&limits, 8512).expect("bounded client");
    let mut out = Queue::with_capacity(jig_client::max_out(&limits));
    let mut front = front();
    Front::<Test>::bind(&mut front, CLIENT, PARTY);
    client.step(NOW, WALL, Event::StartUnlinked { kept: List::with_capacity(0) }, &mut out);
    client.step(NOW, WALL, Event::SignIn { with: jig_client::With::Transport }, &mut out);
    drain(&mut client, &mut front, &mut out);
    for _ in 0..20 {
        for (owner, up) in front.up(NOW, WALL, limits.domain.owners) {
            assert_eq!(owner, CLIENT);
            client.step(NOW, WALL, Event::Up { up }, &mut out);
            drain(&mut client, &mut front, &mut out);
        }
        if client.link().state == Link::Live {
            break;
        }
    }
    assert_eq!((client.link().state, client.link().party), (Link::Live, Some(1)));
}
