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
    for request in out {
        match request {
            Request::Now(now) => match now.as_ref() {
                Now::RunText { .. } => return request,
                Now::Relayed { .. }
                | Now::EffectAnswer { .. }
                | Now::SettledCallRefused { .. }
                | Now::Call { .. }
                | Now::DropCall { .. }
                | Now::Undelivered { .. }
                | Now::SignInRefused { .. }
                | Now::WatchRefused { .. }
                | Now::Account(..)
                | Now::NotesIndexed { .. }
                | Now::NotesRecalled { .. }
                | Now::NotesRefused { .. }
                | Now::NoteBusy { .. }
                | Now::DropPayload { .. }
                | Now::DropAssignment { .. }
                | Now::TurnPayload { .. }
                | Now::AnswerPayload { .. }
                | Now::AcceptedTurn { .. }
                | Now::RefusedPayload { .. }
                | Now::Activate { .. }
                | Now::PrepareAgent { .. }
                | Now::StartPreparation { .. }
                | Now::SeenChat { .. }
                | Now::HistoricalProposal { .. }
                | Now::HistoricalPersonTask { .. }
                | Now::CompleteBrief { .. }
                | Now::BriefCorePlanned { .. }
                | Now::WorkspaceRequest { .. }
                | Now::RunPreparationFailed { .. }
                | Now::RunPrepared { .. }
                | Now::HistoricalEscalation { .. }
                | Now::ProcedureDelegateOutcome { .. }
                | Now::RestoreRefused => {}
            },
            Request::Write(..) | Request::Ask { .. } | Request::Held(..) | Request::Decided => {}
        }
    }
    panic!("current supplied retry reaches the public stream boundary")
}
fn only_decided(out: &[Request]) -> bool {
    for request in out {
        match request {
            Request::Decided => {}
            Request::Write(..) | Request::Ask { .. } | Request::Held(..) | Request::Now(..) => return false,
        }
    }
    true
}
fn has_write(out: &[Request]) -> bool {
    for request in out {
        match request {
            Request::Write(..) => return true,
            Request::Decided | Request::Ask { .. } | Request::Held(..) | Request::Now(..) => {}
        }
    }
    false
}
fn has_watch_end(out: &[Request]) -> bool {
    for request in out {
        match request {
            Request::Held(held) => match held.as_ref() {
                Held::WatchEnded { .. } => return true,
                Held::ApplicationDelivery { .. }
                | Held::ApplicationWatchEnded { .. }
                | Held::ReadLoad { .. }
                | Held::Read { .. }
                | Held::Applied { .. }
                | Held::Delivery { .. }
                | Held::MakeEffect { .. }
                | Held::SettledCall { .. }
                | Held::Assign { .. }
                | Held::Relayed { .. }
                | Held::Inbound { .. }
                | Held::Relay { .. }
                | Held::PeopleReply { .. }
                | Held::CallAnswer { .. }
                | Held::Result { .. }
                | Held::Acknowledge { .. }
                | Held::TaskTerminalAcknowledged { .. }
                | Held::TaskTurnKept { .. }
                | Held::AcknowledgeTurn { .. }
                | Held::Cancel { .. }
                | Held::Refuse { .. }
                | Held::TurnBusy { .. }
                | Held::StopRun { .. }
                | Held::NotesLoad { .. }
                | Held::NotesWritten { .. }
                | Held::NotesDeleted { .. } => {}
            },
            Request::Write(..) | Request::Ask { .. } | Request::Now(..) | Request::Decided => {}
        }
    }
    false
}

#[test]
fn an_unwatched_empty_retry_opens_as_its_current_supplied_identity() {
    let (mut core, limits) = watch_core();
    assert!(only_decided(&report(&mut core, &limits, 7, 0, 0, b"older")));
    assert!(only_decided(&report(&mut core, &limits, 7, 1, 0, b"")));
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
    assert!(!has_write(&opened));
}

#[test]
fn the_last_watch_can_close_and_reopen_after_an_unwatched_retry() {
    let (mut core, limits) = watch_core();
    open_run_watch(&mut core, &limits, 30);
    report(&mut core, &limits, 7, 0, 0, b"original");
    watch_outputs(&mut core, &limits, Event::StreamDelivered { watcher: Token::new(30) });
    let ended = watch_outputs(&mut core, &limits, Event::Unwatch { watcher: Token::new(30) });
    assert!(has_watch_end(&ended));
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
    assert!(only_decided(&stale));
    let supplied = watch_outputs(
        &mut core,
        &limits,
        Event::RunCall { run: Token::new(1), attempt: Token::new(7), call: call.clone() },
    );
    assert!(only_decided(&supplied), "unwatched content decides nothing");
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
