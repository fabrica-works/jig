//! Program observations are content in the existing deployment host row.
use super::{P, Words, list, text};
use crate::Limits;
use crate::owner_deployment_words::ProgramPhrase as G;
use jig_client::contract::{OwnerProgramFailure as F, OwnerProgramLogin, OwnerProgramSignOut, OwnerProgramState};
use skein_lib::List;
use skein_ui::{Builder, Role};
fn failure(b: &mut Builder, words: &Words, reason: F) {
    let phrase = match reason {
        F::Missing => G::Missing,
        F::Deadline => G::Deadline,
        F::Unreadable => G::Unreadable,
        F::Exited => G::Exited,
        F::Unconfigured => G::Unconfigured,
        F::Unknown => G::Unknown,
        F::Closed => G::Closed,
    };
    text(b, words, P::ProgramDetail(phrase), &[]);
}
pub(super) fn render(b: &mut Builder, words: &Words, rows: &List<OwnerProgramLogin>, awaiting: bool) {
    if rows.is_empty() && !awaiting {
        return;
    }
    list(b, words, P::ProgramDetail(G::Programs));
    text(b, words, P::ProgramDetail(if awaiting { G::Awaiting } else { G::Received }), &[]);
    for row in rows {
        b.open(Role::Item);
        crate::literal_segment::render(b, &row.name);
        match &row.state {
            None => text(b, words, P::ProgramDetail(G::StatusUnknown), &[]),
            Some(OwnerProgramState::Usable) => text(b, words, P::ProgramDetail(G::Usable), &[]),
            Some(OwnerProgramState::NotSignedIn) => text(b, words, P::ProgramDetail(G::NotSignedIn), &[]),
            Some(OwnerProgramState::SignedInOtherwise { method }) => {
                text(b, words, P::ProgramDetail(G::Otherwise), &[]);
                crate::literal_segment::render(b, method);
            }
            Some(OwnerProgramState::Failed { reason, detail }) => {
                text(b, words, P::ProgramDetail(G::Failed), &[]);
                failure(b, words, *reason);
                crate::literal_segment::render(b, detail);
            }
        }
        if row.cleanup {
            text(b, words, P::ProgramDetail(G::Cleanup), &[]);
        }
        match &row.sign_out {
            None => {}
            Some(OwnerProgramSignOut::SignedOut) => text(b, words, P::ProgramDetail(G::SignedOut), &[]),
            Some(OwnerProgramSignOut::Failed { reason, detail }) => {
                text(b, words, P::ProgramDetail(G::SignOutFailed), &[]);
                failure(b, words, *reason);
                crate::literal_segment::render(b, detail);
            }
        }
        if row.cut {
            text(b, words, P::Cut, &[]);
        }
        b.close();
    }
    b.close();
}
pub(super) fn fits(rows: &List<OwnerProgramLogin>, limits: &Limits) -> Option<()> {
    if rows.capacity() > limits.rows {
        return None;
    }
    for row in rows {
        if !jig_client::contract::valid_program_name(&row.name) || row.name.len() > usize::try_from(limits.text).ok()? {
            return None;
        }
        match &row.state {
            Some(OwnerProgramState::SignedInOtherwise { method }) => {
                if !jig_client::contract::valid_program_method(method)
                    || method.len() > usize::try_from(limits.text).ok()?
                {
                    return None;
                }
            }
            Some(OwnerProgramState::Failed { detail, .. }) => {
                if detail.len() > usize::try_from(limits.text).ok()? {
                    return None;
                }
            }
            None | Some(OwnerProgramState::Usable | OwnerProgramState::NotSignedIn) => {}
        }
        match &row.sign_out {
            Some(OwnerProgramSignOut::Failed { detail, .. }) => {
                if detail.len() > usize::try_from(limits.text).ok()? {
                    return None;
                }
            }
            None | Some(OwnerProgramSignOut::SignedOut) => {}
        }
    }
    Some(())
}
