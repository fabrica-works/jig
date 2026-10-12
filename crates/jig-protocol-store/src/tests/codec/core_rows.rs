//! Every core row and the transcript frame body (protocol/store.md, 5.1, 8.3).
use super::{call_limits, core_round_trip};
use crate::codec::core as codec;
use crate::tests::fixtures::core_rows as fixtures;
use jig_domain_core as core;
use jig_domain_tasks as tasks;

#[test]
fn the_deployment_header_reads_back_with_its_id() {
    core_round_trip(core::CoreRecord::Deployment(fixtures::deployment()));
}
#[test]
fn a_run_proof_with_its_terminal_reads_back() {
    core_round_trip(core::CoreRecord::RunProof(fixtures::run()));
}
#[test]
fn a_claim_before_its_first_turn_reads_back() {
    let mut run = fixtures::run();
    run.host = jig_domain_fleet::HostKind::Engine;
    run.credentials = Box::new([]);
    run.window = None;
    run.offered = None;
    run.turn = None;
    run.terminal = None;
    core_round_trip(core::CoreRecord::RunProof(run));
}
#[test]
fn a_terminal_archive_reads_back_with_its_end() {
    core_round_trip(core::CoreRecord::Terminal(fixtures::terminal()));
}
#[test]
fn a_goal_projection_reads_back_with_its_phase() {
    core_round_trip(core::CoreRecord::Projection(fixtures::goal()));
}
#[test]
fn a_projection_task_reads_back_with_each_requester() {
    for requester in [tasks::Party::Task(11), tasks::Party::Person(12), tasks::Party::Deployment { project: 2 }] {
        core_round_trip(core::CoreRecord::Projection(fixtures::projection_task(requester)));
    }
}
#[test]
fn a_milestone_reads_back_with_each_identity() {
    for identity in
        [core::MilestoneId::Lifecycle { task: 11, position: 3 }, core::MilestoneId::Revision { task: 11, revision: 4 }]
    {
        core_round_trip(core::CoreRecord::Projection(fixtures::milestone(identity)));
    }
}
#[test]
fn a_milestone_without_carried_values_reads_back() {
    let core::ProjectionRecord::Milestone { goal, mut milestone } =
        fixtures::milestone(core::MilestoneId::Revision { task: 11, revision: 4 })
    else {
        panic!("milestone fixture")
    };
    milestone.phase = None;
    milestone.change = None;
    core_round_trip(core::CoreRecord::Projection(core::ProjectionRecord::Milestone { goal, milestone }));
}
#[test]
fn a_turn_reads_back_with_its_bytes() {
    core_round_trip(core::CoreRecord::Turn(fixtures::turn()));
}
#[test]
fn a_turn_frame_body_reads_back_with_its_version() {
    let limits = call_limits();
    let turn = fixtures::turn();
    let bytes = codec::turn(&turn, &limits).expect("turn within limits");
    assert_eq!(codec::read_turn(&bytes, &limits), Ok(turn));
}
#[test]
fn a_turn_frame_of_another_version_is_refused() {
    let limits = call_limits();
    let mut bytes = codec::turn(&fixtures::turn(), &limits).expect("turn within limits");
    bytes[1] = 2;
    assert_eq!(
        codec::read_turn(&bytes, &limits).expect_err("another core version").reason,
        skein_codec::Reason::Version
    );
}
#[test]
fn both_decision_records_read_back() {
    for row in [
        core::CoreRecord::ProposalDecision(fixtures::proposal()),
        core::CoreRecord::EscalationDecision(fixtures::escalation()),
    ] {
        core_round_trip(row);
    }
}
#[test]
fn a_proposal_without_created_tasks_reads_back() {
    let mut proposal = fixtures::proposal();
    proposal.created = None;
    core_round_trip(core::CoreRecord::ProposalDecision(proposal));
}

#[test]
fn turn_limits_use_the_worker_channels_body_ceiling() {
    assert_eq!(call_limits().core.turn_record_transcript, 8_388_608);
}

#[test]
fn a_turn_exceeding_load_bytes_still_reads_back() {
    let mut domain = super::core_limits();
    domain.load_bytes = 1;
    let limits = codec::limits(&domain).expect("within schema ceilings");
    let turn = fixtures::turn();
    let bytes = codec::turn(&turn, &limits).expect("within the worker channel bound");
    assert_eq!(codec::read_turn(&bytes, &limits), Ok(turn));
}
