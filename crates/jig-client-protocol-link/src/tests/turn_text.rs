use super::*;
use skein_lib::{List, Writer};
use smith_transcript::v6 as smith;

// The golden's first text is "abc"; the second fills another three bytes.
const FIELD_TEXT: u32 = 3;
const TURN_TEXT: u32 = 6;
const TEXT_BLOCKS: u32 = 2; // The golden's text followed by the added text.

fn decoded(second: &[u8]) -> c::TurnRead {
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
    let base = fixtures::limits();
    let client = jig_client_domain::Limits { text: FIELD_TEXT, turn_text: TURN_TEXT, ..base.client };
    let limits = Limits::new(client, base.exchanges).unwrap();
    translate::pages::turn_fields(&writer.finish(), &limits)
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
