use alloc::boxed::Box;
use skein_ui::Act;

#[test]
fn jigs_view_decodes_an_act_on_a_drawing_into_nothing() {
    let mut bindings = crate::JigBindings::new(4, 16);
    for meaning in [
        crate::JigMeaning::TaskNumber,
        crate::JigMeaning::Priority,
        crate::JigMeaning::Action(crate::action::Act::Go { address: crate::address::Address::Inbox { project: None } }),
        crate::JigMeaning::ProjectPoolBudget { project: 1, party: 2 },
    ] {
        let binding = bindings.insert(meaning);
        assert_eq!(crate::decode(&mut bindings, binding.token, Act::Drawing { data: Box::from(*b"a") }), None);
    }
}
