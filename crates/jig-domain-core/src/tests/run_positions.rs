use super::*;

fn resumed(
    core: &mut Core,
    limits: &Limits,
    watcher: u64,
    turn: Option<views::TurnName>,
    offset: u32,
) -> Box<[Request]> {
    watch_outputs(core, limits, Event::WatchRun { watcher: Token::new(watcher), sign_in: 10, task: 1, turn, offset })
}

fn stream(out: &[Request]) -> &Request {
    for request in out {
        if let Request::Now(now) = request
            && let Now::RunText { .. } = now.as_ref()
        {
            return request;
        }
    }
    panic!("run watch offers retained provisional suffix")
}

#[test]
fn newly_opened_run_watches_resume_at_independent_named_offsets() {
    let (mut core, limits) = watch_core();
    open_run_watch(&mut core, &limits, 30);
    run_text_outputs(&mut core, &limits, 7, 1, 0, b"abcdefgh");
    let turn = views::TurnName { attempt: 7, turn: 1, retry: 0 };
    for (watcher, offset, suffix) in [(31, 2, b"cdefgh".as_slice()), (32, 5, b"fgh".as_slice())] {
        let out = resumed(&mut core, &limits, watcher, Some(turn), offset);
        assert_stream(stream(&out), watcher, turn, offset, 8, suffix, false);
        let mut delivered = false;
        for request in &out {
            if let Request::Held(held) = request
                && let Held::Delivery { .. } = held.as_ref()
            {
                delivered = true;
            }
        }
        assert!(delivered);
    }
}

#[test]
fn a_resumed_run_offset_applies_only_to_its_exact_turn_and_attempt() {
    let (mut core, limits) = watch_core();
    open_run_watch(&mut core, &limits, 30);
    run_text_outputs(&mut core, &limits, 7, 2, 0, b"new turn");
    for (watcher, recalled) in [
        (31, Some(views::TurnName { attempt: 7, turn: 1, retry: 0 })),
        (32, Some(views::TurnName { attempt: 6, turn: 2, retry: 0 })),
        (33, None),
    ] {
        let out = resumed(&mut core, &limits, watcher, recalled, 4);
        assert_stream(
            stream(&out),
            watcher,
            views::TurnName { attempt: 7, turn: 2, retry: 0 },
            0,
            8,
            b"new turn",
            false,
        );
    }
}

#[test]
fn a_resumed_run_reports_a_cut_when_its_acknowledged_prefix_was_discarded() {
    let (mut core, limits) = watch_core();
    open_run_watch(&mut core, &limits, 30);
    let mut text = List::with_capacity(limits.views.text_bytes);
    for _ in 0..limits.views.text_bytes {
        text.push(b'a').unwrap();
    }
    let text = text.into_boxed();
    run_text_outputs(&mut core, &limits, 7, 1, 0, &text);
    run_text_outputs(&mut core, &limits, 7, 1, limits.views.text_bytes, &text);
    let turn = views::TurnName { attempt: 7, turn: 1, retry: 0 };
    let out = resumed(&mut core, &limits, 31, Some(turn), 1);
    assert_stream(stream(&out), 31, turn, limits.views.text_bytes, limits.views.text_bytes * 2, &text, true);
}

#[test]
fn run_position_admission_keeps_signin_visibility_and_watch_bounds() {
    let (mut core, limits) = watch_core();
    for (sign_in, task, refusal) in [(99, 1, WatchRefusal::SignedOut), (10, 999, WatchRefusal::Unknown)] {
        let out = watch_outputs(
            &mut core,
            &limits,
            Event::WatchRun { watcher: Token::new(31), sign_in, task, turn: None, offset: u32::MAX },
        );
        assert_watch_refusal(&out[0], refusal);
    }
    let mut people_out = Queue::with_capacity(people::max_out(&limits.people));
    people::step(
        &mut core.people,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people },
        people::Event::Roles { project: 1, holdings: Box::new([]) },
        &mut people_out,
    );
    let denied = resumed(&mut core, &limits, 31, None, u32::MAX);
    assert_watch_refusal(&denied[0], WatchRefusal::NotAllowed);
    let (mut core, mut limits) = watch_core();
    limits.views.watchers_per_party = 1;
    resumed(&mut core, &limits, 30, None, 0);
    let full = resumed(&mut core, &limits, 31, None, 0);
    assert_watch_refusal(&full[0], WatchRefusal::Busy);
}
