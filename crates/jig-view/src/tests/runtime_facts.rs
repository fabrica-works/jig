#![expect(
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary public event fixtures inspect supplied owner facts and trees"
)]
use super::{
    cards::{CLIENT, contains, limits, text, words},
    cards_objects::task_doc,
    pages::{build, deliver},
    work_pages::{entered, list},
};
use crate::JigBindings;
use jig_domain_client::{
    Domain, Request,
    address::{Address, TaskSection},
    contract::*,
};
use skein_lib::{Token, Wall};
use skein_ui::{Builder, Role, Tree};

fn active(stage: RunStage) -> ActiveRun {
    ActiveRun {
        stage,
        phase_since: Wall::from_nanos(7),
        attempt: Some(u64::MAX),
        placement: Some(Wall::from_nanos(13)),
        retry: None,
        observation: None,
    }
}
fn ready(runtime: Runtime, details: ChipPhase, phase: Phase) -> Domain {
    let (mut d, out) = entered(Address::Task { task: 8, section: TaskSection::Activity });
    let watch = out
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Task { .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let mut task = task_doc(&[]);
    task.chip.executor = Executor::Agent { charter: 2 };
    task.chip.phase = phase;
    *task.chip.details = details;
    task.context.as_mut().expect("supplied task context").runtime = runtime;
    task.hold = None;
    deliver(&mut d, watch, Delivery::Task(Box::new(task)));
    d
}
fn shown(value: ActiveRun) -> Tree {
    let d = ready(Runtime::Active(value.clone()), ChipPhase::Active(value), Phase::Running);
    build(&d, 3, words()).0
}
#[test]
fn unobserved_activation_preserves_phase_attempt_and_placement_without_inventing_work_or_start() {
    for stage in [
        RunStage::Idle,
        RunStage::Due,
        RunStage::Preparing,
        RunStage::Claimed { attempt: u64::MAX },
        RunStage::Running { attempt: u64::MAX },
        RunStage::BackingOff { until: Wall::from_nanos(19) },
    ] {
        let tree = shown(active(stage));
        for fact in [
            b"18446744073709551615".as_slice(),
            b"Phase since",
            b"Placement recorded at",
            b"Run observation not supplied",
        ] {
            assert!(contains(&tree, fact));
        }
        assert!(!contains(&tree, b"Observed work: writing"));
        assert!(!contains(&tree, b"Start observed at"));
        assert!(!contains(&tree, b"Preparing a host"));
    }
}
#[test]
fn observed_work_preserves_actual_optional_start_and_whole_tool_call_identity() {
    for (work, expected) in [
        (RunWork::Preparing, b"Observed work: preparing".as_slice()),
        (RunWork::Starting, b"Observed work: starting"),
        (RunWork::Active, b"Observed work: active"),
        (RunWork::Writing, b"Observed work: writing"),
        (RunWork::Waiting, b"Observed work: waiting"),
        (RunWork::Ending, b"Observed work: ending"),
        (
            RunWork::Tool { call: current_identity(b"actual-call/23"), tool: text(b"actual-tool") },
            b"Observed current tool call",
        ),
    ] {
        for started in [None, Some(Wall::EPOCH)] {
            let mut value = active(RunStage::Running { attempt: u64::MAX });
            value.observation = Some(RunObservation {
                channel: Token::new(u64::MAX),
                work: work.clone(),
                since: Wall::from_nanos(17),
                reported_at: Wall::from_nanos(23),
                started,
            });
            let tree = shown(value);
            assert!(contains(&tree, expected));
            if let RunWork::Tool { .. } = &work {
                assert!(contains(&tree, b"actual-tool"));
                assert!(contains(&tree, b"actual-call/23"));
            }
            assert!(contains(&tree, b"Observed work since"));
            assert!(contains(&tree, b"Observation received at"));
            assert_eq!(contains(&tree, b"Start observed at"), started.is_some());
            assert_eq!(contains(&tree, b"Start observation not supplied"), started.is_none());
            if work != RunWork::Writing {
                assert!(!contains(&tree, b"Observed work: writing"));
            }
        }
    }
}
#[test]
fn retry_notices_keep_refusal_and_failure_counters_without_fabricated_reasons() {
    for (retry, expected) in [
        (RetryNotice::Refused { count: u32::MAX }, b"Activation refusal count 4294967295".as_slice()),
        (RetryNotice::Failed { class: FailureClass::Stall, count: 7, allowance: 9 }, b"Failure count 7 of allowance 9"),
    ] {
        let mut value = active(RunStage::BackingOff { until: Wall::from_nanos(29) });
        value.retry = Some(retry);
        let tree = shown(value);
        assert!(contains(&tree, expected));
        assert!(!contains(&tree, b"Reason:"));
        assert!(!contains(&tree, b"Start observed at"));
        match retry {
            RetryNotice::Refused { .. } => assert!(!contains(&tree, b"Failure count")),
            RetryNotice::Failed { .. } | RetryNotice::OwnerFailed { .. } => {}
        }
    }
}
#[test]
fn held_prior_lifecycle_and_optional_closing_facts_keep_their_owner_meaning() {
    for stage in [
        None,
        Some(ClosingStage::Run { attempt: u64::MAX }),
        Some(ClosingStage::Delegates),
        Some(ClosingStage::Effects),
        Some(ClosingStage::Releases),
        Some(ClosingStage::Settled),
    ] {
        for connectors in [None, Some(list([])), Some(list([7, 11]))] {
            let closing = Closing {
                delegates: 2,
                connectors: connectors.clone(),
                stage,
                ending: Some(Ending::Cancelled { by: Actor::Party(17), reason: None, at: None }),
            };
            let was = Some(HeldRuntime::Closing(closing));
            let d = ready(
                Runtime::Held { was: was.clone() },
                ChipPhase::Held {
                    reason: HoldReason::Budget { spent: 2, limit: 2 },
                    since: Wall::EPOCH,
                    revision: Revision { value: 3 },
                    was,
                },
                Phase::Held,
            );
            let tree = build(&d, 3, words()).0;
            assert!(contains(&tree, b"Prior lifecycle before the hold"));
            assert!(contains(&tree, b"Pending ending"));
            assert!(contains(&tree, b"17"));
            assert_eq!(contains(&tree, b"Connector waits not supplied"), connectors.is_none());
            assert_eq!(
                contains(&tree, b"No connector waits"),
                connectors.as_ref().is_some_and(skein_lib::List::is_empty)
            );
            assert_eq!(contains(&tree, b"Waiting for effects to settle"), stage == Some(ClosingStage::Effects));
            if stage != Some(ClosingStage::Effects) {
                assert!(!contains(&tree, b"'s effects settling"));
            }
        }
    }
}
#[test]
fn full_runtime_tools_and_held_closing_fit_the_declared_tree_envelope() {
    let full = text(UTF8_TEXTS[0]);
    let mut run = active(RunStage::Running { attempt: u64::MAX });
    run.retry =
        Some(RetryNotice::Failed { class: FailureClass::Temporary, count: u32::MAX - 1, allowance: u32::MAX - 1 });
    run.observation = Some(RunObservation {
        channel: Token::new(u64::MAX),
        work: RunWork::Tool {
            call: CallIdentity { id: full.clone(), ..current_identity(b"call") },
            tool: full.clone(),
        },
        since: Wall::from_nanos(u64::MAX),
        reported_at: Wall::from_nanos(u64::MAX),
        started: Some(Wall::from_nanos(u64::MAX)),
    });
    let closing = Closing {
        delegates: u32::MAX,
        connectors: Some(list([7, 11])),
        stage: Some(ClosingStage::Effects),
        ending: Some(Ending::Decided {
            by: u64::MAX,
            party: Some(Party { number: u64::MAX, kind: PartyKind::Person, name: full.clone() }),
            how: Some(Decision::Rejected { reason: full.clone() }),
            at: Wall::from_nanos(u64::MAX),
        }),
    };
    let failure = AttemptFailure {
        attempt: u64::MAX,
        class: FailureClass::Permanent,
        reason: full,
        at: Wall::from_nanos(u64::MAX),
    };
    for details in [
        ChipPhase::Active(run.clone()),
        ChipPhase::Closing(closing.clone()),
        ChipPhase::Held {
            reason: HoldReason::Tries { failures: list([failure.clone(), failure.clone()]) },
            since: Wall::EPOCH,
            revision: Revision { value: u64::MAX },
            was: Some(HeldRuntime::Active(run)),
        },
        ChipPhase::Held {
            reason: HoldReason::Tries { failures: list([failure.clone(), failure]) },
            since: Wall::EPOCH,
            revision: Revision { value: u64::MAX },
            was: Some(HeldRuntime::Closing(closing)),
        },
    ] {
        let d = ready(Runtime::Ended, details.clone(), Phase::Held);
        let mut chip = task_doc(&[]).chip;
        *chip.details = details;
        let mut b = Builder::new(&limits());
        let mut bindings = JigBindings::new(64, CLIENT.text);
        b.open(Role::Page);
        crate::cards::chip::chip(&mut b, &d, &chip, words(), &mut bindings);
        b.close();
        let tree = b.finish();
        assert!(tree.len() <= crate::chip_nodes(&CLIENT).unwrap() + 1);
        super::cards_access::check(&tree);
    }
}

fn current_identity(id: &[u8]) -> CallIdentity {
    CallIdentity { owner: Token::new(u64::MAX), sequence: u32::MAX, retry: u32::MAX, id: text(id) }
}

const UTF8_TEXTS: [&[u8]; 1] = [&[b'x'; 128]];
