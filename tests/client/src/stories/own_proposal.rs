//! One family page presents jig's first proposal itself.
use super::*;
const PARTY: u64 = 7;
const ITEM: u64 = 1;
const TASK: u64 = 9;
/// The family's object names a proposal whose funding must be shown.
#[must_use]
pub fn world(seed: u64) -> World<Test, Scripted> {
    let mut world = proposal_world(seed);
    let mut state = Family::default();
    state.documents.insert(
        jig_client_test_domain::family::Subject {
            scope: jig_client_test_domain::family::Scope::Project(1),
            kind: jig_client_test_domain::family::Kind::Item(ITEM),
        },
        jig_client_test_domain::family::Document::Item(jig_client_test_domain::family::Item {
            project: 1,
            key: ITEM,
            revision: Revision { value: 1 },
            state: jig_client_test_domain::family::ItemState::Ready,
            value: 1,
            task: TASK,
            second: None,
            proposals: list([proposal().key]),
            touchable: false,
        }),
    );
    world
        .front()
        .engine
        .act_application(skein_lib::Time::ZERO, Wall::EPOCH, jig_client_test_domain::FIRST, 1, state, vec![])
        .expect("family proposal fixture fits");
    world
}
/// Navigation to the family-owned page.
#[must_use]
pub fn address() -> RootAddress {
    RootAddress::First(jig_client_test_domain::first::Address { project: 1, item: ITEM })
}
/// The same view omitting its decision's effects, for the referee's negative story.
#[must_use]
pub fn hidden(seed: u64) -> (jig_client_test_domain::Domain, jig_client_test_view::View<'static>) {
    let (root, mut view) = crate::test::new(&crate::test::limits(), seed);
    view.hide_proposal_effect();
    (root, view)
}
/// The person sees the application's dialog before deciding.
#[must_use]
pub fn person() -> Vec<skein_fake_person::Step<RootAddress>> {
    use skein_fake_person::{Find, Step, Target};
    use skein_ui::Role;
    const WITHIN: Duration = Duration::from_secs(1);
    vec![
        Step::See { find: Find::named(Role::Region, b"Family proposal"), within: WITHIN },
        Step::Press {
            find: Find::named(Role::Button, b"Accept")
                .within(Box::new([Target::Role { role: Role::Region, name: Box::from(*b"Family proposal") }])),
        },
        Step::See { find: Find::named(Role::Dialog, b"Review family proposal"), within: WITHIN },
        Step::Press { find: Find::named(Role::Button, b"Confirm") },
    ]
}
/// The party this fixture signs in.
pub const PERSON: u64 = PARTY;
