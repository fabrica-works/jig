//! Focused visibility checks through the person’s scripted actions.
use skein_world::domain::{Verdict, heap::Counting};
#[global_allocator]
static HEAP: Counting = Counting;
#[test]
fn clearing_the_filter_shows_the_same_still_waiting_proposal() {
    use jig_client_view::Role;
    use jig_client_world::{Until, stories};
    use jig_fake_person::{Find, Step};
    use skein_lib::Duration;
    let mut world = stories::proposal_world(111);
    let client = world.client(7, stories::inbox());
    world.person(
        client,
        vec![
            Step::See { find: Find::text(b"Resolve the incident"), within: Duration::from_secs(1) },
            Step::Press { find: Find::named(Role::Button, b"Reply") },
            Step::Gone { find: Find::text(b"Resolve the incident"), within: Duration::from_secs(1) },
            Step::Press { find: Find::named(Role::Button, b"All kinds") },
            Step::See { find: Find::text(b"Resolve the incident"), within: Duration::from_secs(1) },
            Step::See { find: Find::named(Role::Button, b"Accept"), within: Duration::from_secs(1) },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert!(world.front().engine.commits().iter().all(|commit| commit.key.is_none()));
}
