//! The settings proposal vocabulary meets at wire 19; task E supplies its domain producers.
use jig_client_domain::contract as c;
use jig_client_protocol_link::{Limits, translate};
use jig_client_protocol_wire::v19 as w;
use jig_client_test_records::settings_proposals as values;
use skein_lib::{Reader, Writer};
macro_rules! encoded {
    ($value:expr) => {{
        let value = $value;
        let mut out = Writer::new(usize::try_from(value.measure()).expect("wire size"));
        value.encode(&mut out).expect("measured");
        out.finish()
    }};
}
fn limits() -> Limits {
    let mut client = jig_client_world::test::limits().client.domain;
    client.rows = 8;
    Limits::new(client, 4).expect("client limits")
}
#[test]
fn settings_proposals_and_running_decisions_keep_the_client_contract() {
    let limits = limits();
    let proposal = values::proposal();
    let ask = c::Ask::Propose {
        project: 0,
        action: c::ProposeAction::Settings(proposal.clone()),
        reason: c::Text::new(Box::from(b"review".as_slice()), limits.client.text).expect("reason"),
    };
    let w::Ask::Propose(wire) = translate::asks::ask(&ask, &limits.wire).expect("ask") else { panic!("proposal") };
    assert_eq!(wire.project(), 0);
    let w::ProposeAction::Settings(wire) = wire.action() else { panic!("settings") };
    assert_eq!(translate::settings_proposals::restore_propose_settings(wire, &limits), Ok(proposal));
    for running in [c::Running::Stays, c::Running::Moves] {
        let ask = c::Ask::Decide {
            project: 0,
            on: c::DecideOn::Proposal {
                proposal: c::ProposalKey { proposer: c::Proposer::Party(3), number: 7 },
                decision: c::Decision::Accepted,
                running: Some(running.clone()),
            },
        };
        let w::Ask::Decide(wire) = translate::asks::ask(&ask, &limits.wire).expect("decision") else {
            panic!("decide")
        };
        let w::DecideOn::Proposal(wire) = wire.on() else { panic!("proposal decision") };
        assert_eq!(
            translate::settings::restore_running(wire.running().as_ref().expect("running choice"), &limits),
            Ok(running)
        );
    }
}
#[test]
fn pending_reviews_keep_origins_values_sizes_and_the_cut_fact() {
    let limits = limits();
    let pending = values::pending();
    let wire = translate::settings_proposals::wire_owner_proposal_action_settings(&pending, &limits.wire)
        .expect("settings action");
    let mut parts = w::OwnerProposal::decode(
        &limits.wire,
        &mut Reader::new(include_bytes!(
            "../../../crates/jig-client-protocol-wire/golden/record_owner_proposal_smallest.bin"
        )),
    )
    .expect("proposal envelope")
    .into_parts();
    parts.action = w::OwnerProposalAction::Settings(Box::new(wire));
    parts.waiting = w::OwnerProposalWaiting::new(
        &limits.wire,
        w::OwnerProposalWaitingParts {
            holder: w::OwnerHistoryHolder::Policy(
                w::OwnerHistoryHolderPolicy::new(
                    &limits.wire,
                    w::OwnerHistoryHolderPolicyParts { project: 0, kind: w::OwnerMessageProposalKind::Settings },
                )
                .expect("settings holder"),
            ),
            since: 0,
        },
    )
    .expect("waiting");

    let wire = w::ReadValue::OwnerProposal(w::OwnerProposal::new(&limits.wire, parts).expect("proposal"));
    let c::ReadValue::OwnerProposal(client) =
        translate::reads::value(&encoded!(&wire), &limits).expect("client proposal")
    else {
        panic!("owner proposal")
    };
    assert_eq!(client.action, c::OwnerProposalAction::Settings(Box::new(pending)));
    assert_eq!(
        client.waiting.holder,
        c::OwnerHistoryHolder::Policy { project: 0, kind: c::OwnerMessageProposalKind::Settings }
    );
    let review = values::review();
    let c::ReviewValue::TooLarge(size) = &review.items.get(0).expect("first review item").would_be else {
        panic!("oversized")
    };
    assert_eq!(size.bytes, values::OVERSIZED_VALUE);
}
#[test]
fn retained_settings_actions_keep_the_revision_they_made() {
    let limits = limits();
    let retained = values::retained();
    let wire = translate::settings_proposals::wire_owner_retained_action_settings(&retained, &limits.wire)
        .expect("retained settings");
    let mut proposal = w::OwnerProposalHistory::decode(
        &limits.wire,
        &mut Reader::new(include_bytes!(
            "../../../crates/jig-client-protocol-wire/golden/record_owner_proposal_history_smallest.bin"
        )),
    )
    .expect("history envelope")
    .into_parts();
    proposal.action = w::OwnerRetainedAction::Settings(wire);
    let key = w::ProposalKey::new(
        &limits.wire,
        w::ProposalKeyParts {
            proposer: w::Proposer::Task(
                w::ProposerTask::new(&limits.wire, w::ProposerTaskParts { value: proposal.proposer })
                    .expect("task proposer"),
            ),
            number: proposal.number,
        },
    )
    .expect("history key");
    proposal.state = w::OwnerProposalHistoryState::Accepted(
        w::OwnerProposalHistoryStateAccepted::new(
            &limits.wire,
            w::OwnerProposalHistoryStateAcceptedParts {
                by: w::OwnerRequester::Person(
                    w::OwnerRequesterPerson::new(&limits.wire, w::OwnerRequesterPersonParts { value: 3 })
                        .expect("accepting party"),
                ),
            },
        )
        .expect("accepted"),
    );

    let value = w::OwnerHistoricalValue::Proposal(
        w::OwnerHistoricalValueProposal::new(
            &limits.wire,
            w::OwnerHistoricalValueProposalParts {
                value: w::OwnerProposalHistory::new(&limits.wire, proposal).expect("history"),
            },
        )
        .expect("historical proposal"),
    );
    let mut historical = w::OwnerHistorical::decode(
        &limits.wire,
        &mut Reader::new(include_bytes!(
            "../../../crates/jig-client-protocol-wire/golden/record_owner_historical_smallest.bin"
        )),
    )
    .expect("historical envelope")
    .into_parts();
    historical.key = w::ObjectKey::Proposal(
        w::ObjectKeyProposal::new(&limits.wire, w::ObjectKeyProposalParts { value: key }).expect("proposal subject"),
    );
    historical.value = value;
    let wire =
        w::ReadValue::Historical(Box::new(w::OwnerHistorical::new(&limits.wire, historical).expect("historical")));
    let c::ReadValue::Historical(client) = translate::reads::value(&encoded!(&wire), &limits).expect("client history")
    else {
        panic!("historical")
    };
    let c::OwnerHistoricalValue::Proposal(proposal) = client.value else { panic!("retained proposal") };
    assert_eq!(proposal.action, c::OwnerRetainedAction::Settings(retained));
}

#[test]
fn a_running_choice_refusal_keeps_its_reason() {
    let limits = limits();
    let refusal = w::Refusal::Malformed(
        w::RefusalMalformed::new(&limits.wire, w::RefusalMalformedParts { at: 0, why: w::Malformed::Running })
            .expect("malformed running"),
    );
    let answer =
        w::Final::Refused(w::Refused::new(&limits.wire, w::RefusedParts { refusal, at: None }).expect("refused"));
    let owned =
        w::Owned::new(&limits.wire, w::OwnedParts { owner: w::Owner::Jig, version: 19, record: encoded!(&answer) })
            .expect("owned refusal");
    assert_eq!(
        translate::answers::answer(w::KeyedAnswer::FinalAnswer(owned), &limits),
        Ok(c::Answer::Refused { refusal: c::Refusal::Malformed { at: 0, why: c::Malformed::Running }, at: None })
    );
}
