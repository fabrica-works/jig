//! The effect stories' explicitly hand-bootstrapped owner roles.
use super::{World, core, people, root};

pub(super) fn signed_in(world: &mut World, person: u64) {
    if world.bootstrap_owners.is_empty() {
        return;
    }
    let key = root::Key::Core(core::Key::People(people::Key::Person(person)));
    let Some(root::Record::Core(core::Record::People(people::Stored::Person { identity, .. }))) =
        world.store.rows.get(&key)
    else {
        return;
    };
    let identity = identity.key.clone();
    let configured = std::mem::take(&mut world.bootstrap_owners);
    for owner in configured {
        if owner.identity == identity {
            let holdings = holdings(world, owner.project, person);
            world.events.push_back(root::Event::Core(core::Event::People(people::Event::Roles {
                project: owner.project,
                holdings,
            })));
        } else {
            world.bootstrap_owners.push(owner);
        }
    }
}

fn holdings(world: &World, project: u32, person: u64) -> Box<[people::Holding]> {
    let key = root::Key::Core(core::Key::People(people::Key::Roles(project)));
    let mut result = Vec::new();
    if let Some(root::Record::Core(core::Record::People(people::Stored::Roles { holdings, .. }))) =
        world.store.rows.get(&key)
    {
        for holding in holdings {
            if holding.person != person {
                result.push(*holding);
            }
        }
    }
    result.push(people::Holding { person, role: people::Role::Owner });
    result.into_boxed_slice()
}
