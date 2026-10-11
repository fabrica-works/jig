//! Independent complete Project fixtures and receiving-boundary proofs.
use super::*;
use jig_core::acts;
use skein_lib::{Duration, List, Wall};

pub(crate) fn limits() -> Limits {
    Limits { rows: 8, text: 200, document_bytes: 100_000, ..crate::test::limits().jig }
}

fn grant_source() -> documents::GrantLine {
    let mut segments = List::with_capacity(3);
    segments.push(Box::from([255, 0])).expect("literal segment");
    segments.push(Box::from([b'/'])).expect("literal slash segment");
    documents::GrantLine {
        connector: u16::MAX,
        kind: 0,
        segments,
        last: documents::PatternEnd::Open(Box::from([0, 255])),
    }
}

fn policy_source() -> documents::Policy {
    let state = owner::tests::amendment_source();
    let mut projections = List::with_capacity(4);
    projections.push(grant_source()).expect("projection");
    let mut roles = List::with_capacity(3);
    roles
        .push(documents::PolicyRole {
            number: u32::MAX,
            authority: state.authority.clone(),
            period_spend: 17,
            requests: authority::Requests(u16::MAX),
            decides: authority::Proposals(u8::MAX),
        })
        .expect("role");
    let mut requirements = List::with_capacity(4);
    for guard in [authority::Guard::Guarded, authority::Guard::Observed { freshness: Duration::ZERO }] {
        requirements
            .push(documents::Requirement {
                grant: grant_source(),
                judge: authority::Judge { connector: 0, requirement: u16::MAX, parameters: u32::MAX },
                guard,
                must_be_guarded: guard == authority::Guard::Guarded,
            })
            .expect("requirement");
    }
    let mut permissions = List::with_capacity(5);
    permissions.push(people::PermissionRole { connector: u16::MAX, permission: 0, role: u32::MAX }).expect("mapping");
    permissions.push(people::PermissionRole { connector: 0, permission: u16::MAX, role: 0 }).expect("mapping");
    documents::Policy {
        ceiling: state.authority,
        period_spend: u64::MAX,
        escalation_role: Some(0),
        projections,
        roles,
        requirements,
        permissions: Some(permissions),
        permissions_cut: true,
        cut: false,
    }
}

fn ledgers() -> List<documents::Ledger> {
    let funders = [
        tasks::Funder::Task(u64::MAX),
        tasks::Funder::Pool { project: u32::MAX, person: 0, period: u64::MAX },
        tasks::Funder::Recurring { project: 0, task: u64::MAX, period: 0 },
        tasks::Funder::Period { project: u32::MAX, period: 0 },
    ];
    let mut values = List::with_capacity(6);
    for (index, source) in funders.into_iter().enumerate() {
        values
            .push(documents::Ledger {
                source,
                parent: Some(funders[(index + 1) % funders.len()]),
                numbers: tasks::Numbers { budget: 0, spent: u64::MAX, spent_below: 17, reserved: 29 },
                left: 31,
                closed: index % 2 == 0,
                began: if index % 2 == 0 { None } else { Some(Wall::EPOCH) },
                next: if index % 2 == 0 { Some(Wall::from_nanos(u64::MAX)) } else { None },
            })
            .expect("ledger");
    }
    values
}

fn members() -> List<documents::Member> {
    let mut values = List::with_capacity(8);
    for role in [
        people::Role::Owner,
        people::Role::Maintainer,
        people::Role::Member,
        people::Role::Observer,
        people::Role::Policy { role: u32::MAX },
    ] {
        values
            .push(documents::Member {
                party: if role == (people::Role::Policy { role: u32::MAX }) { u64::MAX } else { 0 },
                kind: if role == people::Role::Member { people::Kind::Person } else { people::Kind::Service },
                role,
                login: documents::Text { bytes: Box::from([255, 0]), cut: true },
                name: documents::Text { bytes: Box::new([]), cut: false },
            })
            .expect("member");
    }
    values
}

fn recurring_source(overlap: tasks::RecurringOverlap) -> documents::Recurring {
    let state = owner::tests::amendment_source();
    let mut batch = List::with_capacity(7);
    batch
        .push(documents::ProposedTask {
            number: u64::MAX,
            executor: tasks::Executor::Person(tasks::PersonAddress::Role(u32::MAX)),
            spec: state.spec,
            contract: documents::ContractLine::Report { words: u32::MAX },
            authority: state.authority,
            budget: u64::MAX,
            funder: tasks::Funder::Recurring { project: u32::MAX, task: u64::MAX, period: 0 },
            dependencies: state.dependencies,
            resources: List::with_capacity(7),
            wake: state.wake,
            priority: Some(0),
            cut: true,
        })
        .expect("whole proposed task");
    let mut last_batch = List::with_capacity(6);
    last_batch.push(u64::MAX).expect("last batch");
    last_batch.push(0).expect("source order");
    documents::Recurring {
        chip: chips::tests::source_chip(),
        key: u32::MAX,
        batch,
        budget: 0,
        overlap,
        current_period: u64::MAX,
        began: Some(Wall::EPOCH),
        next: None,
        last_period: 17,
        pending_period: Some(0),
        last_batch,
        cut: true,
    }
}

pub(crate) fn source() -> documents::Project {
    let mut acts = List::with_capacity(client::MAX_OFFERED_ACTS);
    for act in [acts::Act::Release, acts::Act::Stop, acts::Act::Release] {
        acts.push(acts::Offered { act, admission: acts::Admission::Allowed }).expect("ordered offers");
    }
    let mut resources = List::with_capacity(7);
    for role in [
        documents::ResourceRole::Owned,
        documents::ResourceRole::Participant,
        documents::ResourceRole::Unavailable,
        documents::ResourceRole::Context,
    ] {
        resources
            .push(documents::ProjectResource {
                role,
                part: tasks::Part {
                    connector: u16::MAX,
                    kind: tasks::PartKind::Outbox,
                    key: tasks::PartKey::Path(grant_source().segments),
                },
            })
            .expect("adopted source resource");
    }
    let mut spend = List::with_capacity(5);
    spend
        .push(documents::SpendLine {
            chip: chips::tests::source_chip(),
            funder: tasks::Funder::Task(0),
            numbers: tasks::Numbers { budget: u64::MAX, spent: 0, spent_below: 7, reserved: 11 },
        })
        .expect("spend");
    let mut recurring = List::with_capacity(4);
    recurring.push(recurring_source(tasks::RecurringOverlap::Skip)).expect("skip");
    recurring.push(recurring_source(tasks::RecurringOverlap::Wait)).expect("wait");
    documents::Project {
        acts,
        number: u32::MAX,
        resources,
        policy: policy_source(),
        members: members(),
        current_period: Some(documents::Ledger {
            source: tasks::Funder::Period { project: u32::MAX, period: u64::MAX },
            parent: None,
            numbers: tasks::Numbers { budget: 2, spent: 3, spent_below: 5, reserved: 7 },
            left: 11,
            closed: false,
            began: None,
            next: Some(Wall::EPOCH),
        }),
        pools: ledgers(),
        configured_period: 0,
        configured_pools: ledgers(),
        configured_pools_cut: true,
        spend,
        recurring,
        cut: false,
    }
}

#[test]
fn whole_project_keeps_all_resource_roles_members_raw_text_and_independent_source_order() {
    let source = source();
    let original = source.clone();
    let actual = document(&source, &limits()).expect("whole source project");
    assert_eq!(actual.number, u32::MAX);
    assert_eq!((actual.resources.capacity(), actual.resources.len()), (7, 4));
    assert_eq!(
        actual.resources.iter().map(|value| value.role).collect::<Vec<_>>(),
        [
            client::OwnerProjectResourceRole::Owned,
            client::OwnerProjectResourceRole::Participant,
            client::OwnerProjectResourceRole::Unavailable,
            client::OwnerProjectResourceRole::Context,
        ]
    );
    for (actual, source) in actual.resources.iter().zip(source.resources.iter()) {
        assert_eq!(actual.part, owner::part(&source.part, &limits()).expect("whole raw resource"));
        let client::OwnerPartKey::Path(path) = &actual.part.key else { panic!("literal path") };
        assert_eq!(path.capacity(), 3);
        assert_eq!(path.get(0).expect("binary segment").bytes(), &[255, 0]);
        assert_eq!(path.get(1).expect("slash segment").bytes(), &[b'/']);
    }
    assert_eq!((actual.members.capacity(), actual.members.len()), (8, 5));
    assert_eq!(
        actual.members.iter().map(|value| value.role).collect::<Vec<_>>(),
        [
            client::HeldRole::Owner,
            client::HeldRole::Maintainer,
            client::HeldRole::Member,
            client::HeldRole::Observer,
            client::HeldRole::Policy { role: u32::MAX },
        ]
    );
    assert_eq!(actual.members.get(2).expect("person").kind, client::PartyKind::Person);
    assert_eq!(actual.members.get(4).expect("service").kind, client::PartyKind::Service);
    assert_eq!(actual.members.get(4).expect("party identity").party, u64::MAX);
    for member in actual.members.iter() {
        assert_eq!(member.login.bytes(), &[255, 0]);
        assert!(member.login.cut);
        assert!(member.name.bytes().is_empty());
        assert!(!member.name.cut);
    }
    assert!(!actual.cut);
    assert_eq!(source, original);
}

#[test]
fn project_policy_keeps_raw_bits_literal_patterns_both_guards_and_independent_permission_loss() {
    let source = source();
    let original = source.clone();
    let policy = document(&source, &limits()).expect("policy").policy;
    assert_eq!(policy.ceiling, owner::authority(&source.policy.ceiling, &limits()).expect("complete ceiling"));
    assert_eq!(policy.period_spend, u64::MAX);
    assert_eq!(policy.escalation_role, Some(0));
    assert_eq!((policy.projections.capacity(), policy.projections.len()), (4, 1));
    let projection = policy.projections.get(0).expect("projection");
    assert_eq!((projection.connector, projection.kind), (u16::MAX, 0));
    assert_eq!(projection.pattern.segments.capacity(), 3);
    assert_eq!(projection.pattern.segments.get(0).expect("binary").bytes(), &[255, 0]);
    assert!(matches!(&projection.pattern.last, client::OwnerPatternEnd::Open(value) if value.bytes() == [0, 255]));
    assert_eq!(policy.roles.capacity(), 3);
    let role = policy.roles.get(0).expect("role");
    assert_eq!((role.number, role.period_spend, role.requests, role.decides), (u32::MAX, 17, u16::MAX, u8::MAX));
    assert_eq!(role.authority, policy.ceiling);
    assert_eq!((policy.requirements.capacity(), policy.requirements.len()), (4, 2));
    for (index, requirement) in policy.requirements.iter().enumerate() {
        assert_eq!((requirement.connector, requirement.kind), (u16::MAX, 0));
        assert_eq!(requirement.pattern, projection.pattern);
        assert_eq!(
            requirement.judge,
            client::PolicyJudge { connector: 0, requirement: u16::MAX, parameters: u32::MAX }
        );
        assert_eq!(requirement.must_be_guarded, index == 0);
    }
    assert_eq!(policy.requirements.get(0).expect("guarded").guard, client::PolicyGuard::Guarded);
    assert_eq!(
        policy.requirements.get(1).expect("observed").guard,
        client::PolicyGuard::Observed { freshness: Duration::ZERO }
    );
    let permissions = policy.permissions.expect("present mappings");
    assert_eq!((permissions.capacity(), permissions.len()), (5, 2));
    assert_eq!(
        permissions.as_slice(),
        &[
            client::PolicyPermission { connector: u16::MAX, permission: 0, role: u32::MAX },
            client::PolicyPermission { connector: 0, permission: u16::MAX, role: 0 },
        ]
    );
    assert!(policy.permissions_cut);
    assert!(!policy.cut);
    assert_eq!(source, original);
}

#[test]
fn project_ledgers_keep_current_and_configured_periods_separate_all_funders_counters_and_optional_clocks() {
    let mut source = source();
    let original = source.clone();
    let actual = document(&source, &limits()).expect("independent ledgers");
    assert_eq!(actual.configured_period, 0);
    let current = actual.current_period.expect("retained current ledger");
    assert_eq!(current.source, client::Funder::Period { project: u32::MAX, period: u64::MAX });
    assert_eq!(current.parent, None);
    assert_eq!(current.numbers, client::OwnerNumbers { budget: 2, spent: 3, spent_below: 5, reserved: 7 });
    assert_eq!((current.left, current.closed, current.began, current.next), (11, false, None, Some(Wall::EPOCH)));
    for values in [&actual.pools, &actual.configured_pools] {
        assert_eq!((values.capacity(), values.len()), (6, 4));
        for (actual, source) in values.iter().zip(source.pools.iter()) {
            assert_eq!(actual, &owner::ledger(source));
            assert_eq!(
                actual.numbers,
                client::OwnerNumbers { budget: 0, spent: u64::MAX, spent_below: 17, reserved: 29 }
            );
            assert_eq!(actual.left, 31);
        }
    }
    assert!(actual.configured_pools_cut);
    assert_eq!(source, original);
    source.current_period = None;
    source.configured_pools = List::with_capacity(6);
    let absent = document(&source, &limits()).expect("no current ledger");
    assert_eq!(absent.current_period, None);
    assert_eq!(absent.configured_period, 0);
    assert_eq!((absent.configured_pools.capacity(), absent.configured_pools.len()), (6, 0));
    assert!(absent.configured_pools_cut);
    let expense = absent.spend.get(0).expect("source expense");
    assert_eq!(absent.spend.capacity(), 5);
    assert_eq!(expense.chip, chips::chip(&source.spend.get(0).expect("source").chip, &limits()).expect("whole chip"));
    assert_eq!(expense.funder, client::Funder::Task(0));
    assert_eq!(expense.numbers, client::OwnerNumbers { budget: u64::MAX, spent: 0, spent_below: 7, reserved: 11 });
}

#[test]
fn project_recurring_keeps_both_overlap_policies_whole_batches_reservations_and_supplied_period_facts() {
    let source = source();
    let original = source.clone();
    let actual = document(&source, &limits()).expect("recurring");
    assert_eq!((actual.recurring.capacity(), actual.recurring.len()), (4, 2));
    for (index, value) in actual.recurring.iter().enumerate() {
        let raw = source.recurring.get(index).expect("source recurring");
        assert_eq!(value.chip, chips::chip(&raw.chip, &limits()).expect("whole template chip"));
        assert_eq!((value.key, value.budget), (u32::MAX, 0));
        assert_eq!(
            value.overlap,
            if index == 0 { client::RecurringOverlap::Skip } else { client::RecurringOverlap::Wait }
        );
        assert_eq!((value.current_period, value.began, value.next), (u64::MAX, Some(Wall::EPOCH), None));
        assert_eq!((value.last_period, value.pending_period), (17, Some(0)));
        assert_eq!((value.batch.capacity(), value.batch.len()), (7, 1));
        let proposed = value.batch.get(0).expect("whole batch task");
        assert_eq!(
            proposed,
            &history::proposed(raw.batch.get(0).expect("source"), &limits()).expect("whole proposed task")
        );
        assert_eq!((proposed.number, proposed.budget, proposed.priority), (u64::MAX, u64::MAX, Some(0)));
        assert_eq!(proposed.resources.capacity(), 7);
        assert!(proposed.resources.is_empty());
        assert_eq!(proposed.funder, client::Funder::Recurring { project: u32::MAX, task: u64::MAX, period: 0 });
        assert_eq!(value.last_batch.capacity(), 6);
        assert_eq!(value.last_batch.as_slice(), &[u64::MAX, 0]);
        assert!(value.cut);
    }
    assert_eq!(source, original);
}

#[test]
fn project_permissions_distinguish_absent_empty_reserved_and_lost_mappings_without_inference() {
    let mut source = source();
    for (permissions, cut, capacity) in [
        (None, false, None),
        (Some(List::with_capacity(0)), false, Some(0)),
        (Some(List::with_capacity(8)), true, Some(8)),
    ] {
        source.policy.permissions = permissions;
        source.policy.permissions_cut = cut;
        source.policy.escalation_role = None;
        source.policy.cut = !cut;
        let actual = document(&source, &limits()).expect("independent absence and loss");
        assert_eq!(actual.policy.permissions.as_ref().map(List::capacity), capacity);
        assert!(actual.policy.permissions.as_ref().is_none_or(List::is_empty));
        assert_eq!(actual.policy.permissions_cut, cut);
        assert_eq!(actual.policy.cut, !cut);
        assert_eq!(actual.policy.escalation_role, None);
        assert!(!actual.cut);
    }
}

#[test]
fn project_read_admits_the_complete_boxed_envelope_at_the_exact_byte_boundary_without_clipping() {
    let source = source();
    let original = source.clone();
    let limits = limits();
    let read = super::super::documents::project_read(&source, &limits).expect("whole read");
    let mut lower = 0;
    let mut upper = limits.document_bytes;
    while lower < upper {
        let middle = lower + (upper - lower) / 2;
        if super::super::documents::project_read(&source, &Limits { document_bytes: middle, ..limits }).is_ok() {
            upper = middle;
        } else {
            lower = middle + 1;
        }
    }
    assert!(lower > 0);
    assert_eq!(super::super::documents::project_read(&source, &Limits { document_bytes: lower, ..limits }), Ok(read));
    assert_eq!(
        super::super::documents::project_read(&source, &Limits { document_bytes: lower - 1, ..limits }).err(),
        Some(super::super::documents::Invalid::Payload)
    );
    assert_eq!(source, original);
}

#[test]
fn project_read_rejects_reserved_empty_collections_final_nested_text_and_the_twenty_sixth_offer_whole() {
    let mut source = source();
    let limits = limits();
    source.acts = List::with_capacity(client::MAX_OFFERED_ACTS);
    for _ in 0..client::MAX_OFFERED_ACTS {
        source
            .acts
            .push(acts::Offered { act: acts::Act::Release, admission: acts::Admission::Allowed })
            .expect("offer");
    }
    assert!(super::super::documents::project_read(&source, &limits).is_ok());
    source.acts = List::with_capacity(client::MAX_OFFERED_ACTS + 1);
    assert!(super::super::documents::project_read(&source, &limits).is_err());
    source.acts = List::with_capacity(client::MAX_OFFERED_ACTS);
    source.policy.permissions = Some(List::with_capacity(9));
    assert!(super::super::documents::project_read(&source, &limits).is_err());
    source.policy.permissions = None;
    source.recurring.get_mut(1).expect("last template").batch.get_mut(0).expect("last nested task").resources =
        List::with_capacity(9);
    assert!(super::super::documents::project_read(&source, &limits).is_err());
    source.recurring.get_mut(1).expect("last template").batch.get_mut(0).expect("last nested task").resources =
        List::with_capacity(7);
    source.members.get_mut(4).expect("final member").name.bytes =
        vec![255; limits.text as usize + 1].into_boxed_slice();
    assert!(super::super::documents::project_read(&source, &limits).is_err());
    source.members.get_mut(4).expect("final member").name.bytes = Box::new([]);
    assert!(super::super::documents::project_read(&source, &limits).is_ok());
}
