use super::*;

const PROJECT: u32 = 1;
const PARTY: u64 = 1;
const CONVERSATION: u64 = 1;
const PERIOD: u64 = 1;
const MAKE: u64 = 110;
const MARK: u8 = 1;
const LEAVE_HELD: u8 = 2;
const CANCEL: u8 = 3;
const ENTRY: u64 = 1;
const UNKNOWN_PARTY: u64 = 2;

fn conversation() -> (Core, Limits) {
    let (mut core, limits) = tree_core();
    sign_in(&mut core, &limits);
    let mut row = new_row(&core, CONVERSATION, tasks::Funder::Period { project: PROJECT, period: PERIOD }, None);
    row.flags.conversation = true;
    send_task(
        &mut core,
        &limits,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(MAKE)),
            creator: tasks::Party::Person(PARTY),
            batch: Box::new([row]),
        },
    );
    (core, limits)
}

fn uncertain_conversation() -> (Core, Limits) {
    let (mut core, limits) = conversation();
    send_task(
        &mut core,
        &limits,
        tasks::Event::Hold { task: CONVERSATION, why: tasks::Hold::Uncertain { entry: ENTRY } },
    );
    (core, limits)
}

#[test]
fn marking_a_conversation_seen_is_admitted() {
    let (mut core, limits) = conversation();
    let subject = Subject::Task { task: CONVERSATION };
    let seen = people::Seen::Conversation { task: CONVERSATION, through: 0 };
    assert_eq!(offers::admit(&core, &limits, PARTY, Act::Seen, &subject), Admission::Allowed);
    assert_eq!(ask_now(&mut core, &limits, MARK, people::Ask::MarkSeen { seen }), people::Outcome::MarkedSeen { seen });
}

#[test]
fn an_uncertain_conversation_keeps_its_amendment_offer_bounded() {
    let (core, limits) = uncertain_conversation();
    let subject = Subject::Task { task: CONVERSATION };
    let Admission::Bounded { .. } = offers::admit(&core, &limits, PARTY, Act::Amend, &subject) else {
        panic!("amend and release remains bounded with an explicit uncertainty choice")
    };
}

fn leave_held(core: &mut Core, limits: &Limits) -> people::Outcome {
    let revision = core.tasks.escalation(CONVERSATION).unwrap().escalation.revision();
    ask_now(
        core,
        limits,
        LEAVE_HELD,
        people::Ask::DecideEscalation {
            project: PROJECT,
            task: CONVERSATION,
            revision,
            decision: people::EscalationDecision::Reject { reason: b"leave held".as_slice().into() },
        },
    )
}

fn routed_conversation() -> (Core, Limits) {
    let (mut core, limits) = uncertain_conversation();
    let revision = core.tasks.escalation(CONVERSATION).unwrap().escalation.revision();
    send_task(
        &mut core,
        &limits,
        tasks::Event::RoutedEscalation {
            task: CONVERSATION,
            revision,
            holder: tasks::EscalationHolder::Person(PARTY),
            entry: ENTRY,
        },
    );
    (core, limits)
}

#[test]
fn leaving_a_conversations_escalation_held_is_admitted() {
    let (mut core, limits) = routed_conversation();
    let subject = Subject::Escalation { task: CONVERSATION };
    let revision = core.tasks.escalation(CONVERSATION).unwrap().escalation.revision();
    assert_eq!(offers::admit(&core, &limits, PARTY, Act::LeaveHeld, &subject), Admission::Allowed);
    assert_eq!(
        leave_held(&mut core, &limits),
        people::Outcome::EscalationDecided {
            task: CONVERSATION,
            revision,
            by: PARTY,
            at: Wall::EPOCH,
            choice: people::EscalationChoice::Rejected
        }
    );
}

#[test]
fn an_unknown_party_has_no_leave_held_offer() {
    let (core, limits) = routed_conversation();
    assert_eq!(
        offers::escalation(&core, &limits, UNKNOWN_PARTY, Act::LeaveHeld, CONVERSATION),
        Err(people::Refusal::Unknown)
    );
}

#[test]
fn cancelling_a_live_conversation_is_admitted() {
    let (core, limits) = conversation();
    let subject = Subject::Task { task: CONVERSATION };
    assert_eq!(offers::admit(&core, &limits, PARTY, Act::Cancel, &subject), Admission::Allowed);
}

#[test]
fn cancelling_a_conversation_left_held_is_admitted() {
    let (mut core, limits) = routed_conversation();
    let _outcome = leave_held(&mut core, &limits);
    core.reclaim();
    let subject = Subject::Task { task: CONVERSATION };
    assert_eq!(offers::admit(&core, &limits, PARTY, Act::Cancel, &subject), Admission::Allowed);
    assert_eq!(
        ask_now(
            &mut core,
            &limits,
            CANCEL,
            people::Ask::Cancel {
                project: PROJECT,
                task: CONVERSATION,
                reason: b"Finish this chat.".as_slice().into(),
                grace: None
            }
        ),
        people::Outcome::Cancelled { task: CONVERSATION }
    );
}
