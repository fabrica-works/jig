//! Fresh named authority admission follows people's retained-key lookup (store.md, 5.1).
use alloc::boxed::Box;
use jig_domain_people as people;

pub(crate) fn resolve(ask: &mut people::Ask, names: &[Box<[u8]>], models: &[crate::Model]) -> bool {
    if !people::valid_model_context(ask, names) {
        return false;
    }
    let mut moved = [0_u64; 64];
    for (old, name) in names.iter().enumerate() {
        let mut found = false;
        for (at, model) in models.iter().enumerate() {
            if *name == model.name {
                *moved.get_mut(old).expect("bounded original names") =
                    1_u64.checked_shl(u32::try_from(at).expect("model index")).unwrap_or(0);
                found = true;
                break;
            }
        }
        if !found {
            return false;
        }
    }
    people::remap_models(ask, &moved);
    true
}
