//! Versions and deployment checked at open (protocol/store.md, 5.3).
use crate::boundary::{Family, OpenRefusal};
use crate::codec::component;

#[must_use]
pub fn current(application: u32, deployment: [u8; 16]) -> component::Format {
    component::Format { component: 2, core: 5, tasks: 9, people: 9, notes: 1, accounts: 1, application, deployment }
}

pub fn check(found: &component::Format, wanted: &component::Format) -> Result<(), OpenRefusal> {
    for (family, found, wanted) in [
        (Family::Component, u32::from(found.component), u32::from(wanted.component)),
        (Family::Core, u32::from(found.core), u32::from(wanted.core)),
        (Family::Tasks, u32::from(found.tasks), u32::from(wanted.tasks)),
        (Family::People, u32::from(found.people), u32::from(wanted.people)),
        (Family::Notes, u32::from(found.notes), u32::from(wanted.notes)),
        (Family::Accounts, u32::from(found.accounts), u32::from(wanted.accounts)),
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
