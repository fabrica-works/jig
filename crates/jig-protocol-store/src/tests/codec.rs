//! The notes family's translation both ways (protocol/store.md 5.1).
use super::fixtures;
use crate::codec::{component, notes};
use jig_domain_notes as domain;

fn a_notes_entry_reads_back_as_it_was_saved() {
    notes_round_trip(domain::Record::Entry(fixtures::note_entry()));
}
fn a_resources_line_reads_back_with_its_pattern() {
    notes_round_trip(domain::Record::Line(fixtures::note_line()));
}
fn a_notes_row_of_another_version_does_not_decode() {
    let limits = notes::limits(&fixtures::NOTES_LIMITS).expect("within the schema's ceilings");
    let entry = domain::Record::Entry(fixtures::note_entry());
    let mut bytes = notes::encode(&entry, &limits).expect("within its limits");
    *bytes.get_mut(1).expect("version prefix") = 2;
    assert_eq!(
        notes::decode(&bytes, &limits),
        Err(crate::notes_v1::Problem { path: crate::notes_v1::Path::RowVersion, reason: skein_codec::Reason::Version })
    );
}
fn notes_limits_past_the_ceilings_are_refused() {
    for field in 0_u8..4 {
        let mut limits = fixtures::NOTES_LIMITS;
        match field {
            0 => limits.pattern_bytes = u32::MAX,
            1 => limits.description_bytes = u32::MAX,
            2 => limits.body_bytes = u32::MAX,
            _ => limits.references = u32::MAX,
        }
        assert_eq!(notes::limits(&limits), None);
    }
}
fn the_format_row_reads_back_with_its_versions_and_deployment() {
    let row =
        component::Format { component: 1, core: 2, tasks: 3, people: 4, notes: 5, application: 6, deployment: [7; 16] };
    let bytes = row.encode().expect("fixed width fields fit");
    assert_eq!(component::Format::decode(&bytes), Ok(row));
}
fn a_transcript_place_reads_back_with_its_window() {
    let row = component::Transcript { length: u64::MAX, window: 42 };
    assert_eq!(component::Transcript::decode(&row.encode().expect("fixed width fields fit")), Ok(row));
}
fn an_attempt_place_reads_back_with_its_first_turn() {
    let row = component::Attempt { first: u64::MAX };
    assert_eq!(component::Attempt::decode(&row.encode().expect("fixed width fields fit")), Ok(row));
}
fn a_notes_pattern_past_its_aggregate_limit_is_refused() {
    let limits = notes::limits(&fixtures::NOTES_LIMITS).expect("within ceilings");
    let mut line = fixtures::note_line();
    line.scope = domain::Scope::Resources {
        project: 3,
        connector: 2,
        pattern: domain::Pattern { segments: Box::new([Box::new([0; 16])]), last: domain::Last::Exact(Box::new([1])) },
    };
    assert_eq!(
        notes::encode(&domain::Record::Line(line), &limits),
        Err(crate::notes_v1::Problem {
            path: crate::notes_v1::Path::PatternSegments,
            reason: skein_codec::Reason::Bound,
        })
    );
}
fn a_party_authored_note_reads_back_in_each_scope() {
    let limits = notes::limits(&fixtures::NOTES_LIMITS).expect("within ceilings");
    for scope in [
        domain::Scope::Deployment,
        domain::Scope::Project { project: u32::MAX },
        domain::Scope::Resources {
            project: 3,
            connector: 2,
            pattern: domain::Pattern { segments: Box::new([]), last: domain::Last::Exact(Box::new([0, 255])) },
        },
    ] {
        let mut entry = fixtures::note_entry();
        entry.scope = scope;
        entry.author = domain::Author::Party { party: u64::MAX };
        let row = domain::Record::Entry(entry);
        assert_eq!(notes::decode(&notes::encode(&row, &limits).expect("within bounds"), &limits), Ok(row));
    }
}
fn a_decoded_pattern_past_its_aggregate_limit_is_refused() {
    let limits = notes::limits(&fixtures::NOTES_LIMITS).expect("within ceilings");
    let mut line = fixtures::note_line();
    line.scope = domain::Scope::Resources {
        project: 3,
        connector: 2,
        pattern: domain::Pattern { segments: Box::new([Box::new([0; 16])]), last: domain::Last::Exact(Box::new([1])) },
    };
    let mut wider = limits;
    wider.segment_bytes = 17;
    wider.pattern_segments = 17;
    let bytes = notes::encode(&domain::Record::Line(line), &wider).expect("within wider bound");
    assert_eq!(
        notes::decode(&bytes, &limits),
        Err(crate::notes_v1::Problem {
            path: crate::notes_v1::Path::PatternSegments,
            reason: skein_codec::Reason::Bound,
        })
    );
}

fn tasks_limits() -> crate::tasks_v2::Limits {
    crate::codec::tasks::limits(&fixtures::TASKS_LIMITS).expect("within the schema's ceilings")
}
fn saved_tasks(row: &jig_domain_tasks::Stored) -> Result<jig_domain_tasks::Stored, crate::tasks_v2::Problem> {
    let limits = tasks_limits();
    let bytes = crate::codec::tasks::encode(row, &limits).expect("within its limits");
    crate::codec::tasks::decode(&bytes, &limits)
}
fn a_period_ledger_reads_back_as_it_was_saved() {
    let row = jig_domain_tasks::Stored::Ledger(fixtures::task_funding());
    assert_eq!(saved_tasks(&row), Ok(row));
}
fn a_pool_ledger_with_its_parent_reads_back() {
    let mut ledger = fixtures::task_funding();
    ledger.parent = Some(ledger.funder);
    ledger.funder = jig_domain_tasks::Funder::Pool { project: 3, person: 7, period: 5 };
    ledger.period = None;
    let row = jig_domain_tasks::Stored::Ledger(ledger);
    assert_eq!(saved_tasks(&row), Ok(row));
}
fn a_writer_slot_reads_back() {
    let row = jig_domain_tasks::Stored::Writer(fixtures::task_writer());
    assert_eq!(saved_tasks(&row), Ok(row));
}
fn an_effect_writer_slot_reads_back() {
    let mut slot = fixtures::task_writer();
    slot.writer = jig_domain_tasks::Writer::Effect { entry: u64::MAX };
    let row = jig_domain_tasks::Stored::Writer(slot);
    assert_eq!(saved_tasks(&row), Ok(row));
}
fn a_closed_ledger_reads_back() {
    let mut ledger = fixtures::task_funding();
    ledger.closed = true;
    ledger.funder = jig_domain_tasks::Funder::Recurring { project: 3, task: 17, period: 5 };
    let row = jig_domain_tasks::Stored::ClosedLedger(ledger);
    assert_eq!(saved_tasks(&row), Ok(row));
}
fn a_pool_slot_report_reads_back() {
    let row = jig_domain_tasks::Stored::Pool(jig_domain_tasks::PoolSlots {
        number: 9,
        pool: fixtures::task_name(),
        slots: 3,
    });
    assert_eq!(saved_tasks(&row), Ok(row));
}

fn saved_wire<T>(
    value: T,
    measure: fn(&T) -> u32,
    encode: fn(&T, &mut skein_lib::Writer) -> Result<(), skein_lib::Overflow>,
    decode: fn(&crate::tasks_v2::Limits, &mut skein_lib::Reader<'_>) -> Result<T, crate::tasks_v2::Problem>,
) -> T {
    let mut writer = skein_lib::Writer::new(usize::try_from(measure(&value)).expect("measured length"));
    encode(&value, &mut writer).expect("measured room");
    decode(&tasks_limits(), &mut skein_lib::Reader::new(&writer.finish())).expect("within bounds")
}
fn saved_authority(
    value: &jig_domain_tasks::Authority,
) -> Result<jig_domain_tasks::Authority, crate::tasks_v2::Problem> {
    use crate::{codec::tasks::value as translate, tasks_v2::Authority};
    let value = translate::authority(value, &tasks_limits())?;
    let value = saved_wire(value, Authority::measure, Authority::encode, Authority::decode);
    translate::read_authority(value, &tasks_limits())
}
fn an_authority_with_every_grant_kind_reads_back() {
    let value = fixtures::task_authority();
    assert_eq!(saved_authority(&value), Ok(value));
}
fn an_authority_without_a_deadline_reads_back() {
    let mut value = fixtures::task_authority();
    value.budget.deadline = None;
    assert_eq!(saved_authority(&value), Ok(value));
}
fn a_specification_with_every_parameter_kind_reads_back() {
    use crate::{codec::tasks::value as translate, tasks_v2::Spec};
    let value = fixtures::task_spec();
    let limits = tasks_limits();
    let encoded = translate::spec(&value, &limits).expect("within bounds");
    let encoded = saved_wire(encoded, Spec::measure, Spec::encode, Spec::decode);
    assert_eq!(translate::read_spec(encoded, &limits), Ok(value));
}
fn each_party_kind_reads_back() {
    use crate::{codec::tasks::value as translate, tasks_v2::Party};
    for value in [
        jig_domain_tasks::Party::Task(u64::MAX),
        jig_domain_tasks::Party::Person(13),
        jig_domain_tasks::Party::Deployment { project: u32::MAX },
    ] {
        let encoded = translate::party(&value, &tasks_limits()).expect("within bounds");
        let encoded = saved_wire(encoded, Party::measure, Party::encode, Party::decode);
        assert_eq!(translate::read_party(encoded), value);
    }
}
fn each_executor_kind_reads_back() {
    use crate::{codec::tasks::value as translate, tasks_v2::Executor};
    use jig_domain_tasks::PersonAddress;
    for value in [
        jig_domain_tasks::Executor::Agent { charter: 7 },
        jig_domain_tasks::Executor::Procedure { connector: 2, code: 11 },
        jig_domain_tasks::Executor::Person(PersonAddress::Person(u64::MAX)),
        jig_domain_tasks::Executor::Person(PersonAddress::Role(u32::MAX)),
    ] {
        let encoded = translate::executor(value, &tasks_limits()).expect("within bounds");
        let encoded = saved_wire(encoded, Executor::measure, Executor::encode, Executor::decode);
        assert_eq!(translate::read_executor(encoded), value);
    }
}
fn each_failure_class_reads_back() {
    use crate::{codec::tasks::value as translate, tasks_v2::Class};
    use jig_domain_tasks::Class as Domain;
    for value in [Domain::Transient, Domain::Permanent, Domain::Run, Domain::Agent, Domain::Lost, Domain::Invalid] {
        let encoded = saved_wire(translate::class(value), Class::measure, Class::encode, Class::decode);
        assert_eq!(translate::read_class(encoded), value);
    }
}
fn each_activation_cause_reads_back() {
    use crate::{codec::tasks::value as translate, tasks_v2::Why};
    use jig_domain_tasks::{Tries, Why as Domain};
    for value in [
        Domain::Made,
        Domain::Ready,
        Domain::Message { message: u64::MAX },
        Domain::Released,
        Domain::Retry { tries: Tries { transient: 1, permanent: 2, run: 3, agent: 4, lost: 5, invalid: 6 } },
        Domain::Narrowed,
        Domain::Changed,
    ] {
        let encoded = translate::why(value, &tasks_limits()).expect("within bounds");
        let encoded = saved_wire(encoded, Why::measure, Why::encode, Why::decode);
        assert_eq!(translate::read_why(encoded), value);
    }
}
fn saved_contract(value: &jig_domain_tasks::Contract) -> Result<jig_domain_tasks::Contract, crate::tasks_v2::Problem> {
    use crate::{codec::tasks::value as translate, tasks_v2::Contract};
    let value = translate::contract(value, &tasks_limits())?;
    let value = saved_wire(value, Contract::measure, Contract::encode, Contract::decode);
    translate::read_contract(value, &tasks_limits())
}
fn each_result_contract_kind_reads_back() {
    use jig_domain_tasks::{Contract, Part, PartKey, PartKind};
    for value in [
        fixtures::task_contract(),
        Contract::Report { words: 13 },
        Contract::Change { connector: 2, kind: 3, words: 17 },
        Contract::Approval {
            state: Box::from(b"approval".as_slice()),
            part: Part { connector: 7, kind: PartKind::Effect, key: PartKey::Number(u64::MAX) },
        },
    ] {
        assert_eq!(saved_contract(&value), Ok(value));
    }
}
fn each_connector_part_kind_reads_back() {
    use crate::{codec::tasks::value as translate, tasks_v2::Part};
    use jig_domain_tasks::{PartKey, PartKind};
    for kind in [
        PartKind::Resource,
        PartKind::Pool,
        PartKind::Result,
        PartKind::Procedure,
        PartKind::Effect,
        PartKind::Outbox,
        PartKind::Hold,
    ] {
        let mut path = skein_lib::List::with_capacity(2);
        path.push(Box::from(b"a\0".as_slice())).expect("room for two segments");
        path.push(Box::from(b"b".as_slice())).expect("room for two segments");
        let value = jig_domain_tasks::Part { connector: 2, kind, key: PartKey::Path(path) };
        let encoded = translate::part(&value, &tasks_limits()).expect("within bounds");
        let encoded = saved_wire(encoded, Part::measure, Part::encode, Part::decode);
        assert_eq!(translate::read_part(encoded, &tasks_limits()), Ok(value));
    }
}

fn bound_problem(path: crate::tasks_v2::Path) -> crate::tasks_v2::Problem {
    crate::tasks_v2::Problem { path, reason: skein_codec::Reason::Bound }
}
fn spec_with_bytes(bytes: usize) -> jig_domain_tasks::Spec {
    jig_domain_tasks::Spec {
        words: Box::new([]),
        parameters: Box::new([jig_domain_tasks::Parameter::Bytes { name: 1, value: skein_lib::bytes::zeroed(bytes) }]),
        inputs: Box::new([]),
    }
}
fn a_specification_at_its_aggregate_byte_bound_reads_back() {
    use crate::{codec::tasks::value as translate, tasks_v2::Spec};
    let value = spec_with_bytes(32);
    let encoded = translate::spec(&value, &tasks_limits()).expect("at its byte bound");
    let encoded = saved_wire(encoded, Spec::measure, Spec::encode, Spec::decode);
    assert_eq!(translate::read_spec(encoded, &tasks_limits()), Ok(value));
}
fn a_specification_past_its_aggregate_byte_bound_is_refused() {
    use crate::codec::tasks::value as translate;
    let mut value = spec_with_bytes(32);
    value.words = Box::new([1]);
    assert_eq!(translate::spec(&value, &tasks_limits()), Err(bound_problem(crate::tasks_v2::Path::SpecWords)));
}
fn a_decoded_specification_past_its_aggregate_byte_bound_is_refused() {
    use crate::codec::tasks::value as translate;
    let mut value = spec_with_bytes(32);
    value.words = Box::new([1]);
    let mut wider = tasks_limits();
    wider.spec_words = 33;
    let encoded = translate::spec(&value, &wider).expect("within wider limit");
    assert_eq!(translate::read_spec(encoded, &tasks_limits()), Err(bound_problem(crate::tasks_v2::Path::SpecWords)));
}
fn authority_with_bytes(bytes: usize) -> jig_domain_tasks::Authority {
    let mut value = fixtures::task_authority();
    value.grants = Box::new([]);
    value.note_resources = Box::new([jig_domain_tasks::ResourceScope {
        connector: 1,
        pattern: jig_domain_tasks::Pattern {
            segments: Box::new([skein_lib::bytes::zeroed(bytes)]),
            last: jig_domain_tasks::Last::Exact(Box::new([])),
        },
    }]);
    value
}
fn an_authority_at_its_shared_byte_bound_reads_back() {
    let value = authority_with_bytes(32);
    assert_eq!(saved_authority(&value), Ok(value));
}
fn authority_past_shared_bytes() -> jig_domain_tasks::Authority {
    let mut value = authority_with_bytes(32);
    value.grants = Box::new([jig_domain_tasks::Grant {
        connector: 2,
        kind: 3,
        pattern: jig_domain_tasks::Pattern {
            segments: Box::new([]),
            last: jig_domain_tasks::Last::Open(Box::new([1])),
        },
    }]);
    value
}
fn grants_and_note_resources_past_their_shared_byte_bound_are_refused() {
    assert_eq!(
        crate::codec::tasks::value::authority(&authority_past_shared_bytes(), &tasks_limits()),
        Err(bound_problem(crate::tasks_v2::Path::AuthorityGrants)),
    );
}
fn decoded_grants_and_note_resources_past_their_shared_byte_bound_are_refused() {
    use crate::codec::tasks::value as translate;
    let mut wider = tasks_limits();
    wider.authority_segment_bytes = 33;
    let encoded = translate::authority(&authority_past_shared_bytes(), &wider).expect("within wider limit");
    assert_eq!(
        translate::read_authority(encoded, &tasks_limits()),
        Err(bound_problem(crate::tasks_v2::Path::AuthorityGrants))
    );
}
fn writer_with_path_bytes(bytes: usize) -> jig_domain_tasks::Stored {
    let mut slot = fixtures::task_writer();
    slot.resource.path = Box::new([skein_lib::bytes::zeroed(bytes), Box::new([1])]);
    jig_domain_tasks::Stored::Writer(slot)
}
fn a_writer_resource_at_its_aggregate_byte_bound_reads_back() {
    let row = writer_with_path_bytes(15);
    assert_eq!(saved_tasks(&row), Ok(row));
}
fn a_writer_resource_past_its_aggregate_byte_bound_is_refused() {
    assert_eq!(
        crate::codec::tasks::encode(&writer_with_path_bytes(16), &tasks_limits()),
        Err(bound_problem(crate::tasks_v2::Path::NamePath))
    );
}
fn a_decoded_writer_resource_past_its_aggregate_byte_bound_is_refused() {
    let mut wider = tasks_limits();
    wider.segment_bytes = 17;
    let bytes = crate::codec::tasks::encode(&writer_with_path_bytes(16), &wider).expect("within wider limit");
    assert_eq!(
        crate::codec::tasks::decode(&bytes, &tasks_limits()),
        Err(bound_problem(crate::tasks_v2::Path::NamePath))
    );
}
fn a_tasks_row_of_another_version_does_not_decode() {
    let limits = tasks_limits();
    let row = jig_domain_tasks::Stored::Ledger(fixtures::task_funding());
    let mut bytes = crate::codec::tasks::encode(&row, &limits).expect("within bounds");
    bytes[1] = 3;
    assert_eq!(
        crate::codec::tasks::decode(&bytes, &limits),
        Err(crate::tasks_v2::Problem { path: crate::tasks_v2::Path::RowVersion, reason: skein_codec::Reason::Version })
    );
}
fn task_funded_accounting_reads_back() {
    let mut value = fixtures::task_funding();
    value.funder = jig_domain_tasks::Funder::Task(u64::MAX);
    let row = jig_domain_tasks::Stored::Ledger(value);
    assert_eq!(saved_tasks(&row), Ok(row));
}

fn tasks_limits_past_the_ceilings_are_refused() {
    for field in 0_u8..11 {
        let mut limits = fixtures::TASKS_LIMITS;
        match field {
            0 => limits.hold_bytes = u32::MAX,
            1 => limits.hold_segments = u32::MAX,
            2 => limits.spec_bytes = u32::MAX,
            3 => limits.parameters = u32::MAX,
            4 => limits.inputs = u32::MAX,
            5 => limits.result_bytes = u32::MAX,
            6 => limits.contract_choices = u32::MAX,
            7 => limits.authority_bytes = u32::MAX,
            8 => limits.authority_segments = u32::MAX,
            9 => limits.executor_kinds = u32::MAX,
            _ => limits.authority_grants = u32::MAX,
        }
        assert_eq!(crate::codec::tasks::limits(&limits), None);
    }
}
fn verdict_rules_at_their_aggregate_byte_bound_read_back() {
    let value = fixtures::task_contract();
    assert_eq!(saved_contract(&value), Ok(value));
}
fn contract_past_shared_bytes() -> jig_domain_tasks::Contract {
    jig_domain_tasks::Contract::Verdict {
        choices: Box::new([jig_domain_tasks::Verdict {
            code: 7,
            words: 17,
            followups: 3,
            fields: Box::new([jig_domain_tasks::FieldRule { name: Box::from(b"field".as_slice()), max: 11 }]),
        }]),
    }
}
fn verdict_rules_past_their_aggregate_byte_bound_are_refused() {
    assert_eq!(
        crate::codec::tasks::value::contract(&contract_past_shared_bytes(), &tasks_limits()),
        Err(bound_problem(crate::tasks_v2::Path::VerdictFields))
    );
}
fn decoded_verdict_rules_past_their_aggregate_byte_bound_are_refused() {
    use crate::codec::tasks::value as translate;
    let mut wider = tasks_limits();
    wider.field_rule_name = 33;
    let encoded = translate::contract(&contract_past_shared_bytes(), &wider).expect("within wider limit");
    assert_eq!(
        translate::read_contract(encoded, &tasks_limits()),
        Err(bound_problem(crate::tasks_v2::Path::VerdictFields))
    );
}

mod phase;

fn core_limits() -> jig_domain_core::Limits {
    fixtures::CORE_LIMITS
}

fn people_limits() -> crate::people_v2::Limits {
    crate::codec::people::limits(&core_limits()).expect("within people's ceilings")
}

fn people_round_trip(row: jig_domain_people::Stored) {
    let limits = people_limits();
    let bytes = crate::codec::people::encode(&row, &limits).expect("within people's bounds");
    assert_eq!(crate::codec::people::decode(&bytes, &limits), Ok(row));
}

fn ask_round_trip(ask: jig_domain_people::Ask) {
    let limits = people_limits();
    let value = crate::codec::people::encode_ask(&ask, &limits).expect("bounded ask");
    let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).expect("bounded length"));
    value.encode(&mut writer).expect("measured room");
    let bytes = writer.finish();
    let read = crate::people_v2::Ask::decode(&limits, &mut skein_lib::Reader::new(&bytes)).expect("saved ask");
    assert_eq!(crate::codec::people::decode_ask(read, &limits), Ok(ask));
}

fn a_person_reads_back_with_its_identity() {
    people_round_trip(jig_domain_people::Stored::Person { number: u64::MAX, identity: fixtures::identity() });
}

fn a_projects_policy_reads_back() {
    people_round_trip(jig_domain_people::Stored::Policy { project: 3, value: fixtures::people_policy() });
}

fn a_policy_larger_than_one_amendment_reads_back() {
    let mut value = fixtures::people_policy();
    let first = value.roles[0].clone();
    let mut second = first.clone();
    second.number = 5;
    value.roles = Box::new([first, second]);
    people_round_trip(jig_domain_people::Stored::Policy { project: 3, value });
}

#[test]
fn shared_people_fields_take_the_larger_policy_bound() {
    let mut domain = core_limits();
    domain.people.amendment_bytes = 16;
    domain.authority.segment_bytes = 32;
    domain.authority.segments = 3;
    let limits = crate::codec::people::limits(&domain).expect("within ceilings");
    assert_eq!(limits.segment_bytes, 32);
    assert_eq!(limits.pattern_segments, 8184);
}

#[test]
fn each_kind_of_ask_reads_back() {
    for ask in fixtures::people_asks() {
        ask_round_trip(ask);
    }
}

fn a_read_chat_position_reads_back() {
    people_round_trip(jig_domain_people::Stored::ReadChat { party: 3, task: 4, through: u64::MAX });
}

fn a_sign_in_reads_back_with_its_wall_expiry() {
    people_round_trip(jig_domain_people::Stored::SignIn {
        number: 2,
        person: 3,
        expires: skein_lib::Wall::from_nanos(u64::MAX),
    });
}

fn every_role_reads_back_in_a_roster() {
    use jig_domain_people::{Holding, Role, Stored};
    for role in [Role::Owner, Role::Maintainer, Role::Member, Role::Observer, Role::Policy { role: u32::MAX }] {
        people_round_trip(Stored::Roles { project: 3, holdings: Box::new([Holding { person: 4, role }]) });
    }
}

fn a_read_results_position_reads_back() {
    people_round_trip(jig_domain_people::Stored::ReadResults { party: 3, through: u64::MAX });
}

fn people_limits_past_their_ceilings_are_refused() {
    let mut limits = core_limits();
    limits.people.amendment_bytes = u32::MAX;
    assert_eq!(crate::codec::people::limits(&limits), None);
}

#[test]
fn a_person_identity_field_past_its_limit_is_refused() {
    let mut identity = fixtures::identity();
    identity.key.subject = Box::new([0; 33]);
    assert_eq!(
        crate::codec::people::encode(&jig_domain_people::Stored::Person { number: 1, identity }, &people_limits()),
        Err(crate::people_v2::Problem {
            path: crate::people_v2::Path::IdentityKeySubject,
            reason: skein_codec::Reason::Bound
        })
    );
}

fn a_people_row_of_another_version_does_not_decode() {
    let limits = people_limits();
    let row = jig_domain_people::Stored::ReadResults { party: 1, through: 2 };
    let mut bytes = crate::codec::people::encode(&row, &limits).expect("fixed row");
    *bytes.get_mut(1).expect("version prefix") = 3;
    assert_eq!(
        crate::codec::people::decode(&bytes, &limits),
        Err(crate::people_v2::Problem {
            path: crate::people_v2::Path::RowVersion,
            reason: skein_codec::Reason::Version
        })
    );
}

mod record;
fn call_limits() -> crate::codec::core::Limits {
    crate::codec::core::limits(&core_limits()).expect("within core ceilings")
}
fn core_round_trip(row: jig_domain_core::CoreRecord) {
    let limits = call_limits();
    let bytes = crate::codec::core::encode(&row, &limits).expect("within core bounds");
    assert_eq!(crate::codec::core::decode(&bytes, &limits), Ok(row));
}
fn a_call_record_of_each_part_reads_back() {
    for part in fixtures::calls::call_parts() {
        core_round_trip(fixtures::calls::call(part));
    }
}
fn a_refused_delegation_reads_back_with_its_problem() {
    core_round_trip(fixtures::calls::call(jig_domain_core::CallPart::DelegationRefused(fixtures::calls::problem())));
}
fn a_recalled_notes_page_reads_back() {
    core_round_trip(fixtures::calls::call(jig_domain_core::CallPart::NoteRecalled {
        entries: Box::new([fixtures::note_entry()]),
        more: true,
    }));
}
fn a_settled_call_reads_back() {
    use jig_domain_core as core;
    for answer in [
        core::SettledAnswer::Host { error: true, body: Box::from(b"denied".as_slice()) },
        core::SettledAnswer::Delivery {
            outcome: core::DeliveryOutcome::Stale,
            evidence: Box::from(b"receipt".as_slice()),
        },
    ] {
        let core::CoreRecord::Call(mut row) = fixtures::calls::call(core::CallPart::Unavailable) else {
            panic!("call fixture")
        };
        row.settled = Some(core::SettledCall {
            serial: 1,
            name: Box::from(b"call".as_slice()),
            tool: Box::from(b"tool".as_slice()),
            answer,
        });
        core_round_trip(core::CoreRecord::Call(row));
    }
}
fn a_finished_terminal_reads_back_with_its_followup_batch() {
    use jig_domain_tasks as tasks;
    let end = tasks::End::FinishedWithFollowups {
        result: tasks::TaskResult::Report { words: Box::from(b"finished".as_slice()) },
        followups: tasks::ResultFollowups::Delegates(Box::new([fixtures::task_creation(2)])),
    };
    end_round_trip(end);
}

fn a_finished_terminal_reads_back_with_its_followup_proposal() {
    use jig_domain_tasks as tasks;
    let end = tasks::End::FinishedWithFollowups {
        result: tasks::TaskResult::Report { words: Box::from(b"finished".as_slice()) },
        followups: tasks::ResultFollowups::Proposal(Box::new(fixtures::task_proposal())),
    };
    end_round_trip(end);
}

fn a_carried_phase_reads_back_as_its_own_family() {
    let phase = fixtures::task_phase();
    let limits = call_limits();
    let bytes = crate::codec::tasks::carried::phase(&phase, &limits.tasks).expect("phase within bounds");
    assert_eq!(crate::codec::tasks::carried::read_phase(&bytes, &limits.tasks), Ok(phase));
}

fn a_carried_change_reads_back_as_its_own_family() {
    let change = jig_domain_tasks::Change::Prioritised { priority: 7 };
    let limits = call_limits();
    let bytes = crate::codec::tasks::carried::change(&change, &limits.tasks).expect("change within bounds");
    assert_eq!(crate::codec::tasks::carried::read_change(&bytes, &limits.tasks), Ok(change));
}

fn core_limits_past_the_settled_answer_ceiling_are_refused() {
    let mut domain = core_limits();
    domain.call_answer_bytes = crate::core_v2::CEILINGS.settled_answer_host_body + 1;
    assert!(crate::codec::core::limits(&domain).is_none());
}

fn end_round_trip(end: jig_domain_tasks::End) {
    let limits = call_limits();
    let bytes = crate::codec::tasks::carried::end(&end, &limits.tasks).expect("terminal within bounds");
    assert_eq!(crate::codec::tasks::carried::read_end(&bytes, &limits.tasks), Ok(end));
}

mod core_rows;

mod people_outcomes;

fn task_ledgers_read_back() {
    table([
        ("a_period_ledger_reads_back_as_it_was_saved", a_period_ledger_reads_back_as_it_was_saved),
        ("a_pool_ledger_with_its_parent_reads_back", a_pool_ledger_with_its_parent_reads_back),
        ("a_closed_ledger_reads_back", a_closed_ledger_reads_back),
        ("task_funded_accounting_reads_back", task_funded_accounting_reads_back),
    ]);
}

fn writer_slots_read_back() {
    table([
        ("a_writer_slot_reads_back", a_writer_slot_reads_back),
        ("an_effect_writer_slot_reads_back", an_effect_writer_slot_reads_back),
    ]);
}

fn authorities_read_back() {
    table([
        ("an_authority_with_every_grant_kind_reads_back", an_authority_with_every_grant_kind_reads_back),
        ("an_authority_without_a_deadline_reads_back", an_authority_without_a_deadline_reads_back),
        ("an_authority_at_its_shared_byte_bound_reads_back", an_authority_at_its_shared_byte_bound_reads_back),
    ]);
}

fn notes_rows_read_back_as_saved() {
    table([
        ("a_notes_entry_reads_back_as_it_was_saved", a_notes_entry_reads_back_as_it_was_saved),
        ("a_resources_line_reads_back_with_its_pattern", a_resources_line_reads_back_with_its_pattern),
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

fn notes_round_trip(row: domain::Record) {
    let limits = notes::limits(&fixtures::NOTES_LIMITS).expect("within the schema's ceilings");
    let bytes = notes::encode(&row, &limits).expect("within its limits");
    assert_eq!(notes::decode(&bytes, &limits), Ok(row));
}

mod pieces;

mod cases;
