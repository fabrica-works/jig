//! Byte-summed pieces preserve the domain's admissions (codec.md, 3).
use super::fixtures;
use crate::codec::{people, tasks};
use crate::{people_v12, tasks_v9};
use jig_domain_people as person;
use jig_domain_tasks as task;
use skein_lib::{List, Reader, Writer};

fn people_limits() -> people_v12::Limits {
    let mut core = fixtures::CORE_LIMITS;
    core.people.amendment_bytes = 8192;
    core.policy_bytes = 8192;
    people::limits(&core).expect("test-sized byte sums")
}
fn pattern(segments: usize) -> person::Pattern {
    let mut values: List<Box<[u8]>> = List::with_capacity(u32::try_from(segments).expect("bounded count"));
    for _ in 0..segments {
        values.push(Box::new([])).expect("same count");
    }
    person::Pattern { segments: values.into_boxed(), last: person::Last::Exact(Box::new([])) }
}
fn amendment() -> person::Amendment {
    let mut value = fixtures::people_amendment();
    value.reason = Box::new([]);
    value.instructions = None;
    value.spec = None;
    value.dependencies = None;
    let authority = value.authority.as_mut().expect("fixture authority");
    authority.grants = Box::new([person::Grant { connector: 1, kind: 2, pattern: pattern(509) }]);
    authority.delegation.kinds = Box::new([]);
    authority.note_resources = Box::new([]);
    value
}

#[test]
fn the_pattern_piece_admits_512_segments_and_refuses_513() {
    let limits = people_limits();
    for (label, segments, passes) in [("at_sum", 512, true), ("past_sum", 513, false)] {
        let value = pattern(segments);
        let encoded = people::policy::encode_pattern(&value, &limits);
        if passes {
            let encoded = encoded.expect(label);
            let mut out = Writer::new(usize::try_from(encoded.measure()).expect("measured size"));
            encoded.encode(&mut out).expect("measured pattern");
            let bytes = out.finish();
            let decoded = people_v12::Pattern::decode(&limits, &mut Reader::new(&bytes)).expect("encoded pattern");
            assert_eq!(people::policy::decode_pattern(decoded, &limits), Ok(value), "{label}");
        } else {
            assert_eq!(encoded, Err(people_problem()), "{label}");
        }
    }
}

#[test]
fn a_whole_amendment_admits_its_full_sum_and_refuses_one_byte_past_it() {
    let limits = people_limits();
    let value = amendment();
    assert_eq!(person::amendment_bytes(&value), Some(8192));
    let encoded = people::policy::encode_amendment(&value, &limits).expect("whole sum fits");
    let mut out = Writer::new(usize::try_from(encoded.measure()).expect("measured size"));
    encoded.encode(&mut out).expect("measured amendment");
    let bytes = out.finish();
    let decoded = people_v12::Amendment::decode(&limits, &mut Reader::new(&bytes)).expect("whole amendment");
    assert_eq!(people::policy::decode_amendment(decoded, &limits), Ok(value.clone()));
    let mut past = value;
    past.reason = Box::new([1]);
    assert_eq!(person::amendment_bytes(&past), Some(8193));
    assert_eq!(people::policy::encode_amendment(&past, &limits), Err(people_problem()), "encoded past_sum");
    let mut parts = encoded.into_parts();
    parts.reason = Box::new([1]);
    let forged = people_v12::Amendment::new(&limits, parts).expect("each scalar still fits");
    assert_eq!(people::policy::decode_amendment(forged, &limits), Err(people_problem()), "decoded past_sum");
}

fn task_limits() -> tasks_v9::Limits {
    let mut domain = fixtures::TASKS_LIMITS;
    domain.result_bytes = 512;
    tasks::limits(&domain).expect("test-sized byte sum")
}
fn fields(count: usize) -> Box<[task::ResultField]> {
    let mut values = List::with_capacity(u32::try_from(count).expect("bounded count"));
    for _ in 0..count {
        values.push(task::ResultField { name: Box::new([1]), value: Box::new([]) }).expect("same count");
    }
    values.into_boxed()
}
fn saved_fields(
    fields: &[task::ResultField],
    limits: &tasks_v9::Limits,
) -> Result<Box<[task::ResultField]>, tasks_v9::Problem> {
    let value = task::TaskResult::Verdict { code: 7, fields: fields.into(), words: Box::new([]) };
    let encoded = tasks::phase::result(&value, limits)?;
    let mut out = Writer::new(usize::try_from(encoded.measure()).expect("measured size"));
    encoded.encode(&mut out).expect("measured verdict");
    let bytes = out.finish();
    let decoded = tasks_v9::TaskResult::decode(limits, &mut Reader::new(&bytes))?;
    match tasks::phase::read_result(decoded, limits)? {
        task::TaskResult::Verdict { fields, .. } => Ok(fields),
        task::TaskResult::Approved { .. }
        | task::TaskResult::Report { .. }
        | task::TaskResult::Change { .. }
        | task::TaskResult::Failure { .. } => unreachable!("verdict fixture"),
    }
}

#[test]
fn verdict_fields_round_trip_at_the_domain_sum_and_refuse_one_past_it() {
    let limits = task_limits();
    for (label, count, passes) in [("at_sum", 512, true), ("past_sum", 513, false)] {
        let fields = fields(count);
        let saved = saved_fields(&fields, &limits);
        if passes {
            assert_eq!(saved, Ok(fields), "{label}");
        } else {
            assert_eq!(
                saved,
                Err(tasks_v9::Problem { path: tasks_v9::Path::TaskResultTag, reason: skein_codec::Reason::Bound }),
                "{label}"
            );
        }
    }
}

#[test]
fn the_encoded_piece_bound_holds_a_verdict_at_the_domain_sum_exactly() {
    let limits = task_limits();
    let value = task::TaskResult::Verdict { code: 7, fields: fields(512), words: Box::new([]) };
    let encoded = tasks::phase::result(&value, &limits).expect("domain sum fits");
    let tasks_v9::TaskResult::Verdict(encoded) = encoded else { unreachable!("verdict fixture") };
    assert_eq!(encoded.fields().len(), 6656);
    assert_eq!(limits.verdict_result_fields, 6656);
    assert_eq!(limits.verdict_fields, 6656);
    assert_eq!(limits.verdict_message_fields, 6656);
}

fn people_problem() -> people_v12::Problem {
    people_v12::Problem { path: people_v12::Path::RowValue, reason: skein_codec::Reason::Bound }
}
