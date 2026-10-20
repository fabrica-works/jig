//! Secret-free program facts (accounts.md, 3.4; client/domain/contract.md, 5.1).
use super::{pieces, w};
use alloc::boxed::Box;
use jig_domain_core::documents as d;
use jig_domain_fleet as fleet;
use skein_lib::{List, Writer};
fn failure(value: fleet::SourceFailure) -> w::OwnerProgramFailure {
    match value {
        fleet::SourceFailure::Missing => w::OwnerProgramFailure::Missing,
        fleet::SourceFailure::Deadline => w::OwnerProgramFailure::Deadline,
        fleet::SourceFailure::Unreadable => w::OwnerProgramFailure::Unreadable,
        fleet::SourceFailure::Exited => w::OwnerProgramFailure::Exited,
        fleet::SourceFailure::Unconfigured => w::OwnerProgramFailure::Unconfigured,
        fleet::SourceFailure::Unknown => w::OwnerProgramFailure::Unknown,
        fleet::SourceFailure::Closed => w::OwnerProgramFailure::Closed,
    }
}
fn state(value: &fleet::LoginState, limits: &w::Limits) -> Result<w::OwnerProgramState, w::Problem> {
    Ok(match value {
        fleet::LoginState::Usable => w::OwnerProgramState::Usable,
        fleet::LoginState::NotSignedIn => w::OwnerProgramState::NotSignedIn,
        fleet::LoginState::SignedInOtherwise(method) => {
            w::OwnerProgramState::SignedInOtherwise(w::OwnerProgramOtherwise::new(
                limits,
                w::OwnerProgramOtherwiseParts { method: skein_lib::bytes::copy_of(method.as_bytes()) },
            )?)
        }
        fleet::LoginState::Failed { reason, detail } => w::OwnerProgramState::Failed(w::OwnerProgramFailed::new(
            limits,
            w::OwnerProgramFailedParts { reason: failure(*reason), detail: detail.clone() },
        )?),
    })
}
pub fn login(value: &d::ProgramLogin, limits: &w::Limits) -> Result<w::OwnerProgramLogin, w::Problem> {
    let state = match &value.state {
        Some(value) => Some(state(value, limits)?),
        None => None,
    };
    let sign_out = match &value.sign_out {
        Some(fleet::SignOutResult::SignedOut) => Some(w::OwnerProgramSignOut::SignedOut),
        Some(fleet::SignOutResult::Failed { reason, detail }) => {
            Some(w::OwnerProgramSignOut::Failed(w::OwnerProgramFailed::new(
                limits,
                w::OwnerProgramFailedParts { reason: failure(*reason), detail: detail.clone() },
            )?))
        }
        None => None,
    };
    w::OwnerProgramLogin::new(
        limits,
        w::OwnerProgramLoginParts { name: value.name.clone(), state, sign_out, cleanup: value.cleanup, cut: value.cut },
    )
}
pub fn logins(
    value: &List<d::ProgramLogin>,
    limits: &w::Limits,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut rows = List::with_capacity(value.len());
    for row in value {
        let row = login(row, limits)?;
        let mut out = Writer::new(usize::try_from(row.measure()).expect("measured row fits"));
        row.encode(&mut out).expect("measured login row");
        rows.push(out.finish()).expect("one piece per supplied row");
    }
    pieces(&rows, bound, path)
}
pub fn host(value: &d::ProgramHost, limits: &w::Limits) -> Result<w::OwnerProgramHost, w::Problem> {
    w::OwnerProgramHost::new(
        limits,
        w::OwnerProgramHostParts {
            name: value.name.clone(),
            channel: value.channel,
            connected: value.connected,
            awaiting_logins: value.awaiting_logins,
            programs: logins(
                &value.programs,
                limits,
                limits.owner_program_host_programs,
                w::Path::OwnerProgramHostPrograms,
            )?,
            cut: value.cut,
        },
    )
}
pub fn hosts(value: &List<d::ProgramHost>, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut rows = List::with_capacity(value.len());
    for row in value {
        let row = host(row, limits)?;
        let mut out = Writer::new(usize::try_from(row.measure()).expect("measured row fits"));
        row.encode(&mut out).expect("measured program host");
        rows.push(out.finish()).expect("one piece per supplied row");
    }
    pieces(&rows, limits.owner_accounts_program_hosts, w::Path::OwnerAccountsProgramHosts)
}
