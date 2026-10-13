//! Turn frame integrity (protocol/store.md, section 8.1).
use super::fixtures;
use crate::transcripts::frame::{self, Unread};
use alloc::boxed::Box;
use skein_lib::Reader;
fn frame() -> Box<[u8]> {
    let turn = fixtures::window_turn(fixtures::LIVE, 2, 3, b"smith's bytes");
    frame::encode(&turn, &fixtures::codec_limits()).expect("bounded frame")
}
#[test]
fn a_frame_decodes_to_the_turn_it_encoded() {
    let limits = fixtures::codec_limits();
    let turn = fixtures::window_turn(fixtures::LIVE, 2, 3, b"smith's bytes");
    let bytes = frame::encode(&turn, &limits).expect("within the turn's limits");
    assert_eq!(frame::decode(&bytes, &limits), Ok(turn));
}
#[test]
fn the_trailing_length_matches_the_leading_one() {
    let bytes = frame();
    let leading = Reader::new(&bytes).u32().expect("leading length");
    let start = bytes.len().checked_sub(4).expect("a whole frame");
    let trailing = Reader::new(bytes.get(start..).expect("trailing bytes")).u32().expect("trailing length");
    assert_eq!(trailing, leading);
    assert_eq!(leading, u32::try_from(bytes.len()).expect("bounded frame"));
}
#[test]
fn a_flipped_byte_fails_the_frames_checksum() {
    let mut bytes = frame();
    *bytes.get_mut(4).expect("first body byte") ^= 1;
    assert_eq!(frame::decode(&bytes, &fixtures::codec_limits()), Err(Unread::Checksum));
}
#[test]
fn a_frame_cut_short_reads_as_short() {
    let bytes = frame();
    let end = bytes.len().checked_sub(1).expect("whole frame");
    assert_eq!(frame::decode(bytes.get(..end).expect("short frame"), &fixtures::codec_limits()), Err(Unread::Short));
}
#[test]
fn crc32c_matches_its_check_value() {
    assert_eq!(frame::crc32c(b"123456789"), 0xe306_9283);
}
#[test]
fn a_turn_past_the_frame_body_bound_is_refused() {
    let mut limits = fixtures::codec_limits();
    limits.core.turn_record_transcript = 4;
    let turn = fixtures::turn(fixtures::LIVE, 1, 1, b"12345");
    assert_eq!(frame::encode(&turn, &limits), None);
}
#[test]
fn a_frame_with_disagreeing_lengths_is_unread() {
    let mut bytes = frame();
    let last = bytes.last_mut().expect("trailing length");
    *last ^= 1;
    assert_eq!(frame::decode(&bytes, &fixtures::codec_limits()), Err(Unread::Lengths));
}
#[test]
fn a_checked_frame_with_no_record_is_unread() {
    let bytes = [0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 12];
    assert_eq!(frame::decode(&bytes, &fixtures::codec_limits()), Err(Unread::Record));
}
