use super::{List, chip_from_rows, chip_root_fact, frame_record, limits, task_record, w, wire_owner_chip_details_row};
use alloc::boxed::Box;
extern crate alloc;

#[must_use]
pub fn frame_two_projects_record() -> w::Frame {
    let lim = limits();
    let mut parts = frame_record().into_parts();
    let mut identity = parts.party.into_parts();
    identity.login = w::DocumentText::new(&lim, w::DocumentTextParts { bytes: Box::from("é".as_bytes()), cut: true })
        .expect("literal UTF-8 login");
    parts.party = w::FrameParty::new(&lim, identity).expect("literal frame party");
    parts.projects = List::with_capacity(2);
    for (project, role, chat_budget) in [(3, w::HeldRole::Owner, 11), (5, w::HeldRole::Observer, 13)] {
        parts
            .projects
            .push(
                w::ProjectRole::new(&lim, w::ProjectRoleParts { project, name: None, role, pool: None, chat_budget })
                    .expect("literal fixture fits"),
            )
            .expect("literal fixture fits");
    }
    let mut chats = List::with_capacity(1);
    chats
        .push(w::Charter::new(&lim, w::CharterParts { number: 17, name: None }).expect("literal chat charter"))
        .expect("one chat");
    let mut goals = List::with_capacity(1);
    goals
        .push(w::Charter::new(&lim, w::CharterParts { number: 19, name: None }).expect("literal goal charter"))
        .expect("one goal");
    let row = w::OfferedCharters::new(&lim, w::OfferedChartersParts { project: 3, chats, goals, cut: true })
        .expect("literal offered charters");
    let mut encoded = skein_lib::Writer::new(usize::try_from(row.measure()).expect("literal size"));
    row.encode(&mut encoded).expect("literal record");
    let mut framed = skein_lib::Writer::new(
        usize::try_from(row.measure()).expect("literal size").checked_add(4).expect("literal piece size"),
    );
    skein_codec::piece::write(&mut framed, &encoded.finish()).expect("literal piece");
    parts.charters = framed.finish();
    w::Frame::new(&lim, parts).expect("literal fixture fits")
}
#[must_use]
pub fn owner_phase_records() -> [w::OwnerChipPhase; 6] {
    let lim = limits();
    [
        w::OwnerChipPhase::Waiting(
            w::OwnerChipPhaseWaiting::new(
                &lim,
                w::OwnerChipPhaseWaitingParts { dependencies: List::with_capacity(0), more: false, holds: false },
            )
            .expect("literal fixture fits"),
        ),
        w::OwnerChipPhase::Active(
            w::OwnerChipPhaseActive::new(&lim, w::OwnerChipPhaseActiveParts { value: w::RunStage::Idle })
                .expect("literal fixture fits"),
        ),
        w::OwnerChipPhase::Person(
            w::OwnerChipPhasePerson::new(
                &lim,
                w::OwnerChipPhasePersonParts {
                    address: w::PersonAddress::Role(
                        w::PersonAddressRole::new(&lim, w::PersonAddressRoleParts { value: 17 })
                            .expect("literal fixture fits"),
                    ),
                    taken_by: Some(19),
                    since: Some(23),
                },
            )
            .expect("literal fixture fits"),
        ),
        w::OwnerChipPhase::Closing(
            w::OwnerChipPhaseClosing::new(&lim, w::OwnerChipPhaseClosingParts { value: w::ClosingStage::Effects })
                .expect("literal fixture fits"),
        ),
        w::OwnerChipPhase::Held(
            w::OwnerChipPhaseHeld::new(
                &lim,
                w::OwnerChipPhaseHeldParts {
                    reason: w::HoldCategory::Uncertain(
                        w::HoldCategoryUncertain::new(&lim, w::HoldCategoryUncertainParts { entry: 29 })
                            .expect("literal fixture fits"),
                    ),
                    revision: 31,
                },
            )
            .expect("literal fixture fits"),
        ),
        w::OwnerChipPhase::Ended,
    ]
}
#[must_use]
pub fn owner_chip_record(phase: w::OwnerChipPhase, compact: w::Phase) -> w::Chip {
    let lim = limits();
    let mut root = match chip_root_fact() {
        w::Fact::Chip(value) => value.into_parts(),
        w::Fact::ChipPhase(_)
        | w::Fact::Closing(_)
        | w::Fact::Ending(_)
        | w::Fact::HeldRuntime(_)
        | w::Fact::HoldReason(_)
        | w::Fact::OwnerChipDetails(_)
        | w::Fact::OwnerChipEnding(_)
        | w::Fact::ResultValue(_) => unreachable!("literal chip root"),
    };
    let mut fields = root.value.into_parts();
    fields.phase = compact;
    root.value = w::ChipFields::new(&lim, fields).expect("literal fixture fits");
    chip_from_rows(&[
        w::Fact::Chip(w::ChipRow::new(&lim, root).expect("literal fixture fits")),
        w::Fact::ChipPhase(
            w::ChipPhaseRow::new(
                &lim,
                w::ChipPhaseRowParts {
                    parent: Some(0),
                    value: w::ChipPhaseFields::Owner(
                        w::ChipPhaseOwnerFields::new(&lim, w::ChipPhaseOwnerFieldsParts { phase, details: 2 })
                            .expect("literal fixture fits"),
                    ),
                },
            )
            .expect("literal fixture fits"),
        ),
        w::Fact::OwnerChipDetails(Box::new(wire_owner_chip_details_row())),
    ])
}
#[must_use]
pub fn owner_waiting_chip_record() -> w::Chip {
    owner_chip_record(owner_phase_records().into_iter().next().expect("literal fixture fits"), w::Phase::Waiting)
}
#[must_use]
pub fn task_with_hold_record() -> w::Task {
    let lim = limits();
    let mut parts = task_record().into_parts();
    parts.chip = owner_waiting_chip_record();
    parts.hold = Some(w::TaskHold::Owner(
        w::TaskHoldOwner::new(&lim, w::TaskHoldOwnerParts { value: hold_record() }).expect("literal fixture fits"),
    ));
    w::Task::new(&lim, parts).expect("literal fixture fits")
}
#[must_use]
pub fn hold_record() -> w::OwnerChipHold {
    let lim = limits();
    w::OwnerChipHold::new(
        &lim,
        w::OwnerChipHoldParts {
            account: None,
            source: None,
            reason: w::HoldCategory::Uncertain(
                w::HoldCategoryUncertain::new(&lim, w::HoldCategoryUncertainParts { entry: 29 })
                    .expect("literal fixture fits"),
            ),
            revision: 31,
            before: w::OwnerPriorPhase::Waiting,
            escalation: None,
            tries: w::OwnerFailureTries::new(
                &lim,
                w::OwnerFailureTriesParts { transient: 2, permanent: 3, run: 5, agent: 7, lost: 11, invalid: 13 },
            )
            .expect("literal fixture fits"),
            hold_wait_since: Some(17),
        },
    )
    .expect("literal fixture fits")
}

#[must_use]
pub fn owner_chip_records() -> [w::Chip; 6] {
    let mut phases = owner_phase_records().into_iter();
    [w::Phase::Waiting, w::Phase::Idle, w::Phase::Active, w::Phase::Closing, w::Phase::Held, w::Phase::Ended]
        .map(|compact| owner_chip_record(phases.next().expect("six literal phases"), compact))
}
