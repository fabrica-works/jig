use super::*;
fn making_script() -> (Script, Ask) {
    let mut setup = live_script();
    let ask = Ask::StartChat { project: 1, words: text(b"hello") };
    let mut made = task();
    made.chip.title = jig_client::contract::DocumentText::complete(text(b"made"));
    setup.rules.push(Rule {
        party: Some(1),
        ask: ask.clone(),
        answer: Answer::Done(Outcome::Made { what: Made::Chat { task: 8 } }),
        changes: vec![Change::Object { project: 1, key: ObjectKey::Task(8), value: ReadValue::Task(Box::new(made)) }],
    });
    (setup, ask)
}
fn scenario_restart_cuts_offline_and_retained_keys() {
    let (setup, ask) = making_script();
    let mut engine = Engine::new(setup, 10).expect("making script");
    send(&mut engine, 0, 11, 1, 1, ask.clone());
    engine
        .write(Time::ZERO, 8, TurnName { attempt: 1, number: 0 }, Wall::EPOCH, &text(b"provisional"), None)
        .expect("run");
    engine.open(Time::ZERO, 11, Token::new(3), Subject::Task { task: 8 }).expect("watch");
    engine.read(Time::ZERO, 11, Token::new(4), Read::Task { task: 8 }).expect("read");
    engine.fault(at(1), Fault::Restart { offline: Duration::from_millis(10) });
    assert!(engine.commits().is_empty());
    assert!(engine.runs.is_empty());
    assert!(engine.keys.is_empty());
    assert!(
        matches!(engine.truth().objects.get(&ObjectKey::Task(8)),Some(ReadValue::Task(doc)) if doc.chip.title.bytes()==b"chat")
    );
    let up = engine.poll(at(1));
    assert!(up.contains(&Up::Answer { attempt: Token::new(1), envelope: Envelope::Lost, answer: None }));
    assert!(up.contains(&Up::Read { token: Token::new(4), terminal: ReadResult::None(ReadNone::Unreachable) }));
    assert!(up.contains(&Up::Ended { token: Token::new(3), why: WatchEnd::Dropped }));
    send(&mut engine, 2, 11, 1, 2, ask.clone());
    assert_eq!(
        engine.poll(at(2)),
        vec![Up::Answer { attempt: Token::new(2), envelope: Envelope::Again, answer: None }]
    );
    assert_eq!(engine.next_due(), Some(at(10_000_001)));
    send(&mut engine, 10_000_001, 11, 1, 5, ask.clone());
    let answered = engine.poll(at(20_000_001));
    assert_eq!(engine.commits().len(), 1, "undurable request was remade once");
    assert!(matches!(final_answer(&answered[0]), Answer::Done(Outcome::Made { .. })));
    engine.fault(at(20_000_002), Fault::Restart { offline: Duration::from_millis(1) });
    send(&mut engine, 21_000_002, 12, 1, 6, ask);
    assert_eq!(final_answer(&engine.poll(at(21_000_002))[0]), final_answer(&answered[0]));
    assert_eq!(engine.commits().len(), 1, "durable key survives the restart across sign-ins");
}
fn scenario_busy_not_ready_loss_sign_out_versions_and_watch_faults() {
    for fault in [Fault::Busy, Fault::NotReady] {
        let (setup, ask) = making_script();
        let mut engine = Engine::new(setup, 11).expect("fault script");
        engine.fault(Time::ZERO, fault);
        send(&mut engine, 0, 11, 1, 1, ask.clone());
        assert_eq!(
            engine.poll(Time::ZERO),
            vec![Up::Answer { attempt: Token::new(1), envelope: Envelope::Again, answer: None }]
        );
        assert!(engine.keys.is_empty());
        assert!(engine.pending.is_empty());
        send(&mut engine, 1, 11, 1, 2, ask);
        drop(engine.poll(at(10_000_001)));
        assert_eq!(engine.commits().len(), 1);
    }
    let (setup, ask) = making_script();
    let mut engine = Engine::new(setup, 12).expect("loss");
    send(&mut engine, 0, 11, 1, 1, ask.clone());
    send(&mut engine, 1, 12, 1, 2, ask.clone());
    engine.fault(at(1), Fault::Lose { attempt: Token::new(1) });
    assert!(engine.poll(at(9_999_999)).is_empty());
    assert_eq!(
        engine.poll(at(10_000_000)),
        vec![
            Up::Answer { attempt: Token::new(1), envelope: Envelope::Lost, answer: None },
            Up::Answer {
                attempt: Token::new(2),
                envelope: Envelope::Final,
                answer: Some(Answer::Done(Outcome::Made { what: Made::Chat { task: 8 } }))
            },
        ],
        "loss targets one attempt while its joined attempt keeps the same durable final answer"
    );
    send(&mut engine, 10_000_001, 12, 1, 3, ask.clone());
    let recovered = engine.poll(at(10_000_001));
    assert!(matches!(final_answer(&recovered[0]), Answer::Done(Outcome::Made { .. })));
    assert_eq!(engine.commits().len(), 1);
    let (setup, ask) = making_script();
    let mut engine = Engine::new(setup, 13).expect("sign-out");
    send(&mut engine, 0, 11, 1, 1, ask.clone());
    engine.read(Time::ZERO, 11, Token::new(2), Read::Frame).expect("read");
    engine.open(Time::ZERO, 11, Token::new(3), Subject::Frame).expect("watch");
    engine.fault(at(1), Fault::SignOut { sign_in: 11 });
    let up = engine.poll(at(1));
    assert!(up.contains(&Up::Answer { attempt: Token::new(1), envelope: Envelope::SignedOut, answer: None }));
    assert!(up.contains(&Up::Read { token: Token::new(2), terminal: ReadResult::None(ReadNone::SignedOut) }));
    assert!(up.contains(&Up::Ended { token: Token::new(3), why: WatchEnd::SignedOut }));
    drop(engine.poll(at(10_000_000)));
    send(&mut engine, 10_000_001, 12, 1, 4, ask);
    assert!(matches!(final_answer(&engine.poll(at(10_000_001))[0]), Answer::Done(Outcome::Made { .. })));
    assert_eq!(engine.commits().len(), 1);
    let mut engine = Engine::new(live_script(), 14).expect("versions");
    engine.open(Time::ZERO, 11, Token::new(1), Subject::Frame).expect("frame");
    drop(engine.poll(Time::ZERO));
    engine.taken(Token::new(1));
    engine.fault(Time::ZERO, Fault::Versions(VersionRange { first: 2, last: 2 }));
    assert_eq!(engine.versions(), VersionRange { first: 2, last: 2 });
    assert!(engine.commits().is_empty(), "a hello range is protocol configuration, not a business commit");
    assert!(engine.poll(at(9_999_999)).is_empty());
    assert!(engine.poll(at(10_000_000)).is_empty(), "changing hello never rewrites Frame");
    let versions = VersionRange { first: u32::MAX, last: u32::MAX };
    engine.fault(at(10_000_000), Fault::Versions(versions));
    assert_eq!(engine.versions(), versions);
    assert!(engine.commits().is_empty(), "full-width ranges remain protocol facts");
    assert!(engine.poll(at(10_000_000)).is_empty(), "a full-width range leaves Frame unchanged");
    engine.fault(at(10_000_000), Fault::DropWatch { watch: Token::new(1) });
    assert_eq!(engine.poll(at(10_000_000)), vec![Up::Ended { token: Token::new(1), why: WatchEnd::Dropped }]);
    engine.open(at(10_000_000), 11, Token::new(2), Subject::Frame).expect("new frame");
    drop(engine.poll(at(10_000_000)));
    engine.stop(at(10_000_000));
    assert_eq!(engine.poll(at(10_000_000)), vec![Up::Ended { token: Token::new(2), why: WatchEnd::Stopping }]);
}
fn scenario_slow_current_documents_preserve_in_flight_and_barriers() {
    let mut engine = Engine::new(live_script(), 15).expect("slow");
    engine.fault(Time::ZERO, Fault::Slow { span: Duration::from_millis(50) });
    engine.open(Time::ZERO, 11, Token::new(1), Subject::Frame).expect("frame");
    let mut latest = frame(1);
    latest.needs.questions = 7;
    engine.act(at(10_000_000), Wall::EPOCH, Change::Frame(latest.clone())).expect("latest frame");
    assert!(engine.poll(at(49_999_999)).is_empty());
    let up = engine.poll(at(60_000_000));
    assert!(up.contains(&Up::Watch {
        token: Token::new(1),
        event: WatchEvent::Delivered(Delivery::Frame(Box::new(latest)))
    }));
    engine.fault(at(60_000_000), Fault::Slow { span: Duration::from_millis(1) });
    let mut latest = frame(1);
    latest.needs.questions = 8;
    engine.act(at(60_000_000), Wall::EPOCH, Change::Frame(latest.clone())).expect("changed while in flight");
    assert!(engine.poll(at(70_000_000)).is_empty(), "slow does not acknowledge a delivery the client still holds");
    engine.taken(Token::new(1));
    assert_eq!(
        engine.poll(at(70_000_000)),
        vec![Up::Watch { token: Token::new(1), event: WatchEvent::Delivered(Delivery::Frame(Box::new(latest))) }]
    );
}
pub(super) fn run() {
    scenario_restart_cuts_offline_and_retained_keys();
    scenario_busy_not_ready_loss_sign_out_versions_and_watch_faults();
    scenario_slow_current_documents_preserve_in_flight_and_barriers();
}
