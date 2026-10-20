//! Family scopes and windows outside the application's opaque records (contract.md, 7.5).
use alloc::boxed::Box;
use jig_client_domain::{Family, contract as c};
use jig_client_protocol_wire::v10 as w;
use skein_lib::{Reader, Writer, bytes};
pub(crate) fn scope(value: c::Scope, limits: &w::Limits) -> Result<w::Scope, w::Problem> {
    Ok(match value {
        c::Scope::Project(value) => w::Scope::Project(w::ScopeProject::new(limits, w::ScopeProjectParts { value })?),
        c::Scope::Deployment => w::Scope::Deployment,
        c::Scope::Own => w::Scope::Own,
    })
}
pub(crate) fn read_scope(value: &w::Scope) -> c::Scope {
    match value {
        w::Scope::Project(value) => c::Scope::Project(value.value()),
        w::Scope::Deployment => c::Scope::Deployment,
        w::Scope::Own => c::Scope::Own,
    }
}
pub(crate) fn request(scope_value: c::Scope, record: &Family, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let value = w::FamilyRequest::new(
        limits,
        w::FamilyRequestParts { scope: scope(scope_value, limits)?, record: bytes::copy_of(&record.bytes) },
    )?;
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("wire size fits"));
    value.encode(&mut writer).expect("measured family request");
    Ok(writer.finish())
}
pub(crate) fn record(
    scope_value: c::Scope,
    window: c::Window,
    record: Family,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let after = match window.after {
        Some(value) => Some(w::Cursor::new(limits, w::CursorParts { value: bytes::copy_of(value.value.bytes()) })?),
        None => None,
    };
    let window = w::Window::new(limits, w::WindowParts { rows: window.rows, after })?;
    let value = w::FamilyRecord::new(
        limits,
        w::FamilyRecordParts { scope: scope(scope_value, limits)?, window, record: record.bytes },
    )?;
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("wire size fits"));
    value.encode(&mut writer).expect("measured family record");
    Ok(writer.finish())
}
pub(crate) fn restore(bytes: &[u8], limits: &w::Limits) -> Result<(c::Scope, Box<[u8]>), w::Problem> {
    let value = w::FamilyRequest::decode(limits, &mut Reader::new(bytes))?;
    Ok((read_scope(value.scope()), value.into_parts().record))
}
pub(crate) fn versions(value: &w::NotServed) -> Option<c::VersionRange> {
    for range in value.served() {
        if range.owner() == value.owner() {
            return Some(c::VersionRange { first: range.first(), last: range.last() });
        }
    }
    None
}
pub(crate) fn family(owner: &w::Owner) -> Option<u16> {
    match owner {
        w::Owner::Jig => None,
        w::Owner::Family(owner) => Some(owner.number()),
    }
}

pub(crate) fn not_served(value: &w::NotServed, out: &mut skein_lib::Queue<crate::Request>) {
    out.push(crate::Request::Hello { family: family(value.owner()), versions: versions(value) });
}
