//! A rejected escalation remains visible while it waits for nobody (people.md, 5.1).
use super::*;

#[test]
fn a_rejected_escalation_still_held_offers_not_waiting() {
    let (mut core, limits) = tree_core();
    sign_in(&mut core, &limits);
    let chat = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, None);
    send_task(
        &mut core,
        &limits,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(110)),
            creator: tasks::Party::Person(1),
            batch: Box::new([chat]),
        },
    );
    send_task(&mut core, &limits, tasks::Event::Hold { task: 1, why: tasks::Hold::Uncertain { entry: 1 } });
    let revision = core.tasks.escalation(1).unwrap().escalation.revision();
    send_task(
        &mut core,
        &limits,
        tasks::Event::RoutedEscalation { task: 1, revision, holder: tasks::EscalationHolder::Person(1), entry: 1 },
    );
    let decision = people::Ask::DecideEscalation {
        project: 1,
        task: 1,
        revision,
        decision: people::EscalationDecision::Reject { reason: b"leave held".as_slice().into() },
    };
    let people::Outcome::EscalationDecided { .. } = ask_now(&mut core, &limits, 2, decision) else {
        unreachable!("the setup decides the escalation");
    };
    for act in [Act::Release, Act::LeaveHeld, Act::PassUp] {
        assert_eq!(
            offers::admit(&core, &limits, 1, act, &Subject::Escalation { task: 1 }),
            Admission::Refused(people::Refusal::NotWaiting)
        );
    }
}
