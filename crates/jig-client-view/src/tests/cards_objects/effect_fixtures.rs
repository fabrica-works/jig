use super::*;

pub(super) fn offered_acts(kinds: &[ActKind], release: ReleaseEffect) -> List<Offered> {
    let mut rows: Vec<_> = kinds
        .iter()
        .map(|act| Offered {
            act: *act,
            admission: Admission::Allowed,
            effect: match act {
                ActKind::Release => Some(Effect::Release(release)),
                ActKind::Cancel => Some(Effect::Cancel(impact())),
                ActKind::Move => Some(Effect::Move { reservation: 13 }),
                _ => None,
            },
        })
        .collect();
    if kinds.contains(&ActKind::Amend) && !kinds.contains(&ActKind::Release) {
        rows.push(Offered {
            act: ActKind::Release,
            admission: Admission::Allowed,
            effect: Some(Effect::Release(release)),
        });
    }
    list(rows)
}
