use super::*;

fn report(core: &mut Core, limits: &Limits, attempt: u64, retry: u32, offset: u32, text: &[u8]) -> Box<[Request]> {
    watch_outputs(
        core,
        limits,
        Event::RunText {
            run: Token::new(1),
            attempt: Token::new(attempt),
            turn: 1,
            retry,
            offset,
            text: Box::from(text),
            call: None,
        },
    )
}
fn current_stream(out: &[Request]) -> &Request {
    out.iter()
        .find(|request| matches!(request, Request::Now(now) if matches!(now.as_ref(), Now::RunText { .. })))
        .expect("current supplied retry reaches the public stream boundary")
}

#[test]
fn an_unwatched_empty_retry_opens_as_its_current_supplied_identity() {
    let (mut core, limits) = watch_core();
    assert!(report(&mut core, &limits, 7, 0, 0, b"older").iter().all(|r| matches!(r, Request::Decided)));
    assert!(report(&mut core, &limits, 7, 1, 0, b"").iter().all(|r| matches!(r, Request::Decided)));
    let opened = watch_outputs(
        &mut core,
        &limits,
        Event::WatchRun {
            watcher: Token::new(30),
            sign_in: 10,
            task: 1,
            turn: Some(views::TurnName { attempt: 7, turn: 1, retry: 0 }),
            offset: 3,
        },
    );
    assert_stream(current_stream(&opened), 30, views::TurnName { attempt: 7, turn: 1, retry: 1 }, 0, 0, b"", false);
    assert!(!opened.iter().any(|r| matches!(r, Request::Write(_))));
}

#[test]
fn the_last_watch_can_close_and_reopen_after_an_unwatched_retry() {
    let (mut core, limits) = watch_core();
    open_run_watch(&mut core, &limits, 30);
    report(&mut core, &limits, 7, 0, 0, b"original");
    watch_outputs(&mut core, &limits, Event::StreamDelivered { watcher: Token::new(30) });
    let ended = watch_outputs(&mut core, &limits, Event::Unwatch { watcher: Token::new(30) });
    assert!(ended.iter().any(|r| matches!(r, Request::Held(held) if matches!(held.as_ref(), Held::WatchEnded { .. }))));
    report(&mut core, &limits, 7, 1, 0, b"replacement");
    let opened = watch_outputs(
        &mut core,
        &limits,
        Event::Watch { watcher: Token::new(31), sign_in: 10, subject: Subject::Run { task: 1 }, window: window() },
    );
    assert_stream(
        current_stream(&opened),
        31,
        views::TurnName { attempt: 7, turn: 1, retry: 1 },
        0,
        11,
        b"replacement",
        false,
    );
}

#[test]
fn opaque_content_calls_are_attempt_fenced_without_deriving_a_text_turn() {
    let (mut core, limits) = watch_core();
    report(&mut core, &limits, 7, 1, 0, b"text");
    let call = views::Call {
        identity: views::CallIdentity { owner: Token::new(91), sequence: 8, retry: 2, id: Box::from(&[0xff, 0][..]) },
        tool: Box::from(&b"wait"[..]),
        input: Box::from(&b"{}"[..]),
        cut: false,
        began: None,
    };
    let stale = watch_outputs(
        &mut core,
        &limits,
        Event::RunCall { run: Token::new(1), attempt: Token::new(6), call: call.clone() },
    );
    assert!(stale.iter().all(|r| matches!(r, Request::Decided)));
    let supplied = watch_outputs(
        &mut core,
        &limits,
        Event::RunCall { run: Token::new(1), attempt: Token::new(7), call: call.clone() },
    );
    assert!(supplied.iter().all(|r| matches!(r, Request::Decided)), "unwatched content decides nothing");
    report(&mut core, &limits, 6, 99, 0, b"stale activation");
    let opened = watch_outputs(
        &mut core,
        &limits,
        Event::Watch { watcher: Token::new(30), sign_in: 10, subject: Subject::Run { task: 1 }, window: window() },
    );
    let Request::Now(now) = current_stream(&opened) else { panic!("now stream") };
    let Now::RunText { turn, text, call: Some(shown), .. } = now.as_ref() else { panic!("opaque current call") };
    assert_eq!(*turn, views::TurnName { attempt: 7, turn: 1, retry: 1 });
    assert_eq!(text.as_ref(), b"text");
    assert_eq!(shown.identity, call.identity);
    assert_eq!(shown.began, Some(Wall::EPOCH));
}
