use skein_ui::phrases::{Language, PhraseLimits, Phrasebook, PluralRule, SpanUnit, WallFormat};

#[test]
fn a_partial_book_has_no_jig_capacity() {
    let language = Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Seconds,
        wall: WallFormat::Clock,
        phrases: Box::new([]),
    };
    let limits = PhraseLimits { languages: 1, phrases: 0, holes: 0, segments: 0, text_bytes: 0, hole_bytes: 0 };
    let book = Phrasebook::load(Box::new([]), Box::new([language]), &limits).unwrap();
    assert_eq!(crate::phrases::jig_capacity(&book), None, "partial jig words do not compose a complete jig page");
}

#[test]
fn english_loads_beside_the_reference_ui_s_phrases_in_one_phrasebook() {
    let book = crate::phrases::english::english(32).expect("both owners load together");
    assert_eq!(
        book.say(0, jig_client::words::Word::Phase(jig_client::contract::Phase::Held).id(), &[]).unwrap().text.as_ref(),
        b"Held"
    );
    assert_eq!(book.say(0, crate::phrases::jig::JigPhrase::Inbox.id(), &[]).unwrap().text.as_ref(), b"Inbox");
    assert!(crate::phrases::jig_capacity(&book).is_some());
}
