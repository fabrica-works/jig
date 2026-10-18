//! Objects offer the engine's acts verbatim; this module translates one act to one ask.
//! Adoption stays in its connector form; generic confirmation emits no adoption ask (pages.md, 6.5).
//! No authority, key or retry is computed here. Revisions come from the confirmation
//! that the person opened. See client/domain/objects.md, 2, 4–9 and 11.
use crate::{
    confirm::Confirmation,
    fields::{Fields, Purpose},
    objects::{Body, Object},
};
use alloc::boxed::Box;
use jig_client::contract::{
    ActKind, Admission, Ask, Decision, Note, ObjectKey, Offered, PersonAnswer, PersonContract, Refusal, Revision, Text,
};
use jig_client::contract::{AnswerOn, DecideOn};
use skein_lib::List;
/// Returns the engine's admission, or not offered when its document has no such act.
#[must_use]
pub fn offered(object: &Object, act: ActKind) -> Admission {
    let admission = match offers(object) {
        Some(rows) => jig_client::facts::offer(rows.as_slice(), act),
        None => None,
    };
    match admission {
        Some(value) => value.clone(),
        None => Admission::Refused(Refusal::NotOffered),
    }
}
/// The engine's whole offer, including its consequence.
#[must_use]
pub fn offer(object: &Object, act: ActKind) -> Option<&Offered> {
    offer_in(offers(object)?, act)
}
pub(crate) fn offers(object: &Object) -> Option<&List<Offered>> {
    let rows = match &object.body {
        Some(Body::Task(document)) => &document.acts,
        Some(Body::Proposal(document)) => &document.acts,
        Some(Body::OwnerProposal(document)) => &document.acts,
        Some(Body::Escalation(document)) => &document.acts,
        Some(Body::OwnerEscalation(document)) => &document.acts,
        Some(Body::Question(document)) => &document.acts,
        Some(Body::PersonTask(document)) => &document.acts,
        Some(Body::OwnerPersonTask(document)) => &document.acts,
        Some(Body::Result(document)) => &document.acts,
        Some(Body::Note { document, .. }) => &document.acts,
        Some(Body::Historical(_) | Body::Chip(_)) | None => return None,
    };
    Some(rows)
}
#[expect(clippy::manual_find, reason = "step code selects offers without closures")]
fn offer_in(rows: &List<Offered>, act: ActKind) -> Option<&Offered> {
    for row in rows {
        if row.act == act {
            return Some(row);
        }
    }
    None
}
/// The revision this act decides; unrelated spend and runtime facts are never named.
#[must_use]
pub fn revision(object: &Object, act: ActKind) -> Option<Revision> {
    match act {
        ActKind::Release | ActKind::LeaveHeld | ActKind::PassUp => match &object.body {
            Some(Body::Escalation(document)) => Some(document.hold.revision),
            Some(Body::OwnerEscalation(document)) => Some(document.task.hold.as_ref()?.revision()),
            Some(Body::Task(document)) => Some(document.hold.as_ref()?.revision()),
            Some(
                Body::Historical(_)
                | Body::Chip(_)
                | Body::Proposal(_)
                | Body::OwnerProposal(_)
                | Body::Question(_)
                | Body::PersonTask(_)
                | Body::OwnerPersonTask(_)
                | Body::Result(_)
                | Body::Note { .. },
            )
            | None => None,
        },
        ActKind::Amend => match &object.body {
            Some(Body::Task(document)) => Some(document.spec_revision),
            Some(Body::Escalation(document)) => Some(document.steering.revision),
            Some(Body::OwnerEscalation(document)) => Some(document.task.spec_revision),
            Some(
                Body::Historical(_)
                | Body::Chip(_)
                | Body::Proposal(_)
                | Body::OwnerProposal(_)
                | Body::Question(_)
                | Body::PersonTask(_)
                | Body::OwnerPersonTask(_)
                | Body::Result(_)
                | Body::Note { .. },
            )
            | None => None,
        },
        ActKind::Answer => match &object.body {
            Some(Body::PersonTask(document)) => Some(document.revision),
            Some(Body::OwnerPersonTask(document)) => Some(document.revision),
            Some(
                Body::Historical(_)
                | Body::Chip(_)
                | Body::Task(_)
                | Body::Proposal(_)
                | Body::OwnerProposal(_)
                | Body::Escalation(_)
                | Body::OwnerEscalation(_)
                | Body::Question(_)
                | Body::Result(_)
                | Body::Note { .. },
            )
            | None => None,
        },
        ActKind::EditNote | ActKind::DeleteNote => match &object.body {
            Some(Body::Note { document, .. }) => Some(document.revision),
            Some(
                Body::Historical(_)
                | Body::Chip(_)
                | Body::Task(_)
                | Body::Proposal(_)
                | Body::OwnerProposal(_)
                | Body::Escalation(_)
                | Body::OwnerEscalation(_)
                | Body::Question(_)
                | Body::PersonTask(_)
                | Body::OwnerPersonTask(_)
                | Body::Result(_),
            )
            | None => None,
        },
        ActKind::Message
        | ActKind::Take
        | ActKind::HandBack
        | ActKind::Withdraw
        | ActKind::Accept
        | ActKind::Reject
        | ActKind::Stop
        | ActKind::Cancel
        | ActKind::Close
        | ActKind::Move
        | ActKind::SetGoal
        | ActKind::Prioritise
        | ActKind::Seen
        | ActKind::Adopt
        | ActKind::ChangePolicy
        | ActKind::SetRoles
        | ActKind::MakeService
        | ActKind::SetPool => None,
    }
}
/// The hold displayed independently of the task's spec revision.
#[must_use]
pub fn hold_revision(object: &Object) -> Option<Revision> {
    match &object.body {
        Some(Body::Task(document)) => {
            let Some(hold) = &document.hold else {
                return None;
            };
            Some(hold.revision())
        }
        Some(Body::Escalation(document)) => Some(document.hold.revision),
        Some(Body::OwnerEscalation(document)) => Some(document.task.hold.as_ref()?.revision()),
        Some(
            Body::Historical(_)
            | Body::Chip(_)
            | Body::Proposal(_)
            | Body::OwnerProposal(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => None,
    }
}
pub(crate) fn words(fields: &Fields, purpose: Purpose) -> Option<Text> {
    Some(fields.get(purpose)?.text.clone())
}
pub(crate) fn request_project(object: &Object) -> Option<u32> {
    match &object.body {
        Some(Body::OwnerPersonTask(document)) => document.project(),
        Some(Body::OwnerEscalation(document)) => Some(document.task.project),
        Some(
            Body::Historical(_)
            | Body::Chip(_)
            | Body::Task(_)
            | Body::Proposal(_)
            | Body::OwnerProposal(_)
            | Body::Escalation(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => object.project,
    }
}
/// Builds one request after the confirmation has checked its current facts and fields.
#[must_use]
pub fn ask(object: &Object, confirmation: &Confirmation, fields: &Fields) -> Option<Ask> {
    let project = request_project(object)?;
    let task = match object.key {
        ObjectKey::Task(task)
        | ObjectKey::Escalation(task)
        | ObjectKey::PersonTask(task)
        | ObjectKey::Result(task)
        | ObjectKey::Question { task, .. } => task,
        ObjectKey::Proposal(_) => return decide(object, confirmation, fields, project),
        ObjectKey::Note { note, .. } => note,
    };
    let choices = &confirmation.choices;
    match confirmation.act {
        ActKind::Accept | ActKind::Reject | ActKind::PassUp | ActKind::Withdraw => {
            decide(object, confirmation, fields, project)
        }
        ActKind::LeaveHeld => Some(Ask::Decide {
            project,
            on: DecideOn::Escalation {
                task,
                revision: confirmation.revision?,
                decision: Decision::LeftHeld { reason: words(fields, Purpose::Reason)? },
            },
        }),
        ActKind::Release => {
            Some(Ask::Release { project, task, revision: confirmation.revision?, again: choices.again })
        }
        ActKind::Amend => amend(confirmation, fields, project, task),

        ActKind::Answer => answer(object, confirmation, fields, project, task),
        ActKind::Take => Some(Ask::Take { project, task }),
        ActKind::HandBack => Some(Ask::HandBack { project, task }),
        ActKind::Message => {
            let words = words(fields, Purpose::Composer { task })?;
            if choices.release {
                Some(releasing(
                    project,
                    task,
                    confirmation.release_revision?,
                    jig_client::contract::Operation::Message { project, task, words },
                ))
            } else {
                Some(Ask::Message { project, task, words })
            }
        }
        ActKind::Stop => Some(Ask::Stop { project, task }),
        ActKind::Cancel => Some(Ask::Cancel { project, task, reason: words(fields, Purpose::Reason)?, grace: None }),
        ActKind::Move => Some(Ask::Move { project, task, reason: words(fields, Purpose::Reason)? }),
        ActKind::Prioritise => Some(priority(project, task, choices.priority?)),
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
            let Some(Body::Note { document, .. }) = &object.body else {
                return None;
            };
            if confirmation.act == ActKind::DeleteNote {
                Some(Ask::DeleteNote { project, note, scope: document.scope.clone(), revision: confirmation.revision? })
            } else {
                if document.cut || document.description.cut || document.body.cut {
                    return None;
                }
                Some(Ask::EditNote {
                    project,
                    note,
                    scope: document.scope.clone(),
                    revision: confirmation.revision?,
                    description: words(fields, Purpose::NoteDescription)?,
                    body: words(fields, Purpose::NoteBody)?,
                    references: document.tasks.clone(),
                })
            }
        }
        ActKind::Close
        | ActKind::SetGoal
        | ActKind::Seen
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
            if confirmation.act == ActKind::Withdraw {
                return Some(Ask::Withdraw { project, proposal: proposal.number });
            }
            let decision = match confirmation.act {
                ActKind::Accept => Decision::Accepted,
                ActKind::Reject => Decision::Rejected { reason: words(fields, Purpose::Reason)? },
                ActKind::PassUp => Decision::PassedUp,
                ActKind::Withdraw
                | ActKind::Message
                | ActKind::Answer
                | ActKind::Take
                | ActKind::HandBack
                | ActKind::Release
                | ActKind::LeaveHeld
                | ActKind::Amend
                | ActKind::Stop
                | ActKind::Cancel
                | ActKind::Close
                | ActKind::Move
                | ActKind::SetGoal
                | ActKind::Prioritise
                | ActKind::Seen
                | ActKind::EditNote
                | ActKind::DeleteNote
                | ActKind::Adopt
                | ActKind::ChangePolicy
                | ActKind::SetRoles
                | ActKind::MakeService
                | ActKind::SetPool => return None,
            };
            Some(Ask::Decide { project, on: DecideOn::Proposal { proposal, decision } })
        }
        ObjectKey::Escalation(task) => {
            if confirmation.act != ActKind::PassUp {
                return None;
            }
            Some(Ask::Decide {
                project,
                on: DecideOn::Escalation { task, revision: confirmation.revision?, decision: Decision::PassedUp },
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
        Some(Body::Question(question)) => Some(Ask::Answer {
            project,
            on: AnswerOn::Question { task, message: question.message, words: words(fields, Purpose::Answer)? },
        }),
        Some(Body::OwnerPersonTask(document)) => {
            let answer = if choices.cannot {
                PersonAnswer::Cannot { reason: words(fields, Purpose::Reason)? }
            } else {
                crate::objects::person_task::answer(document, choices.option, fields)?
            };
            Some(Ask::Answer { project, on: AnswerOn::Person { task, revision: confirmation.revision?, answer } })
        }
        Some(Body::PersonTask(document)) => {
            let answer = if choices.cannot {
                PersonAnswer::Cannot { reason: words(fields, Purpose::Reason)? }
            } else {
                match &document.contract {
                    PersonContract::Choice { options } => {
                        let option = choices.option?;
                        let mut present = false;
                        for row in options {
                            if row.number == option && row.fields.is_empty() {
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
                    PersonContract::Approval { state, .. } => PersonAnswer::Approved {
                        state: state.clone().into_bytes(),
                        words: words(fields, Purpose::Reason),
                    },
                    PersonContract::Words => PersonAnswer::Words(words(fields, Purpose::Answer)?),
                }
            };
            Some(Ask::Answer { project, on: AnswerOn::Person { task, revision: confirmation.revision?, answer } })
        }
        Some(
            Body::Historical(_)
            | Body::Chip(_)
            | Body::Task(_)
            | Body::Proposal(_)
            | Body::OwnerProposal(_)
            | Body::Escalation(_)
            | Body::OwnerEscalation(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => None,
    }
}

/// A note edit or deletion uses its recalled revision and its own offered acts.
#[must_use]
pub fn note_ask(project: u32, note: &Note, act: ActKind, description: Text, body: Text) -> Option<Ask> {
    let offer = offer_in(&note.acts, act)?;
    match &offer.admission {
        Admission::Allowed | Admission::Proposed { .. } => {}
        Admission::Bounded { .. } | Admission::Refused(_) => return None,
    }
    match act {
        ActKind::EditNote => {
            if note.cut || note.description.cut || note.body.cut {
                return None;
            }
            Some(Ask::EditNote {
                project,
                note: note.number,
                scope: note.scope.clone(),
                revision: note.revision,
                description,
                body,
                references: note.tasks.clone(),
            })
        }
        ActKind::DeleteNote => {
            Some(Ask::DeleteNote { project, note: note.number, scope: note.scope.clone(), revision: note.revision })
        }
        ActKind::Message
        | ActKind::Answer
        | ActKind::Take
        | ActKind::HandBack
        | ActKind::Withdraw
        | ActKind::Accept
        | ActKind::Reject
        | ActKind::PassUp
        | ActKind::Release
        | ActKind::LeaveHeld
        | ActKind::Amend
        | ActKind::Stop
        | ActKind::Cancel
        | ActKind::Close
        | ActKind::Move
        | ActKind::SetGoal
        | ActKind::Prioritise
        | ActKind::Seen
        | ActKind::Adopt
        | ActKind::ChangePolicy
        | ActKind::SetRoles
        | ActKind::MakeService
        | ActKind::SetPool => None,
    }
}

fn releasing(project: u32, task: u64, revision: Revision, first: jig_client::contract::Operation) -> Ask {
    let mut operations = List::with_capacity(2);
    operations.push(first).expect("two operations");
    operations
        .push(jig_client::contract::Operation::Release { project, task, revision, again: None })
        .expect("two operations");
    Ask::Compound { project, operations }
}

fn amend(confirmation: &Confirmation, fields: &Fields, project: u32, task: u64) -> Option<Ask> {
    let choices = &confirmation.choices;

    if confirmation.reason_input == crate::confirm::ReasonInput::Missing {
        return None;
    }
    let change = Box::new(choices.change.clone()?);
    let revision = Some(confirmation.revision?);
    let reason = words(fields, Purpose::Reason)?;
    if choices.release {
        Some(releasing(
            project,
            task,
            confirmation.release_revision?,
            jig_client::contract::Operation::Amend { project, task, revision, change, reason },
        ))
    } else if confirmation.proposed.is_some() {
        Some(Ask::Propose {
            project,
            action: jig_client::contract::ProposeAction::Amendment { task, revision: confirmation.revision?, change },
            reason,
        })
    } else {
        Some(Ask::Amend { project, task, revision, change, reason })
    }
}

fn priority(project: u32, task: u64, priority: u32) -> Ask {
    let mut operations = List::with_capacity(1);
    operations
        .push(jig_client::contract::Operation::Amend {
            project,
            task,
            revision: None,
            change: Box::new(jig_client::contract::Amendment {
                addressed: None,
                spec: None,
                wake: None,
                dependencies: None,
                authority: None,
                instructions: None,
                tracked: Some(jig_client::contract::Tracking::Tracked { priority }),
            }),
            reason: Text::new(Box::from([]), 0).expect("empty reason fits"),
        })
        .expect("one operation fits");
    Ask::Compound { project, operations }
}
