//! Bounds and unread evidence for parties' transcript reads.
use super::{
    Box, CHILD, Token, assert_empty, child_store, core, corrupt_disk, fixtures, one_row, party_store, party_submit,
    read_disk, read_result, reads, reply_range, turn_range, turn_reply, waiting_party,
};

#[test]
fn an_unread_turn_is_an_empty_page() {
    let (mut store, mut disk) = party_store();
    corrupt_disk(&mut disk);
    let below = waiting_party(&mut store, turn_range());
    assert_empty(&read_disk(&mut store, below, &disk));
}

#[test]
fn a_turn_row_counts_its_joined_owner_against_the_byte_budget() {
    let (mut store, disk) = party_store();
    let mut range = turn_range();
    range.bytes = 1;
    let below = waiting_party(&mut store, range);
    assert_empty(&read_disk(&mut store, below, &disk));
}

#[test]
fn a_reply_turn_counts_against_the_join_budget() {
    let (mut store, disk) = party_store();
    store.limits.joins = 2;
    let below = waiting_party(&mut store, reply_range());
    assert_empty(&read_disk(&mut store, below, &disk));
}

#[test]
fn a_turns_point_cursor_excludes_the_named_turn() {
    let (mut store, _) = party_store();
    let mut range = turn_range();
    range.after =
        Some(reads::Position::Owner(core::Key::Core(core::CoreKey::Turn { task: CHILD, attempt: 1, turn: 2 })));
    let (events, below) = party_submit(&mut store, Token::new(7), range);
    assert_empty(&read_result(events));
    assert!(below.transcripts.is_empty());
}

#[test]
fn a_zero_row_party_read_does_not_start_io() {
    let (mut store, _) = party_store();
    for mut range in [turn_range(), reply_range()] {
        range.rows = 0;
        let (events, below) = party_submit(&mut store, Token::new(7), range);
        assert_empty(&read_result(events));
        assert!(below.transcripts.is_empty());
    }
}

#[test]
fn a_reply_with_no_transcript_place_returns_without_its_turn() {
    let mut store = child_store();
    fixtures::committed(&mut store, Box::new([fixtures::header(2), fixtures::save(turn_reply())]));
    let (events, below) = party_submit(&mut store, Token::new(7), reply_range());
    let result = read_result(events);
    assert_eq!(one_row(&result).related, None);
    assert!(below.transcripts.is_empty());
}
