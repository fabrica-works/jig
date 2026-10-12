use super::{
    objects,
    spine::{LIMITS, drive, env},
};
use crate::{
    Event,
    contract::*,
    objects::{Body, Card, Left},
};
use skein_lib::{Queue, Time, Wall};

#[test]
fn terminal_task_reads_keep_whole_owner_endings_through_linger_without_result_documents() {
    for ending in [
        OwnerChipEnding::Done(ResultValue::Approved { state: Box::from([255_u8, 0]), words: None }),
        OwnerChipEnding::Failed(DocumentText { bytes: objects::text(b""), cut: true }),
        OwnerChipEnding::Cancelled {
            reason: DocumentText { bytes: objects::text(b"cancelled"), cut: true },
            result: Some(ResultValue::Approved {
                state: Box::from([255_u8, 0]),
                words: Some(DocumentText::complete(objects::text(b""))),
            }),
            by: OwnerRequester::Deployment { project: u32::MAX },
            at: Wall::from_nanos(u64::MAX),
        },
    ] {
        for read_before_drop in [true, false] {
            let (mut domain, page) = objects::live();
            let key = ObjectKey::Task(8);
            let first = objects::watched(&mut domain, page, &[key]);
            let initial = objects::read_token(&first);
            let mut task = super::confirmation::task();
            if !read_before_drop {
                drive(
                    &mut domain,
                    Time::ZERO,
                    Event::Read { read: initial, result: ReadResult::Value(ReadValue::Task(Box::new(task.clone()))) },
                );
                domain.reads.reclaim();
            }
            let dropped = objects::watched(&mut domain, page, &[]);
            let terminal = if read_before_drop {
                assert!(dropped.is_empty());
                initial
            } else {
                objects::read_token(&dropped)
            };
            task.result = Some(TaskResult::Ending(Box::new(ending.clone())));
            let exact = task.clone();
            let at = Time::from_nanos(30);
            let output = drive(
                &mut domain,
                at,
                Event::Read { read: terminal, result: ReadResult::Value(ReadValue::Task(Box::new(task))) },
            );
            assert!(output.is_empty());
            let id = domain.objects.find(key).unwrap();
            let object = domain.objects.get(id).unwrap();
            assert_eq!(
                object.card,
                Card::Leaving {
                    how: Left::OwnerEnded { how: Box::new(ending.clone()) },
                    until: Time::from_nanos(5_000_000_030),
                }
            );
            assert_eq!(object.body, Some(Body::Task(Box::new(exact))));
            assert_eq!(object.sources.len(), 0);
            let duplicate =
                drive(&mut domain, at, Event::Read { read: terminal, result: ReadResult::None(ReadNone::Gone) });
            assert!(duplicate.is_empty());
            assert_eq!(
                domain.objects.get(id).unwrap().card,
                Card::Leaving {
                    how: Left::OwnerEnded { how: Box::new(ending.clone()) },
                    until: Time::from_nanos(5_000_000_030),
                }
            );
            let mut out = Queue::with_capacity(crate::max_out(&LIMITS));
            for _ in 0_u8..4 {
                crate::fire(&mut domain, &env(Time::from_nanos(5_000_000_030)), &mut out);
            }
            assert!(domain.objects.find(key).is_none());
        }
    }
}

#[test]
fn supplied_result_documents_keep_their_own_terminal_facts_and_absent_task_result_is_not_inferred() {
    let mut task = super::confirmation::task();
    task.chip.phase = Phase::Ended;
    assert_eq!(Body::Task(Box::new(task.clone())).terminal(), None);
    let result = objects::result();
    task.result = Some(TaskResult::Document(Box::new(result.clone())));
    let body = Body::Task(Box::new(task));
    assert_eq!(body.ending(), Some(&result.ending));
    assert_eq!(body.terminal(), Some(Left::Ended { how: result.ending }));
}
