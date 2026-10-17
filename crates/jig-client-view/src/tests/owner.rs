use crate::Owner;

#[test]
fn jig_maps_to_one_past_every_application_number() {
    assert_eq!(Owner::Jig.opaque(), skein_ui::Owner { number: 65_536 });
    assert_eq!(Owner::from_opaque(skein_ui::Owner { number: 65_536 }), Some(Owner::Jig));
}

#[test]
fn an_application_number_maps_to_itself_and_back() {
    for number in [0, 1, 42, u16::MAX] {
        let owner = Owner::Application { number };
        assert_eq!(owner.opaque(), skein_ui::Owner { number: u32::from(number) });
        assert_eq!(Owner::from_opaque(skein_ui::Owner { number: u32::from(number) }), Some(owner));
    }
}

#[test]
fn a_number_past_jig_names_no_owner() {
    for number in [65_537, u32::MAX] {
        assert_eq!(Owner::from_opaque(skein_ui::Owner { number }), None);
    }
}

#[test]
fn jig_s_owner_is_the_asker_one_past_every_u16() {
    assert_eq!(Owner::Jig.asker(), jig_client::Asker(65_536));
    assert_eq!(Owner::of_asker(jig_client::Asker(65_536)), Some(Owner::Jig));
}

#[test]
fn an_asker_past_jig_s_is_no_owner() {
    for number in [65_537, u32::MAX] {
        assert_eq!(Owner::of_asker(jig_client::Asker(number)), None);
    }
}
