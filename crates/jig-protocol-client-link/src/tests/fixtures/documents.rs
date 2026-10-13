use super::*;
use skein_lib::{List, Wall};
pub fn text(value: &[u8]) -> c::Text {
    c::Text::new(Box::from(value), 32).unwrap()
}
pub fn document_text(value: &[u8]) -> c::DocumentText {
    c::DocumentText { bytes: text(value), cut: false }
}
pub fn wire_text(value: &[u8]) -> w::DocumentText {
    w::DocumentText::new(&limits().wire, w::DocumentTextParts { bytes: Box::from(value), cut: false }).unwrap()
}
pub fn chip() -> c::Chip {
    c::Chip {
        task: 7,
        title: document_text(b""),
        executor: c::Executor::Agent { charter: 7 },
        phase: c::Phase::Waiting,
        details: Box::new(c::ChipPhase::Starting),
    }
}
pub fn chip_record() -> w::Chip {
    chip_phase_record(w::ChipPhaseFields::Starting)
}
pub fn chip_phase_record(phase: w::ChipPhaseFields) -> w::Chip {
    let lim = limits().wire;

    let root = chip_root_fact();
    let child =
        w::Fact::ChipPhase(w::ChipPhaseRow::new(&lim, w::ChipPhaseRowParts { parent: Some(0), value: phase }).unwrap());
    let first = fact_bytes(&root);
    let second = fact_bytes(&child);
    let mut rows = Writer::new(first.len().checked_add(second.len()).unwrap());
    rows.put(&first).unwrap();
    rows.put(&second).unwrap();
    w::Chip::new(&lim, w::ChipParts { rows: rows.finish() }).unwrap()
}
pub fn chip_pieces(count: u32) -> Box<[u8]> {
    let row = chip_record();
    let mut encoded = Writer::new(usize::try_from(row.measure()).unwrap());
    row.encode(&mut encoded).unwrap();
    let encoded = encoded.finish();
    let mut pieces =
        Writer::new(encoded.len().checked_add(4).unwrap().checked_mul(usize::try_from(count).unwrap()).unwrap());
    for _ in 0..count {
        skein_codec::piece::write(&mut pieces, &encoded).unwrap();
    }
    pieces.finish()
}
pub fn frame_record() -> w::Frame {
    let lim = limits().wire;
    w::Frame::new(
        &lim,
        w::FrameParts {
            party: w::FrameParty::new(
                &lim,
                w::FramePartyParts {
                    number: 7,
                    kind: w::PartyKind::Person,
                    login: w::DocumentText::new(&lim, w::DocumentTextParts { bytes: Box::from([]), cut: false })
                        .unwrap(),
                    name: w::DocumentText::new(&lim, w::DocumentTextParts { bytes: Box::from([]), cut: false })
                        .unwrap(),
                },
            )
            .unwrap(),
            projects: List::with_capacity(0),
            needs: w::Needs::new(&lim, w::NeedsParts { questions: 7, proposals: 7, escalations: 7, person_tasks: 7 })
                .unwrap(),
            needs_total: 7,
            updates: w::Count::new(&lim, w::CountParts { value: 7, limited: false }).unwrap(),
            results_seen: 7,
            charters: Box::from([]),
            cut: false,
            retention: Duration::from_nanos(19),
            deployment_role: None,
        },
    )
    .unwrap()
}
pub fn frame() -> c::Frame {
    c::Frame {
        party: c::FrameParty {
            number: 7,
            kind: c::PartyKind::Person,
            login: c::DocumentText { bytes: text(b""), cut: false },
            name: c::DocumentText { bytes: text(b""), cut: false },
        },
        projects: List::with_capacity(0),
        needs: c::Needs { questions: 7, proposals: 7, escalations: 7, person_tasks: 7 },
        needs_total: 7,
        updates: c::Count { value: 7, limited: false },
        results_seen: 7,
        charters: List::with_capacity(0),
        cut: false,
        retention: Duration::from_nanos(19),
        deployment_role: None,
    }
}
pub fn task_record() -> w::Task {
    let lim = limits().wire;
    w::Task::new(
        &lim,
        w::TaskParts {
            chip: chip_record(),
            goal: None,
            priority: None,
            lineage_chips: Box::from([]),
            inputs: Box::from([]),
            project: 7,
            requester: w::OwnerRequester::Task(
                w::OwnerRequesterTask::new(&lim, w::OwnerRequesterTaskParts { value: 7 }).unwrap(),
            ),
            lineage: List::with_capacity(0),
            spec: wire_specification_document(),
            spec_revision: w::Revision::new(&lim, w::RevisionParts { value: 7 }).unwrap(),
            person_revision: None,
            taken_by: None,
            instructions: None,
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
            subscriptions: w::Subscriptions::new(&lim, w::SubscriptionsParts { entries: List::with_capacity(0) })
                .unwrap(),
            result_contract: w::TaskContractDocument::Approval(
                w::TaskContractDocumentApproval::new(
                    &lim,
                    w::TaskContractDocumentApprovalParts {
                        state: Box::from([]),
                        part: w::OwnerPart::new(
                            &lim,
                            w::OwnerPartParts {
                                connector: 7,
                                kind: w::OwnerPartKind::Resource,
                                key: w::OwnerPartKey::Path(
                                    w::OwnerPartKeyPath::new(
                                        &lim,
                                        w::OwnerPartKeyPathParts { value: List::with_capacity(0) },
                                    )
                                    .unwrap(),
                                ),
                            },
                        )
                        .unwrap(),
                    },
                )
                .unwrap(),
            ),
            dependencies: Box::from([]),
            references: Box::from([]),
            cut: false,
            resources: Box::from([]),
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
            authority: wire_owner_authority_document(),
            hold: None,
            proposals: List::with_capacity(0),
            activity: 7,
            changed: 7,
            acts: Box::from([]),
            result: None,
            delegates: Box::from([]),
            context: None,
        },
    )
    .unwrap()
}
pub fn task() -> c::Task {
    c::Task {
        chip: chip(),
        goal: None,
        priority: None,
        lineage_chips: List::with_capacity(0),
        inputs: List::with_capacity(0),
        project: 7,
        requester: c::OwnerRequester::Task(7),
        lineage: List::with_capacity(0),
        spec: c::SpecificationDocument {
            specification: c::DocumentSpecification {
                words: c::DocumentText { bytes: text(b""), cut: false },
                parameters: List::with_capacity(0),
                inputs: List::with_capacity(0),
            },
            cut: false,
        },
        spec_revision: c::Revision { value: 7 },
        person_revision: None,
        taken_by: None,
        instructions: None,
        wake: c::OwnerWakePolicy {
            words: c::WakeRule::Never,
            notices: c::WakeRule::Never,
            news: c::WakeRule::Never,
            results: c::ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        },
        subscriptions: c::Subscriptions { entries: List::with_capacity(0) },
        result_contract: c::TaskContractDocument::Approval {
            state: Box::from([]),
            part: c::OwnerPart {
                connector: 7,
                kind: c::OwnerPartKind::Resource,
                key: c::OwnerPartKey::Path(List::with_capacity(0)),
            },
        },
        dependencies: List::with_capacity(0),
        references: List::with_capacity(0),
        cut: false,
        resources: List::with_capacity(0),
        budget: c::OwnerBudget {
            numbers: c::OwnerNumbers { budget: 7, spent: 7, spent_below: 7, reserved: 7 },
            left: 7,
            funder: c::Funder::Task(7),
            deadline: None,
        },
        authority: c::OwnerAuthorityDocument {
            authority: c::OwnerAuthority {
                tools: 7,
                grants: List::with_capacity(0),
                delegation: c::OwnerDelegation { kinds: List::with_capacity(0), tasks: 7, depth: 7 },
                spend: 7,
                deadline: None,
                notes: 7,
                note_resources: List::with_capacity(0),
            },
            cut: false,
        },
        hold: None,
        proposals: List::with_capacity(0),
        activity: 7,
        changed: Wall::from_nanos(7),
        acts: List::with_capacity(0),
        result: None,
        delegates: List::with_capacity(0),
        context: None,
    }
}
pub fn run_state_record() -> w::RunState {
    let lim = limits().wire;
    w::RunState::new(&lim, w::RunStateParts { task: 7, attempt: None, committed_turn: 7, writing: None, call: None })
        .unwrap()
}
pub fn run_state() -> c::RunState {
    c::RunState { task: 7, attempt: None, committed_turn: 7, writing: None, call: None }
}
pub fn run_text_record() -> w::RunText {
    let lim = limits().wire;
    w::RunText::new(
        &lim,
        w::RunTextParts {
            attempt: 7,
            began: None,
            turn: w::ProvisionalTurnName::new(&lim, w::ProvisionalTurnNameParts { attempt: 7, number: 7, retry: 7 })
                .unwrap(),
            from: 7,
            through: 7,
            text: Box::from([]),
            cut: false,
            call: None,
        },
    )
    .unwrap()
}
pub fn run_text() -> c::RunText {
    c::RunText {
        attempt: 7,
        began: None,
        turn: c::ProvisionalTurnName { attempt: 7, number: 7, retry: 7 },
        from: 7,
        through: 7,
        text: text(b""),
        cut: false,
        call: None,
    }
}
pub fn current_call_record() -> w::CurrentCall {
    let lim = limits().wire;
    w::CurrentCall::new(
        &lim,
        w::CurrentCallParts {
            identity: w::CallIdentity::new(
                &lim,
                w::CallIdentityParts { owner: 7, sequence: 7, retry: 7, id: Box::from([]) },
            )
            .unwrap(),
            tool: Box::from([]),
            input: Box::from([]),
            cut: false,
            began: None,
        },
    )
    .unwrap()
}
pub fn current_call() -> c::CurrentCall {
    c::CurrentCall {
        identity: c::CallIdentity { owner: Token::new(7), sequence: 7, retry: 7, id: text(b"") },
        tool: text(b""),
        input: text(b""),
        cut: false,
        began: None,
    }
}
pub fn deployment_record() -> w::Deployment {
    let lim = limits().wire;
    w::Deployment::new(
        &lim,
        w::DeploymentParts {
            hosts: Box::from([]),
            accounts: Box::from([]),
            capacity: List::with_capacity(0),
            proposals: 7,
            refusals: Box::from([]),
            parts: Box::from([]),
        },
    )
    .unwrap()
}
pub fn deployment() -> c::Deployment {
    c::Deployment {
        hosts: List::with_capacity(0),
        accounts: List::with_capacity(0),
        capacity: List::with_capacity(0),
        proposals: 7,
        refusals: List::with_capacity(0),
        parts: List::with_capacity(0),
    }
}
pub fn conversations_record() -> w::Conversations {
    let lim = limits().wire;
    w::Conversations::new(
        &lim,
        w::ConversationsParts {
            rows: Box::from([]),
            more: false,
            after: w::Cursor::new(&lim, w::CursorParts { value: Box::from([]) }).unwrap(),
        },
    )
    .unwrap()
}
pub fn conversations() -> c::Conversations {
    c::Conversations { rows: List::with_capacity(0), more: false, after: c::Cursor { value: text(b"") } }
}
pub fn inbox_record() -> w::Inbox {
    let lim = limits().wire;
    w::Inbox::new(
        &lim,
        w::InboxParts {
            entries: Box::from([]),
            needs: w::Needs::new(&lim, w::NeedsParts { questions: 7, proposals: 7, escalations: 7, person_tasks: 7 })
                .unwrap(),
            updates: w::Count::new(&lim, w::CountParts { value: 7, limited: false }).unwrap(),
            results_seen: 7,
            replies_seen: List::with_capacity(0),
            more: false,
        },
    )
    .unwrap()
}
pub fn inbox() -> c::Inbox {
    c::Inbox {
        entries: List::with_capacity(0),
        needs: c::Needs { questions: 7, proposals: 7, escalations: 7, person_tasks: 7 },
        updates: c::Count { value: 7, limited: false },
        results_seen: 7,
        replies_seen: List::with_capacity(0),
        more: false,
    }
}
pub fn goals_record() -> w::Goals {
    let lim = limits().wire;
    w::Goals::new(&lim, w::GoalsParts { project: 7, rows: Box::from([]), more: false }).unwrap()
}
pub fn goals() -> c::Goals {
    c::Goals { project: 7, rows: List::with_capacity(0), more: false }
}
pub fn tasks_record() -> w::Tasks {
    let lim = limits().wire;
    w::Tasks::new(
        &lim,
        w::TasksParts {
            project: 7,
            filter: w::TaskFilter::new(
                &lim,
                w::TaskFilterParts { phase: None, executor: None, goal: None, asked_by_me: false },
            )
            .unwrap(),
            rows: Box::from([]),
            more: false,
        },
    )
    .unwrap()
}
pub fn tasks() -> c::Tasks {
    c::Tasks {
        project: 7,
        filter: c::TaskFilter { phase: None, executor: None, goal: None, asked_by_me: false },
        rows: List::with_capacity(0),
        more: false,
    }
}
pub fn tree_record() -> w::Tree {
    let lim = limits().wire;
    w::Tree::new(
        &lim,
        w::TreeParts {
            task: 7,
            nodes: Box::from([]),
            folded: w::PhaseCounts::new(
                &lim,
                w::PhaseCountsParts {
                    ended: 7,
                    waiting: 7,
                    running: 7,
                    held: 7,
                    closing: 7,
                    done: 7,
                    failed: 7,
                    cancelled: 7,
                },
            )
            .unwrap(),
            held_below: 7,
        },
    )
    .unwrap()
}
pub fn tree() -> c::Tree {
    c::Tree {
        task: 7,
        nodes: List::with_capacity(0),
        folded: c::PhaseCounts {
            ended: 7,
            waiting: 7,
            running: 7,
            held: 7,
            closing: 7,
            done: 7,
            failed: 7,
            cancelled: 7,
        },
        held_below: 7,
    }
}
pub fn task_with_lineage(count: u32) -> w::Task {
    let lim = limits().wire;
    let mut parts = task_record().into_parts();
    parts.lineage_chips = chip_pieces(count);
    w::Task::new(&lim, parts).unwrap()
}
pub fn text_record() -> w::RunText {
    let lim = limits().wire;
    w::RunText::new(
        &lim,
        w::RunTextParts {
            attempt: 41,
            began: Some(u64::MAX),
            turn: w::ProvisionalTurnName::new(&lim, w::ProvisionalTurnNameParts { attempt: 41, number: 7, retry: 3 })
                .unwrap(),
            from: 5,
            through: 11,
            text: Box::from(b"offset".as_slice()),
            cut: true,
            call: None,
        },
    )
    .unwrap()
}
pub fn run_text_expected() -> c::RunText {
    c::RunText {
        attempt: 41,
        began: Some(Wall::from_nanos(u64::MAX)),
        turn: c::ProvisionalTurnName { attempt: 41, number: 7, retry: 3 },
        from: 5,
        through: 11,
        text: text(b"offset"),
        cut: true,
        call: None,
    }
}
pub fn task_with_piece(bytes: &[u8]) -> w::Task {
    let lim = limits().wire;
    let mut parts = task_record().into_parts();
    parts.lineage_chips = Box::from(bytes);
    w::Task::new(&lim, parts).unwrap()
}
pub fn limits_for_document(bytes: u64) -> Limits {
    let mut client = limits().client;
    client.document_bytes = bytes;
    Limits::new(client, 4, Duration::from_secs(10)).unwrap()
}

pub fn wire_specification_document() -> w::SpecificationDocument {
    let lim = limits().wire;
    w::SpecificationDocument::new(
        &lim,
        w::SpecificationDocumentParts {
            specification: w::DocumentSpecification::new(
                &lim,
                w::DocumentSpecificationParts {
                    words: w::DocumentText::new(&lim, w::DocumentTextParts { bytes: Box::from([]), cut: false })
                        .unwrap(),
                    parameters: List::with_capacity(0),
                    inputs: List::with_capacity(0),
                },
            )
            .unwrap(),
            cut: false,
        },
    )
    .unwrap()
}
pub fn wire_owner_authority_document() -> w::OwnerAuthorityDocument {
    let lim = limits().wire;
    w::OwnerAuthorityDocument::new(
        &lim,
        w::OwnerAuthorityDocumentParts {
            authority: w::OwnerAuthority::new(
                &lim,
                w::OwnerAuthorityParts {
                    tools: 7,
                    grants: Box::from([]),
                    delegation: w::OwnerDelegation::new(
                        &lim,
                        w::OwnerDelegationParts { kinds: List::with_capacity(0), tasks: 7, depth: 7 },
                    )
                    .unwrap(),
                    spend: 7,
                    deadline: None,
                    notes: 7,
                    note_resources: Box::from([]),
                },
            )
            .unwrap(),
            cut: false,
        },
    )
    .unwrap()
}
pub fn task_at_row_bound() -> w::Task {
    task_with_lineage(2)
}
pub fn task_with_two_chips() -> c::Task {
    let mut value = task();
    value.lineage_chips = List::with_capacity(2);
    value.lineage_chips.push(chip()).unwrap();
    value.lineage_chips.push(chip()).unwrap();
    value
}
pub fn run_text_one_past_text_limit() -> w::RunText {
    let mut parts = text_record().into_parts();
    parts.text = Box::from([b'x'; 33]);
    w::RunText::new(&w::CEILINGS, parts).unwrap()
}

pub fn conversations_with_cursor() -> w::Conversations {
    let lim = limits().wire;
    let mut parts = conversations_record().into_parts();
    parts.after = w::Cursor::new(&lim, w::CursorParts { value: Box::from([255; 40]) }).unwrap();
    w::Conversations::new(&lim, parts).unwrap()
}
pub fn conversations_cursor_expected() -> c::Conversations {
    let mut value = conversations();
    value.after = c::Cursor { value: c::Text::new(Box::from([255; 40]), 1024).unwrap() };
    value
}
pub fn raw_live_call() -> w::LiveCall {
    let lim = limits().wire;
    let identity = w::CallIdentity::new(
        &lim,
        w::CallIdentityParts { owner: 17, sequence: 2, retry: 3, id: Box::from(b"provider".as_slice()) },
    )
    .unwrap();
    let value = w::CurrentCall::new(
        &lim,
        w::CurrentCallParts {
            identity,
            tool: Box::from(b"read".as_slice()),
            input: Box::from(b"x".as_slice()),
            cut: true,
            began: Some(0),
        },
    )
    .unwrap();
    w::LiveCall::Raw(w::LiveCallRaw::new(&lim, w::LiveCallRawParts { value }).unwrap())
}
pub fn raw_live_call_expected() -> c::LiveCall {
    c::LiveCall::Raw(c::CurrentCall {
        identity: c::CallIdentity { owner: Token::new(17), sequence: 2, retry: 3, id: text(b"provider") },
        tool: text(b"read"),
        input: text(b"x"),
        cut: true,
        began: Some(Wall::from_nanos(0)),
    })
}
pub fn decoded_live_call() -> w::LiveCall {
    let lim = limits().wire;
    let value = w::Call::new(
        &lim,
        w::CallParts {
            number: 11,
            tool: Box::from(b"read".as_slice()),
            input: Box::from(b"x".as_slice()),
            outcome: Some(Box::from(b"yes".as_slice())),
            failed: false,
            objects: List::with_capacity(0),
            parts: Box::from([]),
        },
    )
    .unwrap();
    w::LiveCall::Decoded(w::LiveCallDecoded::new(&lim, w::LiveCallDecodedParts { value }).unwrap())
}
pub fn decoded_live_call_expected() -> c::LiveCall {
    c::LiveCall::Decoded(c::Call {
        number: 11,
        tool: text(b"read"),
        input: text(b"x"),
        outcome: Some(text(b"yes")),
        failed: false,
        objects: List::with_capacity(0),
        parts: List::with_capacity(0),
    })
}
