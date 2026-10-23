//! Project requests through the application's public client, on the scripted engine.
use jig_client::{Asker, Event, Key, contract as c};
use jig_client_scripted_engine::Change;
use jig_client_world::{ClientId, FrontFault, Scripted, Test, Until, World, stories};
use skein_lib::{Duration, Time, Wall};
type ProjectWorld = World<Test, Scripted>;
const PARTY: u64 = 7;
const PROJECT: u32 = 1;
const POLICY: u32 = 1;

fn text(value: &[u8]) -> c::Text {
    c::Text::new(value.into(), 64).expect("project story fixture or delivered fact")
}
fn ready(owner: bool) -> (ProjectWorld, ClientId) {
    let mut world = stories::world(51);
    let mut frame = stories::frame(PARTY);
    frame.deployment_role = if owner { Some(c::HeldRole::Owner) } else { None };
    frame.projects.get_mut(0).expect("project story fixture or delivered fact").role = c::HeldRole::Owner;
    world
        .front()
        .engine
        .act(Time::ZERO, Wall::EPOCH, Change::Me(frame))
        .expect("project story fixture or delivered fact");
    let client = world.client(PARTY, stories::inbox());
    settle(&mut world);
    (world, client)
}
fn settle(world: &mut ProjectWorld) {
    world.run(Until::Time(world.now().saturating_add(Duration::from_millis(80))));
}
fn send(world: &mut ProjectWorld, client: ClientId, number: u64, ask: c::Ask) -> Key {
    let key = Key { asker: Asker::JIG, number };
    world.event(
        client,
        jig_client_test_domain::Event::Client {
            event: Event::Send { name: None, key, request: jig_client::Ask::Jig(Box::new(ask)) },
        },
    );
    key
}
fn make(name: &[u8]) -> c::Ask {
    c::Ask::MakeProject { name: text(name), policy: POLICY }
}
fn answer(world: &mut ProjectWorld) -> c::Answer {
    world
        .front()
        .engine
        .commits()
        .iter()
        .rev()
        .find_map(|commit| commit.answer.clone())
        .expect("project story fixture or delivered fact")
}
fn frame(world: &ProjectWorld, client: ClientId) -> &c::Me {
    let root = world.root(client);
    root.jig().frame(root.client()).document.expect("project story fixture or delivered fact")
}
fn project_made(world: &mut ProjectWorld) -> u32 {
    let c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::ProjectMade { project })) = answer(world) else {
        panic!("make answered ProjectMade");
    };
    project
}

#[test]
fn a_deployment_owner_makes_a_project_and_finds_it_in_me() {
    let (mut world, client) = ready(true);
    let key = send(&mut world, client, 91, make(b"Tally"));
    settle(&mut world);
    let project = project_made(&mut world);
    assert!(project > PROJECT);
    let made = frame(&world, client)
        .projects
        .iter()
        .find(|p| p.project == project)
        .expect("project story fixture or delivered fact");
    assert_eq!(made.name.bytes(), b"Tally");
    assert_eq!(made.role, c::HeldRole::Owner);
    assert!(world.root(client).client().pending(key).is_none(), "the client received its answer");
    assert!(!world.front().engine.truth().frames[&8].projects.iter().any(|p| p.project == project));
}

#[test]
fn an_owner_renames_a_project_and_me_says_the_new_name() {
    let (mut world, client) = ready(true);
    send(&mut world, client, 91, c::Ask::Rename { project: PROJECT, name: text(b"Tally") });
    settle(&mut world);
    assert_eq!(answer(&mut world), c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::Renamed { project: PROJECT })));
    assert_eq!(
        frame(&world, client).projects.get(0).expect("project story fixture or delivered fact").name.bytes(),
        b"Tally"
    );
    assert_eq!(
        world.front().engine.truth().frames[&8]
            .projects
            .get(0)
            .expect("project story fixture or delivered fact")
            .name
            .bytes(),
        b"Tally"
    );
}

#[test]
fn a_second_make_of_one_name_is_refused_as_name_taken() {
    let (mut world, client) = ready(true);
    send(&mut world, client, 91, make(b"Tally"));
    settle(&mut world);
    let project = project_made(&mut world);
    send(&mut world, client, 92, make(b"tally"));
    settle(&mut world);
    assert_eq!(answer(&mut world), c::Answer::Refused { refusal: c::Refusal::NameTaken, at: None });
    assert_eq!(frame(&world, client).projects.iter().filter(|p| p.project == project).count(), 1);
}

#[test]
fn a_make_by_a_party_who_owns_no_deployment_is_refused_by_role() {
    let (mut world, client) = ready(false);
    send(&mut world, client, 91, make(b"Tally"));
    settle(&mut world);
    assert_eq!(answer(&mut world), c::Answer::Refused { refusal: c::Refusal::Role { held: None }, at: None });
    assert_eq!(frame(&world, client).projects.len(), 1);
}

#[test]
fn a_name_of_digits_alone_is_refused_naming_its_rule() {
    let (mut world, client) = ready(true);
    send(&mut world, client, 91, make(b"123"));
    settle(&mut world);
    assert_eq!(
        answer(&mut world),
        c::Answer::Refused {
            refusal: c::Refusal::Malformed { at: 0, why: c::Malformed::Name(c::NameRule::Digits) },
            at: None
        }
    );
}

#[test]
fn a_make_retried_under_its_key_after_a_reload_is_answered_once() {
    let (mut world, client) = ready(true);
    world.fault(FrontFault::NoAnswer);
    let key = send(&mut world, client, 91, make(b"Tally"));
    assert!(world.root(client).client().pending(key).is_some());
    world.reload(client);
    settle(&mut world);
    let project = project_made(&mut world);
    assert_eq!(world.front().engine.commits().iter().filter(|commit| commit.key.is_some()).count(), 1);
    assert_eq!(frame(&world, client).projects.iter().filter(|p| p.project == project).count(), 1);
    assert!(world.root(client).client().pending(key).is_none());
}
