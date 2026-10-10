use super::*;
use jig_client::contract::*;
use skein_lib::{List, Wall, bytes::copy_of};
fn doc(bytes: &[u8]) -> DocumentText {
    DocumentText { bytes: Text::new(copy_of(bytes), 8).unwrap(), cut: true }
}
fn authority() -> OwnerAuthorityDocument {
    OwnerAuthorityDocument {
        authority: OwnerAuthority {
            tools: u64::MAX,
            grants: List::with_capacity(3),
            delegation: OwnerDelegation { kinds: List::with_capacity(3), tasks: u32::MAX, depth: u32::MAX },
            spend: u64::MAX,
            deadline: None,
            notes: u8::MAX,
            note_resources: List::with_capacity(3),
        },
        cut: true,
    }
}
fn state() -> OwnerAmendmentState {
    let mut parameters = List::with_capacity(3);
    parameters.push(ParameterDocument::Bytes { name: u32::MAX, value: doc(&[255; 8]) }).unwrap();
    OwnerAmendmentState {
        addressed: Some(PersonAddress::Party(u64::MAX)),
        taken_by: Some(u64::MAX),
        instructions: Some(doc(b"12345678")),
        spec: SpecificationDocument {
            specification: DocumentSpecification {
                words: doc(b"abcdefgh"),
                parameters,
                inputs: List::with_capacity(3),
            },
            cut: true,
        },
        authority: authority(),
        wake: OwnerWakePolicy {
            words: WakeRule::Immediate,
            notices: WakeRule::Never,
            news: WakeRule::Never,
            results: ResultsWake::Each,
            questions: true,
            answers: false,
            timers: true,
        },
        dependencies: List::with_capacity(3),
        cut: true,
    }
}
fn part() -> OwnerPart {
    let mut segments = List::with_capacity(3);
    segments.push(copy_of(&[255, 0, 254])).unwrap();
    OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Outbox, key: OwnerPartKey::Path(segments) }
}
fn hold() -> OwnerChipHold {
    OwnerChipHold {
        source: Some(part()),
        reason: HoldCategory::StoppedBy { party: u64::MAX },
        revision: u64::MAX,
        before: OwnerPriorPhase::Closing(ClosingStage::Settled),
        escalation: Some(OwnerHoldEscalation::Rejected { by: u64::MAX, reason: doc(b"12345678") }),
        tries: OwnerFailureTries { transient: 1, permanent: 2, run: 3, agent: 4, lost: 5, invalid: 6 },
        hold_wait_since: Some(Wall::from_nanos(u64::MAX)),
    }
}
fn proposal(action: OwnerProposalAction) -> OwnerProposal {
    let mut acts = List::with_capacity(3);
    acts.push(Offered { act: ActKind::Accept, admission: Admission::Allowed }).unwrap();
    OwnerProposal {
        key: ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX },
        project: u32::MAX,
        reason: doc(&[255; 8]),
        action,
        as_holder: true,
        waiting: OwnerProposalWaiting {
            holder: OwnerHistoryHolder::Policy { project: u32::MAX, kind: OwnerMessageProposalKind::Release },
            since: Wall::from_nanos(u64::MAX),
        },
        funding: List::with_capacity(3),
        acts,
    }
}
fn base_bytes() -> u64 {
    u64::try_from(size_of::<OwnerProposal>()).unwrap()
        + 8
        + List::<OwnerFundingOffer>::worst_case(3).unwrap()
        + List::<Offered>::worst_case(3).unwrap()
}
#[test]
fn owner_proposal_exact_whole_action_allocations_include_reserved_lists_boxes_and_opaque_bytes() {
    let bounds = Bounds { rows: 3, text: 8, document_bytes: u64::MAX, ..Bounds::default() };
    let auth = crate::owner_authority::worst_case(&bounds).unwrap();
    assert!(auth > 0);
    let authority_heap = List::<OwnerGrant>::worst_case(3).unwrap()
        + List::<OwnerAuthorityExecutor>::worst_case(3).unwrap()
        + List::<OwnerResourceScope>::worst_case(3).unwrap();
    let before = state();
    let after = state();
    let states = crate::owner_amendment::bytes(&before, &bounds).unwrap()
        + crate::owner_amendment::bytes(&after, &bounds).unwrap();
    let part_heap = List::<Box<[u8]>>::worst_case(3).unwrap() + 3;
    let hold_heap = u64::try_from(size_of::<OwnerChipHold>()).unwrap() + part_heap + 8;
    let cases = [
        (OwnerProposalAction::Batch(List::with_capacity(3)), List::<ProposedTask>::worst_case(3).unwrap()),
        (
            OwnerProposalAction::Effect {
                part: part(),
                authority: authority(),
                attempt: u64::MAX,
                completion: u32::MAX,
                position: u32::MAX,
                purpose: u64::MAX,
            },
            part_heap + authority_heap,
        ),
        (
            OwnerProposalAction::Amend {
                task: OwnerTaskLink::Stored { number: u64::MAX, status: None },
                addressed: Some(AddressChange {
                    revision: Revision { value: u64::MAX },
                    address: PersonAddress::Role(u32::MAX),
                }),
                before: Box::new(before),
                after: Box::new(after),
                reason: doc(&[0; 8]),
                release: Some(0),
            },
            states + 8,
        ),
        (
            OwnerProposalAction::Widen {
                task: OwnerTaskLink::Stored { number: u64::MAX, status: Some(OwnerTaskStatus::Cancelled) },
                before: authority(),
                after: authority(),
            },
            authority_heap * 2,
        ),
        (
            OwnerProposalAction::Release {
                task: OwnerTaskLink::Stored { number: u64::MAX, status: None },
                hold: Some(Box::new(hold())),
            },
            hold_heap,
        ),
        (
            OwnerProposalAction::Release { task: OwnerTaskLink::Stored { number: u64::MAX, status: None }, hold: None },
            0,
        ),
    ];
    for (action, allocated) in cases {
        let supplied = proposal(action);
        let original = supplied.clone();
        let expected = base_bytes() + allocated;
        assert_eq!(bytes(&supplied, &bounds), Some(expected));
        assert!(fits(&supplied, &Bounds { document_bytes: expected, ..bounds }));
        assert!(!fits(&supplied, &Bounds { document_bytes: expected - 1, ..bounds }));
        assert_eq!(supplied, original);
        let boxed = ReadValue::OwnerProposal(Box::new(supplied));
        let whole = u64::try_from(size_of::<ReadValue>()).unwrap() + expected;
        assert!(crate::payload::fits(&boxed, &Bounds { document_bytes: whole, ..bounds }));
        assert!(!crate::payload::fits(&boxed, &Bounds { document_bytes: whole - 1, ..bounds }));
    }
}
#[test]
fn owner_proposal_reserved_empty_and_whole_nested_refusals_never_shorten_input_or_fill_missing_funding() {
    let bounds = Bounds { rows: 3, text: 8, document_bytes: u64::MAX, ..Bounds::default() };
    for capacity in [0, 1, 3] {
        let mut supplied = proposal(OwnerProposalAction::Batch(List::with_capacity(capacity)));
        supplied.funding = List::with_capacity(capacity);
        supplied.acts = List::with_capacity(capacity);
        let expected = u64::try_from(size_of::<OwnerProposal>()).unwrap()
            + 8
            + List::<OwnerFundingOffer>::worst_case(capacity).unwrap()
            + List::<Offered>::worst_case(capacity).unwrap()
            + List::<ProposedTask>::worst_case(capacity).unwrap();
        assert_eq!(bytes(&supplied, &bounds), Some(expected));
        assert!(fits(&supplied, &bounds));
        assert_eq!(supplied.funding.len(), 0);
    }
    let supplied = proposal(OwnerProposalAction::Amend {
        task: OwnerTaskLink::Stored { number: 0, status: None },
        addressed: None,
        before: Box::new(state()),
        after: Box::new(state()),
        reason: doc(b"12345678"),
        release: None,
    });
    let original = supplied.clone();
    assert!(fits(&supplied, &bounds));
    assert!(!fits(&supplied, &Bounds { rows: 2, ..bounds }));
    assert!(!fits(&supplied, &Bounds { text: 7, ..bounds }));
    assert_eq!(supplied, original);
    let too_many = proposal(OwnerProposalAction::Batch(List::with_capacity(4)));
    assert!(!fits(&too_many, &bounds));
    assert_eq!(worst_case(&bounds), Some(u64::MAX));
    assert_eq!(worst_case(&Bounds { document_bytes: 0, ..bounds }), None);
}
