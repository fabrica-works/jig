use super::*;

fn name(retry: u32) -> TurnName {
    TurnName { attempt: 1, turn: 1, retry }
}
fn report(domain: &mut Domain, retry: u32, offset: u32, text: &[u8]) -> Box<[Request]> {
    drive(domain, Event::Text { run: Token::new(7), turn: name(retry), offset, text: Box::from(text), call: None })
}
fn take(domain: &mut Domain, watcher: u64) -> Box<[Request]> {
    drive(domain, Event::StreamDelivered { watcher: Token::new(watcher) })
}
fn expected(watcher: u64, retry: u32, from: u32, text: &[u8], cut: bool) -> Request {
    Request::Stream {
        watcher: Token::new(watcher),
        turn: name(retry),
        from,
        to: from.checked_add(u32::try_from(text.len()).unwrap()).unwrap(),
        text: Box::from(text),
        cut,
        call: None,
    }
}

#[test]
fn empty_retry_replaces_the_prefix_after_the_old_flight_is_taken() {
    let mut domain = Domain::new(&LIMITS);
    follow(&mut domain, 1, 7, None, 0);
    assert_eq!(report(&mut domain, 0, 0, b"earlier").as_ref(), &[expected(1, 0, 0, b"earlier", false)]);
    assert!(report(&mut domain, 1, 0, b"").is_empty());
    let writing = writing(&domain, Token::new(7), 16).unwrap();
    assert_eq!(writing.turn, name(1));
    assert_eq!(writing.text.as_ref(), b"");
    assert_eq!(take(&mut domain, 1).as_ref(), &[expected(1, 1, 0, b"", false)]);
    assert!(report(&mut domain, 0, 7, b"stale").is_empty());
    take(&mut domain, 1);
    assert_eq!(report(&mut domain, 1, 0, b"different").as_ref(), &[expected(1, 1, 0, b"different", false)]);
}

#[test]
fn sole_follower_detaches_and_reopens_the_retained_retry() {
    let mut domain = Domain::new(&LIMITS);
    follow(&mut domain, 1, 7, None, 0);
    ready(&mut domain, 1);
    drive(&mut domain, Event::Delivered { watcher: Token::new(1) });
    report(&mut domain, 1, 0, b"retained");
    take(&mut domain, 1);
    assert_eq!(
        drive(&mut domain, Event::Unwatch { watcher: Token::new(1) }).as_ref(),
        &[Request::Ended { watcher: Token::new(1), end: End::Unwatched }]
    );
    assert!(!domain.follows(Token::new(7)));
    assert_eq!(domain.runs(), 1);
    let opened = follow(&mut domain, 2, 7, None, 0);
    assert_eq!(opened.last(), Some(&expected(2, 1, 0, b"retained", false)));
    drive(&mut domain, Event::RunEnded { run: Token::new(7) });
    assert_eq!(domain.runs(), 0);
    assert!(writing(&domain, Token::new(7), 16).is_none());
}

#[test]
fn unwatched_retry_is_kept_and_opened_with_its_actual_name() {
    let mut domain = Domain::new(&LIMITS);
    assert!(report(&mut domain, 0, 0, b"old").is_empty());
    assert!(report(&mut domain, u32::MAX, 0, b"").is_empty());
    let opened = follow(&mut domain, 1, 7, None, 0);
    assert_eq!(opened.last(), Some(&expected(1, u32::MAX, 0, b"", false)));
    take(&mut domain, 1);
    assert_eq!(
        report(&mut domain, u32::MAX, 0, b"replacement").as_ref(),
        &[expected(1, u32::MAX, 0, b"replacement", false)]
    );
}

#[test]
fn openings_resume_only_a_matching_name_and_retained_offset() {
    let mut domain = Domain::new(&LIMITS);
    report(&mut domain, 1, 0, b"abcdef");
    assert_eq!(follow(&mut domain, 1, 7, Some(name(1)), 3).last(), Some(&expected(1, 1, 3, b"def", false)));
    assert_eq!(follow(&mut domain, 2, 7, Some(name(0)), 3).last(), Some(&expected(2, 1, 0, b"abcdef", false)));
    assert_eq!(follow(&mut domain, 3, 7, Some(name(1)), 99).last(), Some(&expected(3, 1, 0, b"abcdef", false)));
}

#[test]
fn a_late_first_report_after_restart_states_its_retained_start() {
    let mut before = Domain::new(&LIMITS);
    report(&mut before, 1, 0, b"before restart");
    let mut after = Domain::new(&LIMITS);
    assert!(writing(&after, Token::new(7), 16).is_none());
    report(&mut after, 1, 8, b"suffix");
    let opened = follow(&mut after, 1, 7, Some(name(1)), 3);
    assert_eq!(opened.last(), Some(&expected(1, 1, 8, b"suffix", true)));
    let writing = writing(&after, Token::new(7), 16).unwrap();
    assert_eq!((writing.from, writing.to, writing.cut), (8, 14, true));
}

#[test]
fn an_unwatched_suffix_keeps_the_original_byte_bound_and_overlap_fence() {
    let mut domain = Domain::new(&LIMITS);
    report(&mut domain, 1, 0, b"abcdefghijklmnop");
    report(&mut domain, 1, 16, b"qrst");
    assert!(report(&mut domain, 1, 18, b"XX").is_empty());
    assert!(report(&mut domain, 1, 21, b"gap").is_empty());
    assert!(report(&mut domain, 1, 0, b"report beyond sixteen bytes").is_empty());
    assert_eq!(follow(&mut domain, 1, 7, Some(name(1)), 2).last(), Some(&expected(1, 1, 4, b"efghijklmnopqrst", true)));
    assert_eq!(writing(&domain, Token::new(7), 8).unwrap().text.as_ref(), b"mnopqrst");
}

#[test]
fn an_unwatched_call_is_retained_and_an_empty_retry_clears_it() {
    let mut domain = Domain::new(&LIMITS);
    let call = Call {
        identity: CallIdentity { owner: Token::new(42), sequence: 1, retry: 0, id: Box::new([]) },
        tool: Box::from(&b"tool"[..]),
        input: Box::from(&b"input"[..]),
        cut: false,
        began: Some(Wall::EPOCH),
    };
    drive(
        &mut domain,
        Event::Text { run: Token::new(7), turn: name(0), offset: 0, text: Box::new([]), call: Some(call.clone()) },
    );
    assert_eq!(writing(&domain, Token::new(7), 16).unwrap().call, Some(call));
    report(&mut domain, 1, 0, b"");
    assert!(writing(&domain, Token::new(7), 16).unwrap().call.is_none());
    assert_eq!(follow(&mut domain, 1, 7, None, 0).last(), Some(&expected(1, 1, 0, b"", false)));
}

fn call(domain: &mut Domain, owner: u64, sequence: u32, retry: u32, id: &[u8], wall: Wall) {
    let mut out = Queue::with_capacity(max_out(&LIMITS));
    step(
        domain,
        &Env { now: Time::ZERO, wall, limits: LIMITS },
        Event::Called {
            attempt: 1,
            run: Token::new(7),
            call: Call {
                identity: CallIdentity { owner: Token::new(owner), sequence, retry, id: Box::from(id) },
                tool: Box::from(&b"tool"[..]),
                input: Box::from(&b"{}"[..]),
                cut: false,
                began: None,
            },
        },
        &mut out,
    );
}

#[test]
fn the_same_tool_and_input_with_different_ids_starts_a_new_call_clock() {
    let mut domain = Domain::new(&LIMITS);
    report(&mut domain, 0, 0, b"");
    call(&mut domain, 9, 1, 0, b"one", Wall::from_nanos(1));
    let first = writing(&domain, Token::new(7), 16).unwrap().call.unwrap();
    call(&mut domain, 9, 1, 0, b"two", Wall::from_nanos(2));
    let second = writing(&domain, Token::new(7), 16).unwrap().call.unwrap();
    assert_eq!((&first.tool, &first.input), (&second.tool, &second.input));
    assert_ne!(first.identity, second.identity);
    assert_eq!((first.began, second.began), (Some(Wall::from_nanos(1)), Some(Wall::from_nanos(2))));
}

#[test]
fn returning_to_an_earlier_call_restarts_its_current_duration_at_the_public_stream() {
    let mut domain = Domain::new(&LIMITS);
    report(&mut domain, 0, 0, b"");
    call(&mut domain, 9, 1, 0, b"A", Wall::from_nanos(1));
    let first = writing(&domain, Token::new(7), 16).unwrap().call.unwrap();
    call(&mut domain, 9, 1, 0, b"B", Wall::from_nanos(2));
    call(&mut domain, 9, 1, 0, b"A", Wall::from_nanos(3));
    let opened = follow(&mut domain, 1, 7, None, 0);
    let Request::Stream { call: Some(current), .. } = opened.last().unwrap() else { panic!("retained current call") };
    assert_eq!(current.identity, first.identity);
    assert_eq!(first.began, Some(Wall::from_nanos(1)));
    assert_eq!(current.began, Some(Wall::from_nanos(3)));
    take(&mut domain, 1);
    call(&mut domain, 9, 1, 0, b"A", Wall::from_nanos(4));
    let retained = writing(&domain, Token::new(7), 16).unwrap().call.unwrap();
    assert_eq!(retained.began, Some(Wall::from_nanos(3)));
}

#[test]
fn repeated_ids_in_one_completion_replace_payload_and_keep_the_first_seen_clock() {
    let mut domain = Domain::new(&LIMITS);
    report(&mut domain, 0, 0, b"");
    call(&mut domain, 9, 1, 0, b"same", Wall::from_nanos(1));
    let mut next = writing(&domain, Token::new(7), 16).unwrap().call.unwrap();
    next.input = Box::from(&b"new"[..]);
    next.began = None;
    let mut out = Queue::with_capacity(max_out(&LIMITS));
    step(
        &mut domain,
        &Env { now: Time::ZERO, wall: Wall::from_nanos(5), limits: LIMITS },
        Event::Called { run: Token::new(7), attempt: 1, call: next },
        &mut out,
    );
    let shown = writing(&domain, Token::new(7), 16).unwrap().call.unwrap();
    assert_eq!(shown.input.as_ref(), b"new");
    assert_eq!(shown.began, Some(Wall::from_nanos(1)));
    call(&mut domain, 9, 2, 0, b"same", Wall::from_nanos(6));
    let next_completion = writing(&domain, Token::new(7), 16).unwrap().call.unwrap();
    assert_ne!(shown.identity, next_completion.identity);
    assert_eq!(next_completion.began, Some(Wall::from_nanos(6)));
    call(&mut domain, 9, 2, 1, b"same", Wall::from_nanos(7));
    assert_eq!(writing(&domain, Token::new(7), 16).unwrap().call.unwrap().began, Some(Wall::from_nanos(7)));
    call(&mut domain, 10, 2, 1, b"same", Wall::from_nanos(8));
    assert_eq!(writing(&domain, Token::new(7), 16).unwrap().call.unwrap().identity.owner, Token::new(10));
}

#[test]
fn non_utf8_ids_stay_whole_and_an_oversized_identity_cannot_replace_a_call() {
    let mut domain = Domain::new(&LIMITS);
    report(&mut domain, 0, 0, b"");
    call(&mut domain, 9, 1, 0, &[0xff, 0, 0x80], Wall::from_nanos(1));
    let shown = writing(&domain, Token::new(7), 16).unwrap().call.unwrap();
    assert_eq!(shown.identity.id.as_ref(), &[0xff, 0, 0x80]);
    call(&mut domain, 9, 1, 0, b"identity past whole bound", Wall::from_nanos(2));
    assert_eq!(writing(&domain, Token::new(7), 16).unwrap().call, Some(shown));
}

#[test]
fn a_committed_turn_releases_its_bytes_but_its_live_run_keeps_admission() {
    let mut domain = Domain::new(&LIMITS);
    report(&mut domain, 1, 0, b"provisional");
    drive(&mut domain, Event::TurnCommitted { run: Token::new(7), attempt: 1, turn: 1 });
    assert_eq!(domain.runs(), 1);
    assert!(writing(&domain, Token::new(7), 16).is_none());
    assert!(report(&mut domain, 1, 0, b"stale").is_empty());
    assert!(writing(&domain, Token::new(7), 16).is_none());
    text(&mut domain, 2, 0, b"next");
    assert_eq!(writing(&domain, Token::new(7), 16).unwrap().turn.turn, 2);
    drive(&mut domain, Event::RunEnded { run: Token::new(7) });
    assert_eq!(domain.runs(), 0);
}
