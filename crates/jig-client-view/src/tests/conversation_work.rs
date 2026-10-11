#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary view proofs drive public events and inspect selected semantic trees"
)]
use super::cards::{CLIENT, contains, text};
use crate::{Builder, Tree, owner_chip, owner_chip_words::ChipPhrase as P, tree::Node, words::Words};
use jig_client::{
    Domain, Request,
    address::{Address, TaskSection},
    contract::*,
};
use skein_lib::{Token, Wall, bytes::copy_of};

fn words() -> &'static Words {
    static WORDS: std::sync::OnceLock<Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| {
        let mut language = crate::phrases::english::language();
        language.wall = crate::phrases::WallFormat::Nanos;
        Words {
            book: crate::phrases::Phrasebook::load(
                crate::phrases::jig::shapes(CLIENT.text),
                Box::new([language]),
                &crate::phrases::PhraseLimits {
                    languages: 1,
                    phrases: crate::owner_words::phrase_count(),
                    holes: 4,
                    segments: 5000,
                    text_bytes: 100_000,
                    hole_bytes: CLIENT.text,
                },
            )
            .unwrap(),
            language: 0,
            offset_minutes: 0,
        }
    })
}
fn selected() -> &'static Words {
    static WORDS: std::sync::OnceLock<Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| {
        let mut language = crate::phrases::english::language();
        language.wall = crate::phrases::WallFormat::Nanos;
        for phrase in &mut language.phrases {
            if [P::Starting.id(), P::Writing.id(), P::Tool.id()].contains(&phrase.id) {
                for form in &mut phrase.forms {
                    for segment in &mut form.segments {
                        if let crate::phrases::Segment::Text(bytes) = segment {
                            *bytes = copy_of(b"Selected work report");
                        }
                    }
                }
            }
        }
        Words {
            book: crate::phrases::Phrasebook::load(
                crate::phrases::jig::shapes(CLIENT.text),
                Box::new([crate::phrases::english::language(), language]),
                &crate::phrases::PhraseLimits {
                    languages: 2,
                    phrases: crate::owner_words::phrase_count(),
                    holes: 4,
                    segments: 5000,
                    text_bytes: 100_000,
                    hole_bytes: CLIENT.text,
                },
            )
            .unwrap(),
            language: 1,
            offset_minutes: 0,
        }
    })
}
fn report(work: OwnerChipWork, started: Option<Wall>) -> OwnerChipObservation {
    OwnerChipObservation {
        channel: Token::new(u64::MAX),
        work,
        since: Wall::from_nanos(31),
        reported_at: Wall::from_nanos(47),
        started,
    }
}
fn task(observation: Option<OwnerChipObservation>, context: bool) -> Task {
    let mut task = super::cards_objects::task_doc(&[]);
    task.chip.executor = Executor::Agent { charter: 1 };
    let mut details = super::owner_chip::details();
    details.project = task.project;
    details.executor = OwnerExecutor::Agent { charter: 1 };
    details.observation = observation;
    task.chip.phase = Phase::Running;
    task.chip.details = Box::new(ChipPhase::Owner {
        phase: OwnerChipPhase::Active(RunStage::Running { attempt: u64::MAX }),
        details: Box::new(details),
    });
    task.hold = None;
    task.goal = None;
    task.delegates = super::work_pages::list([]);
    if context {
        task.context.as_mut().unwrap().runtime = Runtime::Running {
            attempt: u64::MAX,
            since: Wall::from_nanos(59),
            tool: Some(text(b"independent synthetic runtime")),
        };
    } else {
        task.context = None;
    }
    task
}
fn page(task: Task, section: TaskSection) -> (Domain, Vec<Request>) {
    let (mut domain, requests) = super::work_pages::entered(Address::Task { task: 8, section });
    let watch = requests
        .iter()
        .find_map(|request| match request {
            Request::Open { watch, subject: Subject::Task { task: 8 } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let requests = super::pages::deliver(&mut domain, watch, Delivery::Task(Box::new(task)));
    (domain, requests)
}
fn render(domain: &Domain, words: &Words) -> Tree {
    let (tree, _) = super::pages::build(domain, 3, words);
    super::cards_access::check(&tree);
    assert!(tree.len() <= crate::task_nodes(&CLIENT, 0).unwrap());
    tree
}
#[test]
fn owner_work_activity_without_synthetic_context_shows_all_seven_reports_and_independent_clocks() {
    for (work, label) in [
        (OwnerChipWork::Preparing, b"Reported work preparing".as_slice()),
        (OwnerChipWork::Starting, b"Reported work starting"),
        (OwnerChipWork::Active, b"Reported work active"),
        (OwnerChipWork::Writing, b"Reported work writing"),
        (OwnerChipWork::Tool { call: copy_of(&[255, 0]), tool: copy_of(b"literal/tool") }, b"Reported tool call"),
        (OwnerChipWork::Waiting, b"Reported work waiting"),
        (OwnerChipWork::Ending, b"Reported work ending"),
    ] {
        for started in [None, Some(Wall::EPOCH), Some(Wall::from_nanos(u64::MAX))] {
            let source = report(work.clone(), started);
            let doc = task(Some(source.clone()), false);
            assert!(doc.context.is_none());
            let (domain, _) = page(doc.clone(), TaskSection::Activity);
            let tree = render(&domain, words());
            assert!(contains(&tree, label));
            assert!(contains(&tree, b"Current work since: 31"));
            assert!(contains(&tree, b"Work reported at: 47"));
            assert_eq!(contains(&tree, b"Actual run start unavailable"), started.is_none());
            if let Some(at) = started {
                let caption = words().book.say(0, P::Started.id(), &[crate::phrases::Hole::Wall(at)]).unwrap();
                assert!(contains(&tree, &caption.text));
            }
            match work {
                OwnerChipWork::Tool { .. } => {
                    assert!(contains(&tree, b"\\xFF\\x00"));
                    assert!(contains(&tree, b"literal/tool"));
                }
                OwnerChipWork::Preparing
                | OwnerChipWork::Starting
                | OwnerChipWork::Active
                | OwnerChipWork::Writing
                | OwnerChipWork::Waiting
                | OwnerChipWork::Ending => {}
            }
            let id = domain.objects().find(ObjectKey::Task(8)).unwrap();
            let Some(jig_client::objects::Body::Task(retained)) = &domain.objects().get(id).unwrap().body else {
                panic!("whole task retained");
            };
            assert_eq!(retained.as_ref(), &doc);
        }
    }
}
#[test]
fn owner_work_absence_does_not_fabricate_report_and_preserves_independent_synthetic_runtime() {
    for context in [false, true] {
        for observation in [None, Some(report(OwnerChipWork::Writing, None))] {
            let present = observation.is_some();
            let (domain, _) = page(task(observation, context), TaskSection::Activity);
            let tree = render(&domain, words());
            assert_eq!(contains(&tree, b"Reported work writing"), present);
            assert_eq!(contains(&tree, b"Current work since: 31"), present);
            assert_eq!(contains(&tree, b"independent synthetic runtime"), context);
            assert!(!contains(&tree, b"Placed or adopted"));
            assert!(tree.nodes().iter().any(|node| node.role == crate::Role::Log));
        }
    }
}
#[test]
fn selected_owner_work_activity_preserves_conversation_and_opaque_provisional_text() {
    for work in [
        OwnerChipWork::Starting,
        OwnerChipWork::Writing,
        OwnerChipWork::Tool { call: copy_of(&[255, 0]), tool: copy_of(b"selected/tool") },
    ] {
        let (mut domain, requests) = page(task(Some(report(work, None)), false), TaskSection::Activity);
        let run = requests
            .iter()
            .find_map(|request| match request {
                Request::Open { watch, subject: Subject::Run { .. } } => Some(*watch),
                _ => None,
            })
            .unwrap();
        super::pages::deliver(
            &mut domain,
            run,
            Delivery::Run(Box::new(RunText {
                attempt: u64::MAX,
                began: None,
                turn: ProvisionalTurnName { retry: u32::MAX, attempt: u64::MAX, number: 1 },
                from: 0,
                through: 17,
                text: text(b"provisional reply"),
                cut: false,
                call: None,
            })),
        );
        let tree = render(&domain, selected());
        assert!(contains(&tree, b"Selected work report"));
        assert!(!contains(&tree, b"Reported work starting"));
        assert!(!contains(&tree, b"Reported work writing"));
        assert!(contains(&tree, b"Provisional"));
        assert!(contains(&tree, b"provisional reply"));
        assert!(tree.nodes().iter().any(|node| node.role == crate::Role::Log && node.name.is_some()));
    }
}
fn fragment(value: &OwnerChipObservation, capacity: &crate::Limits) -> Tree {
    let mut builder = Builder::new(capacity);
    owner_chip::observation(&mut builder, value, words());
    builder.finish()
}
#[test]
fn owner_work_fragment_checks_raw_limits_exact_six_leaves_bytes_and_one_under_storage() {
    let value = report(
        OwnerChipWork::Tool {
            call: std::iter::repeat_n(255, usize::try_from(CLIENT.text).unwrap()).collect(),
            tool: std::iter::repeat_n(254, usize::try_from(CLIENT.text).unwrap()).collect(),
        },
        Some(Wall::from_nanos(u64::MAX)),
    );
    let bound = owner_chip::observation_required(&value, &CLIENT, words()).unwrap();
    assert_eq!(bound.nodes, owner_chip::observation_nodes());
    assert_eq!(bound.nodes, 6);
    assert_eq!(bound.depth, 1);
    assert_eq!(bound.bindings, 0);
    assert_eq!(bound.slots, 0);
    let capacity = crate::Limits {
        nodes: bound.nodes,
        depth: bound.depth,
        text_per_node: bound.text_per_node,
        text_bytes: bound.text_bytes,
        drawing_bytes: 0,
        styles_per_node: 0,
        patches: 0,
        bindings: 0,
        slots: 0,
        markdown_depth: 1,
    };
    let tree = fragment(&value, &capacity);
    assert_eq!(tree.len(), bound.nodes);
    assert!(tree.nodes().iter().all(|node| node.binding.is_none() && node.slot.is_none()));
    let total = tree.nodes().iter().map(Node::text_bytes).sum::<u32>();
    let largest = tree.nodes().iter().map(Node::text_bytes).max().unwrap();
    assert!(bound.text_per_node >= largest.checked_mul(2).unwrap());
    assert!(bound.text_bytes >= total);
    let exact = crate::Limits { text_per_node: largest, text_bytes: total, ..capacity };
    assert_eq!(fragment(&value, &exact).nodes(), tree.nodes());
    for smaller in [
        crate::Limits { nodes: 5, ..exact },
        crate::Limits { text_per_node: largest - 1, ..exact },
        crate::Limits { text_bytes: total - 1, ..exact },
    ] {
        std::panic::catch_unwind(|| fragment(&value, &smaller)).unwrap_err();
    }
    assert!(
        owner_chip::observation_required(&value, &jig_client::Limits { text: CLIENT.text - 1, ..CLIENT }, words())
            .is_none()
    );
    assert!(
        owner_chip::observation_required(&value, &jig_client::Limits { text: u32::MAX, ..CLIENT }, words()).is_none()
    );
    assert!(crate::task_nodes(&jig_client::Limits { rows: u32::MAX, ..CLIENT }, 0).is_none());
    assert_eq!(crate::task_nodes(&CLIENT, 7).unwrap(), crate::task_nodes(&CLIENT, 0).unwrap() + 7);
}
#[test]
fn owner_work_overview_and_activity_keep_the_same_supplied_observation_without_full_metadata_in_activity() {
    let value = task(Some(report(OwnerChipWork::Writing, None)), false);
    for section in [TaskSection::Overview, TaskSection::Activity] {
        let (domain, _) = page(value.clone(), section);
        let tree = render(&domain, words());
        assert!(contains(&tree, b"Reported work writing"));
        assert_eq!(contains(&tree, b"Resource holder and writer facts unavailable"), section == TaskSection::Overview);
        assert_eq!(contains(&tree, b"Live delegates: 4294967295"), section == TaskSection::Overview);
    }
}
