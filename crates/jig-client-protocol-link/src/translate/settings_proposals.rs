//! Settings proposal and review facts (client/domain/contract.md, 3.2 and 5.1).
use super::settings::{
    restore_revision_item_edit, restore_revision_item_head, restore_setting, restore_setting_item, restore_setting_set,
    restore_size, wire_revision_item_edit, wire_revision_item_head, wire_setting, wire_setting_item, wire_setting_set,
    wire_size,
};
use crate::Limits;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v19 as w;
use skein_lib::{List, Reader, Writer};
pub fn wire_origin_revision(value: &c::OriginRevision, limits: &w::Limits) -> Result<w::OriginRevision, w::Problem> {
    w::OriginRevision::new(limits, w::OriginRevisionParts { revision: value.revision })
}
pub fn restore_origin_revision(value: &w::OriginRevision, _limits: &Limits) -> Result<c::OriginRevision, w::Problem> {
    Ok(c::OriginRevision { revision: value.revision() })
}
pub fn wire_propose_settings(value: &c::ProposeSettings, limits: &w::Limits) -> Result<w::ProposeSettings, w::Problem> {
    w::ProposeSettings::new(
        limits,
        w::ProposeSettingsParts { against: value.against, items: encode_setting_pieces(&value.items, limits)? },
    )
}
pub fn restore_propose_settings(value: &w::ProposeSettings, limits: &Limits) -> Result<c::ProposeSettings, w::Problem> {
    Ok(c::ProposeSettings { against: value.against(), items: decode_setting_pieces(value.items(), limits)? })
}
pub fn wire_review_item(value: &c::ReviewItem, limits: &w::Limits) -> Result<w::ReviewItem, w::Problem> {
    w::ReviewItem::new(
        limits,
        w::ReviewItemParts {
            setting: wire_setting(&value.setting, limits)?,
            edit: wire_revision_item_edit(&value.edit, limits)?,
            now: wire_review_value(&value.now, limits)?,
            origin: wire_setting_origin(&value.origin, limits)?,
            would_be: wire_review_value(&value.would_be, limits)?,
        },
    )
}
pub fn restore_review_item(value: &w::ReviewItem, limits: &Limits) -> Result<c::ReviewItem, w::Problem> {
    Ok(c::ReviewItem {
        setting: restore_setting(value.setting(), limits)?,
        edit: restore_revision_item_edit(value.edit(), limits)?,
        now: restore_review_value(value.now(), limits)?,
        origin: restore_setting_origin(value.origin(), limits)?,
        would_be: restore_review_value(value.would_be(), limits)?,
    })
}
pub fn wire_settings_review(value: &c::SettingsReview, limits: &w::Limits) -> Result<w::SettingsReview, w::Problem> {
    w::SettingsReview::new(
        limits,
        w::SettingsReviewParts {
            newest: value.newest,
            items: encode_review_pieces(&value.items, limits)?,
            cut: value.cut,
        },
    )
}
pub fn restore_settings_review(value: &w::SettingsReview, limits: &Limits) -> Result<c::SettingsReview, w::Problem> {
    Ok(c::SettingsReview {
        newest: value.newest(),
        items: decode_review_pieces(value.items(), limits)?,
        cut: value.cut(),
    })
}
pub fn wire_owner_proposal_action_settings(
    value: &c::OwnerProposalActionSettings,
    limits: &w::Limits,
) -> Result<w::OwnerProposalActionSettings, w::Problem> {
    w::OwnerProposalActionSettings::new(
        limits,
        w::OwnerProposalActionSettingsParts {
            against: value.against,
            items: encode_heads(&value.items, limits, limits.owner_proposal_action_settings_items)?,
            review: match &value.review {
                Some(item) => Some(wire_settings_review(item, limits)?),
                None => None,
            },
            made: value.made,
        },
    )
}
pub fn restore_owner_proposal_action_settings(
    value: &w::OwnerProposalActionSettings,
    limits: &Limits,
) -> Result<c::OwnerProposalActionSettings, w::Problem> {
    Ok(c::OwnerProposalActionSettings {
        against: value.against(),
        items: decode_heads(value.items(), limits, limits.wire.owner_proposal_action_settings_items)?,
        review: match value.review() {
            Some(item) => Some(restore_settings_review(item, limits)?),
            None => None,
        },
        made: *value.made(),
    })
}
pub fn wire_owner_retained_action_settings(
    value: &c::OwnerRetainedActionSettings,
    limits: &w::Limits,
) -> Result<w::OwnerRetainedActionSettings, w::Problem> {
    w::OwnerRetainedActionSettings::new(
        limits,
        w::OwnerRetainedActionSettingsParts {
            against: value.against,
            items: encode_heads(&value.items, limits, limits.owner_retained_action_settings_items)?,
            made: value.made,
        },
    )
}
pub fn restore_owner_retained_action_settings(
    value: &w::OwnerRetainedActionSettings,
    limits: &Limits,
) -> Result<c::OwnerRetainedActionSettings, w::Problem> {
    Ok(c::OwnerRetainedActionSettings {
        against: value.against(),
        items: decode_heads(value.items(), limits, limits.wire.owner_retained_action_settings_items)?,
        made: *value.made(),
    })
}
pub fn wire_setting_origin(value: &c::SettingOrigin, limits: &w::Limits) -> Result<w::SettingOrigin, w::Problem> {
    Ok(match value {
        c::SettingOrigin::Default(item) => w::SettingOrigin::Default(wire_origin_revision(item, limits)?),
        c::SettingOrigin::Change(item) => w::SettingOrigin::Change(wire_origin_revision(item, limits)?),
        c::SettingOrigin::NotOffered => w::SettingOrigin::NotOffered,
    })
}
pub fn restore_setting_origin(value: &w::SettingOrigin, limits: &Limits) -> Result<c::SettingOrigin, w::Problem> {
    Ok(match value {
        w::SettingOrigin::Default(item) => c::SettingOrigin::Default(restore_origin_revision(item, limits)?),
        w::SettingOrigin::Change(item) => c::SettingOrigin::Change(restore_origin_revision(item, limits)?),
        w::SettingOrigin::NotOffered => c::SettingOrigin::NotOffered,
    })
}
pub fn wire_review_value(value: &c::ReviewValue, limits: &w::Limits) -> Result<w::ReviewValue, w::Problem> {
    Ok(match value {
        c::ReviewValue::Value(item) => w::ReviewValue::Value(wire_setting_set(item, limits)?),
        c::ReviewValue::TooLarge(item) => w::ReviewValue::TooLarge(wire_size(item, limits)?),
        c::ReviewValue::None => w::ReviewValue::None,
    })
}
pub fn restore_review_value(value: &w::ReviewValue, limits: &Limits) -> Result<c::ReviewValue, w::Problem> {
    Ok(match value {
        w::ReviewValue::Value(item) => c::ReviewValue::Value(restore_setting_set(item, limits)?),
        w::ReviewValue::TooLarge(item) => c::ReviewValue::TooLarge(restore_size(item, limits)?),
        w::ReviewValue::None => c::ReviewValue::None,
    })
}
fn encode_heads(
    values: &List<c::RevisionItemHead>,
    limits: &w::Limits,
    bound: u32,
) -> Result<List<w::RevisionItemHead>, w::Problem> {
    if values.len() > bound {
        return Err(super::too_large());
    }
    let mut rows = List::with_capacity(values.len());
    for value in values {
        rows.push(wire_revision_item_head(value, limits)?).expect("same capacity");
    }
    Ok(rows)
}
fn decode_heads(
    values: &List<w::RevisionItemHead>,
    limits: &Limits,
    bound: u32,
) -> Result<List<c::RevisionItemHead>, w::Problem> {
    if values.len() > bound {
        return Err(super::too_large());
    }
    let mut rows = List::with_capacity(values.len());
    for value in values {
        rows.push(restore_revision_item_head(value, limits)?).expect("same capacity");
    }
    Ok(rows)
}
fn encode_setting_pieces(values: &List<c::SettingItem>, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    if values.len() > 128 {
        return Err(super::too_large());
    }
    let mut rows = List::with_capacity(values.len());
    let mut size = 0_u32;
    for value in values {
        let row = wire_setting_item(value, limits)?;
        size = size.checked_add(row.measure()).ok_or(super::too_large())?.checked_add(4).ok_or(super::too_large())?;
        if size > limits.propose_settings_items {
            return Err(super::too_large());
        }
        rows.push(row).expect("same capacity");
    }
    let mut writer = Writer::new(usize::try_from(size).expect("wire size fits"));
    for row in rows.into_boxed() {
        writer.put(&row.measure().to_be_bytes()).expect("measured");
        row.encode(&mut writer).expect("measured");
    }
    Ok(writer.finish())
}
fn decode_setting_pieces(values: &[u8], limits: &Limits) -> Result<List<c::SettingItem>, w::Problem> {
    if values.len() > usize::try_from(limits.wire.propose_settings_items).expect("wire bytes fit") {
        return Err(super::too_large());
    }
    let count = piece_count(values)?;
    let mut reader = Reader::new(values);
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        let piece = piece(&mut reader)?.ok_or(super::too_large())?;
        let mut input = Reader::new(&piece);
        let row = w::SettingItem::decode(&limits.wire, &mut input)?;
        if !input.is_empty() {
            return Err(super::too_large());
        }
        rows.push(restore_setting_item(&row, limits)?).expect("counted pieces");
    }
    Ok(rows)
}
fn encode_review_pieces(values: &List<c::ReviewItem>, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    if values.len() > 128 {
        return Err(super::too_large());
    }
    let mut rows = List::with_capacity(values.len());
    let mut size = 0_u32;
    for value in values {
        let row = wire_review_item(value, limits)?;
        size = size.checked_add(row.measure()).ok_or(super::too_large())?.checked_add(4).ok_or(super::too_large())?;
        if size > limits.settings_review_items {
            return Err(super::too_large());
        }
        rows.push(row).expect("same capacity");
    }
    let mut writer = Writer::new(usize::try_from(size).expect("wire size fits"));
    for row in rows.into_boxed() {
        writer.put(&row.measure().to_be_bytes()).expect("measured");
        row.encode(&mut writer).expect("measured");
    }
    Ok(writer.finish())
}
fn decode_review_pieces(values: &[u8], limits: &Limits) -> Result<List<c::ReviewItem>, w::Problem> {
    if values.len() > usize::try_from(limits.wire.settings_review_items).expect("wire bytes fit") {
        return Err(super::too_large());
    }
    let count = piece_count(values)?;
    let mut reader = Reader::new(values);
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        let piece = piece(&mut reader)?.ok_or(super::too_large())?;
        let mut input = Reader::new(&piece);
        let row = w::ReviewItem::decode(&limits.wire, &mut input)?;
        if !input.is_empty() {
            return Err(super::too_large());
        }
        rows.push(restore_review_item(&row, limits)?).expect("counted pieces");
    }
    Ok(rows)
}
fn piece_count(values: &[u8]) -> Result<u32, w::Problem> {
    let mut reader = Reader::new(values);
    for count in 0_u32..=128 {
        if piece(&mut reader)?.is_none() {
            return Ok(count);
        }
    }
    Err(super::too_large())
}
fn piece(reader: &mut Reader<'_>) -> Result<Option<Box<[u8]>>, w::Problem> {
    match skein_codec::piece::split(reader) {
        Ok(value) => Ok(value),
        Err(reason) => Err(super::piece_problem(reason)),
    }
}
