#![expect(clippy::disallowed_macros, reason = "ordinary test code constructs variable language data")]
use crate::Owner;
use crate::phrases::{
    ChipSpan, Form, Hole, HoleKind, HoleShape, Invalid, Language, Phrase, PhraseId, PhraseLimits, PhraseShape,
    Phrasebook, PluralClass, PluralRule, Segment, SpanUnit, WallFormat,
};
use skein_lib::{Duration, Wall};

const ID: PhraseId = PhraseId { owner: Owner::Other { child: 4 }, number: 1 };
const LIMITS: PhraseLimits =
    PhraseLimits { languages: 2, phrases: 2, holes: 6, segments: 60, text_bytes: 200, hole_bytes: 8 };
fn shape() -> Box<[PhraseShape]> {
    Box::new([PhraseShape {
        id: ID,
        holes: Box::new([
            HoleShape { kind: HoleKind::Number, bytes: 0 },
            HoleShape { kind: HoleKind::Name, bytes: 4 },
            HoleShape { kind: HoleKind::Amount, bytes: 0 },
            HoleShape { kind: HoleKind::Span, bytes: 0 },
            HoleShape { kind: HoleKind::Wall, bytes: 0 },
            HoleShape { kind: HoleKind::Chip, bytes: 8 },
        ]),
        plural: Some(0),
    }])
}
fn language(long: bool) -> Language {
    let forms = [PluralClass::One, PluralClass::Other].map(|class| {
        let prefix: &[u8] = if long {
            b"longer: "
        } else if class == PluralClass::One {
            b"one: "
        } else {
            b"many: "
        };
        let mut segments = vec![Segment::Text(prefix.into())];
        let kinds =
            [HoleKind::Number, HoleKind::Name, HoleKind::Amount, HoleKind::Span, HoleKind::Wall, HoleKind::Chip];
        let indices = if long { [5, 4, 3, 2, 1, 0] } else { [0, 1, 2, 3, 4, 5] };
        for index in indices {
            segments.push(Segment::Hole { index, kind: kinds[usize::from(index)] });
            segments.push(Segment::Text(b" ".as_slice().into()));
        }
        Form { class, segments: segments.into_boxed_slice() }
    });
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Nanos,
        wall: WallFormat::Nanos,
        phrases: Box::new([Phrase { id: ID, forms: Box::new(forms) }]),
    }
}

#[test]
fn loading_refuses_missing_phrases_wrong_holes_and_incomplete_forms() {
    let mut missing = language(false);
    missing.phrases = Box::new([]);
    assert_eq!(Phrasebook::load(shape(), Box::new([missing]), &LIMITS).unwrap_err(), Invalid::Missing { id: ID });
    let mut wrong = language(false);
    wrong.phrases[0].forms[0].segments[1] = Segment::Hole { index: 0, kind: HoleKind::Amount };
    assert_eq!(Phrasebook::load(shape(), Box::new([wrong]), &LIMITS).unwrap_err(), Invalid::Holes { id: ID });
    let mut incomplete = language(false);
    incomplete.phrases[0].forms[0].class = PluralClass::Other;
    assert_eq!(Phrasebook::load(shape(), Box::new([incomplete]), &LIMITS).unwrap_err(), Invalid::Forms { id: ID });
    let mut duplicate = language(false);
    duplicate.phrases[0].forms[0].segments[1] = Segment::Hole { index: 1, kind: HoleKind::Name };
    assert_eq!(Phrasebook::load(shape(), Box::new([duplicate]), &LIMITS).unwrap_err(), Invalid::Holes { id: ID });
}

#[test]
fn plural_selection_hole_values_and_longest_language_capacity_are_checked() {
    let book = Phrasebook::load(shape(), Box::new([language(false), language(true)]), &LIMITS).unwrap();
    let mut values = [
        Hole::Number(1),
        Hole::Name(b"name".as_slice().into()),
        Hole::Amount(3),
        Hole::Span(Duration::from_nanos(4)),
        Hole::Wall(Wall::from_nanos(5)),
        Hole::Chip { task: 9, title: b"a *chip*".as_slice().into() },
    ];
    let singular = book.say(0, ID, &values).unwrap();
    assert_eq!(singular.text.as_ref(), b"one: 1 name 3 4 5 a *chip* ");
    assert_eq!(singular.chips.as_ref(), [ChipSpan { start: 18, end: 26, task: 9 }]);
    values[0] = Hole::Number(0);
    assert!(book.say(0, ID, &values).unwrap().text.starts_with(b"many: 0"));
    assert!(book.say(1, ID, &values).unwrap().text.starts_with(b"longer: a *chip* 5 4 3 name 0"));
    values[0] = Hole::Number(u64::MAX);
    values[2] = Hole::Amount(u64::MAX);
    values[3] = Hole::Span(Duration::from_nanos(u64::MAX));
    values[4] = Hole::Wall(Wall::from_nanos(u64::MAX));
    let longest = book.say(1, ID, &values).unwrap();
    assert_eq!(u32::try_from(longest.text.len()).unwrap(), book.capacity(ID).unwrap());
    values[1] = Hole::Name(b"long name".as_slice().into());
    assert_eq!(book.say(0, ID, &values), Err(Invalid::Values { id: ID }));
    values[1] = Hole::Amount(5);
    assert_eq!(book.say(0, ID, &values), Err(Invalid::Values { id: ID }));
    assert_eq!(book.say(2, ID, &[]), Err(Invalid::Values { id: ID }));
    let mut clock = language(false);
    clock.wall = WallFormat::Clock;
    clock.span_unit = SpanUnit::Seconds;
    let clock_book = Phrasebook::load(shape(), Box::new([clock]), &LIMITS).unwrap();
    let clock_values = [
        Hole::Number(1),
        Hole::Name(b"name".as_slice().into()),
        Hole::Amount(3),
        Hole::Span(Duration::from_secs(4)),
        Hole::Wall(Wall::from_nanos(3_720_000_000_000)),
        Hole::Chip { task: 9, title: b"a *chip*".as_slice().into() },
    ];
    assert_eq!(clock_book.say(0, ID, &clock_values).unwrap().text.as_ref(), b"one: 1 name 3 4 01:02 a *chip* ");
    assert_eq!(clock_book.say(1, ID, &clock_values), Err(Invalid::Language));
    let count_shape = Box::new([PhraseShape {
        id: ID,
        holes: Box::new([HoleShape { kind: HoleKind::Number, bytes: 0 }]),
        plural: Some(0),
    }]);
    let count_language = Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Nanos,
        wall: WallFormat::Nanos,
        phrases: Box::new([Phrase {
            id: ID,
            forms: Box::new([
                Form {
                    class: PluralClass::One,
                    segments: Box::new([
                        Segment::Text(b"a very long singular prefix: ".as_slice().into()),
                        Segment::Hole { index: 0, kind: HoleKind::Number },
                    ]),
                },
                Form {
                    class: PluralClass::Other,
                    segments: Box::new([Segment::Hole { index: 0, kind: HoleKind::Number }]),
                },
            ]),
        }]),
    };
    let count_book = Phrasebook::load(count_shape, Box::new([count_language]), &LIMITS).unwrap();
    assert_eq!(
        count_book.capacity(ID),
        Some(u32::try_from(count_book.say(0, ID, &[Hole::Number(1)]).unwrap().text.len()).unwrap())
    );
    assert!(crate::phrases::worst_case(&LIMITS).unwrap() > 200);
}

#[test]
fn shuffled_languages_keep_owner_scoped_ids_capacities_and_whole_load_refusals() {
    let rows = [
        (PhraseId { owner: Owner::Other { child: u16::MAX }, number: 0 }, b"other maximum".as_slice()),
        (PhraseId { owner: Owner::Jig, number: u16::MAX }, b"jig maximum".as_slice()),
        (PhraseId { owner: Owner::Side { connector: 5 }, number: 1 }, b"side five".as_slice()),
        (PhraseId { owner: Owner::Other { child: 4 }, number: 1 }, b"other four".as_slice()),
        (PhraseId { owner: Owner::Jig, number: 1 }, b"jig one".as_slice()),
        (PhraseId { owner: Owner::Side { connector: 4 }, number: 1 }, b"side four".as_slice()),
    ];
    let shapes = rows
        .iter()
        .map(|(id, _)| PhraseShape { id: *id, holes: Box::new([]), plural: None })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let language = |long: bool| Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Seconds,
        wall: WallFormat::Clock,
        phrases: rows
            .iter()
            .rev()
            .map(|(id, bytes)| Phrase {
                id: *id,
                forms: Box::new([Form {
                    class: PluralClass::Other,
                    segments: if long {
                        Box::new([Segment::Text(bytes.to_vec().repeat(2).into_boxed_slice())])
                    } else {
                        Box::new([Segment::Text((*bytes).into())])
                    },
                }]),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    };
    let limits = PhraseLimits { languages: 2, phrases: 6, holes: 0, segments: 12, text_bytes: 256, hole_bytes: 0 };
    let book = Phrasebook::load(shapes.clone(), Box::new([language(false), language(true)]), &limits).unwrap();
    for (id, bytes) in rows {
        assert_eq!(book.say(0, id, &[]).unwrap().text.as_ref(), bytes);
        assert_eq!(book.say(1, id, &[]).unwrap().text.as_ref(), bytes.to_vec().repeat(2));
        assert_eq!(book.capacity(id), Some(u32::try_from(bytes.len().checked_mul(2).unwrap()).unwrap()));
        assert_eq!(book.clone().say(1, id, &[]), book.say(1, id, &[]));
    }
    let absent = PhraseId { owner: Owner::Other { child: 5 }, number: 1 };
    assert_eq!(book.capacity(absent), None);
    assert_eq!(book.say(0, absent, &[]), Err(Invalid::Unknown { id: absent }));
    assert_eq!(book.jig_capacity(), None, "partial jig words do not compose a complete jig page");
    let mut duplicate_shapes = shapes.clone();
    duplicate_shapes[4].id = duplicate_shapes[0].id;
    assert_eq!(
        Phrasebook::load(duplicate_shapes, Box::new([language(false)]), &limits).unwrap_err(),
        Invalid::Duplicate { id: shapes[0].id },
    );
    let mut duplicate_language = language(false);
    let duplicate = duplicate_language.phrases[0].id;
    duplicate_language.phrases[4].id = duplicate;
    assert_eq!(
        Phrasebook::load(shapes.clone(), Box::new([duplicate_language]), &limits).unwrap_err(),
        Invalid::Duplicate { id: duplicate },
    );
    let mut unknown_language = language(false);
    unknown_language.phrases[4].id = absent;
    assert_eq!(
        Phrasebook::load(shapes, Box::new([unknown_language]), &limits).unwrap_err(),
        Invalid::Unknown { id: absent },
    );
}
