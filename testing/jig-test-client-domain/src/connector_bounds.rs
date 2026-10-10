//! Whole allocation envelopes (programming-model.md, 6.3; root.md, 6–7).
use crate::connector_values::ConnectorLimits as Bounds;
use crate::connector_values::{ConnectorAnswer, ConnectorDocument};
use alloc::boxed::Box;
use core::mem::size_of;
use jig_test_connector::{ClientDocument, ClientResource, OutboxEntry, Path};
use skein_lib::List;

fn path_heap(path: &Path) -> Option<u64> {
    let mut bytes = u64::try_from(path.segments().len())
        .ok()?
        .checked_mul(u64::try_from(size_of::<Box<[u8]>>()).expect("inline carrier size fits u64"))?;
    for segment in path.segments() {
        bytes = bytes.checked_add(u64::try_from(segment.len()).ok()?)?;
    }
    Some(bytes)
}
fn paths_heap(paths: &[Path]) -> Option<u64> {
    let mut bytes = u64::try_from(paths.len())
        .ok()?
        .checked_mul(u64::try_from(size_of::<Path>()).expect("inline carrier size fits u64"))?;
    for path in paths {
        bytes = bytes.checked_add(path_heap(path)?)?;
    }
    Some(bytes)
}
fn resource_heap(rows: &List<ClientResource>) -> Option<u64> {
    let mut bytes = List::<ClientResource>::worst_case(rows.capacity())?;
    for row in rows {
        bytes = bytes.checked_add(path_heap(&row.path)?)?;
    }
    Some(bytes)
}
fn entries_heap(rows: &List<OutboxEntry>) -> Option<u64> {
    let mut bytes = List::<OutboxEntry>::worst_case(rows.capacity())?;
    for row in rows {
        bytes = bytes.checked_add(paths_heap(&row.effect.resources)?)?;
    }
    Some(bytes)
}
/// Every owned allocation, including the boxed whole `ClientDocument` and unused list rows.
#[must_use]
pub fn document_heap(value: &ConnectorDocument) -> Option<u64> {
    let nested = match value.value.as_ref() {
        ClientDocument::Object { .. } | ClientDocument::Result { .. } => 0,
        ClientDocument::Panel { procedure, resources, effects, .. } => {
            let procedure = match procedure {
                Some(value) => path_heap(&value.resource)?,
                None => 0,
            };
            procedure.checked_add(resource_heap(resources)?)?.checked_add(entries_heap(effects)?)?
        }
        ClientDocument::Effect { effect, description, .. } => {
            paths_heap(&effect.resources)?.checked_add(paths_heap(&description.resources)?)?
        }
        ClientDocument::Outbox { entry, description } => {
            paths_heap(&entry.effect.resources)?.checked_add(paths_heap(&description.resources)?)?
        }
        ClientDocument::Hold { resources, effects, .. } => {
            resource_heap(resources)?.checked_add(entries_heap(effects)?)?
        }
        ClientDocument::Project { resources, .. } => resource_heap(resources)?,
        ClientDocument::Deployment { outbox, .. } => entries_heap(outbox)?,
    };
    (u64::try_from(size_of::<ClientDocument>()).expect("inline carrier size fits u64")).checked_add(nested)
}
/// Inline carrier and all owned allocations, excluding allocator overhead.
#[must_use]
pub fn document_bytes(value: &ConnectorDocument) -> Option<u64> {
    (u64::try_from(size_of::<ConnectorDocument>()).expect("inline carrier size fits u64"))
        .checked_add(document_heap(value)?)
}
/// The exact closed terminal has no heap-owned fields.
#[must_use]
pub fn answer_bytes(_value: &ConnectorAnswer) -> u64 {
    u64::try_from(size_of::<ConnectorAnswer>()).expect("inline carrier size fits u64")
}
fn path_fits(path: &Path, bounds: &Bounds) -> bool {
    let Ok(count) = u64::try_from(path.segments().len()) else {
        return false;
    };
    if count > u64::from(bounds.segments) {
        return false;
    }
    for segment in path.segments() {
        let Ok(bytes) = u64::try_from(segment.len()) else {
            return false;
        };
        if bytes > u64::from(bounds.segment_bytes) {
            return false;
        }
    }
    true
}
fn paths_fit(paths: &[Path], bounds: &Bounds) -> bool {
    let Ok(count) = u64::try_from(paths.len()) else {
        return false;
    };
    if count > u64::from(bounds.paths) {
        return false;
    }
    for path in paths {
        if !path_fits(path, bounds) {
            return false;
        }
    }
    true
}
fn resources_fit(rows: &List<ClientResource>, bounds: &Bounds) -> bool {
    if rows.capacity() > bounds.rows {
        return false;
    }
    for row in rows {
        if !path_fits(&row.path, bounds) {
            return false;
        }
    }
    true
}
fn effects_fit(rows: &List<OutboxEntry>, bounds: &Bounds) -> bool {
    if rows.capacity() > bounds.rows {
        return false;
    }
    for row in rows {
        if !paths_fit(&row.effect.resources, bounds) {
            return false;
        }
    }
    true
}
/// Admit the complete shape and allocation price; no field is truncated or defaulted.
/// The consumer validates the original request/subject identity; the owner neither
/// rewrites subjects nor derives absent project or proposal-maker facts.
#[must_use]
pub fn document_fits(value: &ConnectorDocument, bounds: &Bounds) -> bool {
    let shape = match value.value.as_ref() {
        ClientDocument::Object { .. } | ClientDocument::Result { .. } => true,
        ClientDocument::Panel { procedure, resources, effects, .. } => {
            let procedure_fits = match procedure {
                Some(p) => path_fits(&p.resource, bounds),
                None => true,
            };
            procedure_fits && resources_fit(resources, bounds) && effects_fit(effects, bounds)
        }
        ClientDocument::Effect { effect, description, .. } => {
            paths_fit(&effect.resources, bounds) && paths_fit(&description.resources, bounds)
        }
        ClientDocument::Outbox { entry, description } => {
            paths_fit(&entry.effect.resources, bounds) && paths_fit(&description.resources, bounds)
        }
        ClientDocument::Hold { resources, effects, .. } => {
            resources_fit(resources, bounds) && effects_fit(effects, bounds)
        }
        ClientDocument::Project { resources, .. } => resources_fit(resources, bounds),
        ClientDocument::Deployment { outbox, .. } => effects_fit(outbox, bounds),
    };
    if !shape {
        return false;
    }
    match document_bytes(value) {
        Some(bytes) => bytes <= bounds.bytes,
        None => false,
    }
}
/// Every exact outcome, including refusals, uses its whole fixed allocation price.
#[must_use]
pub fn answer_fits(value: &ConnectorAnswer, bounds: &Bounds) -> bool {
    answer_bytes(value) <= bounds.bytes
}
/// Largest complete document admitted by the structural envelope. The configured
/// byte ceiling may admit fewer values; this still bounds every admitted value.
#[must_use]
pub fn document_worst_case(bounds: &Bounds) -> Option<u64> {
    let path = u64::from(bounds.segments).checked_mul(
        u64::try_from(size_of::<Box<[u8]>>())
            .expect("inline carrier size fits u64")
            .checked_add(u64::from(bounds.segment_bytes))?,
    )?;
    let paths = u64::from(bounds.paths)
        .checked_mul((u64::try_from(size_of::<Path>()).expect("inline carrier size fits u64")).checked_add(path)?)?;
    let resources =
        List::<ClientResource>::worst_case(bounds.rows)?.checked_add(u64::from(bounds.rows).checked_mul(path)?)?;
    let effects =
        List::<OutboxEntry>::worst_case(bounds.rows)?.checked_add(u64::from(bounds.rows).checked_mul(paths)?)?;
    let nested = path.checked_add(resources)?.checked_add(effects)?.max(paths.checked_mul(2)?);
    (u64::try_from(size_of::<ConnectorDocument>()).expect("inline carrier size fits u64"))
        .checked_add(u64::try_from(size_of::<ClientDocument>()).expect("inline carrier size fits u64"))?
        .checked_add(nested)
}
