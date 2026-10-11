#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary public proofs inspect complete accessible trees, raw slots and decoded actions"
)]
use crate::{
    Builder, JigBindings, Role, Tree, owner_project as view,
    phrases::{PhraseLimits, Phrasebook, Segment, SpanUnit, WallFormat},
    words::Words,
};
use jig_client::{Domain, contract::*};
use skein_lib::{
    Duration, List, Wall,
    bytes::{copy_of, find},
};
fn list<T>(cap: u32, rows: impl IntoIterator<Item = T>) -> List<T> {
    let mut out = List::with_capacity(cap);
    for row in rows {
        assert!(out.push(row).is_ok());
    }
    out
}
fn limits() -> jig_client::Limits {
    jig_client::Limits { rows: 4, text: 32, ..super::cards::CLIENT }
}
fn doc(bytes: &[u8], cut: bool) -> DocumentText {
    DocumentText { bytes: Text::new(copy_of(bytes), 32).unwrap(), cut }
}
fn pattern() -> OwnerPattern {
    OwnerPattern {
        segments: list(4, [copy_of(&[255, 0, b'/']), copy_of(b"a/b"), copy_of(b"")]),
        last: OwnerPatternEnd::Open(copy_of(&[254, 0, b'/'])),
    }
}
fn part() -> OwnerPart {
    OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Outbox, key: OwnerPartKey::Path(pattern().segments) }
}
fn authority(cut: bool) -> OwnerAuthorityDocument {
    OwnerAuthorityDocument {
        authority: OwnerAuthority {
            tools: u64::MAX,
            grants: list(4, [OwnerGrant { connector: u16::MAX, kind: u16::MAX, pattern: pattern() }]),
            delegation: OwnerDelegation {
                kinds: list(
                    4,
                    [
                        OwnerAuthorityExecutor::Charter(u32::MAX),
                        OwnerAuthorityExecutor::Procedure(u32::MAX),
                        OwnerAuthorityExecutor::Role(u32::MAX),
                    ],
                ),
                tasks: u32::MAX,
                depth: u32::MAX,
            },
            spend: u64::MAX,
            deadline: Some(Wall::EPOCH),
            notes: u8::MAX,
            note_resources: list(4, [OwnerResourceScope { connector: u16::MAX, pattern: pattern() }]),
        },
        cut,
    }
}
fn numbers() -> OwnerNumbers {
    OwnerNumbers { budget: 101, spent: 103, spent_below: 107, reserved: 109 }
}
fn ledger(source: Funder) -> PoolLedger {
    PoolLedger {
        source,
        parent: Some(Funder::Task(u64::MAX)),
        numbers: numbers(),
        left: 113,
        closed: true,
        began: Some(Wall::EPOCH),
        next: None,
    }
}
fn chip(number: u64) -> Chip {
    Chip {
        task: number,
        title: doc(b"source chip", true),
        executor: Executor::Role { project: u32::MAX, role: u32::MAX },
        phase: Phase::Preparing,
        details: Box::new(ChipPhase::Owner {
            phase: OwnerChipPhase::Active(RunStage::Preparing),
            details: Box::new(super::owner_chip::details()),
        }),
    }
}
fn template(number: u64) -> ProposedTask {
    ProposedTask {
        number,
        spec: SpecificationDocument {
            specification: DocumentSpecification {
                words: doc(b"full template source", true),
                parameters: list(4, [ParameterDocument::Bytes { name: u32::MAX, value: doc(&[255, 0, 254], true) }]),
                inputs: list(4, [u64::MAX, 0]),
            },
            cut: true,
        },
        executor: OwnerExecutor::Person(PersonAddress::Role(u32::MAX)),
        contract: TaskContractDocument::Approval { state: copy_of(&[255, 0, 254]), part: part() },
        authority: authority(true),
        budget: u64::MAX,
        funder: Funder::Recurring { project: u32::MAX, task: u64::MAX, period: 0 },
        dependencies: list(4, [u64::MAX, 0]),
        resources: list(4, [part()]),
        wake: OwnerWakePolicy {
            words: WakeRule::Immediate,
            notices: WakeRule::Never,
            news: WakeRule::Never,
            results: ResultsWake::Each,
            questions: true,
            answers: true,
            timers: true,
        },
        priority: Some(u32::MAX),
        cut: true,
    }
}
fn source() -> OwnerProject {
    let funders = [
        Funder::Task(u64::MAX),
        Funder::Pool { project: u32::MAX, person: u64::MAX, period: 0 },
        Funder::Recurring { project: 0, task: u64::MAX, period: u64::MAX },
        Funder::Period { project: u32::MAX, period: u64::MAX },
    ];
    OwnerProject {
        number: u32::MAX,
        acts: List::with_capacity(4),
        resources: list(
            4,
            [
                OwnerProjectResourceRole::Owned,
                OwnerProjectResourceRole::Participant,
                OwnerProjectResourceRole::Unavailable,
                OwnerProjectResourceRole::Context,
            ]
            .map(|role| OwnerProjectResource { part: part(), role }),
        ),
        policy: OwnerProjectPolicy {
            ceiling: authority(true),
            period_spend: u64::MAX,
            escalation_role: None,
            projections: list(
                4,
                [
                    OwnerGrant { connector: u16::MAX, kind: u16::MAX, pattern: pattern() },
                    OwnerGrant { connector: 0, kind: 0, pattern: pattern() },
                ],
            ),
            roles: list(
                4,
                [OwnerProjectPolicyRole {
                    number: u32::MAX,
                    authority: authority(true),
                    period_spend: u64::MAX,
                    requests: u16::MAX,
                    decides: u8::MAX,
                }],
            ),
            requirements: list(
                4,
                [
                    PolicyRequirement {
                        connector: u16::MAX,
                        kind: u16::MAX,
                        pattern: pattern(),
                        judge: PolicyJudge { connector: u16::MAX, requirement: u16::MAX, parameters: u32::MAX },
                        guard: PolicyGuard::Observed { freshness: Duration::from_nanos(u64::MAX) },
                        must_be_guarded: true,
                    },
                    PolicyRequirement {
                        connector: 0,
                        kind: 0,
                        pattern: pattern(),
                        judge: PolicyJudge { connector: 0, requirement: 0, parameters: 0 },
                        guard: PolicyGuard::Guarded,
                        must_be_guarded: false,
                    },
                ],
            ),
            permissions: None,
            permissions_cut: false,
            cut: true,
        },
        members: list(
            4,
            [
                OwnerProjectMember {
                    party: u64::MAX,
                    kind: PartyKind::Service,
                    role: HeldRole::Policy { role: u32::MAX },
                    login: doc(&[255, 0, 254], true),
                    name: doc(b"", false),
                },
                OwnerProjectMember {
                    party: 0,
                    kind: PartyKind::Person,
                    role: HeldRole::Owner,
                    login: doc(b"", false),
                    name: doc(&[254, 0, 255], true),
                },
            ],
        ),
        current_period: None,
        pools: list(4, funders.map(ledger)),
        configured_period: 0,
        configured_pools: List::with_capacity(4),
        configured_pools_cut: false,
        spend: list(
            4,
            funders.into_iter().enumerate().map(|(index, funder)| OwnerProjectSpend {
                chip: chip([9, 0, u64::MAX, 9][index]),
                funder,
                numbers: numbers(),
            }),
        ),
        recurring: list(
            4,
            [Recurring {
                chip: chip(u64::MAX),
                key: u32::MAX,
                batch: list(4, [template(u64::MAX), template(0)]),
                budget: u64::MAX,
                overlap: RecurringOverlap::Wait,
                current_period: 0,
                began: Some(Wall::EPOCH),
                next: None,
                last_period: u64::MAX,
                pending_period: Some(0),
                last_batch: list(4, [u64::MAX, 0, u64::MAX]),
                cut: true,
            }],
        ),
        cut: true,
    }
}
fn make_words(selected: bool) -> Words {
    let mut first = crate::phrases::english::language();
    first.wall = WallFormat::Nanos;
    first.span_unit = SpanUnit::Nanos;
    let mut second = first.clone();
    for p in &mut second.phrases {
        for f in &mut p.forms {
            for s in &mut f.segments {
                if let Segment::Text(bytes) = s {
                    *bytes = copy_of(b"second ");
                }
            }
        }
    }
    Words {
        book: Phrasebook::load(
            crate::phrases::jig::shapes(32),
            Box::new([first, second]),
            &PhraseLimits {
                languages: 2,
                phrases: crate::owner_words::phrase_count(),
                holes: 4,
                segments: 10000,
                text_bytes: 200000,
                hole_bytes: 32,
            },
        )
        .unwrap(),
        language: u16::from(selected),
        offset_minutes: 0,
    }
}
fn words(selected: bool) -> &'static Words {
    static NORMAL: std::sync::OnceLock<Words> = std::sync::OnceLock::new();
    static SELECTED: std::sync::OnceLock<Words> = std::sync::OnceLock::new();
    if selected { SELECTED.get_or_init(|| make_words(true)) } else { NORMAL.get_or_init(|| make_words(false)) }
}
fn capacity(f: view::ProjectFragment) -> crate::Limits {
    crate::Limits {
        nodes: f.nodes,
        depth: f.depth,
        text_per_node: f.text_per_node,
        text_bytes: f.text_bytes,
        drawing_bytes: 0,
        styles_per_node: 2,
        patches: 0,
        bindings: f.bindings,
        slots: f.slots,
        markdown_depth: 4,
    }
}
fn build(value: &OwnerProject, w: &Words) -> (Tree, JigBindings) {
    let f = view::required(value, &limits(), w).expect("whole finite admission");
    let mut b = Builder::new(&capacity(f));
    let mut bindings = JigBindings::new(f.bindings, 32);
    let d = Domain::new(&limits(), 9123);
    view::render(&mut b, value, &d, w, &mut bindings);
    let tree = b.finish();
    super::cards_access::check(&tree);
    assert!(tree.len() <= f.nodes);
    let bytes: u64 = tree
        .nodes()
        .iter()
        .map(|n| {
            assert!(n.text_bytes() <= f.text_per_node);
            u64::from(n.text_bytes())
        })
        .sum();
    assert!(bytes <= u64::from(f.text_bytes));
    (tree, bindings)
}
fn has(tree: &Tree, bytes: &[u8]) -> bool {
    tree.nodes().iter().any(|n| n.text.iter().any(|r| find(&r.text, bytes).is_some()))
}
#[test]
fn owner_project_public_whole_view_keeps_four_resource_roles_opaque_raw_slots_members_and_independent_losses() {
    let value = source();
    let before = value.clone();
    let (tree, bindings) = build(&value, words(false));
    for label in [
        b"Owned".as_slice(),
        b"Participant",
        b"Unavailable",
        b"Context",
        b"Service",
        b"Person",
        b"Project document is incomplete",
        b"Policy is incomplete",
        b"Permission mappings not supplied",
        b"Escalation role not supplied",
    ] {
        assert!(has(&tree, label), "missing {label:?}");
    }
    let raw: Vec<_> = tree.nodes().iter().filter_map(|n| n.slot.and_then(|s| bindings.owner_part(s.token))).collect();
    assert!(raw.len() >= 4);
    for actual in raw {
        assert_eq!(actual, &part());
    }
    assert!(
        tree.nodes()
            .iter()
            .any(|n| n.role == Role::Region && n.name.as_deref() == Some(b"Member: 18446744073709551615"))
    );
    assert_eq!(value, before);
}
#[test]
fn owner_project_optional_ledger_distinguishes_none_supplied_empty_epoch_and_all_four_funding_counters() {
    let mut value = source();
    let (absent, _) = build(&value, words(false));
    assert!(has(&absent, b"Current period not supplied"));
    value.current_period = Some(ledger(Funder::Period { project: 0, period: 0 }));
    let (tree, _) = build(&value, words(false));
    assert!(!has(&tree, b"Current period not supplied"));
    for n in [b"101".as_slice(), b"103", b"107", b"109", b"113", b"18446744073709551615"] {
        assert!(has(&tree, n));
    }
    let supplied = value.current_period.as_ref().unwrap();
    assert_eq!(supplied.began, Some(Wall::EPOCH));
    let mut empty = supplied.clone();
    empty.numbers = OwnerNumbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 };
    empty.left = 0;
    value.current_period = Some(empty);
    let (empty_tree, _) = build(&value, words(false));
    assert!(!has(&empty_tree, b"Current period not supplied"));
    assert_eq!(value.current_period.as_ref().unwrap().numbers.budget, 0);
    assert_eq!(value.current_period.as_ref().unwrap().began, Some(Wall::EPOCH));
    assert_eq!(value.current_period.as_ref().unwrap().next, None);
    assert!(value.current_period.as_ref().unwrap().closed);
    for source in [
        Funder::Task(u64::MAX),
        Funder::Pool { project: u32::MAX, person: u64::MAX, period: 0 },
        Funder::Recurring { project: 0, task: u64::MAX, period: u64::MAX },
        Funder::Period { project: u32::MAX, period: u64::MAX },
    ] {
        assert!(value.pools.iter().any(|p| p.source == source));
    }
}
#[test]
fn owner_project_raw_role_permissions_requirement_guards_and_spend_recurring_source_order_are_public_facts() {
    let value = source();
    let (tree, _) = build(&value, words(false));
    for n in [
        b"65535".as_slice(),
        b"255",
        b"4294967295",
        b"18446744073709551615",
        b"full template source",
        b"Some recurring template facts were omitted",
    ] {
        assert!(has(&tree, n), "missing {n:?}");
    }
    let (_, bindings) = build(&value, words(false));
    let displayed: Vec<_> = tree
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| node.role == Role::Region && node.name.as_deref() == Some(b"Spend".as_slice()))
        .map(|(index, node)| {
            tree.nodes()[index..index + usize::try_from(node.size).unwrap()]
                .iter()
                .find_map(|child| child.binding.and_then(|binding| bindings.address(binding.token)))
                .expect("source expense has its named task link")
        })
        .map(|address| match address {
            jig_client::address::Address::Task { task, .. } => task,
            _ => panic!("expense task link"),
        })
        .collect();
    assert_eq!(displayed, [9, 0, u64::MAX, 9]);
    assert_eq!(value.spend.iter().map(|s| s.chip.task).collect::<Vec<_>>(), [9, 0, u64::MAX, 9]);
    assert_eq!(value.recurring.get(0).unwrap().batch.iter().map(|t| t.number).collect::<Vec<_>>(), [u64::MAX, 0]);
    let region_name = words(false)
        .say(crate::phrases::jig::JigPhrase::RecurringKey, &[crate::phrases::Hole::Number(u64::from(u32::MAX))]);
    let (index, recurring) = tree
        .nodes()
        .iter()
        .enumerate()
        .find(|(_, node)| node.role == Role::Region && node.name.as_deref() == Some(region_name.as_ref()))
        .unwrap();
    let last_links: Vec<_> = tree.nodes()[index..index + usize::try_from(recurring.size).unwrap()]
        .iter()
        .filter(|node| {
            node.role == Role::Link
                && node.name.as_deref().is_some_and(|name| {
                    [0, u64::MAX].into_iter().any(|task| {
                        name == words(false)
                            .say(crate::phrases::jig::JigPhrase::TaskPage, &[crate::phrases::Hole::Number(task)])
                            .as_ref()
                    })
                })
        })
        .filter_map(|node| node.binding.and_then(|binding| bindings.address(binding.token)))
        .map(|address| match address {
            jig_client::address::Address::Task { task, .. } => task,
            _ => panic!("recurring task link"),
        })
        .collect();
    assert_eq!(last_links, [u64::MAX, 0, u64::MAX]);
    assert_eq!(value.recurring.get(0).unwrap().last_batch.as_slice(), [u64::MAX, 0, u64::MAX]);
    assert_eq!(value.resources.capacity(), 4);
    assert_eq!(value.recurring.get(0).unwrap().batch.capacity(), 4);
    assert_eq!(
        value.policy.requirements.get(0).unwrap().guard,
        PolicyGuard::Observed { freshness: Duration::from_nanos(u64::MAX) }
    );
}
#[test]
fn owner_project_public_required_envelope_checks_last_nested_capacity_and_both_loaded_languages_without_clipping() {
    let value = source();
    let before = value.clone();
    for selected in [false, true] {
        let w = words(selected);
        let f = view::required(&value, &limits(), w).unwrap();
        let (tree, _) = build(&value, w);
        assert!(tree.len() <= f.nodes);
        assert!(crate::jig_bindings_worst_case(f.bindings, &limits()).is_some());
        assert_eq!(has(&tree, b"second "), selected);
    }
    let w = words(false);
    let mut small = limits();
    small.rows = 3;
    assert!(view::required(&value, &small, w).is_none());
    let mut changed = value.clone();
    changed.recurring.get_mut(0).unwrap().batch.get_mut(1).unwrap().resources = List::with_capacity(5);
    assert!(view::required(&changed, &limits(), w).is_none());
    let mut changed = value.clone();
    changed.members.get_mut(1).unwrap().name = doc(&[255; 32], true);
    assert!(view::required(&changed, &limits(), w).is_some());
    let mut small = limits();
    small.text = 31;
    assert!(view::required(&changed, &small, w).is_none());
    assert_eq!(value, before);
}
fn native_drive(domain: &mut Domain, event: jig_client::Event) -> Vec<jig_client::Request> {
    let mut out = skein_lib::Queue::with_capacity(jig_client::max_out(&limits()));
    jig_client::step(
        domain,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: Wall::EPOCH, limits: limits() },
        event,
        &mut out,
    );
    let mut result = Vec::new();
    while let Some(value) = out.pop() {
        result.push(value);
    }
    result
}
#[test]
fn owner_project_real_page_decodes_whole_role_edits_preserves_prior_channels_and_reviews_before_after_once() {
    use jig_client::{
        Event, Request,
        action::Act,
        address::{Address, ProjectSection},
    };
    let mut d = Domain::new(&limits(), 9199);
    let opened = native_drive(
        &mut d,
        Event::Start {
            address: Address::Project { project: 4, section: ProjectSection::Settings },
            kept: List::with_capacity(0),
            offset: jig_client::boundary::Offset { minutes: 0 },
        },
    );
    native_drive(&mut d, Event::Hello { versions: VersionRange { first: 1, last: 1 } });
    let f = opened
        .iter()
        .find_map(|r| match r {
            Request::Open { watch, subject: Subject::Frame } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let ready = native_drive(
        &mut d,
        Event::Watch { watch: f, event: WatchEvent::Delivered(Delivery::Frame(Box::new(super::cards::document()))) },
    );
    let read = ready
        .iter()
        .find_map(|r| match r {
            Request::Read { read, what: Read::Project { project: 4 } } => Some(*read),
            _ => None,
        })
        .unwrap();
    let mut value = source();
    value.number = 4;
    value.cut = false;
    value.policy.cut = false;
    value.policy.ceiling.cut = false;
    value.policy.roles.get_mut(0).unwrap().authority.cut = false;
    value.acts.push(Offered { act: ActKind::ChangePolicy, admission: Admission::Allowed }).unwrap();
    native_drive(
        &mut d,
        Event::Read { read, result: ReadResult::Value(ReadValue::OwnerProject(Box::new(value.clone()))) },
    );
    let w = words(false);
    let f = view::worst_case(&limits(), w).unwrap();
    let mut bounds = capacity(f);
    bounds.nodes += 32;
    bounds.text_bytes += bounds.text_per_node * 32;
    let page = |d: &Domain| {
        let mut b = Builder::new(&bounds);
        let mut bindings = JigBindings::new(f.bindings, 32);
        crate::pages::project::project(&mut b, d, w, &mut bindings);
        let tree = b.finish();
        super::cards_access::check(&tree);
        (tree, bindings)
    };
    let (tree, mut bindings) = page(&d);
    let textbox = tree
        .nodes()
        .iter()
        .find(|n| n.role == Role::TextField && n.name.as_deref() == Some(b"Request permission bits 65535".as_slice()))
        .or_else(|| {
            tree.nodes().iter().find(|n| {
                n.role == Role::TextField && n.name.as_ref().is_some_and(|name| find(name, b"65535").is_some())
            })
        })
        .expect("whole requests textbox")
        .binding
        .unwrap()
        .token;
    let act = crate::decode(&mut bindings, textbox, crate::Act::Input { value: copy_of(b"32833") }).unwrap();
    assert!(native_drive(&mut d, Event::Act { act }).is_empty());
    let (tree, mut bindings) = page(&d);
    let field = tree
        .nodes()
        .iter()
        .find(|n| n.role == Role::TextField && n.name.as_ref().is_some_and(|name| find(name, b"255").is_some()))
        .expect("decides textbox")
        .binding
        .unwrap()
        .token;
    let act = crate::decode(&mut bindings, field, crate::Act::Input { value: copy_of(b"129") }).unwrap();
    native_drive(&mut d, Event::Act { act });
    let (tree, mut bindings) = page(&d);
    assert!(has(&tree, b"Before"));
    assert!(has(&tree, b"After"));
    assert!(has(&tree, b"32833"));
    assert!(has(&tree, b"129"));
    let confirm = tree
        .nodes()
        .iter()
        .find(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Confirm".as_slice()))
        .unwrap()
        .binding
        .unwrap()
        .token;
    let act = crate::decode(&mut bindings, confirm, crate::Act::Press).unwrap();
    assert_eq!(act, Act::Confirm);
    let sent = native_drive(&mut d, Event::Act { act });
    let asks: Vec<_> = sent
        .iter()
        .filter_map(|r| match r {
            Request::Send { ask, .. } => Some(ask.as_ref()),
            _ => None,
        })
        .collect();
    assert_eq!(asks.len(), 1);
    let Ask::ChangePolicy { project, edit: PolicyEdit::Role(role) } = asks[0] else { panic!("one whole role") };
    assert_eq!(*project, 4);
    assert_eq!(role.requests, 32833);
    assert_eq!(role.decides, 129);
    assert_eq!(role.authority, value.policy.roles.get(0).unwrap().authority.authority);
    assert_eq!(role.period_spend, u64::MAX);
    assert!(native_drive(&mut d, Event::Act { act: Act::Confirm }).is_empty());
}
#[test]
fn owner_project_public_baselines_preserve_optional_permissions_configured_targets_and_empty_recurring_titles() {
    let mut value = source();
    value.configured_period = 0;
    value.configured_pools.push(ledger(Funder::Pool { project: u32::MAX, person: 0, period: 0 })).unwrap();
    value.policy.permissions = Some(list(
        4,
        [
            PolicyPermission { connector: u16::MAX, permission: u16::MAX, role: u32::MAX },
            PolicyPermission { connector: 0, permission: 0, role: 0 },
        ],
    ));
    value.policy.permissions_cut = true;
    value.recurring.get_mut(0).unwrap().chip.title = doc(b"", true);
    let before = value.clone();
    let (tree, _) = build(&value, words(false));
    assert!(has(&tree, b"Configured pool period: 0"));
    assert!(!has(&tree, b"Permission mappings not supplied"));
    assert!(has(&tree, b"65535"));
    assert!(has(&tree, b"4294967295"));
    assert!(
        tree.nodes().iter().any(|node| node.role == Role::Region
            && node.name.as_ref().is_some_and(|name| find(name, b"4294967295").is_some()))
    );
    assert_eq!(value, before);
    value.policy.permissions = Some(List::with_capacity(4));
    let (tree, _) = build(&value, words(false));
    assert!(!has(&tree, b"Permission mappings not supplied"));
    value.policy.permissions = None;
    let (tree, _) = build(&value, words(false));
    assert!(has(&tree, b"Permission mappings not supplied"));
    value.configured_pools = List::with_capacity(5);
    assert!(view::required(&value, &limits(), words(false)).is_none());
    value.configured_pools = List::with_capacity(4);
    value.policy.permissions = Some(List::with_capacity(5));
    assert!(view::required(&value, &limits(), words(false)).is_none());
}
#[test]
fn owner_project_public_whole_tree_accepts_exact_observed_nodes_and_text_and_refuses_one_under_without_clipping() {
    let value = source();
    let w = words(false);
    let (tree, _) = build(&value, w);
    let f = view::required(&value, &limits(), w).unwrap();
    let mut bound = capacity(f);
    bound.nodes = tree.len();
    bound.text_bytes = tree.nodes().iter().map(|node| node.text_bytes()).sum();
    let render = |bound: crate::Limits| {
        let mut b = Builder::new(&bound);
        let mut bindings = JigBindings::new(f.bindings, 32);
        view::render(&mut b, &value, &Domain::new(&limits(), 9151), w, &mut bindings);
        b.finish()
    };
    let exact = render(bound);
    assert_eq!(exact.len(), tree.len());
    assert_eq!(exact.nodes(), tree.nodes());
    let mut short = bound;
    short.nodes -= 1;
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| render(short))).is_err());
    let mut short = bound;
    short.text_bytes -= 1;
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| render(short))).is_err());
    assert_eq!(value.resources.capacity(), 4);
}
fn native_owner(value: OwnerProject) -> Domain {
    native_owner_at(value, jig_client::address::ProjectSection::Settings)
}
fn native_owner_at(value: OwnerProject, section: jig_client::address::ProjectSection) -> Domain {
    use jig_client::{Event, Request, address::Address};
    let mut domain = Domain::new(&limits(), 9177);
    let out = native_drive(
        &mut domain,
        Event::Start {
            address: Address::Project { project: value.number, section },
            kept: List::with_capacity(0),
            offset: jig_client::boundary::Offset { minutes: 0 },
        },
    );
    native_drive(&mut domain, Event::Hello { versions: VersionRange { first: 1, last: 1 } });
    let watch = out
        .iter()
        .find_map(|request| match request {
            Request::Open { watch, subject: Subject::Frame } => Some(*watch),
            _ => None,
        })
        .unwrap();
    let out = native_drive(
        &mut domain,
        Event::Watch { watch, event: WatchEvent::Delivered(Delivery::Frame(Box::new(super::cards::document()))) },
    );
    let read = out
        .iter()
        .find_map(|request| match request {
            Request::Read { read, what: Read::Project { .. } } => Some(*read),
            _ => None,
        })
        .unwrap();
    native_drive(
        &mut domain,
        Event::Read { read, result: ReadResult::Value(ReadValue::OwnerProject(Box::new(value))) },
    );
    domain
}
fn owner_page(domain: &Domain) -> (Tree, JigBindings) {
    let w = words(false);
    let f = view::worst_case(&limits(), w).unwrap();
    let mut bound = capacity(f);
    bound.nodes += 32;
    bound.text_bytes += 32 * bound.text_per_node;
    let mut b = Builder::new(&bound);
    let mut bindings = JigBindings::new(f.bindings, 32);
    crate::pages::project::project(&mut b, domain, w, &mut bindings);
    let tree = b.finish();
    super::cards_access::check(&tree);
    (tree, bindings)
}
#[test]
fn owner_project_public_service_and_pool_inputs_decode_explicit_values_and_preserve_complete_membership_choices() {
    use jig_client::{Event, Request, action::Act};
    let mut value = source();
    value.number = 4;
    value.cut = false;
    value.policy.cut = false;
    value.acts = list(
        4,
        [ActKind::MakeService, ActKind::SetPool, ActKind::SetRoles]
            .map(|act| Offered { act, admission: Admission::Allowed }),
    );
    let mut domain = native_owner(value.clone());
    let (tree, mut bindings) = owner_page(&domain);
    assert!(
        tree.nodes()
            .iter()
            .all(|node| node.role != Role::TextField || node.name.as_deref() != Some(b"Name".as_slice()))
    );
    let selected = tree
        .nodes()
        .iter()
        .filter(|node| node.role == Role::Button)
        .find_map(|node| {
            node.binding.and_then(|binding| match crate::decode(&mut bindings, binding.token, crate::Act::Press) {
                Some(act @ Act::ProjectServiceRole { role: HeldRole::Policy { role: u32::MAX } }) => Some(act),
                _ => None,
            })
        })
        .unwrap();
    assert!(native_drive(&mut domain, Event::Act { act: selected }).is_empty());
    let (tree, mut bindings) = owner_page(&domain);
    let name = tree
        .nodes()
        .iter()
        .find(|node| node.role == Role::TextField && node.name.as_deref() == Some(b"Name".as_slice()))
        .unwrap();
    assert!(name.value.is_none());
    let act =
        crate::decode(&mut bindings, name.binding.unwrap().token, crate::Act::Input { value: copy_of(b"") }).unwrap();
    let Act::ProjectSettings { ask } = &act else { panic!("explicit service request") };
    assert!(
        matches!(ask.as_ref(),Ask::MakeService{project:4,name,role:HeldRole::Policy{role:u32::MAX}} if name.bytes().is_empty())
    );
    native_drive(&mut domain, Event::Act { act });
    let sent = native_drive(&mut domain, Event::Act { act: Act::Confirm });
    assert!(
        matches!(&sent[..],[Request::Send{ask,..}] if matches!(ask.as_ref(),Ask::MakeService {project:4,role:HeldRole::Policy{role:u32::MAX},name} if name.bytes().is_empty()))
    );
    let mut domain = native_owner(value.clone());
    let (tree, mut bindings) = owner_page(&domain);
    let pool = tree
        .nodes()
        .iter()
        .filter(|node| node.role == Role::TextField)
        .find_map(|node| {
            node.binding.and_then(|binding| {
                match crate::decode(
                    &mut bindings,
                    binding.token,
                    crate::Act::Input { value: copy_of(b"18446744073709551615") },
                ) {
                    Some(Act::ProjectSettings { ask })
                        if matches!(ask.as_ref(), Ask::SetPool { project: 4, party: 0, budget: u64::MAX }) =>
                    {
                        Some((node, Act::ProjectSettings { ask }))
                    }
                    _ => None,
                }
            })
        })
        .unwrap();
    assert!(pool.0.value.is_none());
    assert!(has(&tree, b"No pool exists for this configured target"));
    native_drive(&mut domain, Event::Act { act: pool.1 });
    assert!(domain.project_page().settings.before.is_none());
    assert_eq!(domain.project_page().settings.pool_period, Some(0));
    let sent = native_drive(&mut domain, Event::Act { act: Act::Confirm });
    assert!(
        matches!(&sent[..],[Request::Send{ask,..}] if matches!(ask.as_ref(),Ask::SetPool{project:4,party:0,budget:u64::MAX}))
    );
    let domain = native_owner(value.clone());
    let (tree, mut bindings) = owner_page(&domain);
    let chosen=tree.nodes().iter().filter(|node|node.role==Role::Button).find_map(|node|node.binding.and_then(|binding|{
        match crate::decode(&mut bindings,binding.token,crate::Act::Press) {
            Some(Act::ProjectSettings{ask}) if matches!(ask.as_ref(),Ask::SetRoles{holdings,..} if holdings.len()==2 && holdings.get(0).unwrap().role==HeldRole::Observer)=>Some(ask),_=>None
        }
    })).unwrap();
    let Ask::SetRoles { project, holdings } = chosen.as_ref() else { panic!("whole membership request") };
    assert_eq!(*project, 4);
    assert_eq!(holdings.capacity(), value.members.capacity());
    assert_eq!(holdings.get(0).unwrap().party, u64::MAX);
    assert_eq!(holdings.get(1).unwrap().party, 0);
    assert_eq!(holdings.get(1).unwrap().role, HeldRole::Owner);
}

#[test]
fn owner_project_public_goal_form_decodes_whole_bounded_procedure_and_keeps_proposed_review_before_one_keyed_request() {
    use jig_client::{Event, Request, action::Act, address::ProjectSection, fields::Purpose};
    let mut value = source();
    value.number = 4;
    let executor = GoalExecutor::Procedure { connector: u16::MAX, code: u32::MAX };
    let recipient = Recipient::Roles { project: 4, roles: list(4, [u32::MAX, 0, u32::MAX]) };
    value.acts = list(
        4,
        [Offered {
            act: ActKind::SetGoal,
            admission: Admission::Bounded {
                within: Bounds::Goal {
                    pool_left: 17,
                    charters: list(4, [u32::MAX, 0, u32::MAX]),
                    procedures: list(4, [GoalProcedure { connector: u16::MAX, code: u32::MAX }]),
                    latest: None,
                    budget: 17,
                    maximum_budget: 23,
                    maximum_deadline: None,
                },
                beyond: Beyond::Proposed { to: recipient.clone() },
            },
        }],
    );
    let mut domain = native_owner_at(value.clone(), ProjectSection::Goals);
    assert!(domain.project_page().document.is_none());
    let (tree, mut bindings) = owner_page(&domain);
    let choice = tree
        .nodes()
        .iter()
        .filter_map(|node| {
            let act = crate::decode(&mut bindings, node.binding?.token, crate::Act::Press)?;
            match act {
                Act::GoalForm { form } if form.executor == executor => Some(form),
                _ => None,
            }
        })
        .next()
        .expect("supplied procedure choice");
    assert_eq!(choice.project, 4);
    assert_eq!(choice.budget, 17);
    assert!(native_drive(&mut domain, Event::Act { act: Act::GoalForm { form: choice } }).is_empty());
    let (tree, mut bindings) = owner_page(&domain);
    let words_field = tree.nodes().iter().find(|node| node.role == Role::TextArea).unwrap().binding.unwrap().token;
    let act =
        crate::decode(&mut bindings, words_field, crate::Act::Input { value: copy_of(&[255, 0, b'/', 254]) }).unwrap();
    assert!(matches!(&act, Act::Edit { field: Purpose::NewGoal, text } if text.bytes() == [255, 0, b'/', 254]));
    native_drive(&mut domain, Event::Act { act });
    for (field, amount) in [
        (crate::pages::project::goals::Field::Budget, b"23".as_slice()),
        (crate::pages::project::goals::Field::Priority, b"4294967295".as_slice()),
    ] {
        let (tree, mut bindings) = owner_page(&domain);
        let form = domain.project_page().form.form.unwrap();
        let edited = crate::pages::project::goals::edit(form, field, amount).unwrap();
        let decoded = tree
            .nodes()
            .iter()
            .filter_map(|node| {
                crate::decode(&mut bindings, node.binding?.token, crate::Act::Input { value: copy_of(amount) })
            })
            .find(|act| act == &edited)
            .unwrap();
        assert!(native_drive(&mut domain, Event::Act { act: decoded }).is_empty());
    }
    let (tree, mut bindings) = owner_page(&domain);
    let submit = tree
        .nodes()
        .iter()
        .filter_map(|node| crate::decode(&mut bindings, node.binding?.token, crate::Act::Press))
        .find(|act| matches!(act, Act::Submit { field: Purpose::NewGoal }))
        .unwrap();
    assert!(native_drive(&mut domain, Event::Act { act: submit }).is_empty());
    let review = domain.project_page().form.confirmation.as_ref().unwrap();
    assert_eq!(review.form.executor, executor);
    assert_eq!(review.form.budget, 23);
    assert_eq!(review.form.priority, u32::MAX);
    assert_eq!(review.form.deadline, None);
    assert_eq!(review.words.bytes(), &[255, 0, b'/', 254]);
    assert_eq!(review.pool_left, 17);
    assert!(review.proposed);
    assert_eq!(review.to.as_ref(), Some(&recipient));
    let sent = native_drive(&mut domain, Event::Act { act: Act::Confirm });
    let asks: Vec<_> = sent
        .iter()
        .filter_map(|row| match row {
            Request::Send { ask, .. } => Some(ask.as_ref()),
            _ => None,
        })
        .collect();
    assert_eq!(
        asks,
        [&Ask::SetGoal {
            project: 4,
            spec: Text::new(copy_of(&[255, 0, b'/', 254]), limits().text).unwrap(),
            executor,
            budget: 23,
            priority: u32::MAX,
            deadline: None
        }]
    );
    assert!(native_drive(&mut domain, Event::Act { act: Act::Confirm }).is_empty());
    assert_eq!(domain.project_page().owner.as_deref(), Some(&value));
}

#[test]
fn owner_project_public_offer_ceiling_preserves_twenty_five_in_order_and_rejects_reserved_twenty_six() {
    let w = words(false);
    let mut value = source();
    let original_requirement = view::required(&value, &limits(), w).unwrap();
    for populated in [0, 1, MAX_OFFERED_ACTS] {
        value.acts = List::with_capacity(MAX_OFFERED_ACTS);
        for index in 0..populated {
            value
                .acts
                .push(Offered {
                    act: if index % 2 == 0 { ActKind::SetPool } else { ActKind::ChangePolicy },
                    admission: Admission::Allowed,
                })
                .unwrap();
        }
        let original = value.clone();
        // Standalone facts and supported forms are already charged by their own envelope.
        assert_eq!(view::required(&value, &limits(), w), Some(original_requirement));
        let domain = native_owner(value.clone());
        assert_eq!(domain.project_page().owner.as_deref(), Some(&original));
        let (tree, _) = owner_page(&domain);
        assert!(!tree.is_empty());
        assert_eq!(value, original);
        value.acts = List::with_capacity(MAX_OFFERED_ACTS + 1);
        let oversized = value.clone();
        assert!(view::required(&value, &limits(), w).is_none());
        assert_eq!(value, oversized);
    }
    value.acts = List::with_capacity(MAX_OFFERED_ACTS);
    value
        .acts
        .push(Offered {
            act: ActKind::SetGoal,
            admission: Admission::Bounded {
                within: Bounds::Goal {
                    pool_left: 0,
                    charters: List::with_capacity(5),
                    procedures: List::with_capacity(4),
                    latest: None,
                    budget: 0,
                    maximum_budget: 0,
                    maximum_deadline: None,
                },
                beyond: Beyond::Proposed { to: Recipient::Roles { project: u32::MAX, roles: List::with_capacity(4) } },
            },
        })
        .unwrap();
    assert!(view::required(&value, &limits(), w).is_none());
    value.acts = List::with_capacity(MAX_OFFERED_ACTS);
    value.members = List::with_capacity(5);
    assert!(view::required(&value, &limits(), w).is_none());
}
