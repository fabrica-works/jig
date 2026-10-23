use super::*;
use skein_lib::{List, Writer};
use smith_transcript::v6 as smith;

// The golden's first text is "abc"; the second fills another three bytes.
const FIELD_TEXT: u32 = 3;
const TURN_TEXT: u32 = 6;
const TEXT_BLOCKS: u32 = 2; // The golden's text followed by the added text.

// The golden turn with a second text block appended.
fn turn_with(second: &[u8]) -> Box<[u8]> {
    let source = smith::Turn::decode(&smith::CEILINGS, &mut Reader::new(fixtures::FULL_TURN)).unwrap();
    let mut source = source.into_parts();
    let message = source.messages.get_mut(0).unwrap();
    let mut parts = message.clone().into_parts();
    let mut blocks = List::with_capacity(TEXT_BLOCKS);
    for block in parts.blocks.into_boxed() {
        blocks.push(block).unwrap();
    }
    let text = smith::Text::new(&smith::CEILINGS, smith::TextParts { text: Box::from(second), replay: None }).unwrap();
    blocks.push(smith::Block::Text(text)).unwrap();
    parts.blocks = blocks;
    *message = smith::Message::new(&smith::CEILINGS, parts).unwrap();
    let source = smith::Turn::new(&smith::CEILINGS, source).unwrap();
    let mut writer = Writer::new(usize::try_from(source.measure()).unwrap());
    source.encode(&mut writer).unwrap();
    writer.finish()
}

fn decoded(second: &[u8]) -> c::TurnRead {
    let base = fixtures::limits();
    let client = jig_client_domain::Limits { text: FIELD_TEXT, turn_text: TURN_TEXT, ..base.client };
    let limits = Limits::new(client, base.exchanges).unwrap();
    translate::pages::turn_fields(&turn_with(second), &limits)
}

#[test]
fn a_pages_archived_turn_past_the_live_turns_field_is_decoded() {
    // The live turn's field stops at 65,536 bytes; a page's archived turn is
    // bounded as the archive is (`translate::turn::archive`).
    let mut text = Writer::new(65_536);
    for _ in 0_u32..65_536 {
        text.put(b"a").unwrap();
    }
    let body = turn_with(&text.finish());
    assert!(body.len() > 65_536);
    let base = fixtures::limits();
    let client = jig_client_domain::Limits { document_bytes: 1_048_576, ..base.client };
    let limits = Limits::new(client, base.exchanges).unwrap();
    let turn = translate::pages::archived_turn_fields(&fixtures::single_turn_record(&body), &limits).turn;
    let c::TurnRead::Decoded(_) = turn else { panic!("archived turn decodes whole: {turn:?}") };
}

#[test]
fn a_pages_archived_turn_is_decoded_at_each_bound_and_too_large_one_byte_past() {
    let record = fixtures::single_turn_record(fixtures::FULL_TURN);
    let at = u32::try_from(fixtures::TURN_PAYLOAD_BYTES).unwrap();
    let too_large = c::TurnRead::Undecodable { why: c::TurnProblem::TooLarge { bytes: fixtures::TURN_PAYLOAD_BYTES } };
    // The wire's carried-turn bound, below the client's document bound.
    let mut limits = fixtures::turn_limits(32, 1_048_576);
    limits.wire.activity_turn_turn = 1;
    limits.wire.turn_carried_bytes = at;
    assert_eq!(translate::pages::archived_turn_fields(&record, &limits).turn, fixtures::decoded_turn(b"abc", false));
    limits.wire.turn_carried_bytes = at - 1;
    assert_eq!(translate::pages::archived_turn_fields(&record, &limits).turn, too_large);
    // The client's document bound, below the wire's.
    for (document, expected) in [
        (fixtures::TURN_PAYLOAD_BYTES, fixtures::decoded_turn(b"abc", false)),
        (fixtures::TURN_PAYLOAD_BYTES - 1, too_large),
    ] {
        let mut limits = fixtures::turn_limits(32, document);
        limits.wire.activity_turn_turn = 1;
        assert_eq!(translate::pages::archived_turn_fields(&record, &limits).turn, expected);
    }
}

fn expected(cut: bool) -> c::TurnRead {
    let c::TurnRead::Decoded(mut turn) = fixtures::decoded_turn(b"abc", cut) else { unreachable!() };
    let message = turn.messages.get_mut(0).unwrap();
    let mut blocks = List::with_capacity(TEXT_BLOCKS);
    for block in core::mem::replace(&mut message.blocks, List::with_capacity(0)).into_boxed() {
        blocks.push(block).unwrap();
    }
    blocks.push(c::Block::Text(c::Text::new(Box::from(&b"def"[..]), FIELD_TEXT).unwrap())).unwrap();
    message.blocks = blocks;
    c::TurnRead::Decoded(turn)
}

#[test]
fn texts_above_field_bound_fit_the_turn_budget_whole() {
    assert_eq!(decoded(b"def"), expected(false));
}

#[test]
fn texts_one_past_the_turn_budget_are_cut() {
    assert_eq!(decoded(b"defg"), expected(true));
}
