//! Core durable rows translated exhaustively (protocol/store.md, 5.1).
use super::{Limits, domain, task_problem, too_large, wire};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
pub(super) fn encode_deployment(
    source: &domain::Deployment,
    limits: &Limits,
) -> Result<wire::Deployment, wire::Problem> {
    let domain::Deployment {
        id,
        tasks,
        people,
        sign_ins,
        messages,
        runs,
        calls,
        connector_rows,
        applications,
        commits,
        projects,
    } = source;
    wire::Deployment::new(
        &limits.core,
        wire::DeploymentParts {
            id: *id,
            tasks: *tasks,
            people: *people,
            sign_ins: *sign_ins,
            messages: *messages,
            runs: *runs,
            calls: *calls,
            connector_rows: *connector_rows,
            applications: *applications,
            commits: *commits,
            projects: *projects,
        },
    )
}
pub(super) fn decode_deployment(source: wire::Deployment, _limits: &Limits) -> domain::Deployment {
    let wire::DeploymentParts {
        id,
        tasks,
        people,
        sign_ins,
        messages,
        runs,
        calls,
        connector_rows,
        applications,
        commits,
        projects,
    } = source.into_parts();
    domain::Deployment {
        id,
        tasks,
        people,
        sign_ins,
        messages,
        runs,
        calls,
        connector_rows,
        applications,
        commits,
        projects,
    }
}
pub(super) fn encode_terminal_record(
    source: &domain::TerminalRecord,
    limits: &Limits,
) -> Result<wire::TerminalRecord, wire::Problem> {
    let domain::TerminalRecord { task, attempt, cumulative, end } = source;
    wire::TerminalRecord::new(
        &limits.core,
        wire::TerminalRecordParts {
            task: *task,
            attempt: *attempt,
            cumulative: *cumulative,
            end: wire::EndBytes::new(
                &limits.core,
                wire::EndBytesParts {
                    value: match crate::codec::tasks::carried::end(end, &limits.tasks) {
                        Ok(value) => value,
                        Err(error) => return Err(task_problem(error)),
                    },
                },
            )?,
        },
    )
}
pub(super) fn decode_terminal_record(
    source: wire::TerminalRecord,
    limits: &Limits,
) -> Result<domain::TerminalRecord, wire::Problem> {
    let wire::TerminalRecordParts { task, attempt, cumulative, end } = source.into_parts();
    Ok(domain::TerminalRecord {
        task,
        attempt,
        cumulative,
        end: match crate::codec::tasks::carried::read_end(&end.into_parts().value, &limits.tasks) {
            Ok(value) => value,
            Err(error) => return Err(task_problem(error)),
        },
    })
}
pub(super) fn encode_turn_proof(source: &domain::TurnProof, limits: &Limits) -> Result<wire::TurnProof, wire::Problem> {
    let domain::TurnProof { turn, cumulative, read } = source;
    wire::TurnProof::new(&limits.core, wire::TurnProofParts { turn: *turn, cumulative: *cumulative, read: *read })
}
pub(super) fn decode_turn_proof(source: wire::TurnProof, _limits: &Limits) -> domain::TurnProof {
    let wire::TurnProofParts { turn, cumulative, read } = source.into_parts();
    domain::TurnProof { turn, cumulative, read }
}
#[expect(clippy::manual_map, reason = "the strict subset uses explicit matches without closures")]
pub(super) fn encode_run_proof(source: &domain::RunProof, limits: &Limits) -> Result<wire::RunProof, wire::Problem> {
    let domain::RunProof {
        task,
        attempt,
        conversation,
        accounts,
        placement,
        account_failure,
        account_terminal,
        credentials,
        window,
        host,
        offered,
        outstanding,
        turn,
        terminal,
    } = source;
    wire::RunProof::new(
        &limits.core,
        wire::RunProofParts {
            task: *task,
            attempt: *attempt,
            conversation: *conversation,
            accounts: encode_accounts(accounts, limits)?,
            placement: match placement {
                Some(channel) => Some(channel.raw()),
                None => None,
            },
            account_failure: *account_failure,
            account_terminal: *account_terminal,
            credentials: encode_credentials(credentials, limits)?,
            window: encode_window(*window, limits)?,
            host: super::encode_host_kind(host, limits),
            offered: *offered,
            outstanding: encode_outstanding(outstanding, limits)?,
            turn: match turn {
                Some(value) => Some(encode_turn_proof(value, limits)?),
                None => None,
            },
            terminal: match terminal {
                Some(value) => Some(encode_terminal_record(value, limits)?),
                None => None,
            },
        },
    )
}
#[expect(clippy::manual_map, reason = "the strict subset uses explicit matches without closures")]
pub(super) fn decode_run_proof(source: wire::RunProof, limits: &Limits) -> Result<domain::RunProof, wire::Problem> {
    let wire::RunProofParts {
        task,
        attempt,
        conversation,
        accounts,
        placement,
        account_failure,
        account_terminal,
        credentials,
        window,
        host,
        offered,
        outstanding,
        turn,
        terminal,
    } = source.into_parts();
    Ok(domain::RunProof {
        task,
        attempt,
        conversation,
        accounts: decode_accounts(accounts),
        placement: match placement {
            Some(channel) => Some(skein_lib::Token::new(channel)),
            None => None,
        },
        account_failure,
        account_terminal,
        credentials: decode_credentials(credentials),
        window: match window {
            Some(value) => {
                let result = read_window(value);
                Some(result)
            }
            None => None,
        },
        host: super::decode_host_kind(host, limits),
        offered,
        outstanding: decode_outstanding(outstanding),
        turn: match turn {
            Some(value) => {
                let result = decode_turn_proof(value, limits);
                Some(result)
            }
            None => None,
        },
        terminal: match terminal {
            Some(value) => Some(decode_terminal_record(value, limits)?),
            None => None,
        },
    })
}
pub(super) fn encode_turn_record(
    source: &domain::TurnRecord,
    limits: &Limits,
) -> Result<wire::TurnRecord, wire::Problem> {
    let domain::TurnRecord { opens_window, task, attempt, turn, spent, read, at, transcript } = source;
    wire::TurnRecord::new(
        &limits.core,
        wire::TurnRecordParts {
            opens_window: *opens_window,
            task: *task,
            attempt: *attempt,
            turn: *turn,
            spent: *spent,
            read: *read,
            at: at.as_nanos(),
            transcript: transcript.clone(),
        },
    )
}
pub(super) fn decode_turn_record(source: wire::TurnRecord, _limits: &Limits) -> domain::TurnRecord {
    let wire::TurnRecordParts { opens_window, task, attempt, turn, spent, read, at, transcript } = source.into_parts();
    domain::TurnRecord { opens_window, task, attempt, turn, spent, read, at: Wall::from_nanos(at), transcript }
}
pub(super) fn encode_proposal_decision_record(
    source: &domain::ProposalDecisionRecord,
    limits: &Limits,
) -> Result<wire::ProposalDecisionRecord, wire::Problem> {
    let domain::ProposalDecisionRecord { project, proposer, proposal, kind, by, at, choice, created } = source;
    wire::ProposalDecisionRecord::new(
        &limits.core,
        wire::ProposalDecisionRecordParts {
            project: *project,
            proposer: super::encode_proposer(proposer, limits)?,
            proposal: *proposal,
            kind: super::encode_proposal_kind(kind, limits),
            by: *by,
            at: at.as_nanos(),
            choice: super::encode_proposal_choice(choice, limits),
            created: encode_created(*created, limits)?,
        },
    )
}
pub(super) fn decode_proposal_decision_record(
    source: wire::ProposalDecisionRecord,
    limits: &Limits,
) -> domain::ProposalDecisionRecord {
    let wire::ProposalDecisionRecordParts { project, proposer, proposal, kind, by, at, choice, created } =
        source.into_parts();
    domain::ProposalDecisionRecord {
        project,
        proposer: super::decode_proposer(proposer, limits),
        proposal,
        kind: super::decode_proposal_kind(kind, limits),
        by,
        at: Wall::from_nanos(at),
        choice: super::decode_proposal_choice(choice, limits),
        created: match created {
            Some(value) => {
                let result = read_created(value);
                Some(result)
            }
            None => None,
        },
    }
}
pub(super) fn encode_escalation_decision_record(
    source: &domain::EscalationDecisionRecord,
    limits: &Limits,
) -> Result<wire::EscalationDecisionRecord, wire::Problem> {
    let domain::EscalationDecisionRecord { project, requester, task, revision, by, at, decision } = source;
    wire::EscalationDecisionRecord::new(
        &limits.core,
        wire::EscalationDecisionRecordParts {
            project: *project,
            requester: *requester,
            task: *task,
            revision: *revision,
            by: *by,
            at: at.as_nanos(),
            decision: super::encode_escalation_decision(decision, limits)?,
        },
    )
}
pub(super) fn decode_escalation_decision_record(
    source: wire::EscalationDecisionRecord,
    limits: &Limits,
) -> domain::EscalationDecisionRecord {
    let wire::EscalationDecisionRecordParts { project, requester, task, revision, by, at, decision } =
        source.into_parts();
    domain::EscalationDecisionRecord {
        project,
        requester,
        task,
        revision,
        by,
        at: Wall::from_nanos(at),
        decision: super::decode_escalation_decision(decision, limits),
    }
}
fn encode_window(value: Option<(u64, u32)>, limits: &Limits) -> Result<Option<wire::Window>, wire::Problem> {
    Ok(match value {
        Some((attempt, turn)) => Some(wire::Window::new(&limits.core, wire::WindowParts { attempt, turn })?),
        None => None,
    })
}
fn read_window(value: wire::Window) -> (u64, u32) {
    let wire::WindowParts { attempt, turn } = value.into_parts();
    (attempt, turn)
}
fn encode_created(value: Option<(u64, u64)>, limits: &Limits) -> Result<Option<wire::Created>, wire::Problem> {
    Ok(match value {
        Some((first, last)) => Some(wire::Created::new(&limits.core, wire::CreatedParts { first, last })?),
        None => None,
    })
}
fn read_created(value: wire::Created) -> (u64, u64) {
    let wire::CreatedParts { first, last } = value.into_parts();
    (first, last)
}
fn encode_credentials(values: &[u32], limits: &Limits) -> Result<List<wire::Credential>, wire::Problem> {
    let count = match u32::try_from(values.len()) {
        Ok(value) => value,
        Err(error) => return Err(too_large(error)),
    };
    if count > limits.core.run_proof_credentials {
        return Err(super::problem(skein_codec::Reason::Bound));
    }
    let mut result = List::with_capacity(count);
    for value in values {
        result
            .push(wire::Credential::new(&limits.core, wire::CredentialParts { value: *value })?)
            .expect("same number of credentials");
    }
    Ok(result)
}
fn decode_credentials(values: List<wire::Credential>) -> Box<[u32]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(value.into_parts().value).expect("same number of credentials");
    }
    result.into_boxed()
}

fn encode_accounts(
    values: &[jig_domain_authority::AccountBinding],
    limits: &Limits,
) -> Result<List<wire::AccountBinding>, wire::Problem> {
    let count = match u32::try_from(values.len()) {
        Ok(count) => count,
        Err(error) => return Err(too_large(error)),
    };
    if count > limits.core.run_proof_accounts {
        return Err(super::problem(skein_codec::Reason::Bound));
    }
    let mut result = List::with_capacity(count);
    for value in values {
        result
            .push(wire::AccountBinding::new(
                &limits.core,
                wire::AccountBindingParts { provider: value.provider, account: value.account, source: value.source },
            )?)
            .expect("same binding count");
    }
    Ok(result)
}
fn decode_accounts(values: List<wire::AccountBinding>) -> Box<[jig_domain_authority::AccountBinding]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        let wire::AccountBindingParts { provider, account, source } = value.into_parts();
        result.push(jig_domain_authority::AccountBinding { provider, account, source }).expect("same binding count");
    }
    result.into_boxed()
}

fn encode_outstanding(values: &[u64], limits: &Limits) -> Result<List<wire::Number>, wire::Problem> {
    let count = match u32::try_from(values.len()) {
        Ok(count) => count,
        Err(error) => return Err(too_large(error)),
    };
    if count > limits.core.run_proof_outstanding {
        return Err(super::problem(skein_codec::Reason::Bound));
    }
    let mut result = List::with_capacity(count);
    for &value in values {
        result.push(wire::Number::new(&limits.core, wire::NumberParts { value })?).expect("same offer count");
    }
    Ok(result)
}
fn decode_outstanding(values: List<wire::Number>) -> Box<[u64]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(value.into_parts().value).expect("same offer count");
    }
    result.into_boxed()
}
