const LIMITS: skein_ui::Limits = skein_ui::Limits {
    nodes: 12,
    depth: 4,
    text_per_node: 32,
    text_bytes: 128,
    drawing_bytes: 8,
    styles_per_node: 2,
    patches: 72,
    bindings: 4,
    slots: 4,
    markdown_depth: 2,
    scene_marks: 0,
    scene_bytes: 0,
};

#[test]
fn jig_worst_case_adds_the_page_scratch_to_skein_uis() {
    let ui = skein_ui::worst_case(&LIMITS).expect("small UI limits fit");
    let pages = crate::pages::scratch(&LIMITS).expect("small page limits fit");
    assert_eq!(crate::limits::jig_worst_case(&LIMITS), Some(ui + pages));
}
