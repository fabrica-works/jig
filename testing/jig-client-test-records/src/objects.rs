//! Independent wire literals for single-read documents (testing-strategy.md, 8).
use super::{List, Writer, fact_bytes, limits, owner_waiting_chip_record, task_with_hold_record, w, wire_text};
#[must_use]
pub fn pending_proposal_record() -> w::OwnerProposal {
    let limits = &limits();
    let mut funding = List::with_capacity(2);
    for (source, needed, left, after) in [
        (
            w::Funder::Task(w::FunderTask::new(limits, w::FunderTaskParts { value: 17 }).expect("literal funder")),
            19,
            None,
            Some(23),
        ),
        (
            w::Funder::Period(
                w::FunderPeriod::new(limits, w::FunderPeriodParts { project: 7, period: 29 }).expect("literal funder"),
            ),
            31,
            Some(0),
            None,
        ),
    ] {
        funding
            .push(
                w::OwnerFundingOffer::new(limits, w::OwnerFundingOfferParts { source, needed, left, after })
                    .expect("literal funding"),
            )
            .expect("two offers");
    }
    w::OwnerProposal::new(
        limits,
        w::OwnerProposalParts {
            key: w::ProposalKey::new(
                limits,
                w::ProposalKeyParts {
                    proposer: w::Proposer::Party(
                        w::ProposerParty::new(limits, w::ProposerPartyParts { value: 11 }).expect("literal proposer"),
                    ),
                    number: 13,
                },
            )
            .expect("literal key"),
            project: 7,
            reason: wire_text(b"pending"),
            action: w::OwnerProposalAction::Release(
                w::OwnerProposalActionRelease::new(
                    limits,
                    w::OwnerProposalActionReleaseParts {
                        task: w::OwnerTaskLink::Stored(
                            w::OwnerTaskLinkStored::new(
                                limits,
                                w::OwnerTaskLinkStoredParts { number: 37, status: None },
                            )
                            .expect("literal task link"),
                        ),
                        hold: None,
                    },
                )
                .expect("literal action"),
            ),
            as_holder: true,
            waiting: w::OwnerProposalWaiting::new(
                limits,
                w::OwnerProposalWaitingParts {
                    holder: w::OwnerHistoryHolder::Policy(
                        w::OwnerHistoryHolderPolicy::new(
                            limits,
                            w::OwnerHistoryHolderPolicyParts { project: 7, kind: w::OwnerMessageProposalKind::Amend },
                        )
                        .expect("literal holder"),
                    ),
                    since: 41,
                },
            )
            .expect("literal waiting"),
            funding,
            acts: Box::from([]),
        },
    )
    .expect("literal proposal")
}
#[must_use]
pub fn failed_result_record() -> w::ResultDocument {
    let limits = &limits();
    let root = w::Fact::Ending(
        w::EndingRow::new(
            limits,
            w::EndingRowParts {
                parent: None,
                value: w::EndingFields::Task(
                    w::EndingTaskFields::new(limits, w::EndingTaskFieldsParts { value: 1 })
                        .expect("literal ending pointer"),
                ),
            },
        )
        .expect("literal ending root"),
    );
    let child = w::Fact::OwnerChipEnding(
        w::OwnerChipEndingRow::new(
            limits,
            w::OwnerChipEndingRowParts {
                parent: Some(0),
                value: w::OwnerChipEndingFields::Failed(
                    w::OwnerChipEndingFailedFields::new(
                        limits,
                        w::OwnerChipEndingFailedFieldsParts { value: wire_text(b"failed") },
                    )
                    .expect("literal failed ending"),
                ),
            },
        )
        .expect("literal ending fact"),
    );
    let first = fact_bytes(&root);
    let second = fact_bytes(&child);
    let mut rows = Writer::new(first.len().checked_add(second.len()).expect("literal ending size"));
    rows.put(&first).expect("first fact");
    rows.put(&second).expect("second fact");
    w::ResultDocument::new(
        limits,
        w::ResultDocumentParts {
            task: owner_waiting_chip_record(),
            ending: w::Ending::new(limits, w::EndingParts { rows: rows.finish() }).expect("literal ending"),
            value: None,
            at: Some(47),
            details: Some(
                w::ResultDetails::new(
                    limits,
                    w::ResultDetailsParts {
                        contract: w::TaskContractDocument::Report(
                            w::TaskContractDocumentReport::new(
                                limits,
                                w::TaskContractDocumentReportParts { words: 53 },
                            )
                            .expect("literal contract"),
                        ),
                        requester: w::OwnerRequester::Person(
                            w::OwnerRequesterPerson::new(limits, w::OwnerRequesterPersonParts { value: 59 })
                                .expect("literal requester"),
                        ),
                        executor: w::OwnerExecutor::Agent(
                            w::OwnerExecutorAgent::new(limits, w::OwnerExecutorAgentParts { charter: 7 })
                                .expect("literal executor"),
                        ),
                        chosen_by: Some(61),
                        delegates: Box::from([]),
                        followups: Some(Box::from([])),
                        proposal: None,
                        cut: true,
                    },
                )
                .expect("literal details"),
            ),
            acts: Box::from([]),
        },
    )
    .expect("literal result")
}
#[must_use]
pub fn historical_task_record() -> w::OwnerHistorical {
    let limits = &limits();
    w::OwnerHistorical::new(
        limits,
        w::OwnerHistoricalParts {
            key: w::ObjectKey::Task(
                w::ObjectKeyTask::new(limits, w::ObjectKeyTaskParts { value: 7 }).expect("literal task key"),
            ),
            value: w::OwnerHistoricalValue::Task(
                w::OwnerHistoricalValueTask::new(
                    limits,
                    w::OwnerHistoricalValueTaskParts {
                        document: task_with_hold_record(),
                        result: failed_result_record(),
                    },
                )
                .expect("literal historical task"),
            ),
            at: Some(67),
            by: Some(w::OwnerRequester::Person(
                w::OwnerRequesterPerson::new(limits, w::OwnerRequesterPersonParts { value: 71 })
                    .expect("literal actor"),
            )),
        },
    )
    .expect("literal historical document")
}
#[must_use]
pub fn decided_escalation_record(decision: w::OwnerEscalationDecision) -> w::OwnerHistorical {
    let limits = &limits();
    w::OwnerHistorical::new(
        limits,
        w::OwnerHistoricalParts {
            key: w::ObjectKey::Escalation(
                w::ObjectKeyEscalation::new(limits, w::ObjectKeyEscalationParts { value: 7 })
                    .expect("literal escalation key"),
            ),
            value: w::OwnerHistoricalValue::Escalation(
                w::OwnerHistoricalValueEscalation::new(
                    limits,
                    w::OwnerHistoricalValueEscalationParts {
                        task: task_with_hold_record(),
                        decision: w::OwnerEscalationDecisionRecord::new(
                            limits,
                            w::OwnerEscalationDecisionRecordParts {
                                project: 7,
                                requester: 73,
                                task: 7,
                                revision: 79,
                                by: 83,
                                at: 89,
                                decision,
                            },
                        )
                        .expect("literal decision"),
                        hold: None,
                    },
                )
                .expect("literal historical escalation"),
            ),
            at: Some(97),
            by: Some(w::OwnerRequester::Person(
                w::OwnerRequesterPerson::new(limits, w::OwnerRequesterPersonParts { value: 101 })
                    .expect("literal outer actor"),
            )),
        },
    )
    .expect("literal historical document")
}
#[must_use]
pub fn owner_project_record() -> w::OwnerProject {
    let lim = limits();
    w::OwnerProject::new(
        &lim,
        w::OwnerProjectParts {
            acts: Box::from([]),
            number: 7,
            name: w::DocumentText::new(&lim, w::DocumentTextParts { bytes: Box::from(*b"Tally"), cut: false })
                .expect("project name"),
            resources: Box::from([]),
            policy: w::OwnerProjectPolicy::new(
                &lim,
                w::OwnerProjectPolicyParts {
                    templates: List::with_capacity(0),
                    ceiling: w::OwnerAuthorityDocument::new(
                        &lim,
                        w::OwnerAuthorityDocumentParts {
                            cut: false,
                            authority: w::OwnerAuthority::new(
                                &lim,
                                w::OwnerAuthorityParts {
                                    models: Some(w::ModelSet::All),
                                    tools: 7,
                                    grants: Box::from([]),
                                    delegation: w::OwnerDelegation::new(
                                        &lim,
                                        w::OwnerDelegationParts { kinds: List::with_capacity(0), tasks: 7, depth: 7 },
                                    )
                                    .expect("literal fixture fits"),
                                    spend: 7,
                                    deadline: None,
                                    notes: 7,
                                    note_resources: Box::from([]),
                                },
                            )
                            .expect("literal fixture fits"),
                        },
                    )
                    .expect("literal fixture fits"),
                    period_spend: 7,
                    escalation_role: None,
                    projections: Box::from([]),
                    roles: Box::from([]),
                    requirements: Box::from([]),
                    permissions: None,
                    permissions_cut: false,
                    cut: false,
                },
            )
            .expect("literal fixture fits"),
            members: List::with_capacity(0),
            current_period: None,
            pools: List::with_capacity(0),
            configured_period: 7,
            configured_pools: List::with_capacity(0),
            configured_pools_cut: false,
            spend: Box::from([]),
            recurring: Box::from([]),
            cut: false,
        },
    )
    .expect("literal fixture fits")
}
