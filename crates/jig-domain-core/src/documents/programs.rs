//! Secret-free per-host login observations, joined at read time (accounts.md, 3.4).
use super::DocumentLimits;
use crate::Core;
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_fleet as fleet;
use skein_lib::{List, Token, bytes};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProgramLogin {
    pub name: Box<[u8]>,
    pub state: Option<fleet::LoginState>,
    pub sign_out: Option<fleet::SignOutResult>,
    pub cleanup: bool,
    pub cut: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProgramHost {
    pub name: Box<[u8]>,
    pub channel: u64,
    pub connected: bool,
    pub awaiting_logins: bool,
    pub programs: List<ProgramLogin>,
    pub cut: bool,
}
fn detail(value: &[u8], limits: &DocumentLimits, cut: &mut bool) -> Box<[u8]> {
    let bound = usize::try_from(limits.text_bytes.min(4_096)).unwrap_or(0);
    let end = value.len().min(bound);
    *cut |= end < value.len();
    bytes::copy_of(value.get(..end).expect("bounded detail prefix"))
}
fn state(value: &fleet::LoginState, limits: &DocumentLimits, cut: &mut bool) -> fleet::LoginState {
    match value {
        fleet::LoginState::Usable => fleet::LoginState::Usable,
        fleet::LoginState::NotSignedIn => fleet::LoginState::NotSignedIn,
        fleet::LoginState::SignedInOtherwise(method) => fleet::LoginState::SignedInOtherwise(*method),
        fleet::LoginState::Failed { reason, detail: text } => {
            fleet::LoginState::Failed { reason: *reason, detail: detail(text, limits, cut) }
        }
    }
}
fn signed_out(value: &fleet::SignOutResult, limits: &DocumentLimits, cut: &mut bool) -> fleet::SignOutResult {
    match value {
        fleet::SignOutResult::SignedOut => fleet::SignOutResult::SignedOut,
        fleet::SignOutResult::Failed { reason, detail: text } => {
            fleet::SignOutResult::Failed { reason: *reason, detail: detail(text, limits, cut) }
        }
    }
}
pub(super) fn logins(
    core: &Core,
    limits: &DocumentLimits,
    channel: Token,
    holder: Option<accounts::Holder>,
) -> (List<ProgramLogin>, bool) {
    let mut rows = List::with_capacity(limits.lines);
    let mut cut = false;
    if channel == Token::new(0) {
        return (rows, cut);
    }
    for name in core.fleet.program_names() {
        if let Some(holder) = holder
            && accounts::programs::owner(&core.accounts, name.as_bytes()) != Some(holder)
        {
            continue;
        }
        if rows.room() == 0 {
            cut = true;
            continue;
        }
        let mut row = ProgramLogin {
            name: bytes::copy_of(name.as_bytes()),
            state: None,
            sign_out: None,
            cleanup: core.fleet.program_cleanup(*name),
            cut: false,
        };
        if !core.fleet.awaiting_logins(channel)
            && let Some(logins) = core.fleet.program_logins(channel)
        {
            for login in logins {
                if login.name == *name {
                    row.state = Some(state(&login.state, limits, &mut row.cut));
                }
            }
        }
        if let Some(results) = core.fleet.program_sign_outs(channel)
            && let Some(result) = results.get(name)
        {
            row.sign_out = Some(signed_out(result, limits, &mut row.cut));
        }
        rows.push(row).expect("bounded program facts");
    }
    (rows, cut)
}
pub(super) fn hosts(core: &Core, limits: &crate::Limits, holder: accounts::Holder) -> (List<ProgramHost>, bool) {
    let mut hosts = List::with_capacity(limits.documents.lines);
    let mut cut = false;
    for host in &core.fleet.host_views() {
        let Some(worker) = host.worker else { continue };
        let Some(name) = core.fleet.worker_name(worker) else { continue };
        let (programs, host_cut) = logins(core, &limits.documents, host.channel, Some(holder));
        if programs.is_empty() && !host_cut {
            continue;
        }
        if hosts.room() == 0 {
            cut = true;
            continue;
        }
        hosts
            .push(ProgramHost {
                name: bytes::copy_of(name),
                channel: host.channel.raw(),
                connected: host.connected,
                awaiting_logins: core.fleet.awaiting_logins(host.channel),
                programs,
                cut: host_cut,
            })
            .expect("bounded host facts");
    }
    (hosts, cut)
}
pub(super) fn logins_worst_case(limits: &DocumentLimits) -> Option<u64> {
    List::<ProgramLogin>::worst_case(limits.lines)?.checked_add(
        u64::from(limits.lines).checked_mul(
            u64::from(accounts::NAME).checked_add(u64::from(limits.text_bytes.min(4_096)).checked_mul(2)?)?,
        )?,
    )
}
pub(super) fn hosts_worst_case(limits: &DocumentLimits) -> Option<u64> {
    List::<ProgramHost>::worst_case(limits.lines)?.checked_add(
        u64::from(limits.lines).checked_mul(u64::from(fleet::WORKER_NAME).checked_add(logins_worst_case(limits)?)?)?,
    )
}
