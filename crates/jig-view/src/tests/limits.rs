use crate::ui::tests::fixtures::LIMITS;

#[test]
fn jig_worst_case_adds_the_page_scratch_to_skein_uis() {
    let ui = crate::ui::worst_case(&LIMITS).expect("small UI limits fit");
    let pages = crate::pages::scratch(&LIMITS).expect("small page limits fit");
    assert_eq!(crate::limits::jig_worst_case(&LIMITS), Some(ui + pages));
}
