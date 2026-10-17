use super::*;
const PROJECT: u32 = 1;
const CONNECTOR: u16 = 1;

#[test]
fn a_resource_notes_filter_keeps_the_whole_path_behind_its_cut_label() {
    let words = crate::tests::cards::shared_words();
    let client = crate::tests::cards::CLIENT;
    let path = crate::tests::fixtures::long_resource_path::<128>();
    let pattern = jig_client::contract::exact_resource_pattern(&path).unwrap();
    let scope = NoteScope::Resource { project: PROJECT, connector: CONNECTOR, pattern };
    let expected = NoteFilter { scope: scope.clone(), description: None };
    let capacity = crate::limits::of_words(&client, words).unwrap();
    let mut b = Builder::new(&capacity);
    let mut bindings = JigBindings::new(capacity.bindings, client.text);
    scope_choice(
        &mut b,
        words,
        &mut bindings,
        &NoteFilter { scope: NoteScope::Project(PROJECT), description: None },
        scope,
    );
    let tree = b.finish();
    let mut cut = false;
    let mut button = None;
    for node in tree.nodes() {
        for run in &node.text {
            if run.text.as_ref() == "…".as_bytes() {
                cut = true;
            }
        }
        if node.binding.is_some() {
            button = node.binding;
        }
    }
    assert!(cut);
    let button = button.unwrap();
    assert_eq!(
        crate::binding::decode(&mut bindings, button.token, skein_ui::Act::Press),
        Some(Act::FilterNotes { filter: expected })
    );
}
