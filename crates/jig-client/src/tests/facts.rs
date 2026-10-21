use super::fixtures::*;
use crate::{
    contract::*,
    facts::{self, ChatStatus},
};

struct Row {
    name: &'static str,
    phase: OwnerChipPhase,
    work: Option<OwnerChipWork>,
    wait: WaitsOn,
    run_wait: Option<TaskRunWait>,
    expected: fn() -> ChatStatus,
}
const NOTHING: WaitsOn = WaitsOn::Nothing;
const QUESTION: WaitsOn =
    WaitsOn::Party { whom: Whom::Person(PARTY), entries: 1, task: SUBJECT_TASK, kind: WaitedFor::Question };
const RUNNING: OwnerChipPhase = OwnerChipPhase::Active(RunStage::Running { attempt: ATTEMPT });
#[expect(clippy::too_many_lines, reason = "status precedence cases share one table")]
fn rows() -> Box<[Row]> {
    Box::new([
        Row {
            name: "due",
            phase: OwnerChipPhase::Active(RunStage::Due),
            work: None,
            wait: NOTHING,
            run_wait: None,
            expected: || ChatStatus::Starting,
        },
        Row {
            name: "preparing",
            phase: OwnerChipPhase::Active(RunStage::Preparing),
            work: None,
            wait: NOTHING,
            run_wait: None,
            expected: || ChatStatus::Starting,
        },
        Row {
            name: "claimed",
            phase: OwnerChipPhase::Active(RunStage::Claimed { attempt: ATTEMPT }),
            work: None,
            wait: NOTHING,
            run_wait: None,
            expected: || ChatStatus::Starting,
        },
        Row {
            name: "host preparing",
            phase: RUNNING,
            work: Some(OwnerChipWork::Preparing),
            wait: NOTHING,
            run_wait: None,
            expected: || ChatStatus::Starting,
        },
        Row {
            name: "host starting",
            phase: RUNNING,
            work: Some(OwnerChipWork::Starting),
            wait: NOTHING,
            run_wait: None,
            expected: || ChatStatus::Starting,
        },
        Row {
            name: "writing",
            phase: RUNNING,
            work: Some(OwnerChipWork::Writing),
            wait: NOTHING,
            run_wait: None,
            expected: || ChatStatus::Writing,
        },
        Row {
            name: "working",
            phase: RUNNING,
            work: Some(OwnerChipWork::Active),
            wait: NOTHING,
            run_wait: None,
            expected: || ChatStatus::Working { since: OBSERVED_SINCE },
        },
        Row {
            name: "ending work",
            phase: RUNNING,
            work: Some(OwnerChipWork::Ending),
            wait: NOTHING,
            run_wait: None,
            expected: || ChatStatus::Working { since: OBSERVED_SINCE },
        },
        Row {
            name: "unobserved running",
            phase: RUNNING,
            work: None,
            wait: NOTHING,
            run_wait: None,
            expected: || ChatStatus::Working { since: PHASE_SINCE },
        },
        Row {
            name: "waiting run",
            phase: RUNNING,
            work: Some(OwnerChipWork::Waiting),
            wait: WaitsOn::News,
            run_wait: None,
            expected: || ChatStatus::Parked,
        },
        Row {
            name: "idle",
            phase: OwnerChipPhase::Active(RunStage::Idle),
            work: None,
            wait: NOTHING,
            run_wait: None,
            expected: || ChatStatus::Parked,
        },
        Row {
            name: "reader while writing",
            phase: RUNNING,
            work: Some(OwnerChipWork::Writing),
            wait: QUESTION,
            run_wait: None,
            expected: || ChatStatus::WaitingForYou { kind: WaitedFor::Question, entries: 1 },
        },
        Row {
            name: "reader while due",
            phase: OwnerChipPhase::Active(RunStage::Due),
            work: None,
            wait: QUESTION,
            run_wait: None,
            expected: || ChatStatus::WaitingForYou { kind: WaitedFor::Question, entries: 1 },
        },
        Row {
            name: "another party",
            phase: RUNNING,
            work: None,
            wait: WaitsOn::Party {
                whom: Whom::Person(OTHER_PARTY),
                entries: 1,
                task: SUBJECT_TASK,
                kind: WaitedFor::Proposal,
            },
            run_wait: None,
            expected: || ChatStatus::WaitingFor(Whom::Person(OTHER_PARTY)),
        },
        Row {
            name: "reader role",
            phase: RUNNING,
            work: None,
            wait: WaitsOn::Party {
                whom: Whom::Role { project: PROJECT, role: OWNER_ROLE },
                entries: 2,
                task: SUBJECT_TASK,
                kind: WaitedFor::PersonTask,
            },
            run_wait: None,
            expected: || ChatStatus::WaitingForYou { kind: WaitedFor::PersonTask, entries: 2 },
        },
        Row {
            name: "unheld role",
            phase: RUNNING,
            work: None,
            wait: WaitsOn::Party {
                whom: Whom::Role { project: PROJECT, role: MEMBER_ROLE },
                entries: 1,
                task: SUBJECT_TASK,
                kind: WaitedFor::Proposal,
            },
            run_wait: None,
            expected: || ChatStatus::WaitingFor(Whom::Role { project: PROJECT, role: MEMBER_ROLE }),
        },
        Row {
            name: "another project's role",
            phase: RUNNING,
            work: None,
            wait: WaitsOn::Party {
                whom: Whom::Role { project: OTHER_PROJECT, role: OWNER_ROLE },
                entries: 1,
                task: SUBJECT_TASK,
                kind: WaitedFor::Escalation,
            },
            run_wait: None,
            expected: || ChatStatus::WaitingFor(Whom::Role { project: OTHER_PROJECT, role: OWNER_ROLE }),
        },
        Row {
            name: "backing off",
            phase: OwnerChipPhase::Active(RunStage::BackingOff { until: RETRY_AT }),
            work: None,
            wait: NOTHING,
            run_wait: None,
            expected: || ChatStatus::Retrying { until: RETRY_AT, notice: RETRY_NOTICE },
        },
        Row {
            name: "held before wait",
            phase: OwnerChipPhase::Held { reason: HoldCategory::Stopped, revision: 1 },
            work: None,
            wait: QUESTION,
            run_wait: None,
            expected: || ChatStatus::Held(HoldCategory::Stopped),
        },
        Row {
            name: "closing before wait",
            phase: OwnerChipPhase::Closing(ClosingStage::Delegates),
            work: None,
            wait: QUESTION,
            run_wait: None,
            expected: || ChatStatus::Closing,
        },
        Row {
            name: "ended before wait",
            phase: OwnerChipPhase::Ended,
            work: None,
            wait: QUESTION,
            run_wait: None,
            expected: || ChatStatus::Ended { at: ENDED_AT },
        },
        Row {
            name: "dependencies before refusal",
            phase: OwnerChipPhase::Active(RunStage::Due),
            work: None,
            wait: WaitsOn::Dependencies {
                pending: {
                    let mut pending = skein_lib::List::with_capacity(1);
                    pending.push(SUBJECT_TASK).unwrap();
                    pending
                },
                count: 1,
            },
            run_wait: Some(TaskRunWait::Refused(TaskWaitRefused::Program(OpenProgram::Model))),
            expected: || ChatStatus::Parked,
        },
        Row {
            name: "tool with refusal",
            phase: RUNNING,
            work: Some(tool_work()),
            wait: NOTHING,
            run_wait: Some(TaskRunWait::Refused(TaskWaitRefused::Program(OpenProgram::Model))),
            expected: || ChatStatus::Tool { tool: Box::from(b"search".as_slice()), since: OBSERVED_SINCE },
        },
    ])
}
fn tool_work() -> OwnerChipWork {
    OwnerChipWork::Tool { call: Box::from(b"call".as_slice()), tool: Box::from(b"search".as_slice()) }
}
#[test]
fn chat_status_follows_the_pages_status_table() {
    let me = me();
    for row in &rows() {
        let chip = facts_chip(row.phase.clone(), row.work.clone());
        assert_eq!(
            facts::chat_status(&chip, &row.wait, row.run_wait.as_ref(), PROJECT, &me),
            (row.expected)(),
            "{}",
            row.name
        );
    }
    let chip = facts_chip(RUNNING, Some(tool_work()));
    assert_eq!(
        facts::chat_status(&chip, &NOTHING, None, PROJECT, &me),
        ChatStatus::Tool { tool: Box::from(b"search".as_slice()), since: OBSERVED_SINCE }
    );
}
#[test]
fn a_task_line_and_its_task_document_give_one_status() {
    let limits = jig_client_protocol_link::Limits::new(domain_limits(), 8).unwrap();
    let rows =
        jig_client_protocol_link::translate::documents::tasks(jig_client_test_records::tasks_rows_record(), &limits)
            .unwrap();
    let mut line = rows.rows.get(0).unwrap().clone();
    let mut task = facts_task();
    task.chip = facts_chip(RUNNING, Some(OwnerChipWork::Writing));
    task.waits_on = QUESTION;
    line.chip = task.chip.clone();
    line.waits_on = task.waits_on.clone();
    line.project = task.project;
    let me = me();
    assert_eq!(
        facts::chat_status(&task.chip, &task.waits_on, task.run_wait.as_ref(), task.project, &me),
        ChatStatus::WaitingForYou { kind: WaitedFor::Question, entries: 1 }
    );
    assert_eq!(
        facts::chat_status(&line.chip, &line.waits_on, None, line.project, &me),
        ChatStatus::WaitingForYou { kind: WaitedFor::Question, entries: 1 }
    );
}
#[test]
fn effect_is_none_for_an_act_not_offered() {
    let offers =
        [Offered { act: ActKind::Move, admission: Admission::Allowed, effect: Some(Effect::Move { reservation: 7 }) }];
    assert_eq!(facts::effect(&offers, ActKind::Move), Some(&Effect::Move { reservation: 7 }));
    assert_eq!(facts::offer(&offers, ActKind::Move), Some(&Admission::Allowed));
    assert_eq!(facts::effect(&offers, ActKind::Cancel), None);
    assert_eq!(facts::offer(&offers, ActKind::Cancel), None);
}
const ACTS: &[(ActKind, bool)] = &[
    (ActKind::Message, false),
    (ActKind::Answer, false),
    (ActKind::Take, false),
    (ActKind::HandBack, false),
    (ActKind::Accept, false),
    (ActKind::Reject, true),
    (ActKind::PassUp, false),
    (ActKind::Release, false),
    (ActKind::LeaveHeld, false),
    (ActKind::Amend, false),
    (ActKind::Stop, false),
    (ActKind::Cancel, true),
    (ActKind::Move, false),
    (ActKind::Reorder, false),
    (ActKind::Seen, false),
    (ActKind::EditNote, false),
    (ActKind::DeleteNote, true),
    (ActKind::Adopt, false),
    (ActKind::ChangePolicy, false),
    (ActKind::SetRoles, false),
    (ActKind::MakeService, false),
    (ActKind::SetPool, false),
    (ActKind::Withdraw, false),
];
#[test]
fn dangerous_marks_cancel_reject_and_delete_note_alone() {
    for &(act, expected) in ACTS {
        assert_eq!(facts::dangerous(act), expected, "{act:?}");
    }
}

#[test]
fn refusal_status_only_replaces_an_unplaced_starting_chat() {
    let me = me();
    let cause = TaskWaitRefused::Program(OpenProgram::Model);
    let wait = TaskRunWait::Refused(cause);
    for row in &rows() {
        let chip = facts_chip(row.phase.clone(), row.work.clone());
        let expected = match (&row.phase, &row.wait) {
            (OwnerChipPhase::Active(RunStage::Due | RunStage::Preparing), WaitsOn::Nothing) => {
                ChatStatus::HostRefused(cause)
            }
            _ => (row.expected)(),
        };
        assert_eq!(facts::chat_status(&chip, &row.wait, Some(&wait), PROJECT, &me), expected, "{}", row.name);
    }
}
