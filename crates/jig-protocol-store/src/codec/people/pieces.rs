//! Byte-summed collections, framed without multiplying nested lists (codec.md, 3).
use super::{
    Box, List, Reader, Writer, count, decode_executor, decode_grant, decode_parameter, decode_permission_role,
    decode_requirement, decode_resource_scope, domain, encode_executor, encode_grant, encode_parameter,
    encode_permission_role, encode_requirement, encode_resource_scope, problem, wire,
};

pub(super) const EMPTY: wire::Limits = wire::Limits {
    member_words: 0,
    member_parameters: 0,
    member_dependencies: 0,
    field_rule_name: 0,
    verdict_fields: 0,
    contract_verdict_choices: 0,
    segment_bytes: 0,
    last_exact_value: 0,
    last_open_value: 0,
    pattern_segments: 0,
    parameter_bytes_value: 0,
    ..wire::CEILINGS
};

pub(super) fn piece_bound(sum: u32, owned: usize, item: u64, payload: bool) -> Option<u32> {
    let owned = u64::try_from(owned).ok()?;
    let framing = item.checked_add(u64::try_from(size_of::<u32>()).expect("length size"))?;
    let sum = u64::from(sum);
    let count = sum.checked_div(owned)?;
    let fixed = count.checked_mul(framing)?;
    let bound = if payload && count > 0 {
        if framing >= owned {
            sum.checked_add(count.checked_mul(framing.checked_sub(owned)?)?)?
        } else {
            sum.checked_sub(owned)?.checked_add(framing)?
        }
    } else {
        fixed
    };
    u32::try_from(bound).ok()
}
fn spend(used: &mut u64, item: Option<u64>, sum: u32) -> Result<(), wire::Problem> {
    let Some(item) = item else {
        return Err(problem());
    };
    *used = match used.checked_add(item) {
        Some(value) => value,
        None => return Err(problem()),
    };
    if *used > u64::from(sum) {
        return Err(problem());
    }
    Ok(())
}
fn owned(inline: usize, payload: usize) -> Option<u64> {
    u64::try_from(inline).ok()?.checked_add(u64::try_from(payload).ok()?)
}
fn pattern_bytes(value: &domain::Pattern) -> Option<u64> {
    let mut bytes =
        u64::try_from(value.segments.len()).ok()?.checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?;
    for segment in &value.segments {
        bytes = bytes.checked_add(u64::try_from(segment.len()).ok()?)?;
    }
    let last = match &value.last {
        domain::Last::Exact(value) | domain::Last::Open(value) => value.len(),
    };
    bytes.checked_add(u64::try_from(last).ok()?)
}
fn parameter_bytes(value: &domain::Parameter) -> Option<u64> {
    let payload = match value {
        domain::Parameter::Bytes { value, .. } => value.len(),
        domain::Parameter::Number { .. } | domain::Parameter::Resource { .. } => 0,
    };
    owned(size_of::<domain::Parameter>(), payload)
}
fn grant_bytes(value: &domain::Grant) -> Option<u64> {
    u64::try_from(size_of::<domain::Grant>()).ok()?.checked_add(pattern_bytes(&value.pattern)?)
}
fn scope_bytes(value: &domain::ResourceScope) -> Option<u64> {
    u64::try_from(size_of::<domain::ResourceScope>()).ok()?.checked_add(pattern_bytes(&value.pattern)?)
}
fn requirement_bytes(value: &domain::Requirement) -> Option<u64> {
    u64::try_from(size_of::<domain::Requirement>()).ok()?.checked_add(pattern_bytes(&value.pattern)?)
}
pub(super) fn encode_segments(
    values: &[Box<[u8]>],
    limits: &wire::Limits,
    sum: u32,
    bound: u32,
) -> Result<Box<[u8]>, wire::Problem> {
    let mut items = List::with_capacity(count(values.len())?);
    let mut used = 0;
    for value in values {
        spend(&mut used, owned(size_of::<Box<[u8]>>(), value.len()), sum)?;
        let value = wire::Segment::new(limits, wire::SegmentParts { bytes: value.clone() })?;
        let mut item = Writer::new(usize::try_from(value.measure()).expect("measured size"));
        value.encode(&mut item).expect("measured item");
        items.push(item.finish()).expect("same item count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(problem()),
    }
}
pub(super) fn decode_segments(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
    sum: u32,
) -> Result<Box<[Box<[u8]>]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values =
        List::with_capacity(count(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size"))?);
    let mut used = 0;
    while let Some(bytes) = split(&mut reader)? {
        let value = wire::Segment::decode(limits, &mut Reader::new(&bytes))?;
        let value = value.into_parts().bytes;
        spend(&mut used, owned(size_of::<Box<[u8]>>(), value.len()), sum)?;
        values.push(value).expect("piece contains at most one item per length");
    }
    Ok(values.into_boxed())
}

pub(super) fn encode_delegation_kinds(
    values: &[domain::Executor],
    limits: &wire::Limits,
    sum: u32,
    bound: u32,
) -> Result<Box<[u8]>, wire::Problem> {
    let mut items = List::with_capacity(count(values.len())?);
    let mut used = 0;
    for value in values {
        spend(&mut used, u64::try_from(size_of::<domain::Executor>()).ok(), sum)?;
        let value = encode_executor(*value, limits)?;
        let mut item = Writer::new(usize::try_from(value.measure()).expect("measured size"));
        value.encode(&mut item).expect("measured item");
        items.push(item.finish()).expect("same item count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(problem()),
    }
}
pub(super) fn decode_delegation_kinds(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
    sum: u32,
) -> Result<Box<[domain::Executor]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values =
        List::with_capacity(count(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size"))?);
    let mut used = 0;
    while let Some(bytes) = split(&mut reader)? {
        let value = wire::Executor::decode(limits, &mut Reader::new(&bytes))?;
        let value = decode_executor(value);
        spend(&mut used, u64::try_from(size_of::<domain::Executor>()).ok(), sum)?;
        values.push(value).expect("piece contains at most one item per length");
    }
    Ok(values.into_boxed())
}

pub(super) fn encode_authority_note_resources(
    values: &[domain::ResourceScope],
    limits: &wire::Limits,
    sum: u32,
    bound: u32,
) -> Result<Box<[u8]>, wire::Problem> {
    let mut items = List::with_capacity(count(values.len())?);
    let mut used = 0;
    for value in values {
        spend(&mut used, scope_bytes(value), sum)?;
        let value = encode_resource_scope(value, limits)?;
        let mut item = Writer::new(usize::try_from(value.measure()).expect("measured size"));
        value.encode(&mut item).expect("measured item");
        items.push(item.finish()).expect("same item count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(problem()),
    }
}
pub(super) fn decode_authority_note_resources(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
    sum: u32,
) -> Result<Box<[domain::ResourceScope]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values =
        List::with_capacity(count(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size"))?);
    let mut used = 0;
    while let Some(bytes) = split(&mut reader)? {
        let value = wire::ResourceScope::decode(limits, &mut Reader::new(&bytes))?;
        let value = decode_resource_scope(value, limits)?;
        spend(&mut used, scope_bytes(&value), sum)?;
        values.push(value).expect("piece contains at most one item per length");
    }
    Ok(values.into_boxed())
}

pub(super) fn encode_grant_piece(
    values: &[domain::Grant],
    limits: &wire::Limits,
    sum: u32,
    bound: u32,
) -> Result<Box<[u8]>, wire::Problem> {
    let mut items = List::with_capacity(count(values.len())?);
    let mut used = 0;
    for value in values {
        spend(&mut used, grant_bytes(value), sum)?;
        let value = encode_grant(value, limits)?;
        let mut item = Writer::new(usize::try_from(value.measure()).expect("measured size"));
        value.encode(&mut item).expect("measured item");
        items.push(item.finish()).expect("same item count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(problem()),
    }
}
pub(super) fn decode_grant_piece(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
    sum: u32,
) -> Result<Box<[domain::Grant]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values =
        List::with_capacity(count(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size"))?);
    let mut used = 0;
    while let Some(bytes) = split(&mut reader)? {
        let value = wire::Grant::decode(limits, &mut Reader::new(&bytes))?;
        let value = decode_grant(value, limits)?;
        spend(&mut used, grant_bytes(&value), sum)?;
        values.push(value).expect("piece contains at most one item per length");
    }
    Ok(values.into_boxed())
}

pub(super) fn encode_requirement_piece(
    values: &[domain::Requirement],
    limits: &wire::Limits,
    sum: u32,
    bound: u32,
) -> Result<Box<[u8]>, wire::Problem> {
    let mut items = List::with_capacity(count(values.len())?);
    let mut used = 0;
    for value in values {
        spend(&mut used, requirement_bytes(value), sum)?;
        let value = encode_requirement(value, limits)?;
        let mut item = Writer::new(usize::try_from(value.measure()).expect("measured size"));
        value.encode(&mut item).expect("measured item");
        items.push(item.finish()).expect("same item count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(problem()),
    }
}
pub(super) fn decode_requirement_piece(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
    sum: u32,
) -> Result<Box<[domain::Requirement]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values =
        List::with_capacity(count(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size"))?);
    let mut used = 0;
    while let Some(bytes) = split(&mut reader)? {
        let value = wire::Requirement::decode(limits, &mut Reader::new(&bytes))?;
        let value = decode_requirement(value, limits)?;
        spend(&mut used, requirement_bytes(&value), sum)?;
        values.push(value).expect("piece contains at most one item per length");
    }
    Ok(values.into_boxed())
}

pub(super) fn encode_spec_parameters(
    values: &[domain::Parameter],
    limits: &wire::Limits,
    sum: u32,
    bound: u32,
) -> Result<Box<[u8]>, wire::Problem> {
    let mut items = List::with_capacity(count(values.len())?);
    let mut used = 0;
    for value in values {
        spend(&mut used, parameter_bytes(value), sum)?;
        let value = encode_parameter(value, limits)?;
        let mut item = Writer::new(usize::try_from(value.measure()).expect("measured size"));
        value.encode(&mut item).expect("measured item");
        items.push(item.finish()).expect("same item count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(problem()),
    }
}
pub(super) fn decode_spec_parameters(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
    sum: u32,
) -> Result<Box<[domain::Parameter]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values =
        List::with_capacity(count(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size"))?);
    let mut used = 0;
    while let Some(bytes) = split(&mut reader)? {
        let value = wire::Parameter::decode(limits, &mut Reader::new(&bytes))?;
        let value = decode_parameter(value);
        spend(&mut used, parameter_bytes(&value), sum)?;
        values.push(value).expect("piece contains at most one item per length");
    }
    Ok(values.into_boxed())
}

pub(super) fn encode_numbers(
    values: &[u64],
    limits: &wire::Limits,
    sum: u32,
    bound: u32,
) -> Result<Box<[u8]>, wire::Problem> {
    let mut items = List::with_capacity(count(values.len())?);
    let mut used = 0;
    for value in values {
        spend(&mut used, u64::try_from(size_of::<u64>()).ok(), sum)?;
        let value = wire::Number64::new(limits, wire::Number64Parts { value: *value })?;
        let mut item = Writer::new(usize::try_from(value.measure()).expect("measured size"));
        value.encode(&mut item).expect("measured item");
        items.push(item.finish()).expect("same item count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(problem()),
    }
}
pub(super) fn decode_numbers(bytes: Box<[u8]>, limits: &wire::Limits, sum: u32) -> Result<Box<[u64]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values =
        List::with_capacity(count(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size"))?);
    let mut used = 0;
    while let Some(bytes) = split(&mut reader)? {
        let value = wire::Number64::decode(limits, &mut Reader::new(&bytes))?;
        let value = value.into_parts().value;
        spend(&mut used, u64::try_from(size_of::<u64>()).ok(), sum)?;
        values.push(value).expect("piece contains at most one item per length");
    }
    Ok(values.into_boxed())
}

pub(super) fn encode_permissions(
    values: &[domain::PermissionRole],
    limits: &wire::Limits,
    sum: u32,
    bound: u32,
) -> Result<Box<[u8]>, wire::Problem> {
    let mut items = List::with_capacity(count(values.len())?);
    let mut used = 0;
    for value in values {
        spend(&mut used, u64::try_from(size_of::<domain::PermissionRole>()).ok(), sum)?;
        let value = encode_permission_role(*value, limits)?;
        let mut item = Writer::new(usize::try_from(value.measure()).expect("measured size"));
        value.encode(&mut item).expect("measured item");
        items.push(item.finish()).expect("same item count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(problem()),
    }
}
pub(super) fn decode_permissions(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
    sum: u32,
) -> Result<Box<[domain::PermissionRole]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values =
        List::with_capacity(count(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size"))?);
    let mut used = 0;
    while let Some(bytes) = split(&mut reader)? {
        let value = wire::PermissionRole::decode(limits, &mut Reader::new(&bytes))?;
        let value = decode_permission_role(value);
        spend(&mut used, u64::try_from(size_of::<domain::PermissionRole>()).ok(), sum)?;
        values.push(value).expect("piece contains at most one item per length");
    }
    Ok(values.into_boxed())
}

pub(super) fn split(reader: &mut Reader<'_>) -> Result<Option<Box<[u8]>>, wire::Problem> {
    match skein_codec::piece::split(reader) {
        Ok(value) => Ok(value),
        Err(reason) => Err(wire::Problem { path: wire::Path::RowValue, reason }),
    }
}

pub(super) fn shared_sum(limits: &wire::Limits) -> u32 {
    let framing = wire::Grant::worst_case_bytes(&EMPTY)
        .expect("fixed grant")
        .checked_add(u64::try_from(size_of::<u32>()).expect("length size"))
        .expect("fixed framing");
    let overhead = u64::try_from(size_of::<domain::Grant>())
        .expect("grant size")
        .checked_sub(framing)
        .expect("inline grant is larger than its framing");
    limits.authority_grants.checked_add(u32::try_from(overhead).expect("grant overhead")).expect("schema bounds sum")
}
pub(super) fn permission_sum(limits: &wire::Limits) -> u32 {
    let framing = wire::PermissionRole::worst_case_bytes(&EMPTY)
        .expect("fixed permission")
        .checked_add(u64::try_from(size_of::<u32>()).expect("length size"))
        .expect("fixed framing");
    limits
        .policy_value_permissions
        .checked_div(u32::try_from(framing).expect("permission framing"))
        .expect("nonzero framing")
        .checked_mul(u32::try_from(size_of::<domain::PermissionRole>()).expect("permission size"))
        .expect("decoded size is no larger than its piece")
}
