//! Versions and deployment checked at open (protocol/store.md, 5.3).
use crate::boundary::{Family, OpenRefusal};
use crate::codec::component;

#[must_use]
pub fn current(application: u32, deployment: [u8; 16]) -> component::Format {
    component::Format {
        component: 3,
        core: 6,
        tasks: 9,
        people: 13,
        notes: 1,
        accounts: 2,
        settings: 1,
        application,
        deployment,
    }
}

pub fn check(found: &component::Format, wanted: &component::Format) -> Result<(), OpenRefusal> {
    for (family, found, wanted) in [
        (Family::Component, u32::from(found.component), u32::from(wanted.component)),
        (Family::Core, u32::from(found.core), u32::from(wanted.core)),
        (Family::Tasks, u32::from(found.tasks), u32::from(wanted.tasks)),
        (Family::People, u32::from(found.people), u32::from(wanted.people)),
        (Family::Notes, u32::from(found.notes), u32::from(wanted.notes)),
        (Family::Accounts, u32::from(found.accounts), u32::from(wanted.accounts)),
        (Family::Settings, u32::from(found.settings), u32::from(wanted.settings)),
        (Family::Application, found.application, wanted.application),
    ] {
        if found != wanted {
            return Err(OpenRefusal::Format { family, found, wanted });
        }
    }
    if found.deployment != wanted.deployment {
        return Err(OpenRefusal::Deployment);
    }
    Ok(())
}

/// This build's layouts (store.md, 5.3): computed from the schemas built in,
/// committed beside the generated codecs, and checked against them by the drift test.
#[must_use]
pub fn layouts() -> component::Layouts {
    crate::layouts::LAYOUTS
}

/// Compare in Family's order, before any family's header is decoded.
pub fn check_layouts(found: &component::Layouts, wanted: &component::Layouts) -> Result<(), OpenRefusal> {
    for (family, found, wanted) in [
        (Family::Settings, found.settings, wanted.settings),
        (Family::Component, found.component, wanted.component),
        (Family::Core, found.core, wanted.core),
        (Family::Tasks, found.tasks, wanted.tasks),
        (Family::People, found.people, wanted.people),
        (Family::Notes, found.notes, wanted.notes),
        (Family::Accounts, found.accounts, wanted.accounts),
    ] {
        if found != wanted {
            return Err(OpenRefusal::OlderBuild { why: crate::Older::Layout(family) });
        }
    }
    Ok(())
}
