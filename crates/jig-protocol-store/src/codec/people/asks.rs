//! Asks values retained by people (protocol/store.md, 5.1).
use super::{
    Box, List, Wall, count, decode_amendment, decode_holdings, decode_numbers, decode_pattern, decode_policy_change,
    decode_role, decode_segments, domain, encode_amendment, encode_holdings, encode_numbers, encode_pattern,
    encode_policy_change, encode_role, encode_segments, wire,
};
pub(super) fn encode_seen(value: &domain::Seen, limits: &wire::Limits) -> Result<wire::Seen, wire::Problem> {
    Ok(match value {
        domain::Seen::Results { through } => {
            wire::Seen::Results(wire::SeenResults::new(limits, wire::SeenResultsParts { through: *through })?)
        }
        domain::Seen::Chat { task, through } => {
            wire::Seen::Chat(wire::SeenChat::new(limits, wire::SeenChatParts { task: *task, through: *through })?)
        }
    })
}
pub(super) fn decode_seen(value: wire::Seen) -> domain::Seen {
    match value {
        wire::Seen::Results(value) => {
            let wire::SeenResultsParts { through } = value.into_parts();
            domain::Seen::Results { through }
        }
        wire::Seen::Chat(value) => {
            let wire::SeenChatParts { task, through } = value.into_parts();
            domain::Seen::Chat { task, through }
        }
    }
}

pub(super) fn encode_note_scope(
    value: &domain::NoteScope,
    limits: &wire::Limits,
) -> Result<wire::NoteScope, wire::Problem> {
    Ok(match value {
        domain::NoteScope::Deployment => wire::NoteScope::Deployment,
        domain::NoteScope::Project => wire::NoteScope::Project,
        domain::NoteScope::Goal { goal } => {
            wire::NoteScope::Goal(wire::NoteScopeGoal::new(limits, wire::NoteScopeGoalParts { goal: *goal })?)
        }
        domain::NoteScope::Resources { connector, pattern } => {
            wire::NoteScope::Resources(wire::NoteScopeResources::new(
                limits,
                wire::NoteScopeResourcesParts { connector: *connector, pattern: encode_pattern(pattern, limits)? },
            )?)
        }
    })
}
pub(super) fn decode_note_scope(
    value: wire::NoteScope,
    limits: &wire::Limits,
) -> Result<domain::NoteScope, wire::Problem> {
    Ok(match value {
        wire::NoteScope::Deployment => domain::NoteScope::Deployment,
        wire::NoteScope::Project => domain::NoteScope::Project,
        wire::NoteScope::Goal(value) => {
            let wire::NoteScopeGoalParts { goal } = value.into_parts();
            domain::NoteScope::Goal { goal }
        }
        wire::NoteScope::Resources(value) => {
            let wire::NoteScopeResourcesParts { connector, pattern } = value.into_parts();
            domain::NoteScope::Resources { connector, pattern: decode_pattern(pattern, limits)? }
        }
    })
}

pub(super) fn encode_note_change(
    value: &domain::NoteChange,
    limits: &wire::Limits,
) -> Result<wire::NoteChange, wire::Problem> {
    Ok(match value {
        domain::NoteChange::Correct { description, body, references, recalled } => {
            wire::NoteChange::Correct(wire::NoteChangeCorrect::new(
                limits,
                wire::NoteChangeCorrectParts {
                    description: description.clone(),
                    body: body.clone(),
                    references: encode_numbers(
                        references,
                        limits,
                        limits.parameter_bytes_value,
                        limits.note_change_correct_references,
                    )?,
                    recalled: *recalled,
                },
            )?)
        }
        domain::NoteChange::Delete { recalled } => wire::NoteChange::Delete(wire::NoteChangeDelete::new(
            limits,
            wire::NoteChangeDeleteParts { recalled: *recalled },
        )?),
    })
}
pub(super) fn decode_note_change(
    value: wire::NoteChange,
    limits: &wire::Limits,
) -> Result<domain::NoteChange, wire::Problem> {
    Ok(match value {
        wire::NoteChange::Correct(value) => {
            let wire::NoteChangeCorrectParts { description, body, references, recalled } = value.into_parts();
            domain::NoteChange::Correct {
                description,
                body,
                references: decode_numbers(references, limits, limits.parameter_bytes_value)?,
                recalled,
            }
        }
        wire::NoteChange::Delete(value) => {
            let wire::NoteChangeDeleteParts { recalled } = value.into_parts();
            domain::NoteChange::Delete { recalled }
        }
    })
}

pub(super) fn encode_resource_role(value: domain::ResourceRole, _limits: &wire::Limits) -> wire::ResourceRole {
    match value {
        domain::ResourceRole::Owned => wire::ResourceRole::Owned,
        domain::ResourceRole::Fork => wire::ResourceRole::Fork,
        domain::ResourceRole::Context => wire::ResourceRole::Context,
    }
}
pub(super) fn decode_resource_role(value: wire::ResourceRole) -> domain::ResourceRole {
    match value {
        wire::ResourceRole::Owned => domain::ResourceRole::Owned,
        wire::ResourceRole::Fork => domain::ResourceRole::Fork,
        wire::ResourceRole::Context => domain::ResourceRole::Context,
    }
}

pub(super) fn encode_resource_name(
    value: &domain::ResourceName,
    limits: &wire::Limits,
) -> Result<wire::ResourceName, wire::Problem> {
    let domain::ResourceName { connector, path } = value;
    wire::ResourceName::new(
        limits,
        wire::ResourceNameParts {
            connector: *connector,
            path: encode_segments(path, limits, limits.parameter_bytes_value, limits.resource_name_path)?,
        },
    )
}
pub(super) fn decode_resource_name(
    value: wire::ResourceName,
    limits: &wire::Limits,
) -> Result<domain::ResourceName, wire::Problem> {
    let wire::ResourceNameParts { connector, path } = value.into_parts();
    Ok(domain::ResourceName { connector, path: decode_segments(path, limits, limits.parameter_bytes_value)? })
}

pub(super) fn encode_adoption(
    value: &domain::Adoption,
    limits: &wire::Limits,
) -> Result<wire::Adoption, wire::Problem> {
    let domain::Adoption { resource, role, options } = value;
    wire::Adoption::new(
        limits,
        wire::AdoptionParts {
            resource: encode_resource_name(resource, limits)?,
            role: encode_resource_role(*role, limits),
            options: encode_segments(options, limits, limits.parameter_bytes_value, limits.adoption_options)?,
        },
    )
}
pub(super) fn decode_adoption(value: wire::Adoption, limits: &wire::Limits) -> Result<domain::Adoption, wire::Problem> {
    let wire::AdoptionParts { resource, role, options } = value.into_parts();
    Ok(domain::Adoption {
        resource: decode_resource_name(resource, limits)?,
        role: decode_resource_role(role),
        options: decode_segments(options, limits, limits.parameter_bytes_value)?,
    })
}

pub(super) fn encode_disposal(
    value: &domain::Disposal,
    limits: &wire::Limits,
) -> Result<wire::Disposal, wire::Problem> {
    let domain::Disposal { task, keep } = value;
    wire::Disposal::new(limits, wire::DisposalParts { task: *task, keep: encode_keep(*keep, limits) })
}
pub(super) fn decode_disposal(value: wire::Disposal) -> domain::Disposal {
    let wire::DisposalParts { task, keep } = value.into_parts();
    domain::Disposal { task, keep: decode_keep(keep) }
}

pub(super) fn encode_keep(value: domain::Keep, _limits: &wire::Limits) -> wire::Keep {
    match value {
        domain::Keep::Cancel => wire::Keep::Cancel,
        domain::Keep::MakeMine => wire::Keep::MakeMine,
    }
}
pub(super) fn decode_keep(value: wire::Keep) -> domain::Keep {
    match value {
        wire::Keep::Cancel => domain::Keep::Cancel,
        wire::Keep::MakeMine => domain::Keep::MakeMine,
    }
}

pub(super) fn encode_goal_executor(
    value: domain::GoalExecutor,
    limits: &wire::Limits,
) -> Result<wire::GoalExecutor, wire::Problem> {
    Ok(match value {
        domain::GoalExecutor::Charter(value) => wire::GoalExecutor::Charter(wire::GoalExecutorCharter::new(
            limits,
            wire::GoalExecutorCharterParts { value },
        )?),
        domain::GoalExecutor::Procedure { connector, code } => wire::GoalExecutor::Procedure(
            wire::GoalExecutorProcedure::new(limits, wire::GoalExecutorProcedureParts { connector, code })?,
        ),
    })
}
pub(super) fn decode_goal_executor(value: wire::GoalExecutor) -> domain::GoalExecutor {
    match value {
        wire::GoalExecutor::Charter(value) => {
            let wire::GoalExecutorCharterParts { value } = value.into_parts();
            domain::GoalExecutor::Charter(value)
        }
        wire::GoalExecutor::Procedure(value) => {
            let wire::GoalExecutorProcedureParts { connector, code } = value.into_parts();
            domain::GoalExecutor::Procedure { connector, code }
        }
    }
}

pub(super) fn encode_person_result(
    value: &domain::PersonResult,
    limits: &wire::Limits,
) -> Result<wire::PersonResult, wire::Problem> {
    Ok(match value {
        domain::PersonResult::Approved { state, words } => {
            wire::PersonResult::Approved(wire::PersonResultApproved::new(
                limits,
                wire::PersonResultApprovedParts { state: state.clone(), words: words.clone() },
            )?)
        }
        domain::PersonResult::Report { words } => wire::PersonResult::Report(wire::PersonResultReport::new(
            limits,
            wire::PersonResultReportParts { words: words.clone() },
        )?),
        domain::PersonResult::Verdict { code, words } => wire::PersonResult::Verdict(wire::PersonResultVerdict::new(
            limits,
            wire::PersonResultVerdictParts { code: *code, words: words.clone() },
        )?),
        domain::PersonResult::Failure { reason } => wire::PersonResult::Failure(wire::PersonResultFailure::new(
            limits,
            wire::PersonResultFailureParts { reason: reason.clone() },
        )?),
    })
}
pub(super) fn decode_person_result(value: wire::PersonResult) -> domain::PersonResult {
    match value {
        wire::PersonResult::Approved(value) => {
            let wire::PersonResultApprovedParts { state, words } = value.into_parts();
            domain::PersonResult::Approved { state, words }
        }
        wire::PersonResult::Report(value) => {
            let wire::PersonResultReportParts { words } = value.into_parts();
            domain::PersonResult::Report { words }
        }
        wire::PersonResult::Verdict(value) => {
            let wire::PersonResultVerdictParts { code, words } = value.into_parts();
            domain::PersonResult::Verdict { code, words }
        }
        wire::PersonResult::Failure(value) => {
            let wire::PersonResultFailureParts { reason } = value.into_parts();
            domain::PersonResult::Failure { reason }
        }
    }
}

pub(super) fn encode_proposer(
    value: &domain::Proposer,
    limits: &wire::Limits,
) -> Result<wire::Proposer, wire::Problem> {
    Ok(match value {
        domain::Proposer::Task(value) => {
            wire::Proposer::Task(wire::ProposerTask::new(limits, wire::ProposerTaskParts { value: *value })?)
        }
        domain::Proposer::Party(value) => {
            wire::Proposer::Party(wire::ProposerParty::new(limits, wire::ProposerPartyParts { value: *value })?)
        }
    })
}
pub(super) fn decode_proposer(value: wire::Proposer) -> domain::Proposer {
    match value {
        wire::Proposer::Task(value) => {
            let wire::ProposerTaskParts { value } = value.into_parts();
            domain::Proposer::Task(value)
        }
        wire::Proposer::Party(value) => {
            let wire::ProposerPartyParts { value } = value.into_parts();
            domain::Proposer::Party(value)
        }
    }
}

pub(super) fn encode_proposal_decision(
    value: &domain::ProposalDecision,
    limits: &wire::Limits,
) -> Result<wire::ProposalDecision, wire::Problem> {
    Ok(match value {
        domain::ProposalDecision::Accept => wire::ProposalDecision::Accept,
        domain::ProposalDecision::Reject { reason } => wire::ProposalDecision::Reject(
            wire::ProposalDecisionReject::new(limits, wire::ProposalDecisionRejectParts { reason: reason.clone() })?,
        ),
        domain::ProposalDecision::Pass => wire::ProposalDecision::Pass,
    })
}
pub(super) fn decode_proposal_decision(value: wire::ProposalDecision) -> domain::ProposalDecision {
    match value {
        wire::ProposalDecision::Accept => domain::ProposalDecision::Accept,
        wire::ProposalDecision::Reject(value) => {
            let wire::ProposalDecisionRejectParts { reason } = value.into_parts();
            domain::ProposalDecision::Reject { reason }
        }
        wire::ProposalDecision::Pass => domain::ProposalDecision::Pass,
    }
}

pub(super) fn encode_escalation_decision(
    value: &domain::EscalationDecision,
    limits: &wire::Limits,
) -> Result<wire::EscalationDecision, wire::Problem> {
    Ok(match value {
        domain::EscalationDecision::Release => wire::EscalationDecision::Release,
        domain::EscalationDecision::Reject { reason } => {
            wire::EscalationDecision::Reject(wire::EscalationDecisionReject::new(
                limits,
                wire::EscalationDecisionRejectParts { reason: reason.clone() },
            )?)
        }
        domain::EscalationDecision::Pass => wire::EscalationDecision::Pass,
    })
}
pub(super) fn decode_escalation_decision(value: wire::EscalationDecision) -> domain::EscalationDecision {
    match value {
        wire::EscalationDecision::Release => domain::EscalationDecision::Release,
        wire::EscalationDecision::Reject(value) => {
            let wire::EscalationDecisionRejectParts { reason } = value.into_parts();
            domain::EscalationDecision::Reject { reason }
        }
        wire::EscalationDecision::Pass => domain::EscalationDecision::Pass,
    }
}

#[expect(clippy::too_many_lines, reason = "one exhaustive arm translates each kind of keyed request")]
#[expect(clippy::manual_map, reason = "step code translates optional fields through exhaustive matches")]
pub fn encode_ask(value: &domain::Ask, limits: &wire::Limits) -> Result<wire::Ask, wire::Problem> {
    Ok(match value {
        domain::Ask::Create { project, batch } => wire::Ask::Create(wire::AskCreate::new(
            limits,
            wire::AskCreateParts { project: *project, batch: super::batch::encode(batch, limits)? },
        )?),
        domain::Ask::Close { project, chat, disposals } => wire::Ask::Close(wire::AskClose::new(
            limits,
            wire::AskCloseParts {
                project: *project,
                chat: *chat,
                disposals: encode_ask_close_disposals(disposals, limits)?,
            },
        )?),
        domain::Ask::MarkSeen { seen } => wire::Ask::MarkSeen(wire::AskMarkSeen::new(
            limits,
            wire::AskMarkSeenParts { seen: encode_seen(seen, limits)? },
        )?),
        domain::Ask::Application { scope, connector, digest } => wire::Ask::Application(wire::AskApplication::new(
            limits,
            wire::AskApplicationParts {
                scope: encode_application_scope(*scope, limits)?,
                connector: *connector,
                digest: *digest,
            },
        )?),
        domain::Ask::EditNote { project, name, scope, change } => wire::Ask::EditNote(wire::AskEditNote::new(
            limits,
            wire::AskEditNoteParts {
                project: *project,
                name: *name,
                scope: encode_note_scope(scope, limits)?,
                change: encode_note_change(change, limits)?,
            },
        )?),
        domain::Ask::MakeService { project, name, role } => wire::Ask::MakeService(wire::AskMakeService::new(
            limits,
            wire::AskMakeServiceParts { project: *project, name: name.clone(), role: encode_role(*role, limits)? },
        )?),
        domain::Ask::Adopt { project, adoption } => wire::Ask::Adopt(wire::AskAdopt::new(
            limits,
            wire::AskAdoptParts { project: *project, adoption: encode_adoption(adoption, limits)? },
        )?),
        domain::Ask::SetGoal { project, spec, executor, budget, priority, deadline } => {
            wire::Ask::SetGoal(wire::AskSetGoal::new(
                limits,
                wire::AskSetGoalParts {
                    project: *project,
                    spec: spec.clone(),
                    executor: encode_goal_executor(*executor, limits)?,
                    budget: *budget,
                    priority: *priority,
                    deadline: match deadline {
                        Some(value) => Some(value.as_nanos()),
                        None => None,
                    },
                },
            )?)
        }
        domain::Ask::Stop { project, task } => {
            wire::Ask::Stop(wire::AskStop::new(limits, wire::AskStopParts { project: *project, task: *task })?)
        }
        domain::Ask::Cancel { project, task, reason } => wire::Ask::Cancel(wire::AskCancel::new(
            limits,
            wire::AskCancelParts { project: *project, task: *task, reason: reason.clone() },
        )?),
        domain::Ask::Release { project, task, revision, again } => wire::Ask::Release(wire::AskRelease::new(
            limits,
            wire::AskReleaseParts { project: *project, task: *task, revision: *revision, again: *again },
        )?),
        domain::Ask::TakePerson { project, task } => wire::Ask::TakePerson(wire::AskTakePerson::new(
            limits,
            wire::AskTakePersonParts { project: *project, task: *task },
        )?),
        domain::Ask::HandBackPerson { project, task } => wire::Ask::HandBackPerson(wire::AskHandBackPerson::new(
            limits,
            wire::AskHandBackPersonParts { project: *project, task: *task },
        )?),
        domain::Ask::AnswerPerson { project, task, revision, result } => {
            wire::Ask::AnswerPerson(wire::AskAnswerPerson::new(
                limits,
                wire::AskAnswerPersonParts {
                    project: *project,
                    task: *task,
                    revision: *revision,
                    result: encode_person_result(result, limits)?,
                },
            )?)
        }
        domain::Ask::Move { project, task, reason } => wire::Ask::MoveTask(wire::AskMove::new(
            limits,
            wire::AskMoveParts { project: *project, task: *task, reason: reason.clone() },
        )?),
        domain::Ask::DecideProposal { project, proposer, proposal, decision } => {
            wire::Ask::DecideProposal(wire::AskDecideProposal::new(
                limits,
                wire::AskDecideProposalParts {
                    project: *project,
                    proposer: encode_proposer(proposer, limits)?,
                    proposal: *proposal,
                    decision: encode_proposal_decision(decision, limits)?,
                },
            )?)
        }
        domain::Ask::Say { project, task, words, release } => wire::Ask::Say(wire::AskSay::new(
            limits,
            wire::AskSayParts { project: *project, task: *task, words: words.clone(), release: *release },
        )?),
        domain::Ask::AnswerQuestion { project, task, question, words } => {
            wire::Ask::AnswerQuestion(wire::AskAnswerQuestion::new(
                limits,
                wire::AskAnswerQuestionParts {
                    project: *project,
                    task: *task,
                    question: *question,
                    words: words.clone(),
                },
            )?)
        }
        domain::Ask::Prioritise { project, goals } => wire::Ask::Prioritise(wire::AskPrioritise::new(
            limits,
            wire::AskPrioritiseParts { project: *project, goals: encode_ask_prioritise_goals(goals, limits)? },
        )?),
        domain::Ask::Amend { project, task, revision, amendment, release } => {
            wire::Ask::Amend(Box::new(wire::AskAmend::new(
                limits,
                wire::AskAmendParts {
                    project: *project,
                    task: *task,
                    revision: *revision,
                    amendment: encode_amendment(amendment, limits)?,
                    release: *release,
                },
            )?))
        }
        domain::Ask::SetRoles { project, holdings } => wire::Ask::SetRoles(wire::AskSetRoles::new(
            limits,
            wire::AskSetRolesParts { project: *project, holdings: encode_holdings(holdings, limits)? },
        )?),
        domain::Ask::ChangePolicy { project, change } => wire::Ask::ChangePolicy(wire::AskChangePolicy::new(
            limits,
            wire::AskChangePolicyParts { project: *project, change: encode_policy_change(change, limits)? },
        )?),
        domain::Ask::SetPool { project, person, budget } => wire::Ask::SetPool(wire::AskSetPool::new(
            limits,
            wire::AskSetPoolParts { project: *project, person: *person, budget: *budget },
        )?),
        domain::Ask::DecideEscalation { project, task, revision, decision } => {
            wire::Ask::DecideEscalation(wire::AskDecideEscalation::new(
                limits,
                wire::AskDecideEscalationParts {
                    project: *project,
                    task: *task,
                    revision: *revision,
                    decision: encode_escalation_decision(decision, limits)?,
                },
            )?)
        }
        domain::Ask::StartChat { project, words } => wire::Ask::StartChat(wire::AskStartChat::new(
            limits,
            wire::AskStartChatParts { project: *project, words: words.clone() },
        )?),
    })
}
#[expect(clippy::too_many_lines, reason = "one exhaustive arm translates each kind of keyed request")]
#[expect(clippy::manual_map, reason = "step code translates optional fields through exhaustive matches")]
pub fn decode_ask(value: wire::Ask, limits: &wire::Limits) -> Result<domain::Ask, wire::Problem> {
    Ok(match value {
        wire::Ask::Create(value) => {
            let wire::AskCreateParts { project, batch } = value.into_parts();
            domain::Ask::Create { project, batch: super::batch::decode(batch, limits)? }
        }
        wire::Ask::Close(value) => {
            let wire::AskCloseParts { project, chat, disposals } = value.into_parts();
            domain::Ask::Close { project, chat, disposals: decode_ask_close_disposals(disposals) }
        }
        wire::Ask::MarkSeen(value) => {
            let wire::AskMarkSeenParts { seen } = value.into_parts();
            domain::Ask::MarkSeen { seen: decode_seen(seen) }
        }
        wire::Ask::Application(value) => {
            let wire::AskApplicationParts { scope, connector, digest } = value.into_parts();
            domain::Ask::Application { scope: decode_application_scope(scope), connector, digest }
        }
        wire::Ask::EditNote(value) => {
            let wire::AskEditNoteParts { project, name, scope, change } = value.into_parts();
            domain::Ask::EditNote {
                project,
                name,
                scope: Box::new(decode_note_scope(scope, limits)?),
                change: Box::new(decode_note_change(change, limits)?),
            }
        }
        wire::Ask::MakeService(value) => {
            let wire::AskMakeServiceParts { project, name, role } = value.into_parts();
            domain::Ask::MakeService { project, name, role: decode_role(role) }
        }
        wire::Ask::Adopt(value) => {
            let wire::AskAdoptParts { project, adoption } = value.into_parts();
            domain::Ask::Adopt { project, adoption: decode_adoption(adoption, limits)? }
        }
        wire::Ask::SetGoal(value) => {
            let wire::AskSetGoalParts { project, spec, executor, budget, priority, deadline } = value.into_parts();
            domain::Ask::SetGoal {
                project,
                spec,
                executor: decode_goal_executor(executor),
                budget,
                priority,
                deadline: match deadline {
                    Some(value) => Some(Wall::from_nanos(value)),
                    None => None,
                },
            }
        }
        wire::Ask::Stop(value) => {
            let wire::AskStopParts { project, task } = value.into_parts();
            domain::Ask::Stop { project, task }
        }
        wire::Ask::Cancel(value) => {
            let wire::AskCancelParts { project, task, reason } = value.into_parts();
            domain::Ask::Cancel { project, task, reason }
        }
        wire::Ask::Release(value) => {
            let wire::AskReleaseParts { project, task, revision, again } = value.into_parts();
            domain::Ask::Release { project, task, revision, again }
        }
        wire::Ask::TakePerson(value) => {
            let wire::AskTakePersonParts { project, task } = value.into_parts();
            domain::Ask::TakePerson { project, task }
        }
        wire::Ask::HandBackPerson(value) => {
            let wire::AskHandBackPersonParts { project, task } = value.into_parts();
            domain::Ask::HandBackPerson { project, task }
        }
        wire::Ask::AnswerPerson(value) => {
            let wire::AskAnswerPersonParts { project, task, revision, result } = value.into_parts();
            domain::Ask::AnswerPerson { project, task, revision, result: decode_person_result(result) }
        }
        wire::Ask::MoveTask(value) => {
            let wire::AskMoveParts { project, task, reason } = value.into_parts();
            domain::Ask::Move { project, task, reason }
        }
        wire::Ask::DecideProposal(value) => {
            let wire::AskDecideProposalParts { project, proposer, proposal, decision } = value.into_parts();
            domain::Ask::DecideProposal {
                project,
                proposer: decode_proposer(proposer),
                proposal,
                decision: decode_proposal_decision(decision),
            }
        }
        wire::Ask::Say(value) => {
            let wire::AskSayParts { project, task, words, release } = value.into_parts();
            domain::Ask::Say { project, task, words, release }
        }
        wire::Ask::AnswerQuestion(value) => {
            let wire::AskAnswerQuestionParts { project, task, question, words } = value.into_parts();
            domain::Ask::AnswerQuestion { project, task, question, words }
        }
        wire::Ask::Prioritise(value) => {
            let wire::AskPrioritiseParts { project, goals } = value.into_parts();
            domain::Ask::Prioritise { project, goals: decode_ask_prioritise_goals(goals) }
        }
        wire::Ask::Amend(value) => {
            let wire::AskAmendParts { project, task, revision, amendment, release } = (*value).into_parts();
            domain::Ask::Amend { project, task, revision, amendment: decode_amendment(amendment, limits)?, release }
        }
        wire::Ask::SetRoles(value) => {
            let wire::AskSetRolesParts { project, holdings } = value.into_parts();
            domain::Ask::SetRoles { project, holdings: decode_holdings(holdings) }
        }
        wire::Ask::ChangePolicy(value) => {
            let wire::AskChangePolicyParts { project, change } = value.into_parts();
            domain::Ask::ChangePolicy { project, change: decode_policy_change(change, limits)? }
        }
        wire::Ask::SetPool(value) => {
            let wire::AskSetPoolParts { project, person, budget } = value.into_parts();
            domain::Ask::SetPool { project, person, budget }
        }
        wire::Ask::DecideEscalation(value) => {
            let wire::AskDecideEscalationParts { project, task, revision, decision } = value.into_parts();
            domain::Ask::DecideEscalation { project, task, revision, decision: decode_escalation_decision(decision) }
        }
        wire::Ask::StartChat(value) => {
            let wire::AskStartChatParts { project, words } = value.into_parts();
            domain::Ask::StartChat { project, words }
        }
    })
}

fn encode_ask_close_disposals(
    values: &List<domain::Disposal>,
    limits: &wire::Limits,
) -> Result<List<wire::Disposal>, wire::Problem> {
    let mut result = List::with_capacity(values.len());
    for value in values {
        result.push(encode_disposal(value, limits)?).expect("same number of members");
    }
    Ok(result)
}
fn decode_ask_close_disposals(values: List<wire::Disposal>) -> List<domain::Disposal> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_disposal(value)).expect("same number of members");
    }
    result
}

fn encode_ask_prioritise_goals(
    values: &[(u64, u32)],
    limits: &wire::Limits,
) -> Result<List<wire::Priority>, wire::Problem> {
    let mut result = List::with_capacity(count(values.len())?);
    for value in values {
        result
            .push(wire::Priority::new(limits, wire::PriorityParts { task: value.0, priority: value.1 })?)
            .expect("same number of members");
    }
    Ok(result)
}
fn decode_ask_prioritise_goals(values: List<wire::Priority>) -> Box<[(u64, u32)]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result
            .push({
                let wire::PriorityParts { task, priority } = value.into_parts();
                (task, priority)
            })
            .expect("same number of members");
    }
    result.into_boxed()
}

fn encode_application_scope(
    scope: domain::ApplicationScope,
    limits: &wire::Limits,
) -> Result<wire::ApplicationScope, wire::Problem> {
    Ok(match scope {
        domain::ApplicationScope::Project(project) => {
            wire::ApplicationScope::Project(wire::ScopeProject::new(limits, wire::ScopeProjectParts { project })?)
        }
        domain::ApplicationScope::Deployment => wire::ApplicationScope::Deployment,
        domain::ApplicationScope::Own => wire::ApplicationScope::Own,
    })
}
fn decode_application_scope(scope: wire::ApplicationScope) -> domain::ApplicationScope {
    match scope {
        wire::ApplicationScope::Project(project) => domain::ApplicationScope::Project(project.into_parts().project),
        wire::ApplicationScope::Deployment => domain::ApplicationScope::Deployment,
        wire::ApplicationScope::Own => domain::ApplicationScope::Own,
    }
}
