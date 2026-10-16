use super::*;
#[test]
fn a_read_keeps_its_long_cursor_as_opaque_bytes() {
    let actual = translate::reads::read(&fixtures::history_read(), &fixtures::limits().wire).unwrap();
    let w::Read::History(record) = actual else { panic!("history record") };
    assert_eq!(record.task(), 7);
    assert!(record.subtree());
    assert_eq!(record.after().as_ref().unwrap().value(), fixtures::READ_CURSOR);
}
#[test]
fn a_read_cursor_past_its_wire_bound_is_refused() {
    let mut limits = fixtures::limits().wire;
    limits.cursor_value = 47;
    let problem = translate::reads::read(&fixtures::history_read(), &limits).unwrap_err();
    assert_eq!(problem.path, w::Path::CursorValue);
    assert_eq!(problem.reason, skein_codec::Reason::Bound);
}
#[test]
fn a_notes_read_keeps_its_scope_and_filter_with_its_cursor() {
    let actual = translate::reads::read(&fixtures::notes_read(), &fixtures::limits().wire).unwrap();
    let w::Read::Notes(record) = actual else { panic!("notes record") };
    assert_eq!(record.scope(), &fixtures::notes_scope_record());
    assert_eq!(record.description().as_deref(), Some(b"filter".as_slice()));
    assert_eq!(record.after().as_ref().unwrap().value(), fixtures::READ_CURSOR);
}

#[test]
fn a_read_body_names_jigs_wire_and_keeps_its_read() {
    let record = translate::reads::owned(Readable::Jig(fixtures::history_read()), &fixtures::limits().wire).unwrap();
    assert_eq!(record.owner(), &w::Owner::Jig);
    assert_eq!(record.version(), jig_client_protocol_wire::VERSION);
    let read = w::Read::decode(&fixtures::limits().wire, &mut Reader::new(record.record())).unwrap();
    let w::Read::History(read) = read else { panic!("history read") };
    assert_eq!(read.task(), 7);
    assert_eq!(read.after().as_ref().unwrap().value(), fixtures::READ_CURSOR);
}
#[test]
fn a_family_read_body_keeps_its_opaque_owner_record() {
    let record = translate::reads::owned(Readable::Family(fixtures::family_read()), &fixtures::limits().wire).unwrap();
    let w::Owner::Family(owner) = record.owner() else { panic!("family owner") };
    assert_eq!(owner.number(), 19);
    assert_eq!(record.version(), 23);
    assert_eq!(record.record(), &[255, 0, 1]);
}
#[test]
fn each_wire_none_keeps_its_reason() {
    for (wire, client) in [
        (w::ReadNone::Unknown, c::ReadNone::Unknown),
        (w::ReadNone::Refused, c::ReadNone::Refused),
        (w::ReadNone::Gone, c::ReadNone::Gone),
        (w::ReadNone::Busy, c::ReadNone::Busy),
        (w::ReadNone::SignedOut, c::ReadNone::SignedOut),
    ] {
        assert_eq!(translate::reads::none(&wire), client);
    }
}

#[test]
fn a_read_becomes_one_post_with_its_owner_and_cursor() {
    let (mut h, _) = fixtures::Harness::reading();
    assert_eq!(h.out.len(), 1);
    let Request::Exchange { path, body, .. } = h.out.pop().unwrap() else { panic!("expected read exchange") };
    assert_eq!(path, b"/jig/v1/read");
    let record = w::Owned::decode(&fixtures::limits().wire, &mut Reader::new(&body)).unwrap();
    assert_eq!(record.owner(), &w::Owner::Jig);
    assert_eq!(record.version(), jig_client_protocol_wire::VERSION);
    let w::Read::History(value) = w::Read::decode(&fixtures::limits().wire, &mut Reader::new(record.record())).unwrap()
    else {
        panic!("history read")
    };
    assert_eq!(value.after().as_ref().unwrap().value(), fixtures::READ_CURSOR);
}
#[test]
fn a_read_signed_out_is_answered_signed_out() {
    let (mut h, exchange) = fixtures::Harness::reading();
    h.answer(exchange, 200, &fixtures::read_none_answer(w::ReadNone::SignedOut));
    assert_eq!(h.read_answered(), Some(&Readout::Jig(c::ReadResult::None(c::ReadNone::SignedOut))));
    assert_eq!(h.link.next_deadline(), None);
}
#[test]
fn a_read_with_no_answer_is_unreachable() {
    let (mut h, exchange) = fixtures::Harness::reading();
    h.link.step(Time::ZERO, Event::Failed { exchange }, &mut h.out);
    h.read_unreachable();
    h.answer(exchange, 200, &fixtures::read_none_answer(w::ReadNone::Unknown));
    assert_eq!(h.read_terminals(), 1);
    assert_eq!(h.link.next_deadline(), None);
}
#[test]
fn a_read_past_its_deadline_is_abandoned_and_answered_once() {
    let (mut h, exchange) = fixtures::Harness::reading();
    h.link.fire(Time::ZERO.saturating_add(fixtures::limits().deadline), &mut h.out);
    assert!(h.has(&Request::Abandon { exchange }));
    h.read_unreachable();
    h.answer(exchange, 200, &fixtures::read_none_answer(w::ReadNone::Busy));
    assert_eq!(h.read_terminals(), 1);
    assert_eq!(h.link.next_deadline(), None);
}
#[test]
fn a_read_none_answer_keeps_each_wire_reason_once() {
    for (wire, client) in [
        (w::ReadNone::Unknown, c::ReadNone::Unknown),
        (w::ReadNone::Refused, c::ReadNone::Refused),
        (w::ReadNone::Gone, c::ReadNone::Gone),
        (w::ReadNone::Busy, c::ReadNone::Busy),
    ] {
        let (mut h, exchange) = fixtures::Harness::reading();
        h.answer(exchange, 200, &fixtures::read_none_answer(wire));
        h.answer(exchange, 200, &fixtures::read_none_answer(w::ReadNone::SignedOut));
        assert_eq!(h.read_answered(), Some(&Readout::Jig(c::ReadResult::None(client))));
        assert_eq!(h.read_terminals(), 1);
    }
}
#[test]
fn a_family_read_is_answered_once_with_its_opaque_value() {
    let (mut h, exchange) = fixtures::Harness::reading_family();
    h.answer(exchange, 200, &fixtures::family_read_answer(19, 23));
    h.answer(exchange, 200, &fixtures::family_read_answer(19, 23));
    assert_eq!(h.read_answered(), Some(&Readout::Family { family: 19, record: Some(Box::from([255, 0, 1])) }));
    assert_eq!(h.read_terminals(), 1);
}
#[test]
fn a_family_read_none_stays_under_its_owner() {
    let (mut h, exchange) = fixtures::Harness::reading_family();
    h.answer(exchange, 200, &fixtures::read_none_answer(w::ReadNone::Unknown));
    assert_eq!(h.read_answered(), Some(&Readout::Family { family: 19, record: None }));
}
#[test]
fn a_read_value_from_another_owner_or_version_is_reported() {
    for (owner, version) in [(20, 23), (19, 24)] {
        let (mut h, exchange) = fixtures::Harness::reading_family();
        h.answer(exchange, 200, &fixtures::family_read_answer(owner, version));
        assert_eq!(h.read_answered(), Some(&Readout::Family { family: 19, record: None }));
        assert!(h.has(&Request::Fact { fact: Fact::UndecodableDocument }));
    }
}
#[test]
fn a_failed_read_status_or_malformed_body_is_unreachable() {
    for (status, body, reported) in
        [(503, fixtures::read_none_answer(w::ReadNone::Busy), false), (200, Box::from([255]), true)]
    {
        let (mut h, exchange) = fixtures::Harness::reading();
        h.answer(exchange, status, &body);
        h.read_unreachable();
        assert_eq!(h.has(&Request::Fact { fact: Fact::UndecodableDocument }), reported);
    }
}
#[test]
fn reads_and_attempts_share_the_exchange_admission_bound() {
    let mut limits = fixtures::limits();
    limits.exchanges = 1;
    let mut h = fixtures::Harness::new(&limits);
    h.send(fixtures::ATTEMPT, fixtures::KEY, fixtures::start_chat());
    h.link.step(
        Time::ZERO,
        Event::Read { read: fixtures::READ, what: Readable::Jig(fixtures::history_read()) },
        &mut h.out,
    );
    h.read_unreachable();
    assert_eq!(h.read_terminals(), 1);
    assert_eq!(h.out.len(), 2);
}

#[test]
fn a_read_is_answered_once_with_its_value() {
    let (mut h, exchange) = fixtures::Harness::reading();
    h.answer(exchange, 200, &fixtures::jig_read_value_answer(&fixtures::frame_read_value()));
    h.answer(exchange, 200, &fixtures::read_none_answer(w::ReadNone::Unknown));
    assert_eq!(h.read_answered(), Some(&fixtures::expected_frame_read()));
    assert_eq!(h.read_terminals(), 1);
    assert_eq!(h.link.next_deadline(), None);
}
#[test]
fn a_value_past_the_client_s_limits_is_unreachable_and_reported() {
    for value in fixtures::values_past_client_limits() {
        let (mut h, exchange) = fixtures::Harness::reading();
        h.answer(exchange, 200, &fixtures::jig_read_value_answer(&value));
        h.read_unreachable();
        assert!(h.has(&Request::Fact { fact: Fact::UndecodableDocument }));
        assert_eq!(h.read_terminals(), 1);
    }
}
#[test]
fn each_discriminated_read_value_reaches_its_contract_variant() {
    for (wire, expected) in fixtures::read_value_cases() {
        let (mut h, exchange) = fixtures::Harness::reading();
        h.answer(exchange, 200, &fixtures::jig_read_value_answer(&wire));
        assert_eq!(h.read_answered(), Some(&Readout::Jig(c::ReadResult::Value(expected))));
        assert!(!h.has(&Request::Fact { fact: Fact::UndecodableDocument }));
    }
}
#[test]
fn a_malformed_read_value_is_unreachable_and_reported() {
    for value in fixtures::malformed_read_values() {
        let (mut h, exchange) = fixtures::Harness::reading();
        h.answer(exchange, 200, &fixtures::jig_read_record_answer(value, jig_client_protocol_wire::VERSION));
        h.read_unreachable();
        assert!(h.has(&Request::Fact { fact: Fact::UndecodableDocument }));
    }
}
#[test]
fn a_read_value_of_another_wire_version_is_unreachable_and_reported() {
    let (mut h, exchange) = fixtures::Harness::reading();
    let value = fixtures::encoded_read_value(&fixtures::frame_read_value());
    h.answer(exchange, 200, &fixtures::jig_read_record_answer(value, 2));
    h.read_unreachable();
    assert!(h.has(&Request::Fact { fact: Fact::UndecodableDocument }));
}
#[test]
fn a_read_value_counts_its_discriminator_against_the_document_byte_bound() {
    let value = fixtures::encoded_read_value(&fixtures::frame_read_value());
    let bytes = u64::try_from(value.len()).unwrap();
    let limits = fixtures::read_byte_limits(bytes);
    assert_eq!(translate::reads::value(&value, &limits), Ok(c::ReadValue::Frame(Box::new(fixtures::frame()))));
    let short = fixtures::read_byte_limits(bytes.checked_sub(1).unwrap());
    assert_eq!(translate::reads::value(&value, &short), Err(Undecodable::TooLarge));
}
#[test]
fn a_read_value_rejects_mutated_original_limits() {
    let value = fixtures::encoded_read_value(&fixtures::frame_read_value());
    let mut limits = fixtures::limits();
    limits.wire.cursor_value = 1;
    assert_eq!(translate::reads::value(&value, &limits), Err(Undecodable::TooLarge));
}
#[test]
fn a_read_page_keeps_both_long_cursors_without_parsing_them() {
    let (mut h, exchange) = fixtures::Harness::reading();
    let value = w::ReadValue::Notes(fixtures::note_page_with_cursors());
    h.answer(exchange, 200, &fixtures::jig_read_value_answer(&value));
    let expected = c::ReadValue::Notes(Box::new(fixtures::expected_note_page_with_cursors()));
    assert_eq!(h.read_answered(), Some(&Readout::Jig(c::ReadResult::Value(expected))));
}
#[test]
fn a_read_activity_keeps_each_undecodable_turn_in_its_page() {
    for (record, expected) in fixtures::unread_pages() {
        let (mut h, exchange) = fixtures::Harness::reading();
        h.answer(exchange, 200, &fixtures::jig_read_value_answer(&w::ReadValue::Activity(record)));
        let expected = c::ReadValue::Activity(Box::new(expected));
        assert_eq!(h.read_answered(), Some(&Readout::Jig(c::ReadResult::Value(expected))));
    }
}
