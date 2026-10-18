use jig_client_view::address::ProjectSection;
use jig_client_world::{Until, stories::goals};
use skein_fake_person::{Find, Step};
use skein_lib::Duration;
use skein_ui::Role;
const WAIT: Duration = Duration::from_secs(1);

#[test]
fn moving_a_goal_up_sends_one_compound_request_of_priorities() {
    let mut world = goals::board(8530);
    let client = world.client(goals::PARTY, goals::address(ProjectSection::Goals));
    world.person(
        client,
        vec![
            Step::See { find: Find::text(b"Second goal"), within: WAIT },
            Step::Press { find: Find::exactly_named(Role::Button, b"Move up") },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, skein_world::domain::Verdict::Passed);
    let commits = world.front().engine.commits();
    assert_eq!(commits.iter().filter(|commit| commit.key.is_some()).count(), 1);
    let root = world.root(client);
    let rows = root.jig().borrow(root.client()).project_page().goals;
    assert_eq!(
        rows.as_slice().iter().map(|row| (row.chip.task, row.tracked)).collect::<Vec<_>>(),
        vec![(goals::SECOND, Some(0)), (goals::FIRST, Some(1))]
    );
}
#[test]
fn the_project_overview_shows_the_board_s_first_goals() {
    let mut world = goals::board(8531);
    let client = world.client(goals::PARTY, goals::address(ProjectSection::Overview));
    world.person(
        client,
        vec![
            Step::See { find: Find::text(b"First goal"), within: WAIT },
            Step::See { find: Find::text(b"Second goal"), within: WAIT },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, skein_world::domain::Verdict::Passed);
    let root = world.root(client);
    assert_eq!(
        root.jig().borrow(root.client()).project_page().goals.get(0).expect("first query row").chip.task,
        goals::FIRST
    );
}

fn refused(
    proposable: Option<bool>,
) -> (jig_client_world::World<jig_client_world::Test, jig_client_world::Scripted>, jig_client_world::ClientId) {
    let mut world = goals::refused(8532, proposable);
    let client = world.client(
        goals::PARTY,
        jig_client_test_domain::Address::Jig(jig_client_view::address::Address::task(goals::FIRST)),
    );
    world.person(
        client,
        vec![
            Step::See { find: Find::exactly_named(Role::Button, b"Amend"), within: WAIT },
            Step::Press { find: Find::exactly_named(Role::Button, b"Amend") },
            Step::Type {
                find: Find::named(Role::TextArea, b"New instructions"),
                words: b"changed instructions".as_slice().into(),
            },
            Step::Type { find: Find::named(Role::TextArea, b"Your reason"), words: b"because".as_slice().into() },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::named(Role::Status, b"Refused"), within: WAIT },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, skein_world::domain::Verdict::Passed);
    (world, client)
}
#[test]
fn a_refusal_a_proposal_could_answer_offers_to_propose_with_its_own_confirmation() {
    let (mut world, client) = refused(Some(true));
    goals::offer_proposal(&mut world);
    world.person(
        client,
        vec![
            Step::See { find: Find::text(b"Amendment now proposable"), within: WAIT },
            Step::See { find: Find::exactly_named(Role::Button, b"Propose instead"), within: WAIT },
            Step::Press { find: Find::exactly_named(Role::Button, b"Propose instead") },
            Step::See { find: Find::text(b"This will be proposed"), within: WAIT },
            Step::See { find: Find::text(b"Waiting for project 1, role 1"), within: WAIT },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, skein_world::domain::Verdict::Passed);
    assert_eq!(world.front().engine.commits().iter().filter(|c| c.key.is_some()).count(), 1);
    world.person(client, vec![Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") }]);
    assert_eq!(world.run(Until::Done).verdict, skein_world::domain::Verdict::Passed);
    assert_eq!(world.front().engine.commits().iter().filter(|c| c.key.is_some()).count(), 2);
}
#[test]
fn a_refusal_nobody_could_accept_offers_no_proposal() {
    let (mut world, client) = refused(Some(false));
    assert!(world.tree(client).nodes().iter().all(|node| node.name.as_deref() != Some(b"Propose instead".as_slice())));
    assert_eq!(world.front().engine.commits().iter().filter(|c| c.key.is_some()).count(), 1);
}
