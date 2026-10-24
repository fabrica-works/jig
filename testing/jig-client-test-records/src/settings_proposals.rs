//! Populated settings proposals shared by both wire ends (contract.md, 3.2 and 5.1).
use jig_client::contract as c;
use skein_lib::List;
/// A review's oversized value is described by its size, beyond wire 19's document ceiling.
pub const OVERSIZED_VALUE: u64 = 16_777_217;
#[must_use]
pub fn proposal() -> c::ProposeSettings {
    c::ProposeSettings { against: 6, items: super::settings::settings_change().items }
}
#[must_use]
pub fn heads() -> List<c::RevisionItemHead> {
    let mut items = List::with_capacity(3);
    for (setting, edit) in [
        (c::Setting::Rules, c::RevisionItemEdit::Set),
        (c::Setting::Charter(c::SettingNumber { number: 2 }), c::RevisionItemEdit::Removed),
        (c::Setting::Template(c::SettingNumber { number: 3 }), c::RevisionItemEdit::Reset),
    ] {
        items.push(c::RevisionItemHead { setting, edit }).expect("three heads");
    }
    items
}
#[must_use]
pub fn review() -> c::SettingsReview {
    let mut items = List::with_capacity(3);
    let mut values = super::settings::settings_values().into_boxed().into_vec();
    let value = c::SettingSet { value: values.remove(0) };
    for (origin, now, would_be, edit) in [
        (
            c::SettingOrigin::Default(c::OriginRevision { revision: 2 }),
            c::ReviewValue::Value(value.clone()),
            c::ReviewValue::TooLarge(c::Size { bytes: OVERSIZED_VALUE }),
            c::RevisionItemEdit::Set,
        ),
        (
            c::SettingOrigin::Change(c::OriginRevision { revision: 4 }),
            c::ReviewValue::TooLarge(c::Size { bytes: OVERSIZED_VALUE }),
            c::ReviewValue::None,
            c::RevisionItemEdit::Removed,
        ),
        (c::SettingOrigin::NotOffered, c::ReviewValue::None, c::ReviewValue::Value(value), c::RevisionItemEdit::Reset),
    ] {
        items
            .push(c::ReviewItem {
                setting: c::Setting::Charter(c::SettingNumber { number: 2 }),
                edit,
                now,
                origin,
                would_be,
            })
            .expect("three review items");
    }
    c::SettingsReview { newest: 6, items, cut: true }
}
#[must_use]
pub fn pending() -> c::OwnerProposalActionSettings {
    c::OwnerProposalActionSettings { against: 6, items: heads(), review: Some(review()), made: None }
}
#[must_use]
pub fn retained() -> c::OwnerRetainedActionSettings {
    c::OwnerRetainedActionSettings { against: 6, items: heads(), made: Some(7) }
}
