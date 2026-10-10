#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect public trees and checked language data")]
use crate::{
    Builder, JigBindings, Limits, Tree, owner_chip as view,
    owner_chip_words::{self as data, ChipPhrase, OwnerChipWords},
    phrases::{Invalid, PhraseLimits, Segment},
    words::Words,
};
use jig_client::{Domain, contract::*};
use skein_lib::{Duration, List, Token, Wall, bytes::copy_of};
fn doc(bytes: &[u8], cut: bool) -> DocumentText {
    DocumentText { bytes: Text::new(copy_of(bytes), 256).unwrap(), cut }
}
fn part() -> OwnerPart {
    let mut path = List::with_capacity(2);
    path.push(copy_of(b"a/b")).unwrap();
    path.push(copy_of(&[255, 0])).unwrap();
    OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Resource, key: OwnerPartKey::Path(path) }
}
fn details() -> OwnerChipDetails {
    OwnerChipDetails {
        project: u32::MAX,
        executor: OwnerExecutor::Agent { charter: u32::MAX },
        phase_since: Wall::from_nanos(10),
        run_since: Some(Wall::from_nanos(20)),
        attempt: Some(u64::MAX),
        retry: Some(RetryNotice::Refused { count: u32::MAX }),
        wake: OwnerWakePolicy {
            words: WakeRule::Never,
            notices: WakeRule::Immediate,
            news: WakeRule::Batch { count: u32::MAX, age: Duration::from_nanos(30) },
            results: ResultsWake::LastOrFailure,
            questions: true,
            answers: false,
            timers: true,
        },
        person_wait: None,
        hold: None,
        budget: OwnerBudget {
            numbers: OwnerNumbers { budget: 100, spent: 2, spent_below: 3, reserved: 5 },
            left: 7,
            funder: Funder::Task(u64::MAX),
            deadline: None,
        },
        activity: u64::MAX,
        resources: List::with_capacity(0),
        resources_cut: false,
        resources_observed: false,
        procedure: None,
        live_delegates: u32::MAX,
        closing_connectors: None,
        ending: None,
        ended_at: None,
        observation: None,
    }
}
fn hold() -> OwnerChipHold {
    OwnerChipHold {
        source: None,
        reason: HoldCategory::Budget,
        revision: u64::MAX,
        before: OwnerPriorPhase::Waiting,
        escalation: None,
        tries: OwnerFailureTries { transient: 1, permanent: 2, run: 3, agent: 4, lost: 5, invalid: u32::MAX },
        hold_wait_since: Some(Wall::from_nanos(40)),
    }
}
fn client_limits() -> jig_client::Limits {
    jig_client::Limits { rows: 4, text: 256, objects: 4, ..super::cards::CLIENT }
}
fn phrase_limits() -> PhraseLimits {
    PhraseLimits {
        languages: 2,
        phrases: crate::owner_words::phrase_count(),
        holes: 4,
        segments: 5000,
        text_bytes: 100_000,
        hole_bytes: 256,
    }
}
fn words() -> &'static Words {
    static WORDS: std::sync::OnceLock<Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| {
        let mut language = crate::phrases::english::language();
        language.wall = crate::phrases::WallFormat::Nanos;
        language.span_unit = crate::phrases::SpanUnit::Nanos;
        Words {
            book: crate::phrases::Phrasebook::load(
                crate::phrases::jig::shapes(256),
                Box::new([language]),
                &phrase_limits(),
            )
            .unwrap(),
            language: 0,
            offset_minutes: 0,
        }
    })
}
fn build(value: &OwnerChipDetails, words: &Words) -> (Tree, JigBindings, view::ChipFragment) {
    let client = Domain::new(&client_limits(), 1);
    let f = view::required(value, &client_limits(), words).unwrap();
    let capacity = Limits {
        nodes: f.nodes,
        depth: f.depth,
        text_per_node: f.text_per_node,
        text_bytes: f.text_bytes,
        drawing_bytes: 0,
        styles_per_node: 2,
        patches: 0,
        bindings: f.bindings,
        slots: f.slots,
        markdown_depth: 2,
    };
    let mut b = Builder::new(&capacity);
    let mut bindings = JigBindings::new(f.bindings, 256);
    view::render(&mut b, value, &client, words, &mut bindings);
    let tree = b.finish();
    assert!(tree.len() <= f.nodes);
    assert!(tree.nodes().iter().all(|node| node.text_bytes() <= f.text_per_node));
    (tree, bindings, f)
}
fn has(tree: &Tree, bytes: &[u8]) -> bool {
    tree.nodes().iter().any(|node| node.text.iter().any(|run| run.text.as_ref() == bytes))
}
fn contains(tree: &Tree, bytes: &[u8]) -> bool {
    tree.nodes().iter().any(|node| node.text.iter().any(|run| run.text.windows(bytes.len()).any(|part| part == bytes)))
}
#[test]
fn retained_chip_preserves_concrete_executor_project_and_full_width_owner_scalars() {
    let mut value = details();
    for (executor, label) in [
        (OwnerExecutor::Agent { charter: u32::MAX }, b"Agent charter 4294967295".as_slice()),
        (
            OwnerExecutor::Procedure { connector: u16::MAX, code: u32::MAX },
            b"Procedure connector 65535, code 4294967295",
        ),
        (OwnerExecutor::Person(PersonAddress::Party(u64::MAX)), b"Person 18446744073709551615"),
        (OwnerExecutor::Person(PersonAddress::Role(u32::MAX)), b"Project role 4294967295"),
    ] {
        value.executor = executor;
        let (tree, _, _) = build(&value, words());
        assert_eq!(value.clone().executor, executor);
        assert!(has(&tree, label), "concrete executor identity is never inferred from phase");
        for label in [
            b"Project: 4294967295".as_slice(),
            b"Attempt: 18446744073709551615",
            b"Activity through: 18446744073709551615",
            b"Live delegates: 4294967295",
        ] {
            assert!(has(&tree, label));
        }
    }
}
#[test]
fn placement_and_attempt_do_not_attest_unobserved_work_or_actual_start() {
    let mut value = details();
    let (tree, _, _) = build(&value, words());
    assert!(has(&tree, b"Placed or adopted: 20"));
    assert!(has(&tree, b"Run work unobserved"));
    assert!(!contains(&tree, b"Actual run start:"));
    assert!(!contains(&tree, b"Reported work"));
    let observation = OwnerChipObservation {
        channel: Token::new(u64::MAX),
        work: OwnerChipWork::Tool { call: copy_of(&[255, 0]), tool: copy_of(b"opaque/tool") },
        since: Wall::from_nanos(50),
        reported_at: Wall::from_nanos(60),
        started: None,
    };
    value.observation = Some(observation.clone());
    let (tree, _, _) = build(&value, words());
    assert_eq!(value.observation, Some(observation));
    for label in [
        b"\\xFF\\x00".as_slice(),
        b"opaque/tool",
        b"Current work since: 50",
        b"Work reported at: 60",
        b"Actual run start unavailable",
    ] {
        assert!(has(&tree, label));
    }
    value.observation.as_mut().unwrap().started = Some(Wall::from_nanos(70));
    assert!(has(&build(&value, words()).0, b"Actual run start: 70"));
    for (work, label) in [
        (OwnerChipWork::Preparing, b"Reported work preparing".as_slice()),
        (OwnerChipWork::Starting, b"Reported work starting"),
        (OwnerChipWork::Active, b"Reported work active"),
        (OwnerChipWork::Writing, b"Reported work writing"),
        (OwnerChipWork::Waiting, b"Reported work waiting"),
        (OwnerChipWork::Ending, b"Reported work ending"),
    ] {
        value.observation.as_mut().unwrap().work = work;
        assert!(has(&build(&value, words()).0, label));
    }
}
#[test]
fn closing_connector_absence_empty_and_terminal_clock_are_independent_owner_facts() {
    let mut value = details();
    assert!(has(&build(&value, words()).0, b"Closing connectors unavailable"));
    value.closing_connectors = Some(List::with_capacity(2));
    assert!(has(&build(&value, words()).0, b"No closing connectors"));
    value.closing_connectors.as_mut().unwrap().push(u16::MAX).unwrap();
    let (tree, _, _) = build(&value, words());
    assert!(has(&tree, b"Closing connector: 65535"));
    assert!(!contains(&tree, b"effects: "));
    value.ending = Some(OwnerChipEnding::Failed(doc(b"failed", false)));
    assert!(!contains(&build(&value, words()).0, b"Ended at:"));
    value.ended_at = Some(Wall::from_nanos(80));
    value.ending = None;
    let tree = build(&value, words()).0;
    assert!(has(&tree, b"Ended at: 80"));
    assert!(!has(&tree, b"Failed with reason"));
}
#[test]
fn resource_rows_keep_raw_segmented_parts_and_independent_holder_writer_loss_observation() {
    let mut value = details();
    let reference = part();
    value.resources = List::with_capacity(2);
    value
        .resources
        .push(OwnerResourceLine {
            part: reference.clone(),
            kind: u16::MAX,
            taken: false,
            holder: None,
            writer: Some(OwnerResourceWriter::Effect { entry: u64::MAX }),
            writer_lost: true,
        })
        .unwrap();
    value.resources_cut = true;
    let (tree, bindings, _) = build(&value, words());
    for label in [
        b"a/b".as_slice(),
        b"\\xFF\\x00",
        b"Resource kind: 65535",
        b"Resource not taken",
        b"Writer effect: 18446744073709551615",
        b"Writer lost",
        b"Resource list incomplete",
        b"Resource holder and writer facts unavailable",
    ] {
        assert!(has(&tree, label));
    }
    assert!(!contains(&tree, b"Holder task:"));
    assert!(!contains(&tree, b"Resource is free"));
    for node in tree.nodes() {
        if let Some(slot) = node.slot {
            assert_eq!(bindings.owner_part(slot.token), Some(&reference));
        }
    }
    let row = value.resources.get_mut(0).unwrap();
    row.holder = Some(u64::MAX);
    row.writer = Some(OwnerResourceWriter::Run { task: u64::MAX, attempt: u64::MAX });
    row.taken = true;
    row.writer_lost = false;
    value.resources_observed = true;
    value.resources_cut = false;
    let tree = build(&value, words()).0;
    for label in [
        b"Holder task: 18446744073709551615".as_slice(),
        b"Writer task: 18446744073709551615",
        b"Writer attempt: 18446744073709551615",
        b"Resource taken",
        b"Writer not lost",
        b"Resource holder and writer facts observed",
    ] {
        assert!(has(&tree, label));
    }
    assert!(!has(&tree, b"Resource list incomplete"));
}
#[test]
fn all_hold_categories_and_six_failure_classes_retain_only_supplied_payloads_and_counters() {
    let mut value = details();
    value.hold = Some(hold());
    let reasons = [
        (HoldCategory::Stalled, b"Held stalled".as_slice()),
        (HoldCategory::Dependency { task: u64::MAX }, b"Held dependency task: 18446744073709551615"),
        (HoldCategory::Stopped, b"Held stopped"),
        (HoldCategory::StoppedBy { party: u64::MAX }, b"Held stopped by party: 18446744073709551615"),
        (HoldCategory::Drift, b"Held drift"),
        (HoldCategory::Effects, b"Held effects"),
        (HoldCategory::EffectFailed, b"Held effect failed"),
        (HoldCategory::Uncertain { entry: u64::MAX }, b"Held uncertain entry: 18446744073709551615"),
        (HoldCategory::Budget, b"Held budget"),
        (HoldCategory::Deadline, b"Held deadline"),
        (HoldCategory::HoldsWaited, b"Held waited holds"),
        (HoldCategory::ResourceUnavailable, b"Held resource unavailable"),
        (HoldCategory::ResourceAccess, b"Held resource access"),
        (HoldCategory::PoolLost { pool: u64::MAX }, b"Held lost pool: 18446744073709551615"),
        (HoldCategory::Procedure, b"Held procedure"),
    ];
    for (reason, label) in reasons {
        value.hold.as_mut().unwrap().reason = reason;
        let tree = build(&value, words()).0;
        assert!(has(&tree, label));
        for label in [
            b"Hold revision: 18446744073709551615".as_slice(),
            b"Hold wait began: 40",
            b"Transient tries: 1",
            b"Permanent tries: 2",
            b"Run tries: 3",
            b"Agent tries: 4",
            b"Lost tries: 5",
            b"Invalid tries: 4294967295",
        ] {
            assert!(has(&tree, label));
        }
        assert!(!contains(&tree, b"Previous attempt"));
        assert!(!contains(&tree, b"Escalation waiting since:"));
    }
    for (class, label) in [
        (HoldFailure::Transient, b"Transient failure".as_slice()),
        (HoldFailure::Permanent, b"Permanent failure"),
        (HoldFailure::Run, b"Run failure"),
        (HoldFailure::Agent, b"Agent failure"),
        (HoldFailure::Lost, b"Lost failure"),
        (HoldFailure::Invalid, b"Invalid failure"),
    ] {
        value.hold.as_mut().unwrap().reason = HoldCategory::Failures { class };
        let tree = build(&value, words()).0;
        assert!(has(&tree, label));
        assert!(has(&tree, b"Failure tries exhausted"));
    }
}
#[test]
fn held_prior_stages_have_no_invented_clocks_and_escalations_keep_holder_scope_and_local_loss() {
    let mut value = details();
    value.hold = Some(hold());
    value.run_since = None;
    value.attempt = None;
    let priors = [
        (OwnerPriorPhase::Waiting, b"Prior stage waiting".as_slice()),
        (OwnerPriorPhase::Active(RunStage::Idle), b"Prior stage idle"),
        (OwnerPriorPhase::Active(RunStage::Due), b"Prior stage due"),
        (OwnerPriorPhase::Active(RunStage::Preparing), b"Prior stage preparing"),
        (
            OwnerPriorPhase::Active(RunStage::Claimed { attempt: u64::MAX }),
            b"Prior stage claimed attempt: 18446744073709551615",
        ),
        (
            OwnerPriorPhase::Active(RunStage::Running { attempt: u64::MAX }),
            b"Prior stage running attempt: 18446744073709551615",
        ),
        (OwnerPriorPhase::Active(RunStage::BackingOff { until: Wall::from_nanos(90) }), b"Prior retry at: 90"),
        (
            OwnerPriorPhase::Closing(ClosingStage::Run { attempt: u64::MAX }),
            b"Prior closing run attempt: 18446744073709551615",
        ),
        (OwnerPriorPhase::Closing(ClosingStage::Delegates), b"Prior closing delegates"),
        (OwnerPriorPhase::Closing(ClosingStage::Effects), b"Prior closing effects"),
        (OwnerPriorPhase::Closing(ClosingStage::Releases), b"Prior closing releases"),
        (OwnerPriorPhase::Closing(ClosingStage::Settled), b"Prior closing settled"),
    ];
    for (prior, label) in priors {
        value.hold.as_mut().unwrap().before = prior;
        let tree = build(&value, words()).0;
        assert!(has(&tree, label));
        assert!(!contains(&tree, b"Placed or adopted:"));
        assert!(!contains(&tree, b"Actual run start:"));
    }
    value.hold.as_mut().unwrap().escalation = Some(OwnerHoldEscalation::Routing);
    assert!(has(&build(&value, words()).0, b"Escalation routing"));
    for (holder, label) in [
        (OwnerEscalationHolder::Task(u64::MAX), b"Escalation holder task: 18446744073709551615".as_slice()),
        (OwnerEscalationHolder::Person(u64::MAX), b"Escalation holder person: 18446744073709551615"),
        (OwnerEscalationHolder::Role { project: u32::MAX, role: u32::MAX }, b"Escalation holder role: 4294967295"),
    ] {
        value.hold.as_mut().unwrap().escalation =
            Some(OwnerHoldEscalation::Waiting { holder, entry: u64::MAX, since: Wall::from_nanos(100) });
        let tree = build(&value, words()).0;
        assert!(has(&tree, label));
        assert!(has(&tree, b"Escalation entry: 18446744073709551615"));
        assert!(has(&tree, b"Escalation waiting since: 100"));
        if matches!(holder, OwnerEscalationHolder::Role { .. }) {
            assert!(has(&tree, b"Escalation holder project: 4294967295"));
        }
    }
    value.hold.as_mut().unwrap().escalation =
        Some(OwnerHoldEscalation::Rejected { by: u64::MAX, reason: doc(&[255], true) });
    let tree = build(&value, words()).0;
    assert!(has(&tree, b"Left held by: 18446744073709551615"));
    assert!(has(&tree, b"\\xFF"));
    assert!(has(&tree, b"Supplied words incomplete"));
}
#[test]
fn person_wait_notices_keep_full_message_activity_and_independent_owner_times() {
    let mut value = details();
    value.person_wait = Some(OwnerPersonWait {
        since: Wall::from_nanos(110),
        stall: Duration::from_nanos(120),
        deadline: Wall::from_nanos(130),
        notice: Some(OwnerPersonStallNotice { message: u64::MAX, at: Wall::from_nanos(140), activity: u64::MAX }),
    });
    let tree = build(&value, words()).0;
    for label in [
        b"Person wait began: 110".as_slice(),
        b"Person stall span: 120",
        b"Person stall deadline: 130",
        b"Stall notice message: 18446744073709551615",
        b"Stall notice at: 140",
        b"Stall notice activity: 18446744073709551615",
    ] {
        assert!(has(&tree, label));
    }
}
#[test]
fn complete_chip_endings_keep_result_requester_and_each_independent_reason_loss() {
    let mut value = details();
    value.ending = Some(OwnerChipEnding::Done(ResultValue::Report(doc(b"whole result", true))));
    let tree = build(&value, words()).0;
    assert!(has(&tree, b"whole result"));
    assert!(has(&tree, b"Completed with result"));
    assert!(has(&tree, b"Only part of this text was supplied"));
    for (by, label) in [
        (OwnerRequester::Task(u64::MAX), b"Cancellation requester task: 18446744073709551615".as_slice()),
        (OwnerRequester::Person(u64::MAX), b"Cancellation requester person: 18446744073709551615"),
        (OwnerRequester::Deployment { project: u32::MAX }, b"Cancellation requester deployment project: 4294967295"),
    ] {
        value.ending = Some(OwnerChipEnding::Cancelled {
            reason: doc(b"reason", true),
            result: Some(ResultValue::Words(doc(b"prior result", false))),
            by,
            at: Wall::from_nanos(150),
        });
        let tree = build(&value, words()).0;
        assert!(has(&tree, label));
        assert!(has(&tree, b"Cancelled at: 150"));
        assert!(has(&tree, b"prior result"));
        assert!(has(&tree, b"Supplied words incomplete"));
        assert!(!contains(&tree, b"Ended at:"));
    }
    value.ending = Some(OwnerChipEnding::Failed(doc(b"terminal failure", false)));
    let tree = build(&value, words()).0;
    assert!(has(&tree, b"terminal failure"));
    assert!(!has(&tree, b"Supplied words incomplete"));
}
#[test]
fn maximum_chip_fragment_checks_selected_language_storage_raw_slots_loss_and_arithmetic_bounds() {
    let mut value = details();
    value.hold = Some(hold());
    value.hold.as_mut().unwrap().source = Some(part());
    value.procedure = Some(part());
    value.resources = List::with_capacity(4);
    for _ in 0..4 {
        value
            .resources
            .push(OwnerResourceLine {
                part: part(),
                kind: u16::MAX,
                taken: true,
                holder: Some(u64::MAX),
                writer: Some(OwnerResourceWriter::Run { task: u64::MAX, attempt: u64::MAX }),
                writer_lost: true,
            })
            .unwrap();
    }
    value.resources_cut = true;
    value.closing_connectors = Some(List::with_capacity(4));
    for _ in 0..4 {
        value.closing_connectors.as_mut().unwrap().push(u16::MAX).unwrap();
    }
    value.hold.as_mut().unwrap().escalation =
        Some(OwnerHoldEscalation::Rejected { by: u64::MAX, reason: doc(&[255; 256], true) });
    value.observation = Some(OwnerChipObservation {
        channel: Token::new(u64::MAX),
        work: OwnerChipWork::Tool { call: copy_of(&[255; 256]), tool: copy_of(&[254; 256]) },
        since: Wall::from_nanos(160),
        reported_at: Wall::from_nanos(170),
        started: Some(Wall::from_nanos(180)),
    });
    value.ending = Some(OwnerChipEnding::Cancelled {
        reason: doc(&[253; 256], true),
        result: Some(ResultValue::Change { kind: u16::MAX, part: part(), words: doc(b"change", true) }),
        by: OwnerRequester::Deployment { project: u32::MAX },
        at: Wall::from_nanos(190),
    });
    value.ended_at = Some(Wall::from_nanos(200));
    let (tree, bindings, envelope) = build(&value, words());
    assert!(
        tree.nodes().iter().map(|node| u64::from(node.text_bytes())).sum::<u64>() <= u64::from(envelope.text_bytes)
    );
    assert_eq!(tree.nodes().iter().filter(|node| node.slot.is_some()).count(), 7);
    for node in tree.nodes() {
        if let Some(slot) = node.slot {
            assert!(bindings.owner_part(slot.token).is_some());
        }
    }
    assert!(view::required(&value, &jig_client::Limits { text: u32::MAX, ..client_limits() }, words()).is_none());
    assert!(view::required(&value, &jig_client::Limits { rows: u32::MAX, ..client_limits() }, words()).is_none());
    let mut translated = crate::phrases::english::language();
    for phrase in &mut translated.phrases {
        for form in &mut phrase.forms {
            for segment in &mut form.segments {
                if let Segment::Text(bytes) = segment {
                    *bytes = copy_of(b"selected ");
                }
            }
        }
    }
    let selected =
        OwnerChipWords::load(Box::new([crate::phrases::english::language(), translated]), &phrase_limits(), 1, 0)
            .unwrap();
    let selected_tree = build(&details(), selected.words()).0;
    assert!(has(&selected_tree, b"selected 4294967295"));
    assert!(!has(&selected_tree, b"Run work unobserved"));
    let mut language = data::language();
    language.phrases[0].id = ChipPhrase::PhaseSince.id();
    assert!(matches!(
        crate::phrases::Phrasebook::load(data::shapes(), Box::new([language]), &phrase_limits()),
        Err(Invalid::Duplicate { .. })
    ));
    assert!(matches!(
        OwnerChipWords::load(Box::new([crate::phrases::english::language()]), &phrase_limits(), 1, 0),
        Err(Invalid::Language)
    ));
}
