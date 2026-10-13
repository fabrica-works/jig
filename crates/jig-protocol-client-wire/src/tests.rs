//! Client wire refusals and round trips (protocol/client.md, section 3).
mod fixtures;
use crate::*;
use skein_lib::Reader;

#[test]
fn a_keyed_request_past_its_bound_is_refused_naming_it() {
    let limits = fixtures::limits();
    let request = fixtures::bytes(limits.keyed_request.checked_add(1).expect("small bound"));
    let refused = Keyed::new(&limits, KeyedParts { key: [7; 16], owner: Owner::Jig, version: 1, request });
    assert_eq!(refused, Err(Problem { path: Path::KeyedRequest, reason: skein_codec::Reason::Bound }));
}

#[test]
fn an_owned_record_keeps_its_owners_version() {
    let limits = fixtures::limits();
    let owned = fixtures::owned();
    let bytes = fixtures::encoded(&owned, Owned::measure, Owned::encode);
    assert_eq!(Owned::decode(&limits, &mut Reader::new(&bytes)), Ok(owned));
}

#[test]
fn a_refusal_with_its_facts_round_trips_unchanged() {
    let limits = fixtures::limits();
    let refused = fixtures::refused_moved();
    let bytes = fixtures::encoded(&refused, Refused::measure, Refused::encode);
    assert_eq!(Refused::decode(&limits, &mut Reader::new(&bytes)), Ok(refused));
}

#[test]
fn each_answer_kind_round_trips_unchanged() {
    let limits = fixtures::limits();
    for answer in [Answer::FinalAnswer(fixtures::owned()), Answer::Busy, Answer::NotReady, Answer::SignedOut] {
        let bytes = fixtures::encoded(&answer, Answer::measure, Answer::encode);
        assert_eq!(Answer::decode(&limits, &mut Reader::new(&bytes)), Ok(answer));
    }
}

#[test]
fn a_bearer_token_past_its_bound_is_refused() {
    let limits = fixtures::limits();
    let token = fixtures::bytes(limits.signed_in_token.checked_add(1).expect("small bound"));
    assert_eq!(
        SignedIn::new(&limits, SignedInParts { token, party: 17, expires: 29 }),
        Err(Problem { path: Path::SignedInToken, reason: skein_codec::Reason::Bound })
    );
}

#[test]
fn a_cursor_past_its_bound_is_refused() {
    let limits = fixtures::limits();
    let value = fixtures::bytes(limits.cursor_value.checked_add(1).expect("small bound"));
    assert_eq!(
        Cursor::new(&limits, CursorParts { value }),
        Err(Problem { path: Path::CursorValue, reason: skein_codec::Reason::Bound })
    );
}

#[test]
fn each_read_none_round_trips_unchanged() {
    let limits = fixtures::limits();
    for none in [ReadNone::Unknown, ReadNone::Refused, ReadNone::Gone, ReadNone::Busy, ReadNone::SignedOut] {
        let answer = ReadAnswer::None(fixtures::read_none(none));
        let bytes = fixtures::encoded(&answer, ReadAnswer::measure, ReadAnswer::encode);
        assert_eq!(ReadAnswer::decode(&limits, &mut Reader::new(&bytes)), Ok(answer));
    }
}

#[test]
fn a_read_value_round_trips_with_its_owner_version() {
    let limits = fixtures::limits();
    let answer = ReadAnswer::Value(fixtures::owned());
    let bytes = fixtures::encoded(&answer, ReadAnswer::measure, ReadAnswer::encode);
    assert_eq!(ReadAnswer::decode(&limits, &mut Reader::new(&bytes)), Ok(answer));
}

#[test]
fn a_page_one_row_past_its_limit_is_refused() {
    let limits = fixtures::limits();
    let rows = fixtures::funding_rows(limits.owner_spend_page_rows.checked_add(1).expect("small bound"));
    assert_eq!(
        OwnerSpendPage::new(&limits, OwnerSpendPageParts { rows, before: None, after: None, cut: false }),
        Err(Problem { path: Path::OwnerSpendPageRows, reason: skein_codec::Reason::Bound })
    );
}

#[test]
fn an_activity_line_carries_its_turn_bytes_unchanged() {
    let limits = fixtures::limits();
    let line = fixtures::activity_line(Box::from([0, 255, 1]));
    let bytes = fixtures::encoded(&line, ActivityLine::measure, ActivityLine::encode);
    assert_eq!(ActivityLine::decode(&limits, &mut Reader::new(&bytes)), Ok(line));
}

#[test]
fn a_page_whose_rows_do_not_split_to_its_end_is_refused_as_malformed() {
    let limits = fixtures::limits_with_activity_bytes(4096);
    for rows in fixtures::malformed_activity_pieces() {
        let page = fixtures::activity_page(&limits, rows);
        let bytes = fixtures::encoded(&page, ActivityPage::measure, ActivityPage::encode);
        assert_eq!(
            fixtures::decode_activity_page(&limits, &bytes, 8),
            Err(Problem { path: Path::ActivityPageRows, reason: skein_codec::Reason::Malformed })
        );
    }
}

#[test]
fn a_page_of_many_small_rows_and_one_of_a_few_large_rows_fit_one_bound() {
    let limits = fixtures::limits_with_activity_bytes(4096);
    for (count, width) in [(4, 16), (2, 512)] {
        let rows = fixtures::activity_rows(count, width);
        let bytes = fixtures::encoded_activity_page(&limits, &rows);
        assert_eq!(fixtures::decode_activity_page(&limits, &bytes, 8), Ok(rows));
    }
}

#[test]
fn a_frame_at_its_ceilings_round_trips_unchanged() {
    let limits = fixtures::document_limits();
    let frame = fixtures::frame_at_ceilings(&limits);
    let bytes = fixtures::encoded(&frame, Frame::measure, Frame::encode);
    assert_eq!(Frame::decode(&limits, &mut Reader::new(&bytes)), Ok(frame));
}

#[test]
fn a_task_one_byte_past_its_text_bound_is_refused() {
    let mut limits = fixtures::document_limits();
    let task = fixtures::task(&limits);
    let bytes = fixtures::encoded(&task, Task::measure, Task::encode);
    limits.document_text_bytes -= 1;
    let expected = Err(Problem { path: Path::DocumentTextBytes, reason: skein_codec::Reason::Bound });
    assert_eq!(Task::decode(&limits, &mut Reader::new(&bytes)), expected);
    assert_eq!(Task::new(&limits, task.into_parts()), expected);
}

#[test]
fn a_run_state_without_writing_or_call_round_trips() {
    let limits = fixtures::document_limits();
    let run = fixtures::run_state(&limits);
    let bytes = fixtures::encoded(&run, RunState::measure, RunState::encode);
    assert_eq!(RunState::decode(&limits, &mut Reader::new(&bytes)), Ok(run));
}

#[test]
fn a_chip_with_each_phase_round_trips_unchanged() {
    let limits = fixtures::document_limits();
    for phase in fixtures::chip_phases(&limits) {
        let rows = fixtures::chip_facts(&limits, phase);
        let chip = fixtures::chip(&limits, &rows);
        let bytes = fixtures::encoded(&chip, Chip::measure, Chip::encode);
        assert_eq!(fixtures::decode_chip_facts(&limits, &bytes), rows);
    }
}

#[test]
fn a_hello_naming_two_owners_round_trips_unchanged() {
    let limits = fixtures::stream_limits();
    let hello = fixtures::hello(&limits);
    let bytes = fixtures::encoded(&hello, Hello::measure, Hello::encode);
    assert_eq!(Hello::decode(&limits, &mut Reader::new(&bytes)), Ok(hello));
}

#[test]
fn a_stream_event_at_its_ceiling_round_trips_unchanged() {
    let limits = fixtures::stream_limits();
    for event in fixtures::stream_events(&limits) {
        let bytes = fixtures::encoded(&event, StreamEvent::measure, StreamEvent::encode);
        assert_eq!(StreamEvent::decode(&limits, &mut Reader::new(&bytes)), Ok(event));
    }
}

#[test]
fn an_open_for_a_tree_carries_its_window() {
    let limits = fixtures::stream_limits();
    let open = fixtures::open_tree(&limits);
    let bytes = fixtures::encoded(&open, Open::measure, Open::encode);
    let decoded = Open::decode(&limits, &mut Reader::new(&bytes)).expect("whole open exchange");
    assert_eq!(
        Subject::decode(&limits, &mut Reader::new(decoded.subject().record())),
        Ok(fixtures::tree_subject(&limits))
    );
    assert_eq!(decoded, open);
}

#[test]
fn each_watch_end_but_dropped_round_trips() {
    let limits = fixtures::stream_limits();
    for why in [
        WatchEnd::Closed,
        WatchEnd::Unknown,
        WatchEnd::NotAllowed,
        WatchEnd::Busy,
        WatchEnd::SubjectEnded,
        WatchEnd::SignedOut,
        WatchEnd::Stopping,
    ] {
        let ended = fixtures::ended(&limits, why);
        let bytes = fixtures::encoded(&ended, Ended::measure, Ended::encode);
        assert_eq!(Ended::decode(&limits, &mut Reader::new(&bytes)), Ok(ended));
    }
}

#[test]
fn a_deployment_document_past_its_rows_is_refused() {
    let limits = fixtures::stream_limits();
    let parts =
        fixtures::deployment_parts(&limits, limits.deployment_capacity.checked_add(1).expect("small row bound"));
    assert_eq!(
        Deployment::new(&limits, parts),
        Err(Problem { path: Path::DeploymentCapacity, reason: skein_codec::Reason::Bound })
    );
}

#[test]
fn an_account_waiting_to_ask_again_carries_the_wall_it_asks_at() {
    let limits = fixtures::stream_limits();
    for until in [None, Some(0), Some(u64::MAX)] {
        let state = fixtures::waiting_account_state(&limits, until);
        let account = fixtures::owner_account(&limits, state.clone());
        let bytes = fixtures::encoded(&account, OwnerAccount::measure, OwnerAccount::encode);
        let decoded = OwnerAccount::decode(&limits, &mut Reader::new(&bytes)).expect("whole account document");
        assert_eq!(decoded.state(), &state);
        assert_eq!(decoded, account);
    }
}

#[test]
fn a_historical_goal_holding_keeps_its_opaque_segments() {
    let limits = fixtures::stream_limits();
    let holding = fixtures::historical_goal_holding(&limits);
    let bytes = fixtures::encoded(&holding, OwnerGoalHolding::measure, OwnerGoalHolding::encode);
    assert_eq!(OwnerGoalHolding::decode(&limits, &mut Reader::new(&bytes)), Ok(holding));
}
