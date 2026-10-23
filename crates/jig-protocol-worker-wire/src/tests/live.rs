use super::fixtures::{SIZES_LARGE, SIZES_ONE_SLOT};
use crate::{self as wire, sizes, v9};
use skein_lib::{Reader, Writer};

fn repeated(byte: u8, length: u32) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(length).unwrap());
    for _ in 0..length {
        writer.put(&[byte]).unwrap();
    }
    writer.finish()
}
fn full_piece(kind: wire::LiveKind, bytes: u32) -> wire::Piece {
    wire::Piece::new(
        &wire::CEILINGS,
        wire::PieceParts { turn: u32::MAX, retry: u32::MAX, kind, offset: u64::MAX, bytes: repeated(0xff, bytes) },
    )
    .unwrap()
}
fn input() -> wire::LiveKind {
    wire::LiveKind::Input(
        wire::InputCall::new(
            &wire::CEILINGS,
            wire::InputCallParts { id: repeated(b'i', 256), tool: repeated(b't', 64) },
        )
        .unwrap(),
    )
}
#[test]
fn each_full_piece_and_fact_has_the_hand_measured_body() {
    for (kind, call_bytes) in [(wire::LiveKind::Text, 0), (wire::LiveKind::Reasoning, 0), (input(), 328)] {
        let piece = full_piece(kind, 4096);
        // Turn4 + retry4 + kind1 + optional call (length4+id256+length4+tool64)
        // + offset8 + length4 + content4096.
        assert_eq!(piece.measure(), 4 + 4 + 1 + call_bytes + 8 + 4 + 4096);
        let fact = wire::Fact::new(
            &wire::CEILINGS,
            wire::FactParts { task: u64::MAX, attempt: u64::MAX, value: wire::FactValue::Piece(piece) },
        )
        .unwrap();
        assert_eq!(fact.measure(), 16 + 1 + 21 + call_bytes + 4096);
        let mut writer = Writer::new(usize::try_from(fact.measure()).unwrap());
        fact.encode(&mut writer).unwrap();
        let bytes = writer.finish();
        assert_eq!(wire::Fact::decode(&wire::CEILINGS, &mut Reader::new(&bytes)).unwrap(), fact);
    }
    assert_eq!(wire::Fact::worst_case_bytes(&wire::CEILINGS), Some(4462));
}
#[test]
fn every_size_fixture_holds_a_largest_piece_fact_and_its_frame() {
    for sizes in [SIZES_ONE_SLOT, SIZES_LARGE] {
        let frame = sizes::largest_body(&sizes, wire::Kind::Fact).unwrap() + 8;
        assert_eq!(frame, sizes.piece + 374);
        assert!(sizes.facts >= frame);
        assert!(sizes.facts / 25 >= 1, "room for a fact frame");
    }
    let limits = sizes::limits(&SIZES_ONE_SLOT).unwrap();
    assert_eq!((limits.piece_bytes, limits.input_call_id, limits.input_call_tool), (16, 256, 64));
}
#[test]
fn piece_content_at_its_ceiling_decodes_and_one_past_is_refused() {
    for length in [4096_u32, 4097] {
        let mut writer = Writer::new(21 + usize::try_from(length).unwrap());
        writer.put(&0_u32.to_be_bytes()).unwrap();
        writer.put(&0_u32.to_be_bytes()).unwrap();
        writer.put(&[0]).unwrap();
        writer.put(&0_u64.to_be_bytes()).unwrap();
        writer.put(&length.to_be_bytes()).unwrap();
        writer.put(&repeated(0xff, length)).unwrap();
        let bytes = writer.finish();
        let result = wire::Piece::decode(&wire::CEILINGS, &mut Reader::new(&bytes));
        if length == 4096 {
            assert!(result.is_ok());
        } else {
            assert_eq!(
                result.unwrap_err(),
                v9::Problem { path: v9::Path::PieceBytes, reason: skein_codec::Reason::Bound }
            );
        }
    }
}
#[test]
fn input_call_fields_at_their_ceilings_decode_and_one_past_is_refused() {
    for (id, tool, problem) in
        [(256_u32, 64_u32, None), (257, 64, Some(v9::Path::InputCallId)), (256, 65, Some(v9::Path::InputCallTool))]
    {
        let mut writer = Writer::new(8 + usize::try_from(id + tool).unwrap());
        writer.put(&id.to_be_bytes()).unwrap();
        writer.put(&repeated(b'i', id)).unwrap();
        writer.put(&tool.to_be_bytes()).unwrap();
        writer.put(&repeated(b't', tool)).unwrap();
        let bytes = writer.finish();
        let result = wire::InputCall::decode(&wire::CEILINGS, &mut Reader::new(&bytes));
        match problem {
            None => {
                let call = result.unwrap();
                assert_eq!(call.measure(), 328);
            }
            Some(path) => assert_eq!(result.unwrap_err(), v9::Problem { path, reason: skein_codec::Reason::Bound }),
        }
    }
}
#[test]
fn the_smallest_new_records_have_fixed_widths() {
    let ask = wire::LiveAsk::decode(&wire::CEILINGS, &mut Reader::new(&[0; 3])).unwrap();
    assert_eq!((ask.text(), ask.reasoning(), ask.input(), ask.measure()), (false, false, false, 3));
    assert_eq!(wire::InputCall::decode(&wire::CEILINGS, &mut Reader::new(&[0; 8])).unwrap().measure(), 8);
    assert_eq!(wire::Piece::decode(&wire::CEILINGS, &mut Reader::new(&[0; 21])).unwrap().measure(), 21);
    let mut fact = [0; 38];
    fact[16] = 4; // FactValue::Piece; the piece itself has the first kind, Text.
    assert_eq!(wire::Fact::decode(&wire::CEILINGS, &mut Reader::new(&fact)).unwrap().measure(), 38);
}
