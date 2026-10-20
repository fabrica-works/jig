use super::{limits, owner_chip_records, w};
use skein_lib::{Duration, List, Writer};
pub(super) fn rows<T>(items: impl IntoIterator<Item = T>) -> List<T> {
    let items: Vec<_> = items.into_iter().collect();
    let mut out = List::with_capacity(u32::try_from(items.len()).expect("fixture size"));
    for item in items {
        out.push(item).ok().expect("fixture rows fit");
    }
    out
}
pub(super) fn encoded(measure: u32, write: impl FnOnce(&mut Writer) -> Result<(), skein_lib::Overflow>) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(measure).expect("fixture size"));
    write(&mut writer).expect("literal record encodes");
    writer.finish()
}
pub(super) fn pieces(values: impl IntoIterator<Item = Box<[u8]>>) -> Box<[u8]> {
    let values: Vec<_> = values.into_iter().collect();
    let size = values.iter().map(|v| v.len().checked_add(4).expect("literal piece size")).sum();
    let mut writer = Writer::new(size);
    for value in values {
        skein_codec::piece::write(&mut writer, &value).expect("sized literal pieces");
    }
    writer.finish()
}
#[must_use]
pub fn live_chip_record() -> w::Chip {
    owner_chip_records().into_iter().next().expect("waiting phase fixture")
}
#[must_use]
pub fn live_budget_record() -> w::OwnerBudget {
    let l = limits();
    w::OwnerBudget::new(
        &l,
        w::OwnerBudgetParts {
            numbers: w::OwnerNumbers::new(
                &l,
                w::OwnerNumbersParts { budget: 7, spent: 7, spent_below: 7, reserved: 7 },
            )
            .expect("literal"),
            left: 7,
            funder: w::Funder::Task(w::FunderTask::new(&l, w::FunderTaskParts { value: 7 }).expect("literal")),
            deadline: None,
        },
    )
    .expect("literal")
}
#[must_use]
pub fn live_call_record() -> w::CurrentCall {
    let l = limits();
    w::CurrentCall::new(
        &l,
        w::CurrentCallParts {
            identity: w::CallIdentity::new(
                &l,
                w::CallIdentityParts { owner: 19, sequence: 23, retry: 3, id: Box::from(b"call".as_slice()) },
            )
            .expect("literal"),
            tool: Box::from(b"tool".as_slice()),
            input: Box::from("é".as_bytes()),
            cut: true,
            began: Some(29),
        },
    )
    .expect("literal")
}
#[must_use]
pub fn writing_record() -> w::RunText {
    let l = limits();
    w::RunText::new(
        &l,
        w::RunTextParts {
            attempt: 31,
            began: None,
            turn: w::ProvisionalTurnName::new(&l, w::ProvisionalTurnNameParts { attempt: 31, number: 37, retry: 5 })
                .expect("literal"),
            from: 41,
            through: 43,
            text: Box::from("é".as_bytes()),
            cut: true,
            call: Some(w::LiveCall::Raw(
                w::LiveCallRaw::new(&l, w::LiveCallRawParts { value: live_call_record() }).expect("literal"),
            )),
        },
    )
    .expect("literal")
}
#[must_use]
pub fn run_writing_record() -> w::RunState {
    let l = limits();
    w::RunState::new(
        &l,
        w::RunStateParts {
            task: 7,
            attempt: Some(31),
            committed_turn: 2,
            writing: Some(writing_record()),
            call: Some(live_call_record()),
        },
    )
    .expect("literal")
}
#[must_use]
pub fn live_cursor_record() -> w::Cursor {
    w::Cursor::new(&limits(), w::CursorParts { value: Box::from([255, 0, 3]) }).expect("opaque literal")
}
#[must_use]
pub fn phases_record() -> w::OwnerPhaseCounts {
    w::OwnerPhaseCounts::new(
        &limits(),
        w::OwnerPhaseCountsParts {
            waiting: 1,
            idle: 2,
            starting: 3,
            running: 4,
            backing_off: 5,
            closing: 6,
            held: 7,
            done: 8,
            failed: 9,
            cancelled: 10,
        },
    )
    .expect("literal")
}
#[must_use]
pub fn task_line_record() -> w::TaskLine {
    let l = limits();
    w::TaskLine::new(
        &l,
        w::TaskLineParts {
            chip: live_chip_record(),
            project: 7,
            tracked: Some(19),
            label: 0,
            flags: w::TaskFlags::new(
                &l,
                w::TaskFlagsParts { conversation: false, listed: true, retention: w::Retention::Kept },
            )
            .expect("fixture flags"),
            last_activity: 29,
            ended_at: None,
            budget: live_budget_record(),
            waits_on: w::WaitsOn::Nothing,
            phases: phases_record(),
            held_below: 0,
            conversation: None,
            requester: w::OwnerRequester::Person(
                w::OwnerRequesterPerson::new(&l, w::OwnerRequesterPersonParts { value: 23 }).expect("literal"),
            ),
            changed: 29,
        },
    )
    .expect("literal")
}
#[must_use]
pub fn tasks_rows_record() -> w::Tasks {
    let l = limits();
    let row = task_line_record();
    w::Tasks::new(&l, w::TasksParts { rows: pieces([encoded(row.measure(), |w| row.encode(w))]), more: true })
        .expect("literal")
}
#[must_use]
pub fn stored_link_record() -> w::OwnerTaskLink {
    let l = limits();
    w::OwnerTaskLink::Stored(
        w::OwnerTaskLinkStored::new(
            &l,
            w::OwnerTaskLinkStoredParts { number: 7, status: Some(w::OwnerTaskStatus::Failed) },
        )
        .expect("literal"),
    )
}
#[must_use]
pub fn tree_rows_record() -> w::OwnerTree {
    let l = limits();
    let dependency = w::OwnerTaskLink::Stored(
        w::OwnerTaskLinkStored::new(
            &l,
            w::OwnerTaskLinkStoredParts { number: 19, status: Some(w::OwnerTaskStatus::Failed) },
        )
        .expect("literal"),
    );
    let row = w::OwnerTreeRow::new(
        &l,
        w::OwnerTreeRowParts {
            chip: live_chip_record(),
            parent: 7,
            depth: 1,
            proposals: rows([w::ProposalKey::new(
                &l,
                w::ProposalKeyParts {
                    proposer: w::Proposer::Party(
                        w::ProposerParty::new(&l, w::ProposerPartyParts { value: 11 }).expect("literal"),
                    ),
                    number: 13,
                },
            )
            .expect("literal")]),
            spent: 17,
            dependencies: pieces([encoded(dependency.measure(), |w| dependency.encode(w))]),
            cut: true,
        },
    )
    .expect("literal");
    w::OwnerTree::new(
        &l,
        w::OwnerTreeParts {
            root: live_chip_record(),
            rows: pieces([encoded(row.measure(), |w| row.encode(w))]),
            folded: rows([w::OwnerTreeFold::new(
                &l,
                w::OwnerTreeFoldParts { parent: 7, phases: phases_record(), held_below: 23 },
            )
            .expect("literal")]),
            held_below: 29,
            next: Some(live_cursor_record()),
        },
    )
    .expect("literal")
}
#[must_use]
pub fn live_inbox_cursor_record() -> w::OwnerInboxCursor {
    let l = limits();
    w::OwnerInboxCursor::Inbox(
        w::OwnerInboxCursorInbox::new(
            &l,
            w::OwnerInboxCursorInboxParts {
                task: 7,
                object: w::OwnerInboxObject::Turn(
                    w::OwnerInboxObjectTurn::new(&l, w::OwnerInboxObjectTurnParts { message: 11 }).expect("literal"),
                ),
                at: Some(13),
            },
        )
        .expect("literal"),
    )
}
#[must_use]
pub fn inbox_both_groups_record() -> w::OwnerInbox {
    let l = limits();
    let need = w::OwnerInboxEntry::new(
        &l,
        w::OwnerInboxEntryParts {
            reply: None,
            task: stored_link_record(),
            project: Some(7),
            object: w::OwnerInboxObject::PersonTask,
            activity: Some(17),
            at: Some(13),
            unread: Some(true),
            cursor: live_inbox_cursor_record(),
        },
    )
    .expect("literal");
    let update = w::OwnerInboxEntry::new(
        &l,
        w::OwnerInboxEntryParts {
            reply: Some(task_line_record()),
            task: stored_link_record(),
            project: Some(7),
            object: w::OwnerInboxObject::Turn(
                w::OwnerInboxObjectTurn::new(&l, w::OwnerInboxObjectTurnParts { message: 11 }).expect("literal"),
            ),
            activity: Some(17),
            at: Some(13),
            unread: Some(true),
            cursor: live_inbox_cursor_record(),
        },
    )
    .expect("literal");
    let selection = w::OwnerInboxSelection::new(
        &l,
        w::OwnerInboxSelectionParts { cursor: live_inbox_cursor_record(), goals: rows([37, 41]) },
    )
    .expect("literal");
    w::OwnerInbox::new(
        &l,
        w::OwnerInboxParts {
            party: 19,
            needs: pieces([encoded(need.measure(), |w| need.encode(w))]),
            updates: pieces([encoded(update.measure(), |w| update.encode(w))]),
            next: Some(live_inbox_cursor_record()),
            cached_needs: 23,
            cached_updates: w::Count::new(&l, w::CountParts { value: 29, limited: true }).expect("literal"),
            results_seen: 31,
            updates_time_complete: false,
            selection: Some(pieces([encoded(selection.measure(), |w| selection.encode(w))])),
        },
    )
    .expect("literal")
}
#[must_use]
pub fn host_run_record() -> w::OwnerHostRun {
    let l = limits();
    w::OwnerHostRun::new(
        &l,
        w::OwnerHostRunParts {
            task: stored_link_record(),
            attempt: 11,
            kept: 13,
            listed: true,
            state: w::OwnerAttemptState::Kept,
            location: w::OwnerRunLocation::Adrift(
                w::OwnerRunLocationAdrift::new(
                    &l,
                    w::OwnerRunLocationAdriftParts {
                        channel: Some(17),
                        until: Some(19),
                        remaining: Some(Duration::from_nanos(23)),
                    },
                )
                .expect("literal"),
            ),
            grace_until: Some(29),
        },
    )
    .expect("literal")
}
#[must_use]
pub fn deployment_rows_record() -> w::OwnerDeployment {
    let l = limits();
    let run = host_run_record();
    let chip = live_chip_record();
    let host = w::OwnerHost::new(
        &l,
        w::OwnerHostParts {
            channel: 17,
            kind: w::OwnerHostKind::Engine,
            slots: 2,
            used: 1,
            free: 1,
            draining: true,
            connected: false,
            grace_until: Some(19),
            grace_remaining: Some(Duration::from_nanos(23)),
            runs: pieces([encoded(run.measure(), |w| run.encode(w))]),
            cut: true,
        },
    )
    .expect("literal");
    let account = w::OwnerAccount::new(
        &l,
        w::OwnerAccountParts {
            provider_name: None,
            manage_usage: None,
            number: 41,
            place: 0,
            uses: None,
            provider: Some(3),
            display: super::wire_text(b"Deployment account"),
            holder: w::OwnerAccountHolder::Deployment,
            sources: Box::from([]),
            generation: Some(43),
            state: w::OwnerAccountState::Later(
                w::OwnerAccountStateLater::new(&l, w::OwnerAccountStateLaterParts { until: Some(47) })
                    .expect("literal"),
            ),
            skein_bools: [false, false, true, false, true],
            spent_until: Some(53),
            runs: pieces([encoded(chip.measure(), |w| chip.encode(w))]),
            waiting: pieces([encoded(chip.measure(), |w| chip.encode(w))]),
        },
    )
    .expect("literal");
    w::OwnerDeployment::new(
        &l,
        w::OwnerDeploymentParts {
            store: w::StoreFill::new(&l, w::StoreFillParts { bytes: 31, mark: 37 }).expect("literal"),
            hosts: pieces([encoded(host.measure(), |w| host.encode(w))]),
            attempts: pieces([encoded(run.measure(), |w| run.encode(w))]),
            accounts: pieces([encoded(account.measure(), |w| account.encode(w))]),
            tasks: w::OwnerCapacity::new(&l, w::OwnerCapacityParts { used: 1, limit: 2 }).expect("literal"),
            workers: w::OwnerCapacity::new(&l, w::OwnerCapacityParts { used: 3, limit: 4 }).expect("literal"),
            attempts_capacity: w::OwnerCapacity::new(&l, w::OwnerCapacityParts { used: 5, limit: 6 }).expect("literal"),
            waiting_attempts: 7,
            proposals: w::OwnerCapacity::new(&l, w::OwnerCapacityParts { used: 8, limit: 9 }).expect("literal"),
            projects: rows([w::OwnerProjectCapacity::new(
                &l,
                w::OwnerProjectCapacityParts {
                    project: 7,
                    tasks: w::OwnerCapacity::new(&l, w::OwnerCapacityParts { used: 11, limit: 13 }).expect("literal"),
                    proposals: 17,
                },
            )
            .expect("literal")]),
            grace_bound: Duration::from_nanos(59),
            hello_refusals: rows([hello_refusal_record()]),
            hosts_forgotten: 83,
            hello_refusals_forgotten: 89,
            person_room_refusals: rows([person_refusal_record()]),
            person_room_refusals_forgotten: 109,
            person_answer_retention: Duration::from_nanos(113),
            task_room_refusals: rows([task_refusal_record()]),
            task_room_refusals_forgotten: 149,
            connectors: rows([2, 3]),
            connectors_cut: true,
            cut: false,
        },
    )
    .expect("literal")
}
fn hello_refusal_record() -> w::OwnerHelloRefusalLine {
    let l = limits();
    w::OwnerHelloRefusalLine::new(
        &l,
        w::OwnerHelloRefusalLineParts {
            channel: 61,
            at: 67,
            slots: 2,
            stop_bound: Duration::from_nanos(71),
            why: w::OwnerHelloRefusal::Agent(
                w::OwnerHelloRefusalAgent::new(
                    &l,
                    w::OwnerHelloRefusalAgentParts {
                        why: w::OwnerAgentShortfall::Window(
                            w::OwnerAgentShortfallWindow::new(
                                &l,
                                w::OwnerAgentShortfallWindowParts { charter: 73, model: 79 },
                            )
                            .expect("literal"),
                        ),
                    },
                )
                .expect("literal"),
            ),
        },
    )
    .expect("literal")
}
fn person_refusal_record() -> w::OwnerPersonCapacityRefusal {
    let l = limits();
    w::OwnerPersonCapacityRefusal::new(
        &l,
        w::OwnerPersonCapacityRefusalParts {
            key: w::OwnerRequestKey::new(&l, w::OwnerRequestKeyParts { person: 97, key: [3; 16] }).expect("literal"),
            project: Some(7),
            action: w::OwnerPersonCapacityAction::Proposal(
                w::OwnerPersonCapacityActionProposal::new(
                    &l,
                    w::OwnerPersonCapacityActionProposalParts {
                        proposer: w::Proposer::Party(
                            w::ProposerParty::new(&l, w::ProposerPartyParts { value: 101 }).expect("literal"),
                        ),
                        number: 103,
                    },
                )
                .expect("literal"),
            ),
            at: 107,
            cause: w::OwnerPersonCapacityCause::Limit(
                w::OwnerPersonCapacityCauseLimit::new(
                    &l,
                    w::OwnerPersonCapacityCauseLimitParts { which: w::AdmissionLimit::LiveTasks },
                )
                .expect("literal"),
            ),
        },
    )
    .expect("literal")
}
fn task_refusal_record() -> w::OwnerTaskCapacityRefusal {
    let l = limits();
    w::OwnerTaskCapacityRefusal::new(
        &l,
        w::OwnerTaskCapacityRefusalParts {
            request: w::OwnerTaskCapacityRequest::new(
                &l,
                w::OwnerTaskCapacityRequestParts {
                    project: Some(7),
                    by: w::OwnerRequester::Deployment(
                        w::OwnerRequesterDeployment::new(&l, w::OwnerRequesterDeploymentParts { project: 7 })
                            .expect("literal"),
                    ),
                    action: w::OwnerTaskCapacityAction::Source(
                        w::OwnerTaskCapacityActionSource::new(
                            &l,
                            w::OwnerTaskCapacityActionSourceParts {
                                funder: w::Funder::Task(
                                    w::FunderTask::new(&l, w::FunderTaskParts { value: 127 }).expect("literal"),
                                ),
                            },
                        )
                        .expect("literal"),
                    ),
                    cause: w::OwnerTaskCapacityCause::Inbox(
                        w::OwnerTaskCapacityCauseInbox::new(
                            &l,
                            w::OwnerTaskCapacityCauseInboxParts { task: 131, messages: 2, bytes: 137 },
                        )
                        .expect("literal"),
                    ),
                },
            )
            .expect("literal"),
            at: 139,
        },
    )
    .expect("literal")
}

/// A page fixture with defaults for every field, before a test selects its facts.
#[must_use]
pub fn task_page_record() -> w::TaskPage {
    w::TaskPage::new(&limits(), w::TaskPageParts { live: Box::new([]), live_after: None, ended: w::EndedGroup::None })
        .expect("empty page fits")
}
