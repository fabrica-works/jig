use super::*;
pub(super) fn native_run(rows: u32) -> (Fixture, jig_client::Handle) {
    let limits = crate::Limits { rows, segments: rows.max(4), watches: 8, text: 100, ..super::super::spine::LIMITS };
    let mut fixture = Fixture::new(&limits);
    fixture.drive(
        skein_lib::Time::ZERO,
        crate::Event::Start { seed: 8, place: crate::address::Place::Elsewhere, records: List::with_capacity(0) },
    );
    fixture.ui.conversation = Thread::new(9, &jig_client::thread::Limits { lines: rows, bytes: 65536, text: 4096 });
    let handle = fixture.request_watch(jig_client::contract::Subject::Run { task: 9, turn: None, offset: 0 }).unwrap();
    (fixture, handle)
}
pub(super) fn feed_run(fixture: &mut Fixture, watch: jig_client::Handle, delivery: jig_client::contract::Delivery) {
    fixture
        .drive(skein_lib::Time::ZERO, Watched { watch, event: jig_client::contract::WatchEvent::Delivered(delivery) });
}
pub(super) fn fold_current(fixture: &mut Fixture, identity: jig_client::contract::CallIdentity, fold: Fold) -> bool {
    let projected = fixture.conversation();
    let live = projected.live;
    let current = projected.current_run.cloned();
    fixture.ui.conversation.set_current_fold(identity, fold, live.as_slice(), current.as_ref())
}
pub(super) fn fold_call(fixture: &mut Fixture, name: CallName, fold: Fold) -> bool {
    let live = fixture.conversation().live;
    fixture.ui.conversation.set_fold(name, fold, live.as_slice())
}
pub(super) fn stream(number: u32, from: u32, through: u32, bytes: &[u8], cut: bool) -> RunText {
    RunText {
        attempt: 1,
        began: Some(Wall::EPOCH),
        turn: ProvisionalTurnName { retry: 0, attempt: 1, number },
        from,
        through,
        text: text(bytes),
        cut,
        call: None,
    }
}
