//! Instruction amendments through the task owner's public entrance.
#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary Rust test parent"
)]
use super::revisions::{Harness, LIMITS, task, to};
use crate::{Amendment, Event, Executor, Key, Party, Refusal, Request, Revisions, Stored};
use alloc::{boxed::Box, vec, vec::Vec};

fn change(instructions: &[u8]) -> Amendment {
    Amendment {
        instructions: Some(instructions.into()),
        release: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        reason: b"new evidence".as_slice().into(),
    }
}
fn amend(h: &mut Harness, change: Amendment) -> Vec<Request> {
    h.send(Event::Amend {
        reply_to: to(),
        by: Party::Person(9),
        task: 1,
        message: h.domain.task(1).unwrap().last_message + 1,
        stop_run: false,
        amendment: change,
    })
}

#[test]
fn an_instruction_amendment_is_heard_whole_and_keeps_immutable_settings_across_restart() {
    let mut h = Harness::new();
    h.make(vec![task(1, &[])]);
    h.send(Event::Prepare { reply_to: to(), task: 1 });
    h.send(Event::Claim { reply_to: to(), task: 1, attempt: 1, resumed: false, budget: 10, writes: Box::new([]) });
    h.send(Event::Started { task: 1, attempt: 1 });
    let out = amend(&mut h, change(b"Use the new evidence"));
    assert!(out.iter().any(|r| matches!(r, Request::Relay { task: 1, attempt: 1, word, .. } if word.words.as_ref() == b"Use the new evidence")));
    assert!(!out.iter().any(|r| matches!(r, Request::Stop { .. })));
    assert_eq!(h.domain.task(1).unwrap().revisions, Revisions { spec: 1, hold: 0, person: 0 });
    let Stored::History(history) = &h.stored[&Key::History { task: 1, revision: 1 }] else {
        panic!("history");
    };
    assert_eq!(history.reason.as_ref(), b"new evidence");
    let settings = history.amendment.as_ref().unwrap();
    assert_eq!(settings.before.instructions, None);
    assert_eq!(settings.after.instructions.as_deref(), Some(b"Use the new evidence".as_slice()));
    let owned = crate::task_bytes(h.domain.task(1).unwrap()).unwrap();
    h.restart();
    assert_eq!(h.domain.task(1).unwrap().instructions.as_deref(), Some(b"Use the new evidence".as_slice()));
    assert_eq!(crate::task_bytes(h.domain.task(1).unwrap()), Some(owned));
    amend(&mut h, change(b"Use the new evidence"));
    assert_eq!(h.domain.task(1).unwrap().revisions.spec, 1, "equal replacement keeps semantic revision");
    amend(&mut h, change(b""));
    assert_eq!(h.domain.task(1).unwrap().instructions.as_deref(), Some(b"".as_slice()));
    assert_eq!(h.domain.task(1).unwrap().revisions.spec, 2);
    let Stored::History(history) = &h.stored[&Key::History { task: 1, revision: 3 }] else {
        panic!("history");
    };
    assert_eq!(
        history.amendment.as_ref().unwrap().before.instructions.as_deref(),
        Some(b"Use the new evidence".as_slice())
    );
    assert_eq!(history.amendment.as_ref().unwrap().after.instructions.as_deref(), Some(b"".as_slice()));
}

#[test]
fn unsupported_and_oversized_instruction_edits_leave_the_task_unchanged() {
    for executor in [Executor::Agent { charter: 1 }, Executor::Procedure { connector: 1, code: 1 }] {
        let mut h = Harness::new();
        let mut member = task(1, &[]);
        member.executor = executor;
        h.make(vec![member]);
        let before = h.domain.task(1).unwrap().clone();
        let edit = match executor {
            Executor::Agent { .. } => change(&vec![1; usize::try_from(LIMITS.message_bytes).unwrap() + 1]),
            Executor::Procedure { .. } | Executor::Person(_) => change(b"agent only"),
        };
        let out = amend(&mut h, edit);
        assert!(out.iter().any(|r| matches!(r, Request::Refused { problem, .. } if matches!(problem.why, Refusal::Read | Refusal::Executor))));
        assert_eq!(h.domain.task(1).unwrap(), &before);
        assert!(!out.iter().any(|r| matches!(r, Request::Save { .. } | Request::Relay { .. })));
        let mut corrupt = before;
        corrupt.instructions = Some(vec![1; usize::try_from(LIMITS.message_bytes).unwrap() + 1].into_boxed_slice());
        let mut fresh = Harness::new();
        let out = fresh.send(Event::Restore { record: Stored::Live(Box::new(corrupt)) });
        assert!(out.iter().any(|r| matches!(r, Request::RestoreRefused { .. })));
    }
}
