use jig_client::{contract as c, shapes};
use jig_client_protocol_world::{fixtures, stories as bytes};
use jig_client_view::address::ProjectSection;
use skein_fake_person::{Find, Step};
use skein_lib::Duration;
use skein_ui::Role;
const WAIT: Duration = Duration::from_secs(1);
const FIRST_GOAL: u64 = 2;
const SECOND_GOAL: u64 = 3;
const PROJECT: u32 = 1;
const FIRST_REQUEST: u64 = 9030;
const SECOND_REQUEST: u64 = 9031;

fn ready() -> (bytes::Story, jig_client_world::ClientId) {
    let mut limits = fixtures::owner_limits();
    limits.core.tasks.tasks = 3;
    limits.core.tasks.project_tasks = 3;
    let mut world = jig_client_world::World::new(
        jig_client_world::Settings { seed: fixtures::SEED, ..jig_client_world::Settings::default() },
        jig_client_protocol_world::Protocol::bounded(fixtures::SEED, &limits),
    );
    let client = world.client(fixtures::FIRST_OWNER, jig_client_world::stories::conversation());
    world.person(client, bytes::start());
    bytes::finish(&mut world);
    (world, client)
}
fn goal(world: &mut bytes::Story, client: jig_client_world::ClientId, words: &[u8], priority: u32, request: u64) {
    let root = world.root(client);
    let me = root.jig().borrow(root.client()).frame().document.expect("me delivered");
    let judged = shapes::set_goal(
        me,
        PROJECT,
        shapes::GoalDraft {
            words: jig_client_world::stories::text(words),
            executor: c::OwnerExecutor::Agent { charter: 1 },
            budget: 10,
            deadline: None,
            priority: Some(priority),
        },
        jig_client_world::stories::text(b""),
        fixtures::wall(world.now()),
    );
    let shapes::Judged::Create(ask) = judged else { panic!("goal offered: {judged:?}") };
    world.event(
        client,
        jig_client_test_domain::Event::Client {
            event: jig_client::Event::Send {
                key: jig_client::Key { asker: jig_client::Asker::JIG, number: request },
                request: jig_client::Ask::Jig(Box::new(ask)),
            },
        },
    );
    bytes::finish(world);
}
#[test]
fn moving_a_goal_up_reorders_the_board_in_one_compound_request() {
    let (mut world, client) = ready();
    goal(&mut world, client, b"First goal", 0, FIRST_REQUEST);
    goal(&mut world, client, b"Second goal", 1, SECOND_REQUEST);
    world.person(
        client,
        vec![
            Step::Go {
                address: jig_client_test_domain::Address::Jig(jig_client_view::address::Address::Project {
                    project: PROJECT,
                    section: ProjectSection::Goals,
                }),
            },
            Step::See { find: Find::text(b"Second goal"), within: WAIT },
        ],
    );
    bytes::finish(&mut world);
    let before = world.front().requests_made();
    world.person(
        client,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Move up") },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
        ],
    );
    bytes::finish(&mut world);
    assert_eq!(world.front().requests_made() - before, 1);
    let root = world.root(client);
    let rows = root.jig().borrow(root.client()).project_page().goals;
    assert_eq!(rows.as_slice().iter().map(|row| row.chip.task).collect::<Vec<_>>(), vec![SECOND_GOAL, FIRST_GOAL]);
}
