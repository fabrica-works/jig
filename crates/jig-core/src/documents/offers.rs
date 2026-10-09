//! Whole party-specific offers (`client/domain/contract.md`, 5.3;
//! `client/domain/objects.md`, 11). The acts owner supplies every judgment.
use super::{DocumentLimits, Missing, Subject};
use crate::acts::{self, Act, Admission, Beyond, Bounds, Offered, Recipient};
use crate::{Core, Limits};
use alloc::boxed::Box;
use jig_core_people as people;
use jig_core_tasks as tasks;
use skein_lib::List;

const MAX_ACTS: u32 = 25;

fn receiving_recipient(value: Recipient, limits: &DocumentLimits) -> Result<Recipient, Missing> {
    match value {
        Recipient::Roles { project, roles } => {
            if roles.len() > limits.lines {
                return Err(Missing::TooLarge);
            }
            let mut received = List::with_capacity(limits.lines);
            for role in &roles {
                received.push(*role).expect("whole policy role list fits");
            }
            Ok(Recipient::Roles { project, roles: received })
        }
        other @ (Recipient::Task(_) | Recipient::Party(_) | Recipient::Role { .. }) => Ok(other),
    }
}
fn receiving(value: Admission, limits: &DocumentLimits) -> Result<Admission, Missing> {
    Ok(match value {
        other @ (Admission::Allowed | Admission::Refused(_)) => other,
        Admission::Proposed { to } => Admission::Proposed { to: receiving_recipient(to, limits)? },
        Admission::Bounded { within, beyond } => {
            let fits = match &within {
                Bounds::Goal { charters, .. } => charters.len() <= limits.lines,
                Bounds::Amend { authority, ceiling, grants, .. } => {
                    !authority.cut && !ceiling.cut && grants.len() <= limits.lines
                }
            };
            if !fits {
                return Err(Missing::TooLarge);
            }
            let beyond = match beyond {
                Beyond::Proposed { to } => Beyond::Proposed { to: receiving_recipient(to, limits)? },
                other @ Beyond::Refused(_) => other,
            };
            Admission::Bounded { within, beyond }
        }
    })
}
fn list(core: &Core, limits: &Limits, party: u64, subject: &Subject, kinds: &[Act]) -> Result<List<Offered>, Missing> {
    let mut result = List::with_capacity(u32::try_from(kinds.len()).expect("closed act list"));
    for act in kinds {
        let value = acts::admit(core, limits, party, *act, subject);
        if (*act == Act::Amend || *act == Act::AmendRelease)
            && value == Admission::Refused(people::Refusal::Limit { which: people::LimitKind::Authority })
        {
            return Err(Missing::TooLarge);
        }
        let admission = receiving(value, &limits.documents)?;
        result.push(Offered { act: *act, admission }).expect("one offer per listed act");
    }
    if heap_bytes(&result).ok_or(Missing::TooLarge)? > heap_worst_case(&limits.documents).ok_or(Missing::TooLarge)? {
        return Err(Missing::TooLarge);
    }
    Ok(result)
}

pub(super) fn task(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &tasks::TaskRecord,
) -> Result<List<Offered>, Missing> {
    let mut kinds = List::with_capacity(9);
    let mutable = match row.phase {
        tasks::Phase::Waiting
        | tasks::Phase::Active(_)
        | tasks::Phase::Held { was: tasks::Was::Waiting | tasks::Was::Active(_), .. } => true,
        tasks::Phase::Closing(_) | tasks::Phase::Held { was: tasks::Was::Closing(_), .. } | tasks::Phase::Ended(_) => {
            false
        }
    };
    if mutable {
        if let tasks::Executor::Agent { .. } = row.executor {
            kinds.push(Act::Write).expect("task act bound");
        }
        kinds.push(Act::Amend).expect("task act bound");
        kinds.push(Act::TakeOver).expect("task act bound");
        if core.chat_requester(row).is_some() {
            kinds.push(Act::Close).expect("task act bound");
        }
    }
    match row.phase {
        tasks::Phase::Held { was: tasks::Was::Waiting | tasks::Was::Active(_), .. } => {
            kinds.push(Act::AmendRelease).expect("task act bound");
            kinds.push(Act::Release).expect("task act bound");
        }
        tasks::Phase::Held { was: tasks::Was::Closing(_), .. } => {
            kinds.push(Act::Release).expect("task act bound");
        }
        tasks::Phase::Active(_) => match row.executor {
            tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } => {
                kinds.push(Act::Stop).expect("task act bound");
            }
            tasks::Executor::Person(_) => {}
        },
        tasks::Phase::Waiting | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => {}
    }
    match row.phase {
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
            kinds.push(Act::Cancel).expect("task act bound");
            if core.chat_requester(row).is_some() {
                kinds.push(Act::MarkSeen).expect("task act bound");
            }
        }
        tasks::Phase::Ended(_) => {}
    }
    list(core, limits, party, &Subject::Task { task: row.number }, kinds.as_slice())
}

pub(super) fn subject(core: &Core, limits: &Limits, party: u64, subject: &Subject) -> Result<List<Offered>, Missing> {
    match subject {
        Subject::Project { .. } => list(
            core,
            limits,
            party,
            subject,
            &[
                Act::SetGoal,
                Act::Prioritise,
                Act::Adopt,
                Act::ChangePolicy,
                Act::SetRoles,
                Act::MakeService,
                Act::SetPool,
            ],
        ),
        Subject::Proposal { .. } => list(core, limits, party, subject, &[Act::Accept, Act::Reject, Act::PassUp]),
        Subject::Escalation { task } => {
            let row = core.tasks.task(*task).ok_or(Missing::Unknown)?;
            let kinds = match row.phase {
                tasks::Phase::Held { was: tasks::Was::Closing(_), .. } => {
                    &[Act::Release, Act::LeaveHeld, Act::PassUp, Act::Cancel][..]
                }
                tasks::Phase::Waiting
                | tasks::Phase::Active(_)
                | tasks::Phase::Closing(_)
                | tasks::Phase::Held { was: tasks::Was::Waiting | tasks::Was::Active(_), .. }
                | tasks::Phase::Ended(_) => {
                    &[Act::Release, Act::LeaveHeld, Act::PassUp, Act::AmendRelease, Act::Cancel][..]
                }
            };
            list(core, limits, party, subject, kinds)
        }
        Subject::Question { .. } => list(core, limits, party, subject, &[Act::Answer]),
        Subject::Result { .. } => list(core, limits, party, &Subject::Inbox { party }, &[Act::MarkSeen]),
        Subject::Frame { .. }
        | Subject::Task { .. }
        | Subject::Tree { .. }
        | Subject::Chips { .. }
        | Subject::Inbox { .. }
        | Subject::Goals { .. }
        | Subject::Conversations { .. }
        | Subject::Run { .. }
        | Subject::Deployment
        | Subject::PersonTask { .. }
        | Subject::Note { .. }
        | Subject::Application { .. } => Ok(List::with_capacity(0)),
    }
}

pub(super) fn person(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &tasks::TaskRecord,
) -> Result<List<Offered>, Missing> {
    match row.phase {
        tasks::Phase::Active(tasks::Active::Due | tasks::Active::Idle) => {
            let kinds = match row.executor {
                tasks::Executor::Person(tasks::PersonAddress::Role(_)) => &[Act::Answer, Act::Take, Act::HandBack][..],
                tasks::Executor::Person(tasks::PersonAddress::Person(_))
                | tasks::Executor::Agent { .. }
                | tasks::Executor::Procedure { .. } => &[Act::Answer][..],
            };
            list(core, limits, party, &Subject::PersonTask { task: row.number }, kinds)
        }
        tasks::Phase::Waiting
        | tasks::Phase::Active(
            tasks::Active::Preparing
            | tasks::Active::Claimed { .. }
            | tasks::Active::Running { .. }
            | tasks::Active::BackingOff { .. },
        )
        | tasks::Phase::Closing(_)
        | tasks::Phase::Held { .. }
        | tasks::Phase::Ended(_) => Ok(List::with_capacity(0)),
    }
}

pub(super) fn note(core: &Core, party: u64, project: u32, scope: &people::NoteScope) -> List<Offered> {
    let mut result = List::with_capacity(2);
    for act in [Act::EditNote, Act::DeleteNote] {
        let admission = match acts::note(core, party, project, scope, act) {
            Ok(()) => Admission::Allowed,
            Err(why) => Admission::Refused(why),
        };
        result.push(Offered { act, admission }).expect("two note acts");
    }
    result
}

pub(super) fn heap_worst_case(limits: &DocumentLimits) -> Option<u64> {
    let authority = super::task_tree::authority_worst_case(limits)?;
    let grants = List::<super::GrantLine>::worst_case(limits.lines)?
        .checked_add(u64::from(limits.lines).checked_mul(super::task_tree::grant_worst_case(limits)?)?)?;
    let amended = authority
        .checked_add(u64::try_from(size_of::<super::AuthorityLine>()).ok()?)?
        .checked_mul(2)?
        .checked_add(grants)?;
    let recipient = List::<u32>::worst_case(limits.lines)?;
    // Only Amend and AmendRelease carry the full amendment facts on a task;
    // all other object kinds carry at most one bounded act.
    List::<Offered>::worst_case(MAX_ACTS)?.checked_add(amended.checked_add(recipient)?.checked_mul(2)?)
}

fn path_heap(value: &List<Box<[u8]>>, last: &super::PatternEnd) -> Option<u64> {
    let mut bytes = List::<Box<[u8]>>::worst_case(value.len().checked_add(value.room())?)?;
    for segment in value {
        bytes = bytes.checked_add(u64::try_from(segment.len()).ok()?)?;
    }
    let terminal = match last {
        super::PatternEnd::Exact(value) | super::PatternEnd::Open(value) => value.len(),
    };
    bytes.checked_add(u64::try_from(terminal).ok()?)
}
fn grants_heap(value: &List<super::GrantLine>) -> Option<u64> {
    let mut bytes = List::<super::GrantLine>::worst_case(value.len().checked_add(value.room())?)?;
    for grant in value {
        bytes = bytes.checked_add(path_heap(&grant.segments, &grant.last)?)?;
    }
    Some(bytes)
}
fn authority_heap(value: &super::AuthorityLine) -> Option<u64> {
    let mut bytes = grants_heap(&value.grants)?
        .checked_add(List::<tasks::AuthorityExecutor>::worst_case(
            value.executors.len().checked_add(value.executors.room())?,
        )?)?
        .checked_add(List::<super::NoteScope>::worst_case(
            value.note_resources.len().checked_add(value.note_resources.room())?,
        )?)?;
    for scope in &value.note_resources {
        bytes = bytes.checked_add(path_heap(&scope.segments, &scope.last)?)?;
    }
    Some(bytes)
}
fn recipient_heap(value: &Recipient) -> Option<u64> {
    match value {
        Recipient::Roles { roles, .. } => List::<u32>::worst_case(roles.len().checked_add(roles.room())?),
        Recipient::Task(_) | Recipient::Party(_) | Recipient::Role { .. } => Some(0),
    }
}
fn beyond_heap(value: &Beyond) -> Option<u64> {
    match value {
        Beyond::Proposed { to } => recipient_heap(to),
        Beyond::Refused(_) => Some(0),
    }
}
pub(super) fn heap_bytes(value: &List<Offered>) -> Option<u64> {
    let mut bytes = List::<Offered>::worst_case(value.len().checked_add(value.room())?)?;
    for offer in value {
        bytes = bytes.checked_add(match &offer.admission {
            Admission::Allowed | Admission::Refused(_) => 0,
            Admission::Proposed { to } => recipient_heap(to)?,
            Admission::Bounded { within, beyond } => {
                let inner = match within {
                    Bounds::Goal { charters, .. } => {
                        List::<u32>::worst_case(charters.len().checked_add(charters.room())?)?
                    }
                    Bounds::Amend { grants, authority, ceiling, .. } => grants_heap(grants)?
                        .checked_add(u64::try_from(size_of::<super::AuthorityLine>()).ok()?.checked_mul(2)?)?
                        .checked_add(authority_heap(authority)?)?
                        .checked_add(authority_heap(ceiling)?)?,
                };
                inner.checked_add(beyond_heap(beyond)?)?
            }
        })?;
    }
    Some(bytes)
}
