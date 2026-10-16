use super::*;
fn restored(expected: &Saved) -> fixtures::Harness {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.start_store(fixtures::store_versions(), fixtures::one_store_read(fixtures::store_read(expected.clone())));
    harness
}
fn pending_write(record: Saved) -> fixtures::Harness {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.save(record);
    harness.send_kept(fixtures::ATTEMPT);
    harness
}
#[test]
fn a_table_entry_restores_its_key_and_ask() {
    for ask in fixtures::store_asks().into_boxed() {
        let expected = fixtures::saved_table(ask);
        assert!(restored(&expected).restored(&expected));
    }
}
#[test]
fn a_draft_round_trips_through_the_small_store() {
    let expected = fixtures::saved_draft();
    assert!(restored(&expected).restored(&expected));
}
#[test]
fn a_family_s_bytes_and_storage_key_round_trip_unchanged() {
    let expected = fixtures::saved_family();
    assert!(restored(&expected).restored(&expected));
}
#[test]
fn a_record_of_another_store_version_is_dropped_and_told() {
    for (owner, record) in [
        (Keeper::Jig, fixtures::saved_draft()),
        (Keeper::Table, fixtures::saved_table(fixtures::start_chat())),
        (Keeper::Family(19), fixtures::saved_family()),
    ] {
        let mut harness = fixtures::Harness::new(&fixtures::limits());
        harness.start_store(
            fixtures::store_versions(),
            fixtures::one_store_read(fixtures::other_store_version(fixtures::store_read(record))),
        );
        assert!(harness.has(&Request::Dropped { keeper: owner }));
        assert_eq!(harness.out.len(), 1);
    }
}
#[test]
fn a_keeper_not_named_at_start_is_dropped_and_told() {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.start_store(
        skein_lib::List::with_capacity(0),
        fixtures::one_store_read(fixtures::store_read(fixtures::saved_family())),
    );
    assert!(harness.has(&Request::Dropped { keeper: Keeper::Family(19) }));
    assert_eq!(harness.out.len(), 1);
}
#[test]
fn a_step_s_writes_leave_before_its_exchanges() {
    let mut harness = pending_write(fixtures::saved_draft());
    assert_eq!(harness.exchanges(), 0);
    harness.written();
    let Some(Request::Write { .. }) = harness.out.iter().next() else { panic!("write leaves first") };
    assert_eq!(harness.exchanges(), 1);
}
#[test]
fn a_write_gathers_exchanges_already_in_the_step_s_output_queue() {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.send_kept(fixtures::ATTEMPT);
    harness.save(fixtures::saved_draft());
    assert_eq!(harness.exchanges(), 0);
    harness.written();
    let Some(Request::Write { .. }) = harness.out.iter().next() else { panic!("write leaves first") };
    assert_eq!(harness.exchanges(), 1);
}
#[test]
fn a_failed_write_withholds_the_step_s_exchanges() {
    let mut harness = pending_write(fixtures::saved_table(fixtures::start_chat()));
    harness.write_failed();
    harness.written();
    assert_eq!(harness.exchanges(), 0);
    harness.lost();
    assert_eq!(harness.terminals(), 1);
}
#[test]
fn an_expired_held_exchange_never_leaves_after_the_write_is_durable() {
    let limits = fixtures::limits();
    let mut harness = pending_write(fixtures::saved_draft());
    harness.link.fire(Time::ZERO.saturating_add(limits.deadline), &mut harness.out);
    harness.written();
    assert_eq!(harness.exchanges(), 0);
    harness.lost();
    assert_eq!(harness.terminals(), 1);
}
#[test]
fn a_place_round_trips_every_jig_address() {
    for expected in fixtures::saved_places().into_boxed() {
        assert!(restored(&expected).restored(&expected));
    }
}
#[test]
fn a_table_entry_restores_family_bytes_and_their_wire_version() {
    let expected = Saved::Table {
        version: fixtures::TABLE_VERSION,
        record: skein_lib::RequestRecord {
            key: fixtures::KEY,
            scope: u64::MAX,
            first_sent: None,
            request: Ask::Family(fixtures::family_read()),
        },
    };
    assert!(restored(&expected).restored(&expected));
}
#[test]
fn an_erased_key_waits_for_durability_before_the_step_s_exchange() {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.link.step(Time::ZERO, Event::Erase { key: Erased::Table(fixtures::KEY) }, &mut harness.out);
    harness.send_kept(fixtures::ATTEMPT);
    assert_eq!(harness.exchanges(), 0);
    harness.written();
    let Some(Request::Remove { .. }) = harness.out.iter().next() else { panic!("erase leaves first") };
    assert_eq!(harness.exchanges(), 1);
}
#[test]
fn malformed_kept_bytes_drop_the_owner_and_are_reported() {
    let mut entry = fixtures::store_read(fixtures::saved_draft());
    entry.record = Box::from([255]);
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.start_store(fixtures::store_versions(), fixtures::one_store_read(entry));
    assert!(harness.has(&Request::Dropped { keeper: Keeper::Jig }));
    assert!(harness.has(&Request::Fact { fact: Fact::UndecodableStore }));
    assert_eq!(harness.out.len(), 2);
}
#[test]
fn a_mismatched_table_storage_key_drops_its_record() {
    let mut entry = fixtures::store_read(fixtures::saved_table(fixtures::start_chat()));
    let length = entry.key.len();
    *entry.key.get_mut(length - 1).unwrap() = 0;
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.start_store(fixtures::store_versions(), fixtures::one_store_read(entry));
    assert!(harness.has(&Request::Dropped { keeper: Keeper::Table }));
    assert!(harness.has(&Request::Fact { fact: Fact::UndecodableStore }));
}
#[test]
fn a_write_failure_answers_a_held_read_as_unreachable_once() {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.save(fixtures::saved_draft());
    harness.link.step(
        Time::ZERO,
        Event::Read { read: fixtures::ATTEMPT, what: Readable::Jig(c::Read::Task { task: 7 }) },
        &mut harness.out,
    );
    harness.write_failed();
    harness.written();
    assert_eq!(harness.exchanges(), 0);
    assert!(harness.has(&Request::Read {
        read: fixtures::ATTEMPT,
        result: Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable))
    }));
}
#[test]
fn a_write_failure_drops_a_held_watch_opening() {
    let mut harness = fixtures::Harness::streaming(fixtures::family_subject());
    harness.save(fixtures::saved_draft());
    harness.write_failed();
    harness.written();
    assert_eq!(harness.exchanges(), 0);
    assert_eq!(harness.ended(fixtures::WATCH), Some(c::WatchEnd::Dropped));
}
#[test]
fn a_cancelled_opening_already_in_the_step_queue_is_never_sent_after_writing() {
    let mut harness = fixtures::Harness::streaming(fixtures::family_subject());
    let (exchange, _) = harness.opening();
    harness.link.step(Time::ZERO, Event::Failed { exchange }, &mut harness.out);
    harness.save(fixtures::saved_draft());
    harness.written();
    assert_eq!(harness.exchanges(), 0);
    assert_eq!(harness.ended(fixtures::WATCH), Some(c::WatchEnd::Dropped));
}
#[test]
fn a_family_key_at_its_owner_s_bound_round_trips_without_counting_the_keeper() {
    let limits = fixtures::limits();
    let expected = Saved::Family {
        family: 19,
        version: 7,
        key: skein_lib::bytes::zeroed(usize::try_from(limits.client.document_bytes).unwrap()),
        bytes: Box::from([255, 0, 1]),
    };
    assert!(restored(&expected).restored(&expected));
}
#[test]
fn a_family_key_past_its_owner_s_bound_withholds_the_step_s_exchange() {
    let limits = fixtures::limits();
    let record = Saved::Family {
        family: 19,
        version: 7,
        key: skein_lib::bytes::zeroed(usize::try_from(limits.client.document_bytes + 1).unwrap()),
        bytes: Box::from([255, 0, 1]),
    };
    let mut harness = pending_write(record);
    assert_eq!(harness.exchanges(), 0);
    harness.lost();
    harness.written();
    assert_eq!(harness.exchanges(), 0);
}
