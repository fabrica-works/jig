use super::{objects, spine::drive};
use crate::{
    Event, Request,
    address::{Address, Place, TaskSection},
    contract::*,
    objects::Body,
};
use skein_lib::{List, Time, Wall};

#[test]
fn whole_task_context_absence_and_supplied_empty_are_distinct_in_original_reads() {
    let supplied = super::confirmation::task().context.unwrap();
    for context in [None, Some(supplied.clone())] {
        let (mut domain, page) = objects::live();
        let first = objects::watched(&mut domain, page, &[ObjectKey::Task(8)]);
        let read = objects::read_token(&first);
        let mut row = super::confirmation::task();
        row.context = context.clone();
        let original = row.clone();
        let output = drive(
            &mut domain,
            Time::ZERO,
            Event::Read { read, result: ReadResult::Value(ReadValue::Task(Box::new(row))) },
        );
        let id = domain.objects.find(ObjectKey::Task(8)).unwrap();
        assert_eq!(domain.objects.get(id).unwrap().body, Some(Body::Task(Box::new(original))));
        assert!(output.is_empty());
        let Some(Body::Task(retained)) = &domain.objects.get(id).unwrap().body else {
            panic!("whole task body");
        };
        assert_eq!(retained.context, context);
        assert_eq!(retained.spec_revision.value, 9);
    }
    assert!(supplied.parts.is_empty());
    assert_eq!(supplied.closing, None);
    assert_eq!(supplied.escalation, None);
}

#[test]
fn supplied_task_context_routes_only_its_whole_parts_and_impact_without_recreating_missing_facts() {
    let part = Part::Panel { connector: 2, task: u64::MAX };
    for present in [false, true] {
        let (mut domain, _) = objects::live();
        let opened = drive(
            &mut domain,
            Time::ZERO,
            Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Panels }) },
        );
        let mut task_watch = None;
        for request in &opened {
            if let Request::Open { watch, subject: Subject::Task { task: 8 } } = request {
                assert!(task_watch.replace(*watch).is_none());
            }
        }
        let watch = task_watch.unwrap();
        let mut row = super::confirmation::task();
        row.delegates = List::with_capacity(0);
        let context = row.context.as_mut().unwrap();
        context.parts = List::with_capacity(2);
        context.parts.push(part.clone()).unwrap();
        context.impact.tasks.running = u32::MAX;
        if !present {
            row.context = None;
        }
        let sections = crate::pages::task::sections(&row);
        assert_eq!(crate::pages::task::offered(&sections, TaskSection::Plan), present);
        assert_eq!(crate::pages::task::offered(&sections, TaskSection::Panels), present);
        let exact = row.clone();
        let output = drive(
            &mut domain,
            Time::ZERO,
            Event::Watch { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(row))) },
        );
        let mut parts = 0;
        for request in &output {
            if let Request::ShowPart { part: supplied, .. } = request {
                assert_eq!(supplied, &part);
                parts += 1;
            }
        }
        assert_eq!(parts, u32::from(present));
        let id = domain.objects.find(ObjectKey::Task(8)).unwrap();
        assert_eq!(domain.objects.get(id).unwrap().body, Some(Body::Task(Box::new(exact))));
    }
}

fn finished(phase: Option<OwnerChipPhase>, prior: Option<OwnerPriorPhase>) -> bool {
    let (mut domain, _) = objects::live();
    let opened = drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Activity }) },
    );
    let mut task_watch = None;
    for request in &opened {
        if let Request::Open { watch, subject: Subject::Task { task: 8 } } = request {
            assert!(task_watch.replace(*watch).is_none());
        }
    }
    let watch = task_watch.unwrap();
    assert!(domain.conversation.stream(
        RunText {
            attempt: 1,
            began: None,
            turn: ProvisionalTurnName { retry: 0, attempt: 1, number: 1 },
            from: 0,
            through: 3,
            text: objects::text(b"one"),
            cut: false,
            call: None,
        },
        32
    ));
    let mut row = super::confirmation::task();
    row.context = None;
    row.chip.executor = Executor::Agent { charter: 1 };
    row.activity = 0;
    if let Some(phase) = phase {
        let mut details = super::chip_facts::details();
        details.project = row.project;
        details.executor = OwnerExecutor::Agent { charter: 1 };
        if let Some(before) = prior {
            details.hold = Some(OwnerChipHold {
                source: None,
                reason: HoldCategory::Budget,
                revision: u64::MAX,
                before,
                escalation: None,
                tries: OwnerFailureTries { transient: 1, permanent: 2, run: 3, agent: 4, lost: 5, invalid: 6 },
                hold_wait_since: None,
            });
        }
        row.chip.phase = phase.compact();
        row.chip.details = Box::new(ChipPhase::Owner { phase, details: Box::new(details) });
    }
    drive(&mut domain, Time::ZERO, Event::Watch { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(row))) });
    assert_eq!(domain.conversation.live.is_empty(), domain.conversation.run_finished);
    domain.conversation.run_finished
}

#[test]
fn owner_task_context_absence_finishes_provisional_runs_only_from_supplied_lifecycle_stages() {
    assert!(!finished(None, None));
    for (stage, ended) in [
        (RunStage::Idle, true),
        (RunStage::Due, true),
        (RunStage::Preparing, true),
        (RunStage::Claimed { attempt: u64::MAX }, false),
        (RunStage::Running { attempt: u64::MAX }, false),
        (RunStage::BackingOff { until: Wall::EPOCH }, true),
    ] {
        assert_eq!(finished(Some(OwnerChipPhase::Active(stage)), None), ended);
        assert_eq!(
            finished(
                Some(OwnerChipPhase::Held { reason: HoldCategory::Budget, revision: u64::MAX }),
                Some(OwnerPriorPhase::Active(stage))
            ),
            ended
        );
    }
    for (stage, ended) in [
        (ClosingStage::Run { attempt: u64::MAX }, false),
        (ClosingStage::Delegates, true),
        (ClosingStage::Effects, true),
        (ClosingStage::Releases, true),
        (ClosingStage::Settled, true),
    ] {
        assert_eq!(finished(Some(OwnerChipPhase::Closing(stage)), None), ended);
        assert_eq!(
            finished(
                Some(OwnerChipPhase::Held { reason: HoldCategory::Budget, revision: u64::MAX }),
                Some(OwnerPriorPhase::Closing(stage))
            ),
            ended
        );
    }
    assert!(!finished(Some(OwnerChipPhase::Held { reason: HoldCategory::Budget, revision: u64::MAX }), None));
    assert!(finished(
        Some(OwnerChipPhase::Held { reason: HoldCategory::Budget, revision: u64::MAX }),
        Some(OwnerPriorPhase::Waiting)
    ));
    assert!(finished(Some(OwnerChipPhase::Ended), None));
}
