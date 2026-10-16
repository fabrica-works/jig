//! Party words are UTF-8 at request admission (client/domain/contract.md, 5.3).
use crate::{Ask, EscalationDecision, NoteChange, PersonResult, ProposalDecision};

pub(crate) fn valid(ask: &Ask) -> bool {
    match ask {
        Ask::StartChat { words, .. } | Ask::Say { words, .. } | Ask::AnswerQuestion { words, .. } => text(words),
        Ask::Propose { action: crate::Action::Batch(batch), reason, .. } => text(reason) && batch_words(batch),
        Ask::Create { batch, .. } => batch_words(batch),
        Ask::SetGoal { spec, .. } => text(spec),
        Ask::MakeService { name, .. } => text(name),
        Ask::Move { reason, .. } | Ask::Cancel { reason, .. } => text(reason),
        Ask::DecideEscalation { decision, .. } => match decision {
            EscalationDecision::Reject { reason } => text(reason),
            EscalationDecision::Release | EscalationDecision::Pass => true,
        },
        Ask::DecideProposal { decision, .. } => match decision {
            ProposalDecision::Reject { reason } => text(reason),
            ProposalDecision::Accept | ProposalDecision::Pass => true,
        },
        Ask::AnswerPerson { result, .. } => match result {
            PersonResult::Report { words } | PersonResult::Verdict { words, .. } => text(words),
            PersonResult::Failure { reason } => text(reason),
            PersonResult::Approved { words, .. } => match words {
                Some(words) => text(words),
                None => true,
            },
        },
        Ask::EditNote { change, .. } => match change.as_ref() {
            NoteChange::Correct { description, body, .. } => text(description) && text(body),
            NoteChange::Delete { .. } => true,
        },
        Ask::Amend { amendment, .. } => {
            if !text(&amendment.reason) {
                return false;
            }
            if let Some(instructions) = &amendment.instructions
                && !text(instructions)
            {
                return false;
            }
            if let Some(spec) = &amendment.spec {
                if !text(&spec.words) {
                    return false;
                }
                for parameter in &spec.parameters {
                    match parameter {
                        crate::Parameter::Bytes { value, .. } => {
                            if !text(value) {
                                return false;
                            }
                        }
                        crate::Parameter::Number { .. } | crate::Parameter::Resource { .. } => {}
                    }
                }
            }
            true
        }
        Ask::Compound { .. }
        | Ask::Close { .. }
        | Ask::MarkSeen { .. }
        | Ask::Application { .. }
        | Ask::Adopt { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::Stop { .. }
        | Ask::Release { .. }
        | Ask::SetRoles { .. }
        | Ask::ChangePolicy { .. }
        | Ask::SetPool { .. }
        | Ask::Prioritise { .. }
        | Ask::Withdraw { .. } => true,
    }
}

#[expect(clippy::disallowed_methods, reason = "client/domain/contract.md, 5.3 requires UTF-8 text validation")]
fn text(bytes: &[u8]) -> bool {
    core::str::from_utf8(bytes).is_ok()
}

fn batch_words(batch: &[crate::Member]) -> bool {
    for member in batch {
        if !text(&member.words) {
            return false;
        }
        for parameter in &member.parameters {
            match parameter {
                crate::Parameter::Bytes { value, .. } => {
                    if !text(value) {
                        return false;
                    }
                }
                crate::Parameter::Number { .. } | crate::Parameter::Resource { .. } => {}
            }
        }
    }
    true
}
