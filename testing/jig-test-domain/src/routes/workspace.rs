//! Gather credential names from the connectors' typed workspace items.
use super::{Domain, Limits, List, tasks};
use alloc::boxed::Box;

pub(super) fn credentials(
    domain: &mut Domain,
    limits: &Limits,
    project: u32,
    spec: &tasks::Spec,
) -> Option<Box<[u32]>> {
    let capacity = limits.connector.resources_per_task.checked_mul(2).expect("two connectors' item bound");
    let mut names = List::with_capacity(capacity);
    for number in [1, 2] {
        let mut paths = List::with_capacity(limits.connector.resources_per_task);
        for parameter in &spec.parameters {
            match parameter {
                tasks::Parameter::Resource { connector, resource, .. } => {
                    if *connector == number {
                        let path = crate::numbered(domain, number).specification_resource(*resource)?.path.clone();
                        if !paths.as_slice().contains(&path) {
                            paths.push(path).ok()?;
                        }
                    }
                }
                tasks::Parameter::Number { .. } | tasks::Parameter::Bytes { .. } => {}
            }
        }
        let items = crate::numbered(domain, number).workspace_items(&limits.connector, project, paths.as_slice());
        for item in items {
            if let Some(credential) = item.credential
                && !names.as_slice().contains(&credential)
            {
                names.push(credential).expect("one credential per gathered item");
            }
        }
    }
    Some(names.into_boxed())
}
