//! Shared amendment payloads for the tests.
/// An amendment retaining every field, with the supplied explanation.
#[must_use]
pub fn person_unchanged(reason: &[u8]) -> jig_domain_people::Amendment {
    jig_domain_people::Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        reason: reason.into(),
    }
}

pub fn member(world: &mut crate::World) -> u64 {
    let call = world.signin(10, 1);
    assert!(world.reply(call).is_none());
    world.commit_all();
    let jig_domain_people::Reply::SignedIn { person, .. } =
        world.reply(call).expect("test values are admitted and fit")
    else {
        panic!("signed in")
    };
    world.roles(1, Box::new([jig_domain_people::Holding { person, role: jig_domain_people::Role::Member }]));
    world.commit_all();
    person
}
