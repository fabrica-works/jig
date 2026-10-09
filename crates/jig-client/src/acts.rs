//! Objects offer the engine's acts verbatim; this module translates one act to one ask.
//! No authority, key or retry is computed here. Revisions come from the confirmation
//! that the person opened. See client/domain/objects.md, 2, 4–9 and 11.
use crate::{
    confirm::Confirmation,
    contract::{
        ActKind, Admission, Ask, Decision, Note, ObjectKey, Offered, PersonAnswer, PersonContract, Priority, Refusal,
        Revision, Text,
    },
    fields::{Fields, Purpose},
    objects::{Body, Object},
};
use alloc::boxed::Box;
use skein_lib::List;
/// Returns the engine's admission, or not offered when its document has no such act.
#[must_use]
pub fn offered(object: &Object, act: ActKind) -> Admission {
    let rows = match &object.body {
        Some(Body::Task(document)) => &document.acts,
        Some(Body::Proposal(document)) => &document.acts,
        Some(Body::Escalation(document)) => &document.acts,
        Some(Body::Question(document)) => &document.acts,
        Some(Body::PersonTask(document)) => &document.acts,
        Some(Body::Result(document)) => &document.acts,
        Some(Body::Note { document, .. }) => &document.acts,
        Some(Body::Chip(_)) | None => return Admission::Refused(Refusal::NotOffered),
    };
    admission(rows, act)
}
fn admission(rows: &List<Offered>, act: ActKind) -> Admission {
    for row in rows {
        if row.act == act {
            return row.admission.clone();
        }
    }
    Admission::Refused(Refusal::NotOffered)
}
/// The revision this act decides; unrelated spend and runtime facts are never named.
#[must_use]
pub fn revision(object: &Object, act: ActKind) -> Option<Revision> {
    match act {
        ActKind::Release | ActKind::LeaveHeld | ActKind::PassUp => match &object.body {
            Some(Body::Escalation(document)) => Some(document.hold.revision),
            Some(Body::Task(document)) => Some(document.hold.as_ref()?.revision),
            Some(
                Body::Chip(_)
                | Body::Proposal(_)
                | Body::Question(_)
                | Body::PersonTask(_)
                | Body::Result(_)
                | Body::Note { .. },
            )
            | None => None,
        },
        ActKind::Amend | ActKind::AmendRelease => match &object.body {
            Some(Body::Task(document)) => Some(document.spec_revision),
            Some(Body::Escalation(document)) => Some(document.steering.revision),
            Some(
                Body::Chip(_)
                | Body::Proposal(_)
                | Body::Question(_)
                | Body::PersonTask(_)
                | Body::Result(_)
                | Body::Note { .. },
            )
            | None => None,
        },
        ActKind::Answer => match &object.body {
            Some(Body::PersonTask(document)) => Some(document.revision),
            Some(
                Body::Chip(_)
                | Body::Task(_)
                | Body::Proposal(_)
                | Body::Escalation(_)
                | Body::Question(_)
                | Body::Result(_)
                | Body::Note { .. },
            )
            | None => None,
        },
        ActKind::EditNote | ActKind::DeleteNote => match &object.body {
            Some(Body::Note { document, .. }) => Some(document.revision),
            Some(
                Body::Chip(_)
                | Body::Task(_)
                | Body::Proposal(_)
                | Body::Escalation(_)
                | Body::Question(_)
                | Body::PersonTask(_)
                | Body::Result(_),
            )
            | None => None,
        },
        ActKind::Write
        | ActKind::Take
        | ActKind::HandBack
        | ActKind::Accept
        | ActKind::Reject
        | ActKind::Stop
        | ActKind::Cancel
        | ActKind::Close
        | ActKind::TakeOver
        | ActKind::SetGoal
        | ActKind::Prioritise
        | ActKind::MarkSeen
        | ActKind::Adopt
        | ActKind::ChangePolicy
        | ActKind::SetRoles
        | ActKind::MakeService
        | ActKind::SetPool => None,
    }
}
fn words(fields: &Fields, purpose: Purpose) -> Option<Text> {
    Some(fields.get(purpose)?.text.clone())
}
/// Builds one request after the confirmation has checked its current facts and fields.
#[must_use]
pub fn ask(object: &Object, confirmation: &Confirmation, fields: &Fields) -> Option<Ask> {
    let project = object.project?;
    let task = match object.key {
        ObjectKey::Task(task)
        | ObjectKey::Escalation(task)
        | ObjectKey::PersonTask(task)
        | ObjectKey::Result(task)
        | ObjectKey::Question { task, .. } => task,
        ObjectKey::Proposal(key) => key.proposer,
        ObjectKey::Note { note, .. } => note,
    };
    let choices = &confirmation.choices;
    match confirmation.act {
        ActKind::Accept | ActKind::Reject | ActKind::PassUp => decide(object, confirmation, fields, project),
        ActKind::LeaveHeld => Some(Ask::DecideEscalation {
            project,
            task,
            revision: confirmation.revision?,
            decision: Decision::LeftHeld { reason: words(fields, Purpose::Reason)? },
        }),
        ActKind::Release => {
            Some(Ask::Release { project, task, revision: confirmation.revision?, again: choices.again })
        }
        ActKind::Amend | ActKind::AmendRelease => Some(Ask::Amend {
            project,
            task,
            revision: confirmation.revision?,
            change: choices.change.clone()?,
            release: confirmation.act == ActKind::AmendRelease,
        }),
        ActKind::Answer => answer(object, confirmation, fields, project, task),
        ActKind::Take => Some(Ask::Take { project, task }),
        ActKind::HandBack => Some(Ask::HandBack { project, task }),
        ActKind::Write => Some(Ask::Write {
            project,
            task,
            words: words(fields, Purpose::Composer { task })?,
            release: choices.release,
        }),
        ActKind::Stop => Some(Ask::Stop { project, task }),
        ActKind::Cancel => Some(Ask::Cancel { project, task, reason: words(fields, Purpose::Reason)? }),
        ActKind::Close => Some(Ask::Close { project, chat: task, disposals: choices.disposals.clone() }),
        ActKind::TakeOver => Some(Ask::TakeOver { project, task, reason: words(fields, Purpose::Reason)? }),
        ActKind::Prioritise => {
            let mut goals = List::with_capacity(1);
            goals.push(Priority { task, priority: choices.priority? }).expect("one priority fits");
            Some(Ask::Prioritise { project, goals })
        }
        ActKind::EditNote | ActKind::DeleteNote => {
            let note = match object.key {
                ObjectKey::Note { note, .. } => note,
                ObjectKey::Task(_)
                | ObjectKey::Proposal(_)
                | ObjectKey::Escalation(_)
                | ObjectKey::Question { .. }
                | ObjectKey::PersonTask(_)
                | ObjectKey::Result(_) => return None,
            };
            if confirmation.act == ActKind::DeleteNote {
                Some(Ask::DeleteNote { project, note, revision: confirmation.revision? })
            } else {
                Some(Ask::EditNote {
                    project,
                    note,
                    revision: confirmation.revision?,
                    description: words(fields, Purpose::NoteDescription)?,
                    body: words(fields, Purpose::NoteBody)?,
                })
            }
        }
        ActKind::SetGoal
        | ActKind::MarkSeen
        | ActKind::Adopt
        | ActKind::ChangePolicy
        | ActKind::SetRoles
        | ActKind::MakeService
        | ActKind::SetPool => None,
    }
}
fn decide(object: &Object, confirmation: &Confirmation, fields: &Fields, project: u32) -> Option<Ask> {
    match object.key {
        ObjectKey::Proposal(proposal) => {
            let decision = match confirmation.act {
                ActKind::Accept => Decision::Accepted,
                ActKind::Reject => Decision::Rejected { reason: words(fields, Purpose::Reason)? },
                ActKind::PassUp => Decision::PassedUp,
                ActKind::Write
                | ActKind::Answer
                | ActKind::Take
                | ActKind::HandBack
                | ActKind::Release
                | ActKind::LeaveHeld
                | ActKind::Amend
                | ActKind::AmendRelease
                | ActKind::Stop
                | ActKind::Cancel
                | ActKind::Close
                | ActKind::TakeOver
                | ActKind::SetGoal
                | ActKind::Prioritise
                | ActKind::MarkSeen
                | ActKind::EditNote
                | ActKind::DeleteNote
                | ActKind::Adopt
                | ActKind::ChangePolicy
                | ActKind::SetRoles
                | ActKind::MakeService
                | ActKind::SetPool => return None,
            };
            Some(Ask::DecideProposal { project, proposal, decision })
        }
        ObjectKey::Escalation(task) => {
            if confirmation.act != ActKind::PassUp {
                return None;
            }
            Some(Ask::DecideEscalation {
                project,
                task,
                revision: confirmation.revision?,
                decision: Decision::PassedUp,
            })
        }
        ObjectKey::Task(_)
        | ObjectKey::Question { .. }
        | ObjectKey::PersonTask(_)
        | ObjectKey::Result(_)
        | ObjectKey::Note { .. } => None,
    }
}

fn answer(object: &Object, confirmation: &Confirmation, fields: &Fields, project: u32, task: u64) -> Option<Ask> {
    let choices = &confirmation.choices;
    match &object.body {
        Some(Body::Question(question)) => Some(Ask::AnswerQuestion {
            project,
            task,
            message: question.message,
            words: words(fields, Purpose::Answer)?,
        }),
        Some(Body::PersonTask(document)) => {
            let answer = if choices.cannot {
                PersonAnswer::Cannot { reason: words(fields, Purpose::Reason)? }
            } else {
                match &document.contract {
                    PersonContract::Choice { options } => {
                        let option = choices.option?;
                        let mut present = false;
                        for row in options {
                            if row.number == option {
                                present = true;
                            }
                        }
                        if !present {
                            return None;
                        }
                        let reason = match words(fields, Purpose::Reason) {
                            Some(words) => words,
                            None => Text::new(Box::from([]), 0).expect("empty text fits"),
                        };
                        PersonAnswer::Choice { option, reason }
                    }
                    PersonContract::Approval { state, .. } => {
                        let reason = match words(fields, Purpose::Reason) {
                            Some(words) => words,
                            None => Text::new(Box::from([]), 0).expect("empty text fits"),
                        };
                        PersonAnswer::Approval { state: state.clone(), approved: true, reason }
                    }
                    PersonContract::Words => PersonAnswer::Words(words(fields, Purpose::Answer)?),
                }
            };
            Some(Ask::AnswerPerson { project, task, revision: confirmation.revision?, answer })
        }
        Some(
            Body::Chip(_)
            | Body::Task(_)
            | Body::Proposal(_)
            | Body::Escalation(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => None,
    }
}

/// A note edit or deletion uses its recalled revision and its own offered acts.
#[must_use]
pub fn note_ask(project: u32, note: &Note, act: ActKind, description: Text, body: Text) -> Option<Ask> {
    match admission(&note.acts, act) {
        Admission::Allowed | Admission::Proposed { .. } => {}
        Admission::Bounded { .. } | Admission::Refused(_) => return None,
    }
    match act {
        ActKind::EditNote => {
            Some(Ask::EditNote { project, note: note.number, revision: note.revision, description, body })
        }
        ActKind::DeleteNote => Some(Ask::DeleteNote { project, note: note.number, revision: note.revision }),
        ActKind::Write
        | ActKind::Answer
        | ActKind::Take
        | ActKind::HandBack
        | ActKind::Accept
        | ActKind::Reject
        | ActKind::PassUp
        | ActKind::Release
        | ActKind::LeaveHeld
        | ActKind::Amend
        | ActKind::AmendRelease
        | ActKind::Stop
        | ActKind::Cancel
        | ActKind::Close
        | ActKind::TakeOver
        | ActKind::SetGoal
        | ActKind::Prioritise
        | ActKind::MarkSeen
        | ActKind::Adopt
        | ActKind::ChangePolicy
        | ActKind::SetRoles
        | ActKind::MakeService
        | ActKind::SetPool => None,
    }
}
