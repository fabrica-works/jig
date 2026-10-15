use crate::{Owner, ui};

#[test]
fn jig_maps_to_one_past_every_application_number() {
    assert_eq!(Owner::Jig.opaque(), ui::Owner { number: 65_536 });
    assert_eq!(Owner::from_opaque(ui::Owner { number: 65_536 }), Some(Owner::Jig));
}

#[test]
fn an_application_number_maps_to_itself_and_back() {
    for number in [0, 1, 42, u16::MAX] {
        let owner = Owner::Application { number };
        assert_eq!(owner.opaque(), ui::Owner { number: u32::from(number) });
        assert_eq!(Owner::from_opaque(ui::Owner { number: u32::from(number) }), Some(owner));
    }
}

#[test]
fn a_number_past_jig_names_no_owner() {
    for number in [65_537, u32::MAX] {
        assert_eq!(Owner::from_opaque(ui::Owner { number }), None);
    }
}
