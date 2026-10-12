use super::revisions::{Harness, task};
use crate::{
    Escalation, EscalationHolder, MessageKind, Party, Proposal, ProposalAction, ProposalHolder, ProposalState, Word,
};
use alloc::boxed::Box;
use skein_lib::Wall;

fn word(number: u64) -> Word {
    Word {
        number,
        from: Party::Person(9),
        kind: MessageKind::Words,
        words: Box::new([1]),
        at: Wall::EPOCH,
        hits: 1,
        eligible: true,
    }
}

#[test]
fn next_waiting_merges_words_and_decisions_in_number_order() {
    let mut h = Harness::new();
    h.make([task(1, &[]), task(2, &[]), task(3, &[])].into());
    crate::domain::task_mut(&mut h.domain, 1).expect("holder").record.inbox = Box::new([word(9), word(3)]);
    crate::domain::task_mut(&mut h.domain, 2).expect("proposer").record.proposal = Some(Box::new(Proposal {
        number: 5,
        proposer: 2,
        project: 1,
        action: ProposalAction::Release { task: 3 },
        reason: Box::new([]),
        as_holder: false,
        state: ProposalState::Pending { holder: ProposalHolder::Task(1), since: Wall::EPOCH },
    }));
    crate::domain::task_mut(&mut h.domain, 3).expect("held").record.escalation =
        Escalation::Waiting { revision: 1, holder: EscalationHolder::Task(1), entry: 7, since: Wall::EPOCH };
    let mut after = None;
    for expected in [3, 5, 7, 9] {
        let next = h.domain.next_waiting(1, after).expect("waiting message");
        assert_eq!(next.number, expected);
        after = Some(next.number);
    }
    assert_eq!(h.domain.next_waiting(1, after), None);
    assert_eq!(h.domain.next_waiting(99, None), None);
}
