use super::*;
pub fn merged_targets() -> [(&'static str, Ask); 4] {
    let limits = limits();
    let proposal = ProposalKey::new(
        &limits,
        ProposalKeyParts {
            proposer: Proposer::Party(
                ProposerParty::new(&limits, ProposerPartyParts { value: u64::MAX }).expect("party"),
            ),
            number: u64::MAX,
        },
    )
    .expect("proposal");
    let decide = [
        (
            "proposal",
            DecideOn::Proposal(
                DecideProposal::new(&limits, DecideProposalParts { proposal, decision: Decision::Accepted })
                    .expect("proposal decision"),
            ),
        ),
        (
            "escalation",
            DecideOn::Escalation(
                DecideEscalation::new(
                    &limits,
                    DecideEscalationParts { task: u64::MAX, revision: revision(&limits), decision: Decision::PassedUp },
                )
                .expect("escalation decision"),
            ),
        ),
    ];
    let answer = [
        (
            "question",
            AnswerOn::Question(
                AnswerQuestion::new(
                    &limits,
                    AnswerQuestionParts { task: u64::MAX, message: u64::MAX, words: b"answer".as_slice().into() },
                )
                .expect("question answer"),
            ),
        ),
        (
            "person",
            AnswerOn::Person(
                AnswerPerson::new(
                    &limits,
                    AnswerPersonParts {
                        task: u64::MAX,
                        revision: revision(&limits),
                        answer: PersonAnswer::Words(
                            PersonAnswerWords::new(
                                &limits,
                                PersonAnswerWordsParts { value: b"answer".as_slice().into() },
                            )
                            .expect("answer words"),
                        ),
                    },
                )
                .expect("person answer"),
            ),
        ),
    ];
    let [(proposal_name, proposal), (escalation_name, escalation)] = decide;
    let [(question_name, question), (person_name, person)] = answer;
    [
        (
            proposal_name,
            Ask::Decide(Decide::new(&limits, DecideParts { project: u32::MAX, on: proposal }).expect("decide")),
        ),
        (
            escalation_name,
            Ask::Decide(Decide::new(&limits, DecideParts { project: u32::MAX, on: escalation }).expect("decide")),
        ),
        (
            question_name,
            Ask::Answer(Answer::new(&limits, AnswerParts { project: u32::MAX, on: question }).expect("answer")),
        ),
        (
            person_name,
            Ask::Answer(Answer::new(&limits, AnswerParts { project: u32::MAX, on: person }).expect("answer")),
        ),
    ]
}
