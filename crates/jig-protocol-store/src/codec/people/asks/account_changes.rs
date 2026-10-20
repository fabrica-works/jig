//! Holder account changes retain only LLM key digests (domain/accounts.md, section 5).
use super::{domain, wire};
use alloc::boxed::Box;
use skein_lib::{List, bytes};
pub(super) fn encode(
    value: &domain::AccountChange,
    limits: &wire::Limits,
) -> Result<wire::AccountChange, wire::Problem> {
    Ok(match value {
        domain::AccountChange::DeclareProgram { provider, join, name } => {
            wire::AccountChange::DeclareProgram(wire::AccountChangeDeclareProgram::new(
                limits,
                wire::AccountChangeDeclareProgramParts { provider: *provider, join: *join, name: name.clone() },
            )?)
        }
        domain::AccountChange::CheckProgram { source } => wire::AccountChange::CheckProgram(
            wire::AccountChangeCheckProgram::new(limits, wire::AccountChangeCheckProgramParts { source: *source })?,
        ),
        domain::AccountChange::AddKey { provider, join, digest } => {
            wire::AccountChange::AddKey(wire::AccountChangeAddKey::new(
                limits,
                wire::AccountChangeAddKeyParts { provider: *provider, join: *join, digest: bytes::copy_of(digest) },
            )?)
        }
        domain::AccountChange::ReplaceKey { source, digest } => {
            wire::AccountChange::ReplaceKey(wire::AccountChangeReplaceKey::new(
                limits,
                wire::AccountChangeReplaceKeyParts { source: *source, digest: bytes::copy_of(digest) },
            )?)
        }
        domain::AccountChange::RemoveKey { source } => wire::AccountChange::RemoveKey(
            wire::AccountChangeRemoveKey::new(limits, wire::AccountChangeRemoveKeyParts { source: *source })?,
        ),
        domain::AccountChange::Resume { source } => wire::AccountChange::Resume(wire::AccountChangeResume::new(
            limits,
            wire::AccountChangeResumeParts { source: *source },
        )?),
        domain::AccountChange::Remove { account } => wire::AccountChange::Remove(wire::AccountChangeRemove::new(
            limits,
            wire::AccountChangeRemoveParts { account: *account },
        )?),
        domain::AccountChange::MoveAccount { account, place } => {
            wire::AccountChange::MoveAccount(wire::AccountChangeMoveAccount::new(
                limits,
                wire::AccountChangeMoveAccountParts { account: *account, place: *place },
            )?)
        }
        domain::AccountChange::MoveSource { account, source, place } => {
            wire::AccountChange::MoveSource(wire::AccountChangeMoveSource::new(
                limits,
                wire::AccountChangeMoveSourceParts { account: *account, source: *source, place: *place },
            )?)
        }
        domain::AccountChange::SetUses { account, uses } => {
            wire::AccountChange::SetUses(wire::AccountChangeSetUses::new(
                limits,
                wire::AccountChangeSetUsesParts { account: *account, uses: encode_uses(uses.as_deref(), limits)? },
            )?)
        }
    })
}
pub(super) fn decode(value: wire::AccountChange) -> Result<domain::AccountChange, wire::Problem> {
    Ok(match value {
        wire::AccountChange::DeclareProgram(value) => {
            let value = value.into_parts();
            if !domain::program_name(&value.name) {
                return Err(wire::Problem {
                    path: wire::Path::AccountChangeDeclareProgramName,
                    reason: skein_codec::Reason::Malformed,
                });
            }
            domain::AccountChange::DeclareProgram { provider: value.provider, join: value.join, name: value.name }
        }
        wire::AccountChange::CheckProgram(value) => domain::AccountChange::CheckProgram { source: value.source() },
        wire::AccountChange::AddKey(value) => {
            let value = value.into_parts();
            domain::AccountChange::AddKey {
                provider: value.provider,
                join: value.join,
                digest: digest(value.digest, wire::Path::AccountChangeAddKeyDigest)?,
            }
        }
        wire::AccountChange::ReplaceKey(value) => {
            let value = value.into_parts();
            domain::AccountChange::ReplaceKey {
                source: value.source,
                digest: digest(value.digest, wire::Path::AccountChangeReplaceKeyDigest)?,
            }
        }
        wire::AccountChange::RemoveKey(value) => {
            let value = value.into_parts();
            domain::AccountChange::RemoveKey { source: value.source }
        }
        wire::AccountChange::Resume(value) => {
            let value = value.into_parts();
            domain::AccountChange::Resume { source: value.source }
        }
        wire::AccountChange::Remove(value) => {
            let value = value.into_parts();
            domain::AccountChange::Remove { account: value.account }
        }
        wire::AccountChange::MoveAccount(value) => {
            let value = value.into_parts();
            domain::AccountChange::MoveAccount { account: value.account, place: value.place }
        }
        wire::AccountChange::MoveSource(value) => {
            let value = value.into_parts();
            domain::AccountChange::MoveSource { account: value.account, source: value.source, place: value.place }
        }
        wire::AccountChange::SetUses(value) => {
            let value = value.into_parts();
            domain::AccountChange::SetUses { account: value.account, uses: decode_uses(value.uses) }
        }
    })
}
fn encode_uses(
    uses: Option<&[domain::Use]>,
    limits: &wire::Limits,
) -> Result<Option<List<wire::AccountUse>>, wire::Problem> {
    let Some(uses) = uses else { return Ok(None) };
    let mut out = List::with_capacity(limits.account_change_set_uses_uses_some);
    for value in uses {
        let value = match value {
            domain::Use::Party(number) => wire::AccountUse::Party(wire::AccountUseParty::new(
                limits,
                wire::AccountUsePartyParts { number: *number },
            )?),
            domain::Use::Project(number) => wire::AccountUse::Project(wire::AccountUseProject::new(
                limits,
                wire::AccountUseProjectParts { number: *number },
            )?),
        };
        if out.push(value).is_err() {
            return Err(wire::Problem {
                path: wire::Path::AccountChangeSetUsesUses,
                reason: skein_codec::Reason::Bound,
            });
        }
    }
    Ok(Some(out))
}
fn decode_uses(uses: Option<List<wire::AccountUse>>) -> Option<Box<[domain::Use]>> {
    let uses = uses?;
    let mut out = List::with_capacity(uses.len());
    for value in uses.into_boxed() {
        out.push(match value {
            wire::AccountUse::Party(value) => domain::Use::Party(value.into_parts().number),
            wire::AccountUse::Project(value) => domain::Use::Project(value.into_parts().number),
        })
        .expect("account uses room");
    }
    Some(out.into_boxed())
}
fn digest(bytes: Box<[u8]>, path: wire::Path) -> Result<[u8; 32], wire::Problem> {
    match bytes.as_ref().try_into() {
        Ok(value) => Ok(value),
        Err(_) => Err(wire::Problem { path, reason: skein_codec::Reason::Bound }),
    }
}
