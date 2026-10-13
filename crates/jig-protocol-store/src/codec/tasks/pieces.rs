//! Verdict pieces retain the domain's byte sum (codec.md, 3; store.md, 5.1).
use super::{Box, Reader, Writer, domain, wire};
use skein_lib::List;
fn admit(used: &mut usize, name: &[u8], extra: usize, limit: u32, path: wire::Path) -> Result<(), wire::Problem> {
    *used = used.saturating_add(name.len()).saturating_add(extra);
    if name.is_empty() || *used > usize::try_from(limit).expect("bound fits usize") {
        return Err(wire::Problem { path, reason: skein_codec::Reason::Bound });
    }
    Ok(())
}
fn split(reader: &mut Reader<'_>, path: wire::Path) -> Result<Option<Box<[u8]>>, wire::Problem> {
    match skein_codec::piece::split(reader) {
        Ok(value) => Ok(value),
        Err(reason) => Err(wire::Problem { path, reason }),
    }
}

pub(super) fn fields(
    values: &[domain::ResultField],
    limits: &wire::Limits,
    bound: u32,
    path: wire::Path,
) -> Result<Box<[u8]>, wire::Problem> {
    let mut items = List::with_capacity(u32::try_from(values.len()).expect("bounded field count"));
    let mut used = 0;
    for field in values {
        let domain::ResultField { name, value } = field;
        admit(&mut used, name, value.len(), limits.result_field_value, path)?;
        let field =
            wire::ResultField::new(limits, wire::ResultFieldParts { name: name.clone(), value: value.clone() })?;
        let mut item = Writer::new(usize::try_from(field.measure()).expect("measured size"));
        field.encode(&mut item).expect("measured field");
        items.push(item.finish()).expect("same field count");
    }
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(reason) => Err(wire::Problem { path, reason }),
    }
}
pub(super) fn read_fields(
    bytes: Box<[u8]>,
    limits: &wire::Limits,
    path: wire::Path,
) -> Result<Box<[domain::ResultField]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values = List::with_capacity(
        u32::try_from(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size")).expect("bounded piece"),
    );
    let mut used = 0;
    while let Some(bytes) = split(&mut reader, path)? {
        let wire::ResultFieldParts { name, value } =
            wire::ResultField::decode(limits, &mut Reader::new(&bytes))?.into_parts();
        admit(&mut used, &name, value.len(), limits.result_field_value, path)?;
        values.push(domain::ResultField { name, value }).expect("piece bounds item count");
    }
    Ok(values.into_boxed())
}

pub(super) fn rules(values: &[domain::FieldRule], limits: &wire::Limits) -> Result<Box<[u8]>, wire::Problem> {
    let mut items = List::with_capacity(u32::try_from(values.len()).expect("bounded field count"));
    let mut used = 0;
    for field in values {
        let domain::FieldRule { name, max } = field;
        admit(
            &mut used,
            name,
            usize::try_from(*max).expect("field bound fits usize"),
            limits.field_rule_name,
            wire::Path::VerdictFields,
        )?;
        let field = wire::FieldRule::new(limits, wire::FieldRuleParts { name: name.clone(), max: *max })?;
        let mut item = Writer::new(usize::try_from(field.measure()).expect("measured size"));
        field.encode(&mut item).expect("measured field");
        items.push(item.finish()).expect("same field count");
    }
    match crate::codec::pieces::join(items, limits.verdict_fields) {
        Ok(bytes) => Ok(bytes),
        Err(reason) => Err(wire::Problem { path: wire::Path::VerdictFields, reason }),
    }
}
pub(super) fn read_rules(bytes: Box<[u8]>, limits: &wire::Limits) -> Result<Box<[domain::FieldRule]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut values = List::with_capacity(
        u32::try_from(bytes.len().checked_div(size_of::<u32>()).expect("nonzero length size")).expect("bounded piece"),
    );
    let mut used = 0;
    while let Some(bytes) = split(&mut reader, wire::Path::VerdictFields)? {
        let wire::FieldRuleParts { name, max } =
            wire::FieldRule::decode(limits, &mut Reader::new(&bytes))?.into_parts();
        admit(
            &mut used,
            &name,
            usize::try_from(max).expect("field bound fits usize"),
            limits.field_rule_name,
            wire::Path::VerdictFields,
        )?;
        values.push(domain::FieldRule { name, max }).expect("piece bounds item count");
    }
    Ok(values.into_boxed())
}
