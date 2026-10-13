//! Field-for-field client contract translation (protocol/client.md, 3 and 5).
use crate::translate::too_large;
use alloc::boxed::Box;
use jig_domain_client::contract as c;
use jig_protocol_client_wire::v7 as w;
use skein_lib::{List, Writer, bytes};
pub(crate) fn wire_revision(value: c::Revision, limits: &w::Limits) -> Result<w::Revision, w::Problem> {
    w::Revision::new(limits, w::RevisionParts { value: value.value })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn wire_person_answer(value: &c::PersonAnswer, limits: &w::Limits) -> Result<w::PersonAnswer, w::Problem> {
    Ok(match value {
        c::PersonAnswer::Choice { option, reason } => w::PersonAnswer::Choice(w::PersonAnswerChoice::new(
            limits,
            w::PersonAnswerChoiceParts { option: *option, reason: bytes::copy_of(reason.bytes()) },
        )?),
        c::PersonAnswer::Approved { state, words } => {
            let opt0 = match words {
                Some(item) => Some(bytes::copy_of(item.bytes())),
                None => None,
            };
            w::PersonAnswer::Approved(w::PersonAnswerApproved::new(
                limits,
                w::PersonAnswerApprovedParts { state: bytes::copy_of(state), words: opt0 },
            )?)
        }
        c::PersonAnswer::Words(value) => w::PersonAnswer::Words(w::PersonAnswerWords::new(
            limits,
            w::PersonAnswerWordsParts { value: bytes::copy_of(value.bytes()) },
        )?),
        c::PersonAnswer::Cannot { reason } => w::PersonAnswer::Cannot(w::PersonAnswerCannot::new(
            limits,
            w::PersonAnswerCannotParts { reason: bytes::copy_of(reason.bytes()) },
        )?),
    })
}

pub(crate) fn wire_proposer(value: &c::Proposer, limits: &w::Limits) -> Result<w::Proposer, w::Problem> {
    Ok(match value {
        c::Proposer::Task(value) => {
            w::Proposer::Task(w::ProposerTask::new(limits, w::ProposerTaskParts { value: *value })?)
        }
        c::Proposer::Party(value) => {
            w::Proposer::Party(w::ProposerParty::new(limits, w::ProposerPartyParts { value: *value })?)
        }
    })
}

pub(crate) fn wire_proposal_key(value: &c::ProposalKey, limits: &w::Limits) -> Result<w::ProposalKey, w::Problem> {
    w::ProposalKey::new(
        limits,
        w::ProposalKeyParts { proposer: wire_proposer(&value.proposer, limits)?, number: value.number },
    )
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn wire_decision(value: &c::Decision, limits: &w::Limits) -> Result<w::Decision, w::Problem> {
    Ok(match value {
        c::Decision::Accepted => w::Decision::Accepted,
        c::Decision::Rejected { reason } => w::Decision::Rejected(w::DecisionRejected::new(
            limits,
            w::DecisionRejectedParts { reason: bytes::copy_of(reason.bytes()) },
        )?),
        c::Decision::PassedUp => w::Decision::PassedUp,
        c::Decision::Released { again } => {
            let opt0 = match again {
                Some(item) => Some(*item),
                None => None,
            };
            w::Decision::Released(w::DecisionReleased::new(limits, w::DecisionReleasedParts { again: opt0 })?)
        }
        c::Decision::LeftHeld { reason } => w::Decision::LeftHeld(w::DecisionLeftHeld::new(
            limits,
            w::DecisionLeftHeldParts { reason: bytes::copy_of(reason.bytes()) },
        )?),
    })
}

pub(crate) fn wire_person_address(
    value: &c::PersonAddress,
    limits: &w::Limits,
) -> Result<w::PersonAddress, w::Problem> {
    Ok(match value {
        c::PersonAddress::Party(value) => {
            w::PersonAddress::Party(w::PersonAddressParty::new(limits, w::PersonAddressPartyParts { value: *value })?)
        }
        c::PersonAddress::Role(value) => {
            w::PersonAddress::Role(w::PersonAddressRole::new(limits, w::PersonAddressRoleParts { value: *value })?)
        }
    })
}

pub(crate) fn wire_address_change(
    value: &c::AddressChange,
    limits: &w::Limits,
) -> Result<w::AddressChange, w::Problem> {
    w::AddressChange::new(
        limits,
        w::AddressChangeParts {
            revision: wire_revision(value.revision, limits)?,
            address: wire_person_address(&value.address, limits)?,
        },
    )
}

pub(crate) fn wire_parameter(value: &c::Parameter, limits: &w::Limits) -> Result<w::Parameter, w::Problem> {
    Ok(match value {
        c::Parameter::Number { name, value } => w::Parameter::Number(w::ParameterNumber::new(
            limits,
            w::ParameterNumberParts { name: *name, value: *value },
        )?),
        c::Parameter::Bytes { name, value } => w::Parameter::Bytes(w::ParameterBytes::new(
            limits,
            w::ParameterBytesParts { name: *name, value: bytes::copy_of(value.bytes()) },
        )?),
        c::Parameter::Resource { name, connector, resource } => w::Parameter::Resource(w::ParameterResource::new(
            limits,
            w::ParameterResourceParts { name: *name, connector: *connector, resource: *resource },
        )?),
    })
}

pub(crate) fn wire_specification(value: &c::Specification, limits: &w::Limits) -> Result<w::Specification, w::Problem> {
    let mut list0 = List::with_capacity(value.parameters.len());
    for item in &value.parameters {
        list0.push(wire_parameter(item, limits)?).expect("reserved supplied list");
    }
    let mut list1 = List::with_capacity(value.inputs.len());
    for item in &value.inputs {
        list1.push(*item).expect("reserved supplied list");
    }
    w::Specification::new(
        limits,
        w::SpecificationParts { words: bytes::copy_of(value.words.bytes()), parameters: list0, inputs: list1 },
    )
}

pub(crate) fn wire_wake_rule(value: &c::WakeRule, limits: &w::Limits) -> Result<w::WakeRule, w::Problem> {
    Ok(match value {
        c::WakeRule::Never => w::WakeRule::Never,
        c::WakeRule::Immediate => w::WakeRule::Immediate,
        c::WakeRule::Batch { count, age } => {
            w::WakeRule::Batch(w::WakeRuleBatch::new(limits, w::WakeRuleBatchParts { count: *count, age: *age })?)
        }
    })
}

pub(crate) fn wire_results_wake(value: c::ResultsWake, _limits: &w::Limits) -> w::ResultsWake {
    match value {
        c::ResultsWake::Never => w::ResultsWake::Never,
        c::ResultsWake::Each => w::ResultsWake::Each,
        c::ResultsWake::LastOrFailure => w::ResultsWake::LastOrFailure,
    }
}

pub(crate) fn wire_owner_wake_policy(
    value: &c::OwnerWakePolicy,
    limits: &w::Limits,
) -> Result<w::OwnerWakePolicy, w::Problem> {
    w::OwnerWakePolicy::new(
        limits,
        w::OwnerWakePolicyParts {
            words: wire_wake_rule(&value.words, limits)?,
            notices: wire_wake_rule(&value.notices, limits)?,
            news: wire_wake_rule(&value.news, limits)?,
            results: wire_results_wake(value.results, limits),
            questions: value.questions,
            answers: value.answers,
            timers: value.timers,
        },
    )
}

pub(crate) fn wire_owner_pattern_end(
    value: &c::OwnerPatternEnd,
    limits: &w::Limits,
) -> Result<w::OwnerPatternEnd, w::Problem> {
    Ok(match value {
        c::OwnerPatternEnd::Exact(value) => w::OwnerPatternEnd::Exact(w::OwnerPatternEndExact::new(
            limits,
            w::OwnerPatternEndExactParts { value: bytes::copy_of(value) },
        )?),
        c::OwnerPatternEnd::Open(value) => w::OwnerPatternEnd::Open(w::OwnerPatternEndOpen::new(
            limits,
            w::OwnerPatternEndOpenParts { value: bytes::copy_of(value) },
        )?),
    })
}

pub(crate) fn wire_owner_pattern(value: &c::OwnerPattern, limits: &w::Limits) -> Result<w::OwnerPattern, w::Problem> {
    let mut list0 = List::with_capacity(value.segments.len());
    for item in &value.segments {
        list0.push(bytes::copy_of(item)).expect("reserved supplied list");
    }
    w::OwnerPattern::new(
        limits,
        w::OwnerPatternParts { segments: list0, last: wire_owner_pattern_end(&value.last, limits)? },
    )
}

pub(crate) fn wire_owner_grant(value: &c::OwnerGrant, limits: &w::Limits) -> Result<w::OwnerGrant, w::Problem> {
    w::OwnerGrant::new(
        limits,
        w::OwnerGrantParts {
            connector: value.connector,
            kind: value.kind,
            pattern: wire_owner_pattern(&value.pattern, limits)?,
        },
    )
}

fn wire_owner_grant_piece(rows: &List<w::OwnerGrant>, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut length = 0_u32;
    for row in rows {
        length = length.checked_add(row.measure()).ok_or(too_large())?;
        length = length.checked_add(4).ok_or(too_large())?;
    }
    if length > limits.owned_record {
        return Err(too_large());
    }
    let mut writer = Writer::new(usize::try_from(length).expect("u32 fits usize"));
    for row in rows {
        let mut item = Writer::new(usize::try_from(row.measure()).expect("u32 fits usize"));
        row.encode(&mut item).expect("measured row");
        let item = item.finish();
        skein_codec::piece::write(&mut writer, &item).expect("measured piece");
    }
    Ok(writer.finish())
}

pub(crate) fn wire_owner_authority_executor(
    value: c::OwnerAuthorityExecutor,
    limits: &w::Limits,
) -> Result<w::OwnerAuthorityExecutor, w::Problem> {
    Ok(match value {
        c::OwnerAuthorityExecutor::Charter(value) => w::OwnerAuthorityExecutor::Charter(
            w::OwnerAuthorityExecutorCharter::new(limits, w::OwnerAuthorityExecutorCharterParts { value })?,
        ),
        c::OwnerAuthorityExecutor::Procedure(value) => w::OwnerAuthorityExecutor::Procedure(
            w::OwnerAuthorityExecutorProcedure::new(limits, w::OwnerAuthorityExecutorProcedureParts { value })?,
        ),
        c::OwnerAuthorityExecutor::Role(value) => w::OwnerAuthorityExecutor::Role(w::OwnerAuthorityExecutorRole::new(
            limits,
            w::OwnerAuthorityExecutorRoleParts { value },
        )?),
    })
}

pub(crate) fn wire_owner_delegation(
    value: &c::OwnerDelegation,
    limits: &w::Limits,
) -> Result<w::OwnerDelegation, w::Problem> {
    let mut list0 = List::with_capacity(value.kinds.len());
    for item in &value.kinds {
        list0.push(wire_owner_authority_executor(*item, limits)?).expect("reserved supplied list");
    }
    w::OwnerDelegation::new(limits, w::OwnerDelegationParts { kinds: list0, tasks: value.tasks, depth: value.depth })
}

pub(crate) fn wire_owner_resource_scope(
    value: &c::OwnerResourceScope,
    limits: &w::Limits,
) -> Result<w::OwnerResourceScope, w::Problem> {
    w::OwnerResourceScope::new(
        limits,
        w::OwnerResourceScopeParts { connector: value.connector, pattern: wire_owner_pattern(&value.pattern, limits)? },
    )
}

fn wire_owner_resource_scope_piece(
    rows: &List<w::OwnerResourceScope>,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut length = 0_u32;
    for row in rows {
        length = length.checked_add(row.measure()).ok_or(too_large())?;
        length = length.checked_add(4).ok_or(too_large())?;
    }
    if length > limits.owned_record {
        return Err(too_large());
    }
    let mut writer = Writer::new(usize::try_from(length).expect("u32 fits usize"));
    for row in rows {
        let mut item = Writer::new(usize::try_from(row.measure()).expect("u32 fits usize"));
        row.encode(&mut item).expect("measured row");
        let item = item.finish();
        skein_codec::piece::write(&mut writer, &item).expect("measured piece");
    }
    Ok(writer.finish())
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn wire_owner_authority(
    value: &c::OwnerAuthority,
    limits: &w::Limits,
) -> Result<w::OwnerAuthority, w::Problem> {
    let mut list0 = List::with_capacity(value.grants.len());
    for item in &value.grants {
        list0.push(wire_owner_grant(item, limits)?).expect("reserved supplied list");
    }
    let opt1 = match value.deadline {
        Some(item) => Some(item.as_nanos()),
        None => None,
    };
    let mut list2 = List::with_capacity(value.note_resources.len());
    for item in &value.note_resources {
        list2.push(wire_owner_resource_scope(item, limits)?).expect("reserved supplied list");
    }
    w::OwnerAuthority::new(
        limits,
        w::OwnerAuthorityParts {
            tools: value.tools,
            grants: wire_owner_grant_piece(&list0, limits)?,
            delegation: wire_owner_delegation(&value.delegation, limits)?,
            spend: value.spend,
            deadline: opt1,
            notes: value.notes,
            note_resources: wire_owner_resource_scope_piece(&list2, limits)?,
        },
    )
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn wire_amendment(value: &c::Amendment, limits: &w::Limits) -> Result<w::Amendment, w::Problem> {
    let opt0 = match &value.addressed {
        Some(item) => Some(wire_address_change(item, limits)?),
        None => None,
    };
    let opt1 = match &value.spec {
        Some(item) => Some(wire_specification(item, limits)?),
        None => None,
    };
    let opt2 = match &value.wake {
        Some(item) => Some(wire_owner_wake_policy(item, limits)?),
        None => None,
    };
    let opt3 = match &value.dependencies {
        Some(item) => {
            let mut list0 = List::with_capacity((item).len());
            for item in item {
                list0.push(*item).expect("reserved supplied list");
            }
            Some(list0)
        }
        None => None,
    };
    let opt4 = match &value.authority {
        Some(item) => Some(wire_owner_authority(item, limits)?),
        None => None,
    };
    let opt5 = match &value.instructions {
        Some(item) => Some(bytes::copy_of(item.bytes())),
        None => None,
    };
    w::Amendment::new(
        limits,
        w::AmendmentParts {
            addressed: opt0,
            spec: opt1,
            wake: opt2,
            dependencies: opt3,
            authority: opt4,
            instructions: opt5,
        },
    )
}

pub(crate) fn wire_disposal(value: &c::Disposal, limits: &w::Limits) -> Result<w::Disposal, w::Problem> {
    Ok(match value {
        c::Disposal::Cancel { task } => {
            w::Disposal::Cancel(w::DisposalCancel::new(limits, w::DisposalCancelParts { task: *task })?)
        }
        c::Disposal::TakeOver { task } => {
            w::Disposal::TakeOver(w::DisposalTakeOver::new(limits, w::DisposalTakeOverParts { task: *task })?)
        }
    })
}

pub(crate) fn wire_goal_executor(value: c::GoalExecutor, limits: &w::Limits) -> Result<w::GoalExecutor, w::Problem> {
    Ok(match value {
        c::GoalExecutor::Charter(value) => {
            w::GoalExecutor::Charter(w::GoalExecutorCharter::new(limits, w::GoalExecutorCharterParts { value })?)
        }
        c::GoalExecutor::Procedure { connector, code } => w::GoalExecutor::Procedure(w::GoalExecutorProcedure::new(
            limits,
            w::GoalExecutorProcedureParts { connector, code },
        )?),
    })
}

pub(crate) fn wire_priority(value: &c::Priority, limits: &w::Limits) -> Result<w::Priority, w::Problem> {
    w::Priority::new(limits, w::PriorityParts { task: value.task, priority: value.priority })
}

pub(crate) fn wire_seen(value: &c::Seen, limits: &w::Limits) -> Result<w::Seen, w::Problem> {
    Ok(match value {
        c::Seen::Results { through } => {
            w::Seen::Results(w::SeenResults::new(limits, w::SeenResultsParts { through: *through })?)
        }
        c::Seen::Replies { chat, through } => {
            w::Seen::Replies(w::SeenReplies::new(limits, w::SeenRepliesParts { chat: *chat, through: *through })?)
        }
    })
}

pub(crate) fn wire_note_scope(value: &c::NoteScope, limits: &w::Limits) -> Result<w::NoteScope, w::Problem> {
    Ok(match value {
        c::NoteScope::Deployment => w::NoteScope::Deployment,
        c::NoteScope::Project(value) => {
            w::NoteScope::Project(w::NoteScopeProject::new(limits, w::NoteScopeProjectParts { value: *value })?)
        }
        c::NoteScope::Resource { project, connector, pattern } => w::NoteScope::Resource(w::NoteScopeResource::new(
            limits,
            w::NoteScopeResourceParts {
                project: *project,
                connector: *connector,
                pattern: wire_owner_pattern(pattern, limits)?,
            },
        )?),
        c::NoteScope::Goal { project, goal } => {
            w::NoteScope::Goal(w::NoteScopeGoal::new(limits, w::NoteScopeGoalParts { project: *project, goal: *goal })?)
        }
    })
}

pub(crate) fn wire_policy_edit_role(
    value: &c::PolicyEditRole,
    limits: &w::Limits,
) -> Result<w::PolicyEditRole, w::Problem> {
    w::PolicyEditRole::new(
        limits,
        w::PolicyEditRoleParts {
            number: value.number,
            authority: wire_owner_authority(&value.authority, limits)?,
            period_spend: value.period_spend,
            requests: value.requests,
            decides: value.decides,
        },
    )
}

pub(crate) fn wire_policy_judge(value: c::PolicyJudge, limits: &w::Limits) -> Result<w::PolicyJudge, w::Problem> {
    w::PolicyJudge::new(
        limits,
        w::PolicyJudgeParts {
            connector: value.connector,
            requirement: value.requirement,
            parameters: value.parameters,
        },
    )
}

pub(crate) fn wire_policy_guard(value: &c::PolicyGuard, limits: &w::Limits) -> Result<w::PolicyGuard, w::Problem> {
    Ok(match value {
        c::PolicyGuard::Guarded => w::PolicyGuard::Guarded,
        c::PolicyGuard::Observed { freshness } => w::PolicyGuard::Observed(w::PolicyGuardObserved::new(
            limits,
            w::PolicyGuardObservedParts { freshness: *freshness },
        )?),
    })
}

pub(crate) fn wire_policy_requirement(
    value: &c::PolicyRequirement,
    limits: &w::Limits,
) -> Result<w::PolicyRequirement, w::Problem> {
    w::PolicyRequirement::new(
        limits,
        w::PolicyRequirementParts {
            connector: value.connector,
            kind: value.kind,
            pattern: wire_owner_pattern(&value.pattern, limits)?,
            judge: wire_policy_judge(value.judge, limits)?,
            guard: wire_policy_guard(&value.guard, limits)?,
            must_be_guarded: value.must_be_guarded,
        },
    )
}

fn wire_policy_requirement_piece(
    rows: &List<w::PolicyRequirement>,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut length = 0_u32;
    for row in rows {
        length = length.checked_add(row.measure()).ok_or(too_large())?;
        length = length.checked_add(4).ok_or(too_large())?;
    }
    if length > limits.owned_record {
        return Err(too_large());
    }
    let mut writer = Writer::new(usize::try_from(length).expect("u32 fits usize"));
    for row in rows {
        let mut item = Writer::new(usize::try_from(row.measure()).expect("u32 fits usize"));
        row.encode(&mut item).expect("measured row");
        let item = item.finish();
        skein_codec::piece::write(&mut writer, &item).expect("measured piece");
    }
    Ok(writer.finish())
}

pub(crate) fn wire_policy_permission(
    value: c::PolicyPermission,
    limits: &w::Limits,
) -> Result<w::PolicyPermission, w::Problem> {
    w::PolicyPermission::new(
        limits,
        w::PolicyPermissionParts { connector: value.connector, permission: value.permission, role: value.role },
    )
}

pub(crate) fn wire_policy_edit(value: &c::PolicyEdit, limits: &w::Limits) -> Result<w::PolicyEdit, w::Problem> {
    Ok(match value {
        c::PolicyEdit::ProjectSpend { period_spend } => w::PolicyEdit::ProjectSpend(w::PolicyEditProjectSpend::new(
            limits,
            w::PolicyEditProjectSpendParts { period_spend: *period_spend },
        )?),
        c::PolicyEdit::Role(value) => w::PolicyEdit::Role(w::PolicyEditRoleValue::new(
            limits,
            w::PolicyEditRoleValueParts { value: wire_policy_edit_role(value, limits)? },
        )?),
        c::PolicyEdit::Requirements { requirements } => {
            let mut list0 = List::with_capacity((requirements).len());
            for item in requirements {
                list0.push(wire_policy_requirement(item, limits)?).expect("reserved supplied list");
            }
            w::PolicyEdit::Requirements(w::PolicyEditRequirements::new(
                limits,
                w::PolicyEditRequirementsParts { requirements: wire_policy_requirement_piece(&list0, limits)? },
            )?)
        }
        c::PolicyEdit::Projections { grants } => {
            let mut list0 = List::with_capacity((grants).len());
            for item in grants {
                list0.push(wire_owner_grant(item, limits)?).expect("reserved supplied list");
            }
            w::PolicyEdit::Projections(w::PolicyEditProjections::new(
                limits,
                w::PolicyEditProjectionsParts { grants: wire_owner_grant_piece(&list0, limits)? },
            )?)
        }
        c::PolicyEdit::Permissions { mappings } => {
            let mut list0 = List::with_capacity((mappings).len());
            for item in mappings {
                list0.push(wire_policy_permission(*item, limits)?).expect("reserved supplied list");
            }
            w::PolicyEdit::Permissions(w::PolicyEditPermissions::new(
                limits,
                w::PolicyEditPermissionsParts { mappings: list0 },
            )?)
        }
    })
}

pub(crate) fn wire_held_role(value: c::HeldRole, limits: &w::Limits) -> Result<w::HeldRole, w::Problem> {
    Ok(match value {
        c::HeldRole::Owner => w::HeldRole::Owner,
        c::HeldRole::Maintainer => w::HeldRole::Maintainer,
        c::HeldRole::Member => w::HeldRole::Member,
        c::HeldRole::Observer => w::HeldRole::Observer,
        c::HeldRole::Policy { role } => {
            w::HeldRole::Policy(w::HeldRolePolicy::new(limits, w::HeldRolePolicyParts { role })?)
        }
    })
}

pub(crate) fn wire_holding(value: &c::Holding, limits: &w::Limits) -> Result<w::Holding, w::Problem> {
    w::Holding::new(limits, w::HoldingParts { party: value.party, role: wire_held_role(value.role, limits)? })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
#[expect(clippy::too_many_lines, reason = "one exhaustive field translation arm per client ask")]
pub fn ask(value: &c::Ask, limits: &w::Limits) -> Result<w::Ask, w::Problem> {
    Ok(match value {
        c::Ask::StartChat { project, words } => w::Ask::StartChat(w::StartChat::new(
            limits,
            w::StartChatParts { project: *project, words: bytes::copy_of(words.bytes()) },
        )?),
        c::Ask::Write { project, task, words, release } => w::Ask::Write(w::Write::new(
            limits,
            w::WriteParts { project: *project, task: *task, words: bytes::copy_of(words.bytes()), release: *release },
        )?),
        c::Ask::AnswerQuestion { project, task, message, words } => w::Ask::AnswerQuestion(w::AnswerQuestion::new(
            limits,
            w::AnswerQuestionParts {
                project: *project,
                task: *task,
                message: *message,
                words: bytes::copy_of(words.bytes()),
            },
        )?),
        c::Ask::AnswerPerson { project, task, revision, answer } => w::Ask::AnswerPerson(w::AnswerPerson::new(
            limits,
            w::AnswerPersonParts {
                project: *project,
                task: *task,
                revision: wire_revision(*revision, limits)?,
                answer: wire_person_answer(answer, limits)?,
            },
        )?),
        c::Ask::Take { project, task } => {
            w::Ask::Take(w::Take::new(limits, w::TakeParts { project: *project, task: *task })?)
        }
        c::Ask::HandBack { project, task } => {
            w::Ask::HandBack(w::HandBack::new(limits, w::HandBackParts { project: *project, task: *task })?)
        }
        c::Ask::DecideProposal { project, proposal, decision } => w::Ask::DecideProposal(w::DecideProposal::new(
            limits,
            w::DecideProposalParts {
                project: *project,
                proposal: wire_proposal_key(proposal, limits)?,
                decision: wire_decision(decision, limits)?,
            },
        )?),
        c::Ask::DecideEscalation { project, task, revision, decision } => {
            w::Ask::DecideEscalation(w::DecideEscalation::new(
                limits,
                w::DecideEscalationParts {
                    project: *project,
                    task: *task,
                    revision: wire_revision(*revision, limits)?,
                    decision: wire_decision(decision, limits)?,
                },
            )?)
        }
        c::Ask::Release { project, task, revision, again } => {
            let opt0 = match again {
                Some(item) => Some(*item),
                None => None,
            };
            w::Ask::Release(w::Release::new(
                limits,
                w::ReleaseParts {
                    project: *project,
                    task: *task,
                    revision: wire_revision(*revision, limits)?,
                    again: opt0,
                },
            )?)
        }
        c::Ask::Amend { project, task, revision, change, release, reason } => {
            let opt0 = match release {
                Some(item) => Some(wire_revision(*item, limits)?),
                None => None,
            };
            w::Ask::Amend(Box::new(w::Amend::new(
                limits,
                w::AmendParts {
                    project: *project,
                    task: *task,
                    revision: wire_revision(*revision, limits)?,
                    change: wire_amendment(change, limits)?,
                    release: opt0,
                    reason: bytes::copy_of(reason.bytes()),
                },
            )?))
        }
        c::Ask::Stop { project, task } => {
            w::Ask::Stop(w::Stop::new(limits, w::StopParts { project: *project, task: *task })?)
        }
        c::Ask::Cancel { project, task, reason } => w::Ask::Cancel(w::Cancel::new(
            limits,
            w::CancelParts { project: *project, task: *task, reason: bytes::copy_of(reason.bytes()) },
        )?),
        c::Ask::Close { project, chat, disposals } => {
            let mut list0 = List::with_capacity((disposals).len());
            for item in disposals {
                list0.push(wire_disposal(item, limits)?).expect("reserved supplied list");
            }
            w::Ask::Close(w::Close::new(limits, w::CloseParts { project: *project, chat: *chat, disposals: list0 })?)
        }
        c::Ask::TakeOver { project, task, reason } => w::Ask::TakeOver(w::TakeOver::new(
            limits,
            w::TakeOverParts { project: *project, task: *task, reason: bytes::copy_of(reason.bytes()) },
        )?),
        c::Ask::SetGoal { project, spec, executor, budget, priority, deadline } => {
            let opt0 = match deadline {
                Some(item) => Some(item.as_nanos()),
                None => None,
            };
            w::Ask::SetGoal(w::SetGoal::new(
                limits,
                w::SetGoalParts {
                    project: *project,
                    spec: bytes::copy_of(spec.bytes()),
                    executor: wire_goal_executor(*executor, limits)?,
                    budget: *budget,
                    priority: *priority,
                    deadline: opt0,
                },
            )?)
        }
        c::Ask::Prioritise { project, goals } => {
            let mut list0 = List::with_capacity((goals).len());
            for item in goals {
                list0.push(wire_priority(item, limits)?).expect("reserved supplied list");
            }
            w::Ask::Prioritise(w::Prioritise::new(limits, w::PrioritiseParts { project: *project, goals: list0 })?)
        }
        c::Ask::MarkSeen { seen } => {
            w::Ask::MarkSeen(w::MarkSeen::new(limits, w::MarkSeenParts { seen: wire_seen(seen, limits)? })?)
        }
        c::Ask::EditNote { project, note, scope, revision, description, body, references } => {
            let mut list0 = List::with_capacity((references).len());
            for item in references {
                list0.push(*item).expect("reserved supplied list");
            }
            w::Ask::EditNote(w::EditNote::new(
                limits,
                w::EditNoteParts {
                    project: *project,
                    note: *note,
                    scope: wire_note_scope(scope, limits)?,
                    revision: wire_revision(*revision, limits)?,
                    description: bytes::copy_of(description.bytes()),
                    body: bytes::copy_of(body.bytes()),
                    references: list0,
                },
            )?)
        }
        c::Ask::DeleteNote { project, note, scope, revision } => w::Ask::DeleteNote(w::DeleteNote::new(
            limits,
            w::DeleteNoteParts {
                project: *project,
                note: *note,
                scope: wire_note_scope(scope, limits)?,
                revision: wire_revision(*revision, limits)?,
            },
        )?),
        c::Ask::ChangePolicy { project, edit } => w::Ask::ChangePolicy(w::ChangePolicy::new(
            limits,
            w::ChangePolicyParts { project: *project, edit: wire_policy_edit(edit, limits)? },
        )?),
        c::Ask::SetRoles { project, holdings } => {
            let mut list0 = List::with_capacity((holdings).len());
            for item in holdings {
                list0.push(wire_holding(item, limits)?).expect("reserved supplied list");
            }
            w::Ask::SetRoles(w::SetRoles::new(limits, w::SetRolesParts { project: *project, holdings: list0 })?)
        }
        c::Ask::MakeService { project, name, role } => w::Ask::MakeService(w::MakeService::new(
            limits,
            w::MakeServiceParts {
                project: *project,
                name: bytes::copy_of(name.bytes()),
                role: wire_held_role(*role, limits)?,
            },
        )?),
        c::Ask::SetPool { project, party, budget } => w::Ask::SetPool(w::SetPool::new(
            limits,
            w::SetPoolParts { project: *project, party: *party, budget: *budget },
        )?),
    })
}
