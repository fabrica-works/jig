use super::*;
fn case_0() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Closing { chat: 8, cancelled: 3, kept: 3 }),
        owner(w::OwnerOutcome::Closing(
            w::OwnerOutcomeClosing::new(&wire(), w::OwnerOutcomeClosingParts { chat: 8, cancelled: 3, kept: 3 })
                .expect("Closing"),
        )),
    )
}
fn case_1() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::NoteEdited { name: 8 }),
        owner(w::OwnerOutcome::NoteEdited(
            w::OwnerOutcomeNoteEdited::new(&wire(), w::OwnerOutcomeNoteEditedParts { name: 8 }).expect("NoteEdited"),
        )),
    )
}
fn case_2() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Adopted { project: 3 }),
        owner(w::OwnerOutcome::Adopted(
            w::OwnerOutcomeAdopted::new(&wire(), w::OwnerOutcomeAdoptedParts { project: 3 }).expect("Adopted"),
        )),
    )
}
fn case_3() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::GoalStarted { task: 8 }),
        owner(w::OwnerOutcome::GoalStarted(
            w::OwnerOutcomeGoalStarted::new(&wire(), w::OwnerOutcomeGoalStartedParts { task: 8 }).expect("GoalStarted"),
        )),
    )
}
fn case_5() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Stopped { task: 8 }),
        owner(w::OwnerOutcome::Stopped(
            w::OwnerOutcomeStopped::new(&wire(), w::OwnerOutcomeStoppedParts { task: 8 }).expect("Stopped"),
        )),
    )
}
fn case_6() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Cancelled { task: 8 }),
        owner(w::OwnerOutcome::Cancelled(
            w::OwnerOutcomeCancelled::new(&wire(), w::OwnerOutcomeCancelledParts { task: 8 }).expect("Cancelled"),
        )),
    )
}
fn case_7() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Released { task: 8 }),
        owner(w::OwnerOutcome::Released(
            w::OwnerOutcomeReleased::new(&wire(), w::OwnerOutcomeReleasedParts { task: 8 }).expect("Released"),
        )),
    )
}
fn case_8() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::PersonTaken { task: 8 }),
        owner(w::OwnerOutcome::PersonTaken(
            w::OwnerOutcomePersonTaken::new(&wire(), w::OwnerOutcomePersonTakenParts { task: 8 }).expect("PersonTaken"),
        )),
    )
}
fn case_9() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::PersonHandedBack { task: 8 }),
        owner(w::OwnerOutcome::PersonHandedBack(
            w::OwnerOutcomePersonHandedBack::new(&wire(), w::OwnerOutcomePersonHandedBackParts { task: 8 })
                .expect("PersonHandedBack"),
        )),
    )
}
fn case_10() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::PersonAnswered { task: 8 }),
        owner(w::OwnerOutcome::PersonAnswered(
            w::OwnerOutcomePersonAnswered::new(&wire(), w::OwnerOutcomePersonAnsweredParts { task: 8 })
                .expect("PersonAnswered"),
        )),
    )
}
fn case_11() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Moved { task: 8 }),
        owner(w::OwnerOutcome::Moved(
            w::OwnerOutcomeMoved::new(&wire(), w::OwnerOutcomeMovedParts { task: 8 }).expect("Moved"),
        )),
    )
}
fn case_12() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Said { task: 8, message: 8, released: true }),
        owner(w::OwnerOutcome::Said(
            w::OwnerOutcomeSaid::new(&wire(), w::OwnerOutcomeSaidParts { task: 8, message: 8, released: true })
                .expect("Said"),
        )),
    )
}
fn case_13() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::QuestionAnswered { task: 8, question: 8, message: 8 }),
        owner(w::OwnerOutcome::QuestionAnswered(
            w::OwnerOutcomeQuestionAnswered::new(
                &wire(),
                w::OwnerOutcomeQuestionAnsweredParts { task: 8, question: 8, message: 8 },
            )
            .expect("QuestionAnswered"),
        )),
    )
}
fn case_14() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Prioritised { project: 3 }),
        owner(w::OwnerOutcome::Prioritised(
            w::OwnerOutcomePrioritised::new(&wire(), w::OwnerOutcomePrioritisedParts { project: 3 })
                .expect("Prioritised"),
        )),
    )
}
fn case_15() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Amended { task: 8, released: true }),
        owner(w::OwnerOutcome::Amended(
            w::OwnerOutcomeAmended::new(&wire(), w::OwnerOutcomeAmendedParts { task: 8, released: true })
                .expect("Amended"),
        )),
    )
}
fn case_16() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::RolesSet { project: 3 }),
        owner(w::OwnerOutcome::RolesSet(
            w::OwnerOutcomeRolesSet::new(&wire(), w::OwnerOutcomeRolesSetParts { project: 3 }).expect("RolesSet"),
        )),
    )
}
fn case_18() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::PolicyChanged { project: 3 }),
        owner(w::OwnerOutcome::PolicyChanged(
            w::OwnerOutcomePolicyChanged::new(&wire(), w::OwnerOutcomePolicyChangedParts { project: 3 })
                .expect("PolicyChanged"),
        )),
    )
}
fn case_19() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::PoolSet { project: 3, person: 8 }),
        owner(w::OwnerOutcome::PoolSet(
            w::OwnerOutcomePoolSet::new(&wire(), w::OwnerOutcomePoolSetParts { project: 3, person: 8 })
                .expect("PoolSet"),
        )),
    )
}
fn case_20() -> (p::Reply, w::Final) {
    (
        p::Reply::Outcome(p::Outcome::Started { task: 8 }),
        owner(w::OwnerOutcome::Started(
            w::OwnerOutcomeStarted::new(&wire(), w::OwnerOutcomeStartedParts { task: 8 }).expect("Started"),
        )),
    )
}
pub fn owner_outcomes() -> [(p::Reply, w::Final); 19] {
    [
        case_0(),
        case_1(),
        case_2(),
        case_3(),
        case_5(),
        case_6(),
        case_7(),
        case_8(),
        case_9(),
        case_10(),
        case_11(),
        case_12(),
        case_13(),
        case_14(),
        case_15(),
        case_16(),
        case_18(),
        case_19(),
        case_20(),
    ]
}
pub fn decision_outcomes() -> [(p::Reply, w::Final); 2] {
    let limits = wire();
    let proposal = w::ProposalKey::new(
        &limits,
        w::ProposalKeyParts {
            proposer: w::Proposer::Party(
                w::ProposerParty::new(&limits, w::ProposerPartyParts { value: 7 }).expect("party"),
            ),
            number: 9,
        },
    )
    .expect("proposal");
    [
        (
            p::Reply::Outcome(p::Outcome::ProposalDecided {
                proposer: p::Proposer::Party(7),
                proposal: 9,
                by: 11,
                at: Wall::from_nanos(13),
                choice: p::ProposalChoice::Passed,
            }),
            owner(w::OwnerOutcome::ProposalDecided(
                w::OwnerOutcomeProposalDecided::new(
                    &limits,
                    w::OwnerOutcomeProposalDecidedParts {
                        proposal,
                        by: 11,
                        at: 13,
                        choice: w::OwnerProposalChoice::Passed,
                    },
                )
                .expect("decision"),
            )),
        ),
        (
            p::Reply::Outcome(p::Outcome::EscalationDecided {
                task: 8,
                revision: 6,
                by: 11,
                at: Wall::from_nanos(13),
                choice: p::EscalationChoice::Released,
            }),
            owner(w::OwnerOutcome::EscalationDecided(
                w::OwnerOutcomeEscalationDecided::new(
                    &limits,
                    w::OwnerOutcomeEscalationDecidedParts {
                        task: 8,
                        revision: revision(6),
                        by: 11,
                        at: 13,
                        choice: w::OwnerEscalationChoice::Released,
                    },
                )
                .expect("decision"),
            )),
        ),
    ]
}
pub fn seen_outcomes() -> [(p::Reply, w::Final); 2] {
    let limits = wire();
    [
        (
            p::Reply::Outcome(p::Outcome::MarkedSeen { seen: p::Seen::Results { through: 15 } }),
            owner(w::OwnerOutcome::MarkedSeen(
                w::OwnerOutcomeMarkedSeen::new(
                    &limits,
                    w::OwnerOutcomeMarkedSeenParts {
                        seen: w::Seen::Results(
                            w::SeenResults::new(&limits, w::SeenResultsParts { through: 15 }).expect("results"),
                        ),
                    },
                )
                .expect("seen"),
            )),
        ),
        (
            p::Reply::Outcome(p::Outcome::MarkedSeen { seen: p::Seen::Chat { task: 8, through: 15 } }),
            owner(w::OwnerOutcome::MarkedSeen(
                w::OwnerOutcomeMarkedSeen::new(
                    &limits,
                    w::OwnerOutcomeMarkedSeenParts {
                        seen: w::Seen::Replies(
                            w::SeenReplies::new(&limits, w::SeenRepliesParts { chat: 8, through: 15 })
                                .expect("replies"),
                        ),
                    },
                )
                .expect("seen"),
            )),
        ),
    ]
}
