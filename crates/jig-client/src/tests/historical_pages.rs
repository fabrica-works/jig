#![expect(
    clippy::wildcard_enum_match_arm,
    reason = "ordinary public-boundary proofs collect emitted requests and select their original identities"
)]
use super::{
    objects,
    spine::{LIMITS, delivered, drive, frame, start},
};
use crate::{
    Domain, Event, Request,
    action::{Act, End},
    address::{Address, Place, ProjectSection, TaskSection},
    contract::*,
};
use skein_lib::{List, Time, Token, Wall};
fn cursor(bytes: &[u8]) -> Cursor {
    Cursor { value: objects::text(bytes) }
}
fn rows<T>(values: impl IntoIterator<Item = T>) -> List<T> {
    let mut rows = List::with_capacity(2);
    for value in values {
        rows.push(value).ok().unwrap();
    }
    rows
}
fn ended(cut: bool) -> OwnerEndedPage {
    let mut chip = objects::chip(u64::MAX);
    chip.title = DocumentText { bytes: objects::text(b""), cut: true };
    chip.phase = Phase::Ended;
    chip.details = Box::new(ChipPhase::Ended { how: Ending::Done, at: Wall::EPOCH });
    OwnerEndedPage {
        rows: rows([OwnerEndedDelegate {
            chip,
            ended: OwnerChipEnding::Cancelled {
                reason: DocumentText { bytes: objects::text(b"source reason"), cut: true },
                result: None,
                by: OwnerRequester::Deployment { project: u32::MAX },
                at: Wall::from_nanos(u64::MAX),
            },
            at: None,
        }]),
        before: Some(cursor(&[255, 0, 47])),
        after: Some(cursor(&[0, 254])),
        cut,
    }
}
fn funding(project: u32) -> OwnerSpendPage {
    OwnerSpendPage {
        rows: rows([OwnerFundingRecord {
            period: None,
            funder: Funder::Period { project, period: u64::MAX },
            parent: Some(Funder::Recurring { project: u32::MAX, task: 0, period: 7 }),
            numbers: OwnerNumbers { budget: 11, spent: 13, spent_below: 17, reserved: u64::MAX },
            made: u32::MAX,
            closed: false,
        }]),
        before: Some(cursor(&[255, 0, 47])),
        after: Some(cursor(&[0, 254])),
        cut: true,
    }
}
fn begin() -> Domain {
    let mut d = Domain::new(&LIMITS, 91);
    let token = start(&mut d, List::with_capacity(0));
    let out = delivered(&mut d, Time::ZERO, token, frame(7));
    super::pages::finish_initial_inbox_reads(&mut d, &out);
    d
}
fn ended_read() -> (Domain, Token) {
    let mut d = begin();
    let out = drive(
        &mut d,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Plan }) },
    );
    let mut watch = None;
    for request in &out {
        if let Request::Open { watch, subject: Subject::Task { task: 8 } } = request {
            watch = Some(*watch);
            break;
        }
    }
    let watch = watch.unwrap();
    let mut task = super::confirmation::task();
    task.chip.phase = Phase::Done;
    task.delegates = rows([LiveDelegate { chip: objects::chip(9), takeover_budget: 0 }]);
    task.chip.details = Box::new(ChipPhase::Ended { how: Ending::Done, at: Wall::EPOCH });
    let out =
        drive(&mut d, Time::ZERO, Event::Watch { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(task))) });
    let mut token = None;
    for request in &out {
        if let Request::Read { read, what: Read::Ended { task: 8, after: None } } = request {
            token = Some(*read);
            break;
        }
    }
    let token = token.unwrap();
    (d, token)
}
fn spend_read() -> (Domain, Token, Token) {
    let mut d = begin();
    let out = drive(
        &mut d,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Project { project: 1, section: ProjectSection::Spend }) },
    );
    let mut spend = None;
    for request in &out {
        if let Request::Read { read, what: Read::Spend { project: 1, after: None } } = request {
            spend = Some(*read);
            break;
        }
    }
    let spend = spend.unwrap();
    let mut project = None;
    for request in &out {
        if let Request::Read { read, what: Read::Project { project: 1 } } = request {
            project = Some(*read);
            break;
        }
    }
    let project = project.unwrap();
    (d, spend, project)
}
#[test]
fn historical_owner_original_read_callbacks_keep_complete_rows_cuts_and_two_opaque_edges() {
    let (mut d, token) = ended_read();
    let page = ended(true);
    let original = page.clone();
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: token, result: ReadResult::Value(ReadValue::OwnerEnded(Box::new(page))) },
    );
    assert_eq!(d.task_page().owner_ended.as_deref(), Some(&original));
    assert_eq!(d.task_page().owner_ended.as_ref().unwrap().rows.capacity(), 2);
    assert!(d.task_page().ended_cut);
    assert_eq!(d.task_page().ended_before, original.before);
    assert_eq!(d.task_page().ended_after, original.after);
    assert!(d.objects().find(ObjectKey::Result(u64::MAX)).is_none());
    let (mut d, token, _) = spend_read();
    let page = funding(1);
    let original = page.clone();
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: token, result: ReadResult::Value(ReadValue::OwnerSpend(Box::new(page))) },
    );
    assert_eq!(d.project_page().owner_spend.as_deref(), Some(&original));
    assert!(d.project_page().spend.is_empty());
    assert_eq!(d.project_page().spend_before, original.before);
    assert_eq!(d.project_page().spend_after, original.after);
}
#[test]
fn historical_owner_wrong_original_query_foreign_funder_and_stale_terminal_cannot_replace_pages() {
    let (mut d, token, wrong) = spend_read();
    let page = funding(1);
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: wrong, result: ReadResult::Value(ReadValue::OwnerSpend(Box::new(page.clone()))) },
    );
    assert!(d.project_page().owner_spend.is_none());
    for wrong_page in [
        funding(2),
        OwnerSpendPage {
            rows: rows([OwnerFundingRecord { funder: Funder::Task(1), ..page.rows.get(0).unwrap().clone() }]),
            ..page.clone()
        },
    ] {
        drive(
            &mut d,
            Time::ZERO,
            Event::Read { read: token, result: ReadResult::Value(ReadValue::OwnerSpend(Box::new(wrong_page))) },
        );
        assert!(d.project_page().owner_spend.is_none());
    }
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: token, result: ReadResult::Value(ReadValue::OwnerSpend(Box::new(page.clone()))) },
    );
    assert_eq!(d.project_page().owner_spend.as_deref(), Some(&page));
    drive(&mut d, Time::ZERO, Event::Read { read: token, result: ReadResult::None(ReadNone::Gone) });
    assert_eq!(d.project_page().owner_spend.as_deref(), Some(&page));
    let (mut d, token) = ended_read();
    drive(&mut d, Time::ZERO, Event::Went { place: Place::Elsewhere });
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: token, result: ReadResult::Value(ReadValue::OwnerEnded(Box::new(ended(true)))) },
    );
    assert!(d.task_page().owner_ended.is_none());
    assert!(d.task_page().ended_task.is_none());
}
#[test]
fn historical_owner_more_keeps_original_window_tokens_and_each_supplied_opaque_cursor() {
    let (mut d, token) = ended_read();
    let page = ended(true);
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: token, result: ReadResult::Value(ReadValue::OwnerEnded(Box::new(page.clone()))) },
    );
    for (end, edge) in [(End::Before, page.before), (End::After, page.after)] {
        let out = drive(
            &mut d,
            Time::ZERO,
            Event::Act { act: Act::More { window: Token::new(crate::pages::task::ENDED_WINDOW), end } },
        );
        let expected = Read::Ended { task: 8, after: edge };
        let mut found = false;
        for request in &out {
            if let Request::Read { what, .. } = request
                && *what == expected
            {
                found = true;
                break;
            }
        }
        assert!(found);
    }
    let (mut d, token, _) = spend_read();
    let page = funding(1);
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: token, result: ReadResult::Value(ReadValue::OwnerSpend(Box::new(page.clone()))) },
    );
    for (end, edge) in [(End::Before, page.before), (End::After, page.after)] {
        let out = drive(
            &mut d,
            Time::ZERO,
            Event::Act { act: Act::More { window: Token::new(crate::pages::project::SPEND_WINDOW), end } },
        );
        let expected = Read::Spend { project: 1, after: edge };
        let mut found = false;
        for request in &out {
            if let Request::Read { what, .. } = request
                && *what == expected
            {
                found = true;
                break;
            }
        }
        assert!(found);
    }
}
#[test]
fn historical_owner_fresh_whole_page_clears_obsolete_page_loss_without_changing_row_loss_or_clocks() {
    let (mut d, token) = ended_read();
    let page = ended(true);
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: token, result: ReadResult::Value(ReadValue::OwnerEnded(Box::new(page.clone()))) },
    );
    let out = drive(
        &mut d,
        Time::ZERO,
        Event::Act { act: Act::More { window: Token::new(crate::pages::task::ENDED_WINDOW), end: End::After } },
    );
    let mut token = None;
    for request in &out {
        if let Request::Read { read, what: Read::Ended { .. } } = request {
            token = Some(*read);
            break;
        }
    }
    let token = token.unwrap();
    let mut fresh = page.clone();
    fresh.cut = false;
    fresh.before = None;
    fresh.after = None;
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: token, result: ReadResult::Value(ReadValue::OwnerEnded(Box::new(fresh.clone()))) },
    );
    assert_eq!(d.task_page().owner_ended.as_deref(), Some(&fresh));
    assert!(!d.task_page().ended_cut);
    assert!(fresh.rows.get(0).unwrap().chip.title.cut);
    assert_eq!(fresh.rows.get(0).unwrap().at, None);
    assert_eq!(fresh.rows.get(0).unwrap().ended, page.rows.get(0).unwrap().ended);
    drive(&mut d, Time::ZERO, Event::Went { place: Place::Elsewhere });
    assert!(d.task_page().owner_ended.is_none());
}
#[test]
fn historical_owner_old_result_backed_ended_pages_stay_on_their_original_object_path() {
    let (mut d, token) = ended_read();
    let result = objects::result();
    drive(
        &mut d,
        Time::ZERO,
        Event::Read {
            read: token,
            result: ReadResult::Value(ReadValue::Ended(Box::new(ObjectPage {
                rows: rows([result.clone()]),
                before: None,
                after: None,
                cut: true,
            }))),
        },
    );
    assert!(d.task_page().owner_ended.is_none());
    let id = d.objects().find(ObjectKey::Result(result.task.task)).unwrap();
    assert_eq!(d.objects().get(id).unwrap().body, Some(crate::objects::Body::Result(Box::new(result))));
    assert!(d.task_page().ended_cut);
}

#[test]
fn historical_owner_wrong_query_and_reserved_capacity_refusals_preserve_the_original_live_read() {
    let (mut d, spend, project) = spend_read();
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: project, result: ReadResult::Value(ReadValue::OwnerEnded(Box::new(ended(true)))) },
    );
    assert!(d.task_page().owner_ended.is_none());
    let mut over = funding(1);
    over.rows = List::with_capacity(LIMITS.rows + 1);
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: spend, result: ReadResult::Value(ReadValue::OwnerSpend(Box::new(over))) },
    );
    assert!(d.project_page().owner_spend.is_none());
    let mut over = funding(1);
    over.after = Some(Cursor { value: Text::new(skein_lib::bytes::copy_of(&[0; 33]), 33).unwrap() });
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: spend, result: ReadResult::Value(ReadValue::OwnerSpend(Box::new(over))) },
    );
    assert!(d.project_page().owner_spend.is_none());
    let actual = funding(1);
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: spend, result: ReadResult::Value(ReadValue::OwnerSpend(Box::new(actual.clone()))) },
    );
    assert_eq!(d.project_page().owner_spend.as_deref(), Some(&actual));
    let (mut d, token) = ended_read();
    let mut over = ended(true);
    over.rows = List::with_capacity(LIMITS.rows + 1);
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: token, result: ReadResult::Value(ReadValue::OwnerEnded(Box::new(over))) },
    );
    assert!(d.task_page().owner_ended.is_none());
    let actual = ended(false);
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: token, result: ReadResult::Value(ReadValue::OwnerEnded(Box::new(actual.clone()))) },
    );
    assert_eq!(d.task_page().owner_ended.as_deref(), Some(&actual));
}
#[test]
fn historical_owner_fresh_funding_page_resets_loss_and_navigation_clears_only_its_retained_state() {
    let (mut d, token, _) = spend_read();
    let page = funding(1);
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: token, result: ReadResult::Value(ReadValue::OwnerSpend(Box::new(page.clone()))) },
    );
    let out = drive(
        &mut d,
        Time::ZERO,
        Event::Act { act: Act::More { window: Token::new(crate::pages::project::SPEND_WINDOW), end: End::After } },
    );
    let mut token = None;
    for request in &out {
        if let Request::Read { read, what: Read::Spend { .. } } = request {
            token = Some(*read);
            break;
        }
    }
    let token = token.unwrap();
    let mut fresh = page.clone();
    fresh.cut = false;
    fresh.before = None;
    fresh.after = None;
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: token, result: ReadResult::Value(ReadValue::OwnerSpend(Box::new(fresh.clone()))) },
    );
    assert_eq!(d.project_page().owner_spend.as_deref(), Some(&fresh));
    assert!(!d.project_page().spend_cut);
    assert!(d.project_page().spend_before.is_none() && d.project_page().spend_after.is_none());
    assert_eq!(fresh.rows, page.rows);
    drive(&mut d, Time::ZERO, Event::Went { place: Place::Elsewhere });
    assert!(d.project_page().owner_spend.is_none());
    drive(
        &mut d,
        Time::ZERO,
        Event::Read { read: token, result: ReadResult::Value(ReadValue::OwnerSpend(Box::new(page))) },
    );
    assert!(d.project_page().owner_spend.is_none());
}

#[test]
fn historical_owner_old_spend_period_budget_pool_and_task_envelope_stays_independent() {
    let (mut d, token, _) = spend_read();
    let period = SpendPeriod {
        period: u64::MAX,
        budget: Budget { total: 11, own: 13, ended: 17, reserved: 19, left: 23, funder: Actor::Party(7) },
        pools: List::with_capacity(2),
        tasks: List::with_capacity(2),
    };
    drive(
        &mut d,
        Time::ZERO,
        Event::Read {
            read: token,
            result: ReadResult::Value(ReadValue::Spend(Box::new(SpendPage {
                rows: rows([period.clone()]),
                before: None,
                after: Some(cursor(b"old exact edge")),
                cut: true,
            }))),
        },
    );
    assert!(d.project_page().owner_spend.is_none());
    assert_eq!(d.project_page().spend.get(0), Some(&period));
    assert!(d.project_page().spend_cut);
    assert_eq!(d.project_page().spend_after, Some(cursor(b"old exact edge")));
}
