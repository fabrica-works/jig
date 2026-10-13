use super::*;
use skein_lib::{List, Wall};
pub fn chip_from_rows(rows: &[w::Fact]) -> w::Chip {
    let mut length = 0_usize;
    for row in rows {
        length = length.checked_add(usize::try_from(row.measure()).unwrap()).unwrap().checked_add(4).unwrap();
    }
    let mut bytes = Writer::new(length);
    for row in rows {
        bytes.put(&fact_bytes(row)).unwrap();
    }
    w::Chip::new(&limits().wire, w::ChipParts { rows: bytes.finish() }).unwrap()
}
pub fn phases() -> [(w::Chip, c::Chip); 13] {
    [
        phase_owner(),
        phase_active(),
        phase_waiting(),
        phase_idle(),
        phase_starting(),
        phase_running(),
        phase_waiting_for_words(),
        phase_backing_off(),
        phase_procedure(),
        phase_person(),
        phase_closing(),
        phase_held(),
        phase_ended(),
    ]
}
pub fn phase_owner() -> (w::Chip, c::Chip) {
    let lim = limits().wire;
    let rows = [
        chip_root_fact(),
        w::Fact::ChipPhase(
            w::ChipPhaseRow::new(
                &lim,
                w::ChipPhaseRowParts {
                    parent: Some(0),
                    value: w::ChipPhaseFields::Owner(
                        w::ChipPhaseOwnerFields::new(
                            &lim,
                            w::ChipPhaseOwnerFieldsParts {
                                phase: w::OwnerChipPhase::Waiting(
                                    w::OwnerChipPhaseWaiting::new(
                                        &lim,
                                        w::OwnerChipPhaseWaitingParts {
                                            dependencies: List::with_capacity(0),
                                            more: false,
                                            holds: false,
                                        },
                                    )
                                    .unwrap(),
                                ),
                                details: 2,
                            },
                        )
                        .unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
        w::Fact::OwnerChipDetails(Box::new(wire_owner_chip_details_row())),
    ];
    let record = chip_from_rows(&rows);
    let mut expected = chip();
    expected.details = Box::new(c::ChipPhase::Owner {
        phase: c::OwnerChipPhase::Waiting { dependencies: List::with_capacity(0), more: false, holds: false },
        details: Box::new(c::OwnerChipDetails {
            project: 7,
            executor: c::OwnerExecutor::Agent { charter: 7 },
            phase_since: Wall::from_nanos(7),
            run_since: None,
            attempt: None,
            retry: None,
            wake: c::OwnerWakePolicy {
                words: c::WakeRule::Never,
                notices: c::WakeRule::Never,
                news: c::WakeRule::Never,
                results: c::ResultsWake::Never,
                questions: false,
                answers: false,
                timers: false,
            },
            person_wait: None,
            hold: None,
            budget: c::OwnerBudget {
                numbers: c::OwnerNumbers { budget: 7, spent: 7, spent_below: 7, reserved: 7 },
                left: 7,
                funder: c::Funder::Task(7),
                deadline: None,
            },
            activity: 7,
            resources: List::with_capacity(0),
            resources_cut: false,
            resources_observed: false,
            procedure: None,
            live_delegates: 7,
            closing_connectors: None,
            ending: None,
            ended_at: None,
            observation: None,
        }),
    });
    (record, expected)
}
pub fn phase_active() -> (w::Chip, c::Chip) {
    let lim = limits().wire;
    let rows = [
        chip_root_fact(),
        w::Fact::ChipPhase(
            w::ChipPhaseRow::new(
                &lim,
                w::ChipPhaseRowParts {
                    parent: Some(0),
                    value: w::ChipPhaseFields::Active(
                        w::ChipPhaseActiveFields::new(
                            &lim,
                            w::ChipPhaseActiveFieldsParts {
                                value: w::ActiveRun::new(
                                    &lim,
                                    w::ActiveRunParts {
                                        stage: w::RunStage::Idle,
                                        phase_since: 7,
                                        attempt: None,
                                        placement: None,
                                        retry: None,
                                        observation: None,
                                    },
                                )
                                .unwrap(),
                            },
                        )
                        .unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
    ];
    let record = chip_from_rows(&rows);
    let mut expected = chip();
    expected.details = Box::new(c::ChipPhase::Active(c::ActiveRun {
        stage: c::RunStage::Idle,
        phase_since: Wall::from_nanos(7),
        attempt: None,
        placement: None,
        retry: None,
        observation: None,
    }));
    (record, expected)
}
pub fn phase_waiting() -> (w::Chip, c::Chip) {
    let lim = limits().wire;
    let rows = [
        chip_root_fact(),
        w::Fact::ChipPhase(
            w::ChipPhaseRow::new(
                &lim,
                w::ChipPhaseRowParts {
                    parent: Some(0),
                    value: w::ChipPhaseFields::Waiting(
                        w::ChipPhaseWaitingFields::new(
                            &lim,
                            w::ChipPhaseWaitingFieldsParts { dependencies: List::with_capacity(0), hold: None },
                        )
                        .unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
    ];
    let record = chip_from_rows(&rows);
    let mut expected = chip();
    expected.details = Box::new(c::ChipPhase::Waiting { dependencies: List::with_capacity(0), hold: None });
    (record, expected)
}
pub fn phase_idle() -> (w::Chip, c::Chip) {
    let lim = limits().wire;
    let rows = [
        chip_root_fact(),
        w::Fact::ChipPhase(
            w::ChipPhaseRow::new(
                &lim,
                w::ChipPhaseRowParts {
                    parent: Some(0),
                    value: w::ChipPhaseFields::Idle(
                        w::ChipPhaseIdleFields::new(
                            &lim,
                            w::ChipPhaseIdleFieldsParts {
                                wake: w::WakePolicy::new(
                                    &lim,
                                    w::WakePolicyParts {
                                        words: false,
                                        results: false,
                                        topics: List::with_capacity(0),
                                        subscriptions: Box::from([]),
                                    },
                                )
                                .unwrap(),
                            },
                        )
                        .unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
    ];
    let record = chip_from_rows(&rows);
    let mut expected = chip();
    expected.details = Box::new(c::ChipPhase::Idle {
        wake: c::WakePolicy {
            words: false,
            results: false,
            topics: List::with_capacity(0),
            subscriptions: List::with_capacity(0),
        },
    });
    (record, expected)
}
pub fn phase_starting() -> (w::Chip, c::Chip) {
    let lim = limits().wire;
    let rows = [
        chip_root_fact(),
        w::Fact::ChipPhase(
            w::ChipPhaseRow::new(&lim, w::ChipPhaseRowParts { parent: Some(0), value: w::ChipPhaseFields::Starting })
                .unwrap(),
        ),
    ];
    let record = chip_from_rows(&rows);
    let mut expected = chip();
    expected.details = Box::new(c::ChipPhase::Starting);
    (record, expected)
}
pub fn phase_running() -> (w::Chip, c::Chip) {
    let lim = limits().wire;
    let rows = [
        chip_root_fact(),
        w::Fact::ChipPhase(
            w::ChipPhaseRow::new(
                &lim,
                w::ChipPhaseRowParts {
                    parent: Some(0),
                    value: w::ChipPhaseFields::Running(
                        w::ChipPhaseRunningFields::new(
                            &lim,
                            w::ChipPhaseRunningFieldsParts { attempt: 7, since: 7, tool: None },
                        )
                        .unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
    ];
    let record = chip_from_rows(&rows);
    let mut expected = chip();
    expected.details = Box::new(c::ChipPhase::Running { attempt: 7, since: Wall::from_nanos(7), tool: None });
    (record, expected)
}
pub fn phase_waiting_for_words() -> (w::Chip, c::Chip) {
    let lim = limits().wire;
    let rows = [
        chip_root_fact(),
        w::Fact::ChipPhase(
            w::ChipPhaseRow::new(
                &lim,
                w::ChipPhaseRowParts {
                    parent: Some(0),
                    value: w::ChipPhaseFields::WaitingForWords(
                        w::ChipPhaseWaitingForWordsFields::new(
                            &lim,
                            w::ChipPhaseWaitingForWordsFieldsParts { attempt: 7, since: 7 },
                        )
                        .unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
    ];
    let record = chip_from_rows(&rows);
    let mut expected = chip();
    expected.details = Box::new(c::ChipPhase::WaitingForWords { attempt: 7, since: Wall::from_nanos(7) });
    (record, expected)
}
pub fn phase_backing_off() -> (w::Chip, c::Chip) {
    let lim = limits().wire;
    let rows = [
        chip_root_fact(),
        w::Fact::ChipPhase(
            w::ChipPhaseRow::new(
                &lim,
                w::ChipPhaseRowParts {
                    parent: Some(0),
                    value: w::ChipPhaseFields::BackingOff(
                        w::ChipPhaseBackingOffFields::new(
                            &lim,
                            w::ChipPhaseBackingOffFieldsParts {
                                at: 7,
                                attempt: 7,
                                tries: 7,
                                class: w::FailureClass::HostLost,
                                reason: Box::from([]),
                            },
                        )
                        .unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
    ];
    let record = chip_from_rows(&rows);
    let mut expected = chip();
    expected.details = Box::new(c::ChipPhase::BackingOff {
        at: Wall::from_nanos(7),
        attempt: 7,
        tries: 7,
        class: c::FailureClass::HostLost,
        reason: text(b""),
    });
    (record, expected)
}
pub fn phase_procedure() -> (w::Chip, c::Chip) {
    let lim = limits().wire;
    let rows = [
        chip_root_fact(),
        w::Fact::ChipPhase(
            w::ChipPhaseRow::new(
                &lim,
                w::ChipPhaseRowParts {
                    parent: Some(0),
                    value: w::ChipPhaseFields::Procedure(
                        w::ChipPhaseProcedureFields::new(
                            &lim,
                            w::ChipPhaseProcedureFieldsParts {
                                panel: w::Part::Connector(
                                    w::PartConnector::new(&lim, w::PartConnectorParts { connector: 7 }).unwrap(),
                                ),
                            },
                        )
                        .unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
    ];
    let record = chip_from_rows(&rows);
    let mut expected = chip();
    expected.details = Box::new(c::ChipPhase::Procedure { panel: c::Part::Connector { connector: 7 } });
    (record, expected)
}
pub fn phase_person() -> (w::Chip, c::Chip) {
    let lim = limits().wire;
    let rows = [
        chip_root_fact(),
        w::Fact::ChipPhase(
            w::ChipPhaseRow::new(
                &lim,
                w::ChipPhaseRowParts {
                    parent: Some(0),
                    value: w::ChipPhaseFields::Person(
                        w::ChipPhasePersonFields::new(
                            &lim,
                            w::ChipPhasePersonFieldsParts {
                                addressed: w::Recipient::Task(
                                    w::RecipientTask::new(&lim, w::RecipientTaskParts { value: 7 }).unwrap(),
                                ),
                                taken: None,
                                since: None,
                            },
                        )
                        .unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
    ];
    let record = chip_from_rows(&rows);
    let mut expected = chip();
    expected.details = Box::new(c::ChipPhase::Person { addressed: c::Recipient::Task(7), taken: None, since: None });
    (record, expected)
}
pub fn phase_closing() -> (w::Chip, c::Chip) {
    let lim = limits().wire;
    let rows = [
        chip_root_fact(),
        w::Fact::ChipPhase(
            w::ChipPhaseRow::new(
                &lim,
                w::ChipPhaseRowParts {
                    parent: Some(0),
                    value: w::ChipPhaseFields::Closing(
                        w::ChipPhaseClosingFields::new(&lim, w::ChipPhaseClosingFieldsParts { value: 2 }).unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
        w::Fact::Closing(
            w::ClosingRow::new(
                &lim,
                w::ClosingRowParts {
                    parent: Some(1),
                    value: w::ClosingFields::new(
                        &lim,
                        w::ClosingFieldsParts { delegates: 7, connectors: None, stage: None, ending: None },
                    )
                    .unwrap(),
                },
            )
            .unwrap(),
        ),
    ];
    let record = chip_from_rows(&rows);
    let mut expected = chip();
    expected.details =
        Box::new(c::ChipPhase::Closing(c::Closing { delegates: 7, connectors: None, stage: None, ending: None }));
    (record, expected)
}
pub fn phase_held() -> (w::Chip, c::Chip) {
    let lim = limits().wire;
    let rows = [
        chip_root_fact(),
        w::Fact::ChipPhase(
            w::ChipPhaseRow::new(
                &lim,
                w::ChipPhaseRowParts {
                    parent: Some(0),
                    value: w::ChipPhaseFields::Held(
                        w::ChipPhaseHeldFields::new(
                            &lim,
                            w::ChipPhaseHeldFieldsParts {
                                reason: 2,
                                since: 7,
                                revision: w::Revision::new(&lim, w::RevisionParts { value: 7 }).unwrap(),
                                was: None,
                            },
                        )
                        .unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
        w::Fact::HoldReason(
            w::HoldReasonRow::new(
                &lim,
                w::HoldReasonRowParts {
                    parent: Some(1),
                    value: w::HoldReasonFields::Tries(
                        w::HoldReasonTriesFields::new(
                            &lim,
                            w::HoldReasonTriesFieldsParts { failures: List::with_capacity(0) },
                        )
                        .unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
    ];
    let record = chip_from_rows(&rows);
    let mut expected = chip();
    expected.details = Box::new(c::ChipPhase::Held {
        reason: c::HoldReason::Tries { failures: List::with_capacity(0) },
        since: Wall::from_nanos(7),
        revision: c::Revision { value: 7 },
        was: None,
    });
    (record, expected)
}
pub fn phase_ended() -> (w::Chip, c::Chip) {
    let lim = limits().wire;
    let rows = [
        chip_root_fact(),
        w::Fact::ChipPhase(
            w::ChipPhaseRow::new(
                &lim,
                w::ChipPhaseRowParts {
                    parent: Some(0),
                    value: w::ChipPhaseFields::Ended(
                        w::ChipPhaseEndedFields::new(&lim, w::ChipPhaseEndedFieldsParts { how: 2, at: 7 }).unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
        w::Fact::Ending(
            w::EndingRow::new(
                &lim,
                w::EndingRowParts {
                    parent: Some(1),
                    value: w::EndingFields::Task(
                        w::EndingTaskFields::new(&lim, w::EndingTaskFieldsParts { value: 3 }).unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
        w::Fact::OwnerChipEnding(
            w::OwnerChipEndingRow::new(
                &lim,
                w::OwnerChipEndingRowParts {
                    parent: Some(2),
                    value: w::OwnerChipEndingFields::Done(
                        w::OwnerChipEndingDoneFields::new(&lim, w::OwnerChipEndingDoneFieldsParts { value: 4 })
                            .unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
        w::Fact::ResultValue(
            w::ResultValueRow::new(
                &lim,
                w::ResultValueRowParts {
                    parent: Some(3),
                    value: w::ResultValueFields::Report(
                        w::ResultValueReportFields::new(
                            &lim,
                            w::ResultValueReportFieldsParts {
                                value: w::DocumentText::new(
                                    &lim,
                                    w::DocumentTextParts { bytes: Box::from([]), cut: false },
                                )
                                .unwrap(),
                            },
                        )
                        .unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
    ];
    let record = chip_from_rows(&rows);
    let mut expected = chip();
    expected.details = Box::new(c::ChipPhase::Ended {
        how: c::Ending::Task(Box::new(c::OwnerChipEnding::Done(c::ResultValue::Report(c::DocumentText {
            bytes: text(b""),
            cut: false,
        })))),
        at: Wall::from_nanos(7),
    });
    (record, expected)
}
pub fn chip_root_fact() -> w::Fact {
    let lim = limits().wire;
    w::Fact::Chip(
        w::ChipRow::new(
            &lim,
            w::ChipRowParts {
                parent: None,
                value: w::ChipFields::new(
                    &lim,
                    w::ChipFieldsParts {
                        task: 7,
                        title: wire_text(b""),
                        executor: w::Executor::Agent(
                            w::ExecutorAgent::new(&lim, w::ExecutorAgentParts { charter: 7 }).unwrap(),
                        ),
                        phase: w::Phase::Waiting,
                        details: 1,
                    },
                )
                .unwrap(),
            },
        )
        .unwrap(),
    )
}

pub fn wire_owner_chip_details_row() -> w::OwnerChipDetailsRow {
    let lim = limits().wire;
    w::OwnerChipDetailsRow::new(
        &lim,
        w::OwnerChipDetailsRowParts {
            parent: Some(1),
            value: w::OwnerChipDetailsFields::new(
                &lim,
                w::OwnerChipDetailsFieldsParts {
                    project: 7,
                    executor: w::OwnerExecutor::Agent(
                        w::OwnerExecutorAgent::new(&lim, w::OwnerExecutorAgentParts { charter: 7 }).unwrap(),
                    ),
                    phase_since: 7,
                    run_since: None,
                    attempt: None,
                    retry: None,
                    wake: w::OwnerWakePolicy::new(
                        &lim,
                        w::OwnerWakePolicyParts {
                            words: w::WakeRule::Never,
                            notices: w::WakeRule::Never,
                            news: w::WakeRule::Never,
                            results: w::ResultsWake::Never,
                            questions: false,
                            answers: false,
                            timers: false,
                        },
                    )
                    .unwrap(),
                    person_wait: None,
                    hold: None,
                    budget: w::OwnerBudget::new(
                        &lim,
                        w::OwnerBudgetParts {
                            numbers: w::OwnerNumbers::new(
                                &lim,
                                w::OwnerNumbersParts { budget: 7, spent: 7, spent_below: 7, reserved: 7 },
                            )
                            .unwrap(),
                            left: 7,
                            funder: w::Funder::Task(w::FunderTask::new(&lim, w::FunderTaskParts { value: 7 }).unwrap()),
                            deadline: None,
                        },
                    )
                    .unwrap(),
                    activity: 7,
                    resources: Box::from([]),
                    resources_cut: false,
                    resources_observed: false,
                    procedure: None,
                    live_delegates: 7,
                    closing_connectors: None,
                    ending: None,
                    ended_at: None,
                    observation: None,
                },
            )
            .unwrap(),
        },
    )
    .unwrap()
}
