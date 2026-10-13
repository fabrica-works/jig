//! Wire asks into the people vocabulary (client/domain/contract.md, 3).
use super::Untranslatable;
use alloc::boxed::Box;
use jig_domain_people as p;
use jig_protocol_client_wire::v7 as w;
use skein_lib::{List, Reader, Wall};
#[expect(clippy::manual_map, reason = "step code matches optional fields")]
#[expect(clippy::too_many_lines, reason = "one total match names every wire ask")]
pub(crate) fn ask(record: w::Ask, limits: &w::Limits) -> Result<p::Ask, Untranslatable> {
    Ok(match record {
        w::Ask::StartChat(value) => {
            let v = value.into_parts();
            p::Ask::StartChat { project: v.project, words: v.words }
        }
        w::Ask::Write(value) => {
            let v = value.into_parts();
            p::Ask::Say { project: v.project, task: v.task, words: v.words, release: v.release }
        }
        w::Ask::AnswerQuestion(value) => {
            let v = value.into_parts();
            p::Ask::AnswerQuestion { project: v.project, task: v.task, question: v.message, words: v.words }
        }
        w::Ask::AnswerPerson(value) => {
            let v = value.into_parts();
            p::Ask::AnswerPerson {
                project: v.project,
                task: v.task,
                revision: v.revision.value(),
                result: person_answer(v.answer),
            }
        }
        w::Ask::Take(value) => {
            let v = value.into_parts();
            p::Ask::TakePerson { project: v.project, task: v.task }
        }
        w::Ask::HandBack(value) => {
            let v = value.into_parts();
            p::Ask::HandBackPerson { project: v.project, task: v.task }
        }
        w::Ask::DecideProposal(value) => {
            let v = value.into_parts();
            p::Ask::DecideProposal {
                project: v.project,
                proposer: proposer(v.proposal.proposer()),
                proposal: v.proposal.number(),
                decision: proposal_decision(v.decision)?,
            }
        }
        w::Ask::DecideEscalation(value) => {
            let v = value.into_parts();
            p::Ask::DecideEscalation {
                project: v.project,
                task: v.task,
                revision: v.revision.value(),
                decision: escalation_decision(v.decision)?,
            }
        }
        w::Ask::Release(value) => {
            let v = value.into_parts();
            p::Ask::Release { project: v.project, task: v.task, revision: v.revision.value(), again: v.again }
        }
        w::Ask::Amend(value) => {
            let v = value.into_parts();
            p::Ask::Amend {
                project: v.project,
                task: v.task,
                revision: v.revision.value(),
                amendment: amendment(v.change, v.reason, limits)?,
                release: match v.release {
                    Some(value) => Some(revision(value)),
                    None => None,
                },
            }
        }
        w::Ask::Stop(value) => {
            let v = value.into_parts();
            p::Ask::Stop { project: v.project, task: v.task }
        }
        w::Ask::Cancel(value) => {
            let v = value.into_parts();
            p::Ask::Cancel { project: v.project, task: v.task, reason: v.reason }
        }
        w::Ask::TakeOver(value) => {
            let v = value.into_parts();
            p::Ask::Move { project: v.project, task: v.task, reason: v.reason }
        }
        w::Ask::SetGoal(value) => {
            let v = value.into_parts();
            p::Ask::SetGoal {
                project: v.project,
                spec: v.spec,
                executor: goal_executor(v.executor),
                budget: v.budget,
                priority: v.priority,
                deadline: match v.deadline {
                    Some(value) => Some(Wall::from_nanos(value)),
                    None => None,
                },
            }
        }
        w::Ask::MarkSeen(value) => {
            let v = value.into_parts();
            p::Ask::MarkSeen { seen: seen(v.seen) }
        }
        w::Ask::MakeService(value) => {
            let v = value.into_parts();
            p::Ask::MakeService { project: v.project, name: v.name, role: role(v.role) }
        }
        w::Ask::SetPool(value) => {
            let v = value.into_parts();
            p::Ask::SetPool { project: v.project, person: v.party, budget: v.budget }
        }
        w::Ask::EditNote(value) => {
            let v = value.into_parts();
            p::Ask::EditNote {
                project: v.project,
                name: v.note,
                scope: Box::new(note_scope(v.scope, v.project, w::Path::EditNoteScope)?),
                change: Box::new(p::NoteChange::Correct {
                    description: v.description,
                    body: v.body,
                    references: v.references.into_boxed(),
                    recalled: recalled(v.revision, w::Path::EditNoteRevision)?,
                }),
            }
        }
        w::Ask::DeleteNote(value) => {
            let v = value.into_parts();
            p::Ask::EditNote {
                project: v.project,
                name: v.note,
                scope: Box::new(note_scope(v.scope, v.project, w::Path::DeleteNoteScope)?),
                change: Box::new(p::NoteChange::Delete {
                    recalled: recalled(v.revision, w::Path::DeleteNoteRevision)?,
                }),
            }
        }
        w::Ask::ChangePolicy(value) => {
            let v = value.into_parts();
            p::Ask::ChangePolicy { project: v.project, change: policy(v.edit, limits)? }
        }
        w::Ask::Close(value) => {
            let v = value.into_parts();
            let mut disposals = List::with_capacity(v.disposals.capacity());
            for item in v.disposals.into_boxed() {
                disposals.push(disposal(item)).expect("same capacity");
            }
            p::Ask::Close { project: v.project, chat: v.chat, disposals }
        }
        w::Ask::Prioritise(value) => {
            let v = value.into_parts();
            let mut goals = List::with_capacity(v.goals.capacity());
            for item in v.goals.into_boxed() {
                goals.push((item.task(), item.priority())).expect("same capacity");
            }
            p::Ask::Prioritise { project: v.project, goals: goals.into_boxed() }
        }
        w::Ask::SetRoles(value) => {
            let v = value.into_parts();
            let mut holdings = List::with_capacity(v.holdings.capacity());
            for item in v.holdings.into_boxed() {
                let item = item.into_parts();
                holdings.push(p::Holding { person: item.party, role: role(item.role) }).expect("same capacity");
            }
            p::Ask::SetRoles { project: v.project, holdings: holdings.into_boxed() }
        }
    })
}
fn revision(value: w::Revision) -> u64 {
    value.value()
}
fn recalled(value: w::Revision, path: w::Path) -> Result<u32, Untranslatable> {
    match u32::try_from(value.value()) {
        Ok(value) => Ok(value),
        Err(_) => Err(Untranslatable { path }),
    }
}
fn proposer(value: &w::Proposer) -> p::Proposer {
    match value {
        w::Proposer::Task(v) => p::Proposer::Task(v.value()),
        w::Proposer::Party(v) => p::Proposer::Party(v.value()),
    }
}
fn proposal_decision(value: w::Decision) -> Result<p::ProposalDecision, Untranslatable> {
    match value {
        w::Decision::Accepted => Ok(p::ProposalDecision::Accept),
        w::Decision::Rejected(v) => Ok(p::ProposalDecision::Reject { reason: v.into_parts().reason }),
        w::Decision::PassedUp => Ok(p::ProposalDecision::Pass),
        w::Decision::Released(_) | w::Decision::LeftHeld(_) => {
            Err(Untranslatable { path: w::Path::DecideProposalDecision })
        }
    }
}
fn escalation_decision(value: w::Decision) -> Result<p::EscalationDecision, Untranslatable> {
    match value {
        w::Decision::Released(v) => match v.again() {
            None => Ok(p::EscalationDecision::Release),
            Some(_) => Err(Untranslatable { path: w::Path::DecideEscalationDecision }),
        },
        w::Decision::Rejected(v) => Ok(p::EscalationDecision::Reject { reason: v.into_parts().reason }),
        w::Decision::LeftHeld(v) => Ok(p::EscalationDecision::Reject { reason: v.into_parts().reason }),
        w::Decision::PassedUp => Ok(p::EscalationDecision::Pass),
        w::Decision::Accepted => Err(Untranslatable { path: w::Path::DecideEscalationDecision }),
    }
}
fn person_answer(value: w::PersonAnswer) -> p::PersonResult {
    match value {
        w::PersonAnswer::Choice(v) => {
            let v = v.into_parts();
            p::PersonResult::Verdict { code: v.option, words: v.reason }
        }
        w::PersonAnswer::Approved(v) => {
            let v = v.into_parts();
            p::PersonResult::Approved { state: v.state, words: v.words }
        }
        w::PersonAnswer::Words(v) => p::PersonResult::Report { words: v.into_parts().value },
        w::PersonAnswer::Cannot(v) => p::PersonResult::Failure { reason: v.into_parts().reason },
    }
}
fn disposal(value: w::Disposal) -> p::Disposal {
    match value {
        w::Disposal::Cancel(v) => p::Disposal { task: v.task(), keep: p::Keep::Cancel },
        w::Disposal::TakeOver(v) => p::Disposal { task: v.task(), keep: p::Keep::MakeMine },
    }
}
fn goal_executor(value: w::GoalExecutor) -> p::GoalExecutor {
    match value {
        w::GoalExecutor::Charter(v) => p::GoalExecutor::Charter(v.value()),
        w::GoalExecutor::Procedure(v) => p::GoalExecutor::Procedure { connector: v.connector(), code: v.code() },
    }
}
fn seen(value: w::Seen) -> p::Seen {
    match value {
        w::Seen::Results(v) => p::Seen::Results { through: v.through() },
        w::Seen::Replies(v) => p::Seen::Chat { task: v.chat(), through: v.through() },
    }
}
fn role(value: w::HeldRole) -> p::Role {
    match value {
        w::HeldRole::Owner => p::Role::Owner,
        w::HeldRole::Maintainer => p::Role::Maintainer,
        w::HeldRole::Member => p::Role::Member,
        w::HeldRole::Observer => p::Role::Observer,
        w::HeldRole::Policy(v) => p::Role::Policy { role: v.role() },
    }
}
fn pattern(value: w::OwnerPattern) -> p::Pattern {
    let v = value.into_parts();
    p::Pattern {
        segments: v.segments.into_boxed(),
        last: match v.last {
            w::OwnerPatternEnd::Exact(v) => p::Last::Exact(v.into_parts().value),
            w::OwnerPatternEnd::Open(v) => p::Last::Open(v.into_parts().value),
        },
    }
}
fn note_scope(value: w::NoteScope, project: u32, path: w::Path) -> Result<p::NoteScope, Untranslatable> {
    Ok(match value {
        w::NoteScope::Deployment => p::NoteScope::Deployment,
        w::NoteScope::Project(v) => {
            if v.value() != project {
                return Err(Untranslatable { path });
            }
            p::NoteScope::Project
        }
        w::NoteScope::Goal(v) => {
            if v.project() != project {
                return Err(Untranslatable { path });
            }
            p::NoteScope::Goal { goal: v.goal() }
        }
        w::NoteScope::Resource(v) => {
            let v = v.into_parts();
            if v.project != project {
                return Err(Untranslatable { path });
            }
            p::NoteScope::Resources { connector: v.connector, pattern: pattern(v.pattern) }
        }
    })
}
fn spec(value: w::Specification) -> p::Spec {
    let v = value.into_parts();
    let mut parameters = List::with_capacity(v.parameters.capacity());
    for item in v.parameters.into_boxed() {
        let item = match item {
            w::Parameter::Number(v) => p::Parameter::Number { name: v.name(), value: v.value() },
            w::Parameter::Bytes(v) => {
                let v = v.into_parts();
                p::Parameter::Bytes { name: v.name, value: v.value }
            }
            w::Parameter::Resource(v) => {
                p::Parameter::Resource { name: v.name(), connector: v.connector(), resource: v.resource() }
            }
        };
        parameters.push(item).expect("same capacity");
    }
    p::Spec { words: v.words, parameters: parameters.into_boxed(), inputs: v.inputs.into_boxed() }
}
fn wake_rule(value: w::WakeRule) -> p::WakeRule {
    match value {
        w::WakeRule::Never => p::WakeRule::Never,
        w::WakeRule::Immediate => p::WakeRule::Immediate,
        w::WakeRule::Batch(v) => p::WakeRule::Batch { count: v.count(), age: v.age() },
    }
}
fn wake(value: w::OwnerWakePolicy) -> p::WakePolicy {
    let v = value.into_parts();
    p::WakePolicy {
        words: wake_rule(v.words),
        notices: wake_rule(v.notices),
        news: wake_rule(v.news),
        results: match v.results {
            w::ResultsWake::Never => p::ResultsWake::Never,
            w::ResultsWake::Each => p::ResultsWake::Each,
            w::ResultsWake::LastOrFailure => p::ResultsWake::LastOrFailure,
        },
        questions: v.questions,
        answers: v.answers,
        timers: v.timers,
    }
}
fn address(value: w::AddressChange) -> p::AddressChange {
    let v = value.into_parts();
    p::AddressChange {
        revision: v.revision.value(),
        address: match v.address {
            w::PersonAddress::Party(v) => p::PersonAddress::Person(v.value()),
            w::PersonAddress::Role(v) => p::PersonAddress::Role(v.value()),
        },
    }
}
#[expect(clippy::manual_map, reason = "optional step fields use named total functions")]
fn amendment(value: w::Amendment, reason: Box<[u8]>, limits: &w::Limits) -> Result<p::Amendment, Untranslatable> {
    let v = value.into_parts();
    let authority = match v.authority {
        Some(value) => Some(authority(value, limits)?),
        None => None,
    };
    Ok(p::Amendment {
        addressed: match v.addressed {
            Some(value) => Some(address(value)),
            None => None,
        },
        instructions: v.instructions,
        spec: match v.spec {
            Some(value) => Some(spec(value)),
            None => None,
        },
        wake: match v.wake {
            Some(value) => Some(wake(value)),
            None => None,
        },
        dependencies: match v.dependencies {
            Some(value) => Some(List::into_boxed(value)),
            None => None,
        },
        authority,
        reason,
    })
}
fn count(bytes: &[u8], path: w::Path) -> Result<u32, Untranslatable> {
    let mut reader = Reader::new(bytes);
    let mut count = 0_u32;
    for _ in 0..bytes.len() {
        if reader.is_empty() {
            break;
        }
        let length = reader.u32().ok_or(Untranslatable { path })?;
        reader.bytes(length).ok_or(Untranslatable { path })?;
        count = count.checked_add(1).ok_or(Untranslatable { path })?;
    }
    Ok(count)
}
fn item(reader: &mut Reader<'_>, path: w::Path) -> Result<Box<[u8]>, Untranslatable> {
    match skein_codec::piece::split(reader) {
        Ok(Some(bytes)) => Ok(bytes),
        Ok(None) | Err(_) => Err(Untranslatable { path }),
    }
}
fn sum(total: &mut u64, bytes: Option<u64>, ceiling: u32, path: w::Path) -> Result<(), Untranslatable> {
    *total = total.checked_add(bytes.ok_or(Untranslatable { path })?).ok_or(Untranslatable { path })?;
    if *total > u64::from(ceiling) {
        return Err(Untranslatable { path });
    }
    Ok(())
}
fn grant(value: w::OwnerGrant) -> p::Grant {
    let v = value.into_parts();
    p::Grant { connector: v.connector, kind: v.kind, pattern: pattern(v.pattern) }
}
fn grants(bytes: &[u8], ceiling: u32, path: w::Path, limits: &w::Limits) -> Result<Box<[p::Grant]>, Untranslatable> {
    let count = count(bytes, path)?;
    let mut values = List::with_capacity(count);
    let mut reader = Reader::new(bytes);
    let mut total = 0;
    for _ in 0..count {
        let bytes = item(&mut reader, path)?;
        let value = match w::OwnerGrant::decode(limits, &mut Reader::new(&bytes)) {
            Ok(value) => grant(value),
            Err(_) => return Err(Untranslatable { path }),
        };
        sum(&mut total, pattern_bytes(&value.pattern, size_of::<p::Grant>()), ceiling, path)?;
        values.push(value).expect("counted pieces");
    }
    Ok(values.into_boxed())
}
#[expect(clippy::manual_map, reason = "step code matches optional fields")]
fn authority(value: w::OwnerAuthority, limits: &w::Limits) -> Result<p::Authority, Untranslatable> {
    let v = value.into_parts();
    let delegation = v.delegation.into_parts();
    let mut kinds = List::with_capacity(delegation.kinds.capacity());
    for kind in delegation.kinds.into_boxed() {
        kinds
            .push(match kind {
                w::OwnerAuthorityExecutor::Charter(v) => p::Executor::Charter(v.value()),
                w::OwnerAuthorityExecutor::Procedure(v) => p::Executor::Procedure(v.value()),
                w::OwnerAuthorityExecutor::Role(v) => p::Executor::Role(v.value()),
            })
            .expect("same capacity");
    }
    let path = w::Path::OwnerAuthorityNoteResources;
    let count = count(&v.note_resources, path)?;
    let mut resources = List::with_capacity(count);
    let mut reader = Reader::new(&v.note_resources);
    let mut total = 0;
    for _ in 0..count {
        let bytes = item(&mut reader, path)?;
        let value = match w::OwnerResourceScope::decode(limits, &mut Reader::new(&bytes)) {
            Ok(value) => value.into_parts(),
            Err(_) => return Err(Untranslatable { path }),
        };
        let pattern = pattern(value.pattern);
        let size = pattern_bytes(&pattern, size_of::<p::ResourceScope>());
        sum(&mut total, size, limits.owner_authority_note_resources, path)?;
        resources.push(p::ResourceScope { connector: value.connector, pattern }).expect("counted pieces");
    }
    Ok(p::Authority {
        tools: v.tools,
        grants: grants(&v.grants, limits.owner_authority_grants, w::Path::OwnerAuthorityGrants, limits)?,
        delegation: p::Delegation { kinds: kinds.into_boxed(), tasks: delegation.tasks, depth: delegation.depth },
        spend: v.spend,
        deadline: match v.deadline {
            Some(value) => Some(Wall::from_nanos(value)),
            None => None,
        },
        notes: v.notes,
        note_resources: resources.into_boxed(),
    })
}
fn requirement(value: w::PolicyRequirement) -> p::Requirement {
    let v = value.into_parts();
    p::Requirement {
        connector: v.connector,
        kind: v.kind,
        pattern: pattern(v.pattern),
        judge: p::Judge {
            connector: v.judge.connector(),
            requirement: v.judge.requirement(),
            parameters: v.judge.parameters(),
        },
        guard: match v.guard {
            w::PolicyGuard::Guarded => p::Guard::Guarded,
            w::PolicyGuard::Observed(v) => p::Guard::Observed { freshness: v.freshness() },
        },
        must_be_guarded: v.must_be_guarded,
    }
}
fn policy(value: w::PolicyEdit, limits: &w::Limits) -> Result<p::PolicyChange, Untranslatable> {
    Ok(match value {
        w::PolicyEdit::ProjectSpend(v) => p::PolicyChange::ProjectSpend { period_spend: v.period_spend() },
        w::PolicyEdit::Role(v) => {
            let v = v.into_parts().value.into_parts();
            p::PolicyChange::Role(p::PolicyRole {
                number: v.number,
                authority: authority(v.authority, limits)?,
                period_spend: v.period_spend,
                requests: v.requests,
                decides: v.decides,
            })
        }
        w::PolicyEdit::Requirements(v) => {
            let path = w::Path::PolicyEditRequirementsRequirements;
            let count = count(v.requirements(), path)?;
            let mut values = List::with_capacity(count);
            let mut reader = Reader::new(v.requirements());
            let mut total = 0;
            for _ in 0..count {
                let bytes = item(&mut reader, path)?;
                let value = match w::PolicyRequirement::decode(limits, &mut Reader::new(&bytes)) {
                    Ok(value) => requirement(value),
                    Err(_) => return Err(Untranslatable { path }),
                };
                sum(
                    &mut total,
                    p::requirements_bytes(core::slice::from_ref(&value)),
                    limits.policy_edit_requirements_requirements,
                    path,
                )?;
                values.push(value).expect("counted pieces");
            }
            p::PolicyChange::Requirements { requirements: values.into_boxed() }
        }
        w::PolicyEdit::Projections(v) => p::PolicyChange::Projections {
            grants: grants(
                v.grants(),
                limits.policy_edit_projections_grants,
                w::Path::PolicyEditProjectionsGrants,
                limits,
            )?,
        },
        w::PolicyEdit::Permissions(v) => {
            let v = v.into_parts();
            let mut mappings = List::with_capacity(v.mappings.capacity());
            for value in v.mappings.into_boxed() {
                mappings
                    .push(p::PermissionRole {
                        connector: value.connector(),
                        permission: value.permission(),
                        role: value.role(),
                    })
                    .expect("same capacity");
            }
            p::PolicyChange::Permissions { mappings: mappings.into_boxed() }
        }
    })
}

fn pattern_bytes(value: &p::Pattern, fixed: usize) -> Option<u64> {
    let mut bytes = u64::try_from(fixed).ok()?;
    bytes = bytes.checked_add(
        u64::try_from(value.segments.len()).ok()?.checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?,
    )?;
    for segment in &value.segments {
        bytes = bytes.checked_add(u64::try_from(segment.len()).ok()?)?;
    }
    bytes.checked_add(match &value.last {
        p::Last::Exact(bytes) | p::Last::Open(bytes) => u64::try_from(bytes.len()).ok()?,
    })
}
