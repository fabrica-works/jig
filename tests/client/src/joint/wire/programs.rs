//! Program facts translated independently in the domain-only client world.
use super::owner;
use jig_client::DomainLimits as Limits;
use jig_client_domain::contract as c;
use jig_domain_core::documents as d;
use jig_domain_fleet as f;
use skein_lib::List;
fn failure(value: f::SourceFailure) -> c::OwnerProgramFailure {
    match value {
        f::SourceFailure::Missing => c::OwnerProgramFailure::Missing,
        f::SourceFailure::Deadline => c::OwnerProgramFailure::Deadline,
        f::SourceFailure::Unreadable => c::OwnerProgramFailure::Unreadable,
        f::SourceFailure::Exited => c::OwnerProgramFailure::Exited,
        f::SourceFailure::Unconfigured => c::OwnerProgramFailure::Unconfigured,
        f::SourceFailure::Unknown => c::OwnerProgramFailure::Unknown,
        f::SourceFailure::Closed => c::OwnerProgramFailure::Closed,
    }
}
fn bounded(value: &[u8], maximum: u32) -> Result<Box<[u8]>, owner::Invalid> {
    if value.len() > usize::try_from(maximum).map_err(|_| owner::Invalid::TooLarge)? {
        return Err(owner::Invalid::TooLarge);
    }
    Ok(value.into())
}
pub(super) fn configured_name(value: &[u8]) -> Result<Box<[u8]>, owner::Invalid> {
    if core::str::from_utf8(value).is_err() {
        return Err(owner::Invalid::Program);
    }
    bounded(value, c::PROGRAM_HOST_NAME)
}
pub(super) fn program_name(value: &[u8]) -> Result<Box<[u8]>, owner::Invalid> {
    if !c::valid_program_name(value) {
        return Err(owner::Invalid::Program);
    }
    bounded(value, c::PROGRAM_NAME)
}
fn method_name(value: &[u8]) -> Result<Box<[u8]>, owner::Invalid> {
    if !c::valid_program_method(value) {
        return Err(owner::Invalid::Program);
    }
    bounded(value, c::PROGRAM_NAME)
}
fn state(value: &f::LoginState) -> Result<c::OwnerProgramState, owner::Invalid> {
    Ok(match value {
        f::LoginState::Usable => c::OwnerProgramState::Usable,
        f::LoginState::NotSignedIn => c::OwnerProgramState::NotSignedIn,
        f::LoginState::SignedInOtherwise(method) => {
            c::OwnerProgramState::SignedInOtherwise { method: method_name(method.as_bytes())? }
        }
        f::LoginState::Failed { reason, detail } => {
            c::OwnerProgramState::Failed { reason: failure(*reason), detail: bounded(detail, c::PROGRAM_DETAIL)? }
        }
    })
}
pub(super) fn logins(
    value: &List<d::ProgramLogin>,
    limits: &Limits,
) -> Result<List<c::OwnerProgramLogin>, owner::Invalid> {
    owner::list(value, limits, |row| {
        Ok(c::OwnerProgramLogin {
            name: program_name(&row.name)?,
            state: match &row.state {
                Some(value) => Some(state(value)?),
                None => None,
            },
            sign_out: match &row.sign_out {
                Some(f::SignOutResult::SignedOut) => Some(c::OwnerProgramSignOut::SignedOut),
                Some(f::SignOutResult::Failed { reason, detail }) => Some(c::OwnerProgramSignOut::Failed {
                    reason: failure(*reason),
                    detail: bounded(detail, c::PROGRAM_DETAIL)?,
                }),
                None => None,
            },
            cleanup: row.cleanup,
            cut: row.cut,
        })
    })
}
pub(super) fn hosts(
    value: &List<d::ProgramHost>,
    limits: &Limits,
) -> Result<List<c::OwnerProgramHost>, owner::Invalid> {
    owner::list(value, limits, |row| {
        Ok(c::OwnerProgramHost {
            name: configured_name(&row.name)?,
            channel: row.channel,
            connected: row.connected,
            awaiting_logins: row.awaiting_logins,
            programs: logins(&row.programs, limits)?,
            cut: row.cut,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> Limits {
        crate::test::limits().client.domain
    }
    fn host(name: &[u8]) -> List<d::ProgramHost> {
        crate::test::list([d::ProgramHost {
            name: name.into(),
            channel: 7,
            connected: true,
            awaiting_logins: false,
            programs: List::with_capacity(0),
            cut: false,
        }])
    }
    fn login(name: &[u8], state: Option<f::LoginState>) -> List<d::ProgramLogin> {
        crate::test::list([d::ProgramLogin { name: name.into(), state, sign_out: None, cleanup: false, cut: false }])
    }
    #[test]
    fn malformed_program_names_are_refused() {
        assert_eq!(logins(&login(b"not portable", None), &limits()), Err(owner::Invalid::Program));
    }
    #[test]
    fn malformed_configured_names_are_refused() {
        assert_eq!(hosts(&host(&[255]), &limits()), Err(owner::Invalid::Program));
    }
    #[test]
    fn configured_names_hold_at_the_wire_bound() {
        let mut name = vec![b'a'; usize::try_from(c::PROGRAM_HOST_NAME).expect("name bound")];
        let value = hosts(&host(&name), &limits()).expect("exact bound");
        assert_eq!(value.get(0).expect("host").name.as_ref(), name);
        name.push(b'a');
        assert_eq!(hosts(&host(&name), &limits()), Err(owner::Invalid::TooLarge));
    }
    #[test]
    fn program_details_hold_at_the_wire_bound() {
        let mut detail = vec![b'a'; usize::try_from(c::PROGRAM_DETAIL).expect("detail bound")];
        let rows = login(
            b"personal",
            Some(f::LoginState::Failed { reason: f::SourceFailure::Deadline, detail: detail.clone().into() }),
        );
        let value = logins(&rows, &limits()).expect("exact bound");
        assert_eq!(
            value.get(0).expect("login").state,
            Some(c::OwnerProgramState::Failed {
                reason: c::OwnerProgramFailure::Deadline,
                detail: detail.clone().into()
            })
        );
        detail.push(b'a');
        let rows = login(
            b"personal",
            Some(f::LoginState::Failed { reason: f::SourceFailure::Deadline, detail: detail.into() }),
        );
        assert_eq!(logins(&rows, &limits()), Err(owner::Invalid::TooLarge));
    }
    #[test]
    fn host_names_stay_whole_below_the_display_bound() {
        let limits = Limits { text: 1, turn_text: 1, ..limits() };
        let value = hosts(&host(b"configured-worker"), &limits).expect("independent name bound");
        assert_eq!(value.get(0).expect("host").name.as_ref(), b"configured-worker");
    }
}
