//! Complete durable tasks and all remaining row kinds (protocol/store.md, 5.1).
use super::{bound_problem, fixtures, saved_tasks, tasks_limits};
use crate::{codec::tasks, tasks_v3 as wire};
use jig_domain_tasks as domain;
use skein_lib::{Duration, Token, Wall};

fn activity(line: domain::Line) -> domain::Stored {
    domain::Stored::Activity(domain::ActivityLine { task: 17, position: 29, at: Wall::from_nanos(31), line })
}
fn live_record(record: domain::TaskRecord) -> domain::Stored {
    domain::Stored::Live(Box::new(record))
}
fn message_row(kind: domain::MessageKind) -> domain::Stored {
    let mut word = fixtures::task_word(5);
    word.kind = kind;
    activity(domain::Line::Heard(word))
}
fn current_record() -> domain::TaskRecord {
    let mut record = fixtures::task_record();
    record.proposal = None;
    record.recurring = None;
    record
}

fn a_task_records_label_flags_and_last_activity_read_back() {
    let mut record = current_record();
    record.label = 0x0102_0304;
    record.flags = domain::Flags { conversation: false, listed: true, retention: domain::Retention::ForgottenOnceRead };
    record.last_activity = Wall::from_nanos(9_000_000_000);
    let row = live_record(record);
    assert_eq!(saved_tasks(&row), Ok(row));
}
fn a_live_task_record_reads_back_whole() {
    let row = domain::Stored::Live(Box::new(fixtures::task_record()));
    assert_eq!(saved_tasks(&row), Ok(row));
}
fn an_ended_task_with_its_result_reads_back() {
    let mut record = fixtures::task_record();
    record.phase = domain::Phase::Ended(domain::Ending::Done(domain::TaskResult::Report {
        words: Box::from(b"result".as_slice()),
    }));
    let row = domain::Stored::Ended(Box::new(record));
    assert_eq!(saved_tasks(&row), Ok(row));
}
fn an_activity_line_of_each_kind_reads_back() {
    for line in [
        domain::Line::Words {
            message: 5,
            by: domain::Party::Person(31),
            kind: domain::MessageKind::Question,
            words: Box::from(b"question".as_slice()),
        },
        domain::Line::Heard(fixtures::task_word(5)),
        domain::Line::Notified { recipient: domain::Party::Task(19), word: fixtures::task_word(5) },
        domain::Line::RunBegan {
            attempt: 3,
            why: domain::Why::Retry { tries: fixtures::task_record().tries },
            resumed: true,
        },
        domain::Line::RunEnded { attempt: 3, end: domain::EndKind::Failed { class: domain::Class::Lost }, spent: 7 },
        domain::Line::Turn { reply: 37, attempt: 3, turn: 5, read: Some(5) },
        domain::Line::Decided { revision: 3 },
        domain::Line::Milestone { position: 13 },
    ] {
        let row = activity(line);
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_terminal_activity_kind_reads_back() {
    for end in
        [domain::EndKind::Parked, domain::EndKind::Finished, domain::EndKind::Cancelled, domain::EndKind::Refused]
    {
        let row = activity(domain::Line::RunEnded { attempt: 3, end, spent: 7 });
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_index_row_reads_back() {
    for row in [
        domain::Stored::Update(domain::Update {
            party: 31,
            at: Wall::from_nanos(41),
            task: 17,
            object: domain::UpdateObject::Result,
        }),
        domain::Stored::Update(domain::Update {
            party: 31,
            at: Wall::from_nanos(41),
            task: 17,
            object: domain::UpdateObject::Reply { message: domain::Newest::of(37), activity: domain::Newest::of(29) },
        }),
        domain::Stored::Result(domain::ResultRow { party: 31, position: 43, task: 17 }),
        domain::Stored::ConversationNewest(domain::Conversation { party: 31, task: 17 }),
        domain::Stored::ConversationOldest(domain::Conversation { party: 31, task: 17 }),
        domain::Stored::Goal(domain::GoalRow {
            project: 3,
            task: 17,
            endings: domain::Endings { done: u32::MAX, failed: 3, cancelled: 5 },
        }),
        domain::Stored::EndedDelegate(domain::Edge { requester: 17, task: 19 }),
        domain::Stored::Delegate(domain::Edge { requester: 17, task: 19 }),
        domain::Stored::EndedByParty(domain::EndedByParty {
            party: 31,
            label: 3,
            ended_at: Wall::from_nanos(41),
            task: 17,
        }),
    ] {
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_stub_status_reads_back() {
    for phase in [domain::Status::Done, domain::Status::Failed, domain::Status::Cancelled] {
        let row = domain::Stored::Stub(domain::Stub { task: 17, phase, result: Token::new(19) });
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_message_kind_reads_back() {
    for kind in [
        domain::MessageKind::Escalation { task: 17, revision: 3 },
        domain::MessageKind::Proposal { proposer: 17, proposal: 43, kind: domain::ProposalKind::Widen },
        domain::MessageKind::PersonStalled { task: 19 },
        domain::MessageKind::ProposalDecision { proposal: 43, accepted: false },
        domain::MessageKind::Words,
        domain::MessageKind::EffectNotMade { entry: 11 },
        domain::MessageKind::Amendment { revision: 3 },
        domain::MessageKind::Question,
        domain::MessageKind::Answer { question: 5 },
        domain::MessageKind::Notice { subscription: 109, target: 19, state: domain::NoticeState::Held { revision: 1 } },
        domain::MessageKind::Timer { subscription: 109 },
        domain::MessageKind::News { subscription: 109, class: domain::NewsClass::Wakes },
        domain::MessageKind::Result(domain::ResultKind::Report),
    ] {
        let row = message_row(kind);
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_result_message_kind_reads_back() {
    for kind in [
        domain::ResultKind::Approved { state: Box::from(b"state".as_slice()), words_present: true },
        domain::ResultKind::Report,
        domain::ResultKind::Verdict {
            code: 7,
            fields: Box::new([domain::ResultField {
                name: Box::from(b"field".as_slice()),
                value: Box::from(b"value".as_slice()),
            }]),
        },
        domain::ResultKind::Change { connector: 2, kind: 3, resource: 11 },
        domain::ResultKind::Failed,
        domain::ResultKind::Cancelled,
    ] {
        let row = message_row(domain::MessageKind::Result(kind));
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_notice_state_reads_back() {
    for state in [
        domain::NoticeState::Held { revision: 1 },
        domain::NoticeState::Done,
        domain::NoticeState::Failed,
        domain::NoticeState::Cancelled,
    ] {
        let row = message_row(domain::MessageKind::Notice { subscription: 109, target: 19, state });
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_news_class_reads_back() {
    for class in [domain::NewsClass::Wakes, domain::NewsClass::Kept, domain::NewsClass::Dropped] {
        let row = message_row(domain::MessageKind::News { subscription: 109, class });
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_escalation_state_reads_back() {
    for escalation in [
        domain::Escalation::Unheld { revision: 3 },
        domain::Escalation::Routing { revision: 5 },
        domain::Escalation::Rejected { revision: 7, by: 31, reason: Box::from(b"rejected".as_slice()) },
    ] {
        let mut record = current_record();
        record.escalation = escalation;
        let row = live_record(record);
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_escalation_holder_reads_back() {
    for holder in [
        domain::EscalationHolder::Task(19),
        domain::EscalationHolder::Person(31),
        domain::EscalationHolder::Role { project: 3, role: 7 },
    ] {
        let mut record = current_record();
        record.escalation = domain::Escalation::Waiting { revision: 3, holder, entry: 43, since: Wall::from_nanos(47) };
        let row = live_record(record);
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_invalid_result_reason_reads_back() {
    for reason in [
        domain::InvalidResult::Form,
        domain::InvalidResult::Verdict,
        domain::InvalidResult::Words,
        domain::InvalidResult::Change,
        domain::InvalidResult::Followups,
    ] {
        let mut record = current_record();
        record.invalid_result = Some(reason);
        let row = live_record(record);
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_subscription_kind_reads_back() {
    for kind in [
        domain::SubscriptionKind::Task { target: 19, held: true, result: false },
        domain::SubscriptionKind::Timer { at: Wall::from_nanos(59), period: Some(Duration::from_nanos(3)) },
        domain::SubscriptionKind::Timer { at: Wall::from_nanos(59), period: None },
        domain::SubscriptionKind::Topic { connector: 2, topic: 113 },
    ] {
        let mut record = current_record();
        record.subscriptions = Box::new([domain::Subscription { number: 109, kind }]);
        let row = live_record(record);
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn a_task_record_without_optional_values_reads_back() {
    let mut record = fixtures::task_record();
    record.instructions = None;
    record.grace = None;
    record.run_since = None;
    record.retry_notice = None;
    record.ended_at = None;
    record.proposal = None;
    record.taken_by = None;
    record.person_answer = None;
    record.person_wait = None;
    record.recurring = None;
    record.tracked = None;
    record.hold_wait_since = None;
    record.last_answer = None;
    record.invalid_result = None;
    record.question_lines = Box::new([domain::QuestionLine { message: 5, line: 7, answer: None }]);
    let row = live_record(record);
    assert_eq!(saved_tasks(&row), Ok(row));
}
fn a_task_record_at_its_inbox_bound_reads_back() {
    let mut record = fixtures::task_record();
    let mut first = fixtures::task_word(5);
    first.words = skein_lib::bytes::zeroed(16);
    let mut second = fixtures::task_word(7);
    second.words = skein_lib::bytes::zeroed(16);
    record.inbox = Box::new([first, second]);
    let row = live_record(record);
    assert_eq!(saved_tasks(&row), Ok(row));
}
#[test]
fn a_task_record_past_its_inbox_count_is_refused() {
    let mut record = fixtures::task_record();
    record.inbox = Box::new([
        fixtures::task_word(5),
        fixtures::task_word(7),
        fixtures::task_word(11),
        fixtures::task_word(13),
        fixtures::task_word(17),
    ]);
    assert_eq!(
        tasks::encode(&live_record(record), &tasks_limits()),
        Err(bound_problem(wire::Path::TaskInterestsInbox))
    );
}
#[test]
fn a_task_record_inbox_decoded_above_its_count_is_refused() {
    let limits = tasks_limits();
    let mut wider = limits;
    wider.task_interests_inbox = 5;
    let mut record = fixtures::task_record();
    record.inbox = Box::new([
        fixtures::task_word(5),
        fixtures::task_word(7),
        fixtures::task_word(11),
        fixtures::task_word(13),
        fixtures::task_word(17),
    ]);
    let bytes = tasks::encode(&live_record(record), &wider).expect("within wider count");
    assert_eq!(tasks::decode(&bytes, &limits), Err(bound_problem(wire::Path::TaskInterestsInbox)));
}
#[test]
fn record_count_limits_past_the_schema_ceilings_are_refused() {
    for field in 0_u8..6 {
        let mut limits = fixtures::TASKS_LIMITS;
        match field {
            0 => limits.inbox_messages = u32::MAX,
            1 => limits.delegates = u32::MAX,
            2 => limits.references = u32::MAX,
            3 => limits.subscriptions = u32::MAX,
            4 => limits.saved_resources = u32::MAX,
            _ => limits.authority_segments = u32::MAX,
        }
        assert_eq!(tasks::limits(&limits), None);
    }
}
fn a_result_word_uses_the_larger_shared_word_bound() {
    let mut record = fixtures::task_record();
    let mut word = fixtures::task_word(5);
    word.kind = domain::MessageKind::Result(domain::ResultKind::Report);
    word.words = skein_lib::bytes::zeroed(32);
    record.inbox = Box::new([word]);
    let row = live_record(record);
    assert_eq!(saved_tasks(&row), Ok(row));
}
#[test]
fn an_activity_words_line_past_its_own_word_bound_is_refused() {
    let row = activity(domain::Line::Words {
        message: 5,
        by: domain::Party::Person(31),
        kind: domain::MessageKind::Words,
        words: skein_lib::bytes::zeroed(17),
    });
    assert_eq!(tasks::encode(&row, &tasks_limits()), Err(bound_problem(wire::Path::ActivityWordsWords)));
}
fn a_person_wait_without_a_stall_notice_reads_back() {
    let mut record = fixtures::task_record();
    record.person_wait.as_mut().expect("fixture wait").notice = None;
    let row = live_record(record);
    assert_eq!(saved_tasks(&row), Ok(row));
}

fn live_task_records_read_back_whole() {
    table([
        ("a_live_task_record_reads_back_whole", a_live_task_record_reads_back_whole),
        ("a_task_record_without_optional_values_reads_back", a_task_record_without_optional_values_reads_back),
        ("a_task_record_at_its_inbox_bound_reads_back", a_task_record_at_its_inbox_bound_reads_back),
    ]);
}

type Case = (&'static str, fn());

fn table<const N: usize>(cases: [Case; N]) {
    let mut failures = [None; N];
    let mut passed = true;
    for ((name, case), failure) in cases.into_iter().zip(&mut failures) {
        if std::panic::catch_unwind(case).is_err() {
            *failure = Some(name);
            passed = false;
        }
    }
    assert!(passed, "{failures:?}");
}

#[test]
fn task_records_read_back_as_written() {
    table([
        (
            "a_task_records_label_flags_and_last_activity_read_back",
            a_task_records_label_flags_and_last_activity_read_back,
        ),
        ("an_ended_task_with_its_result_reads_back", an_ended_task_with_its_result_reads_back),
        ("an_activity_line_of_each_kind_reads_back", an_activity_line_of_each_kind_reads_back),
        ("each_terminal_activity_kind_reads_back", each_terminal_activity_kind_reads_back),
        ("each_index_row_reads_back", each_index_row_reads_back),
        ("each_stub_status_reads_back", each_stub_status_reads_back),
        ("each_message_kind_reads_back", each_message_kind_reads_back),
        ("each_result_message_kind_reads_back", each_result_message_kind_reads_back),
        ("each_notice_state_reads_back", each_notice_state_reads_back),
        ("each_news_class_reads_back", each_news_class_reads_back),
        ("each_escalation_state_reads_back", each_escalation_state_reads_back),
        ("each_escalation_holder_reads_back", each_escalation_holder_reads_back),
        ("each_invalid_result_reason_reads_back", each_invalid_result_reason_reads_back),
        ("each_subscription_kind_reads_back", each_subscription_kind_reads_back),
        ("a_person_wait_without_a_stall_notice_reads_back", a_person_wait_without_a_stall_notice_reads_back),
        ("live_task_records_read_back_whole", live_task_records_read_back_whole),
        ("a_result_word_uses_the_larger_shared_word_bound", a_result_word_uses_the_larger_shared_word_bound),
    ]);
}
