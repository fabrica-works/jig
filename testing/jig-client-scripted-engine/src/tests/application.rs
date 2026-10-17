use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
struct OwnDocument {
    revision: u64,
    words: Text,
    rows: List<Text>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct OwnAsk {
    words: Text,
    transport: Text,
    core: Option<List<Ask>>,
    raw: bool,
}
impl payload::Payload for OwnDocument {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.words.heap(bounds)?.checked_add(self.rows.heap(bounds)?)
    }
}
impl payload::Payload for OwnAsk {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.words.heap(bounds)?.checked_add(self.transport.heap(bounds)?)?.checked_add(self.core.heap(bounds)?)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct OwnApplication {
    document: OwnDocument,
}
impl Application for OwnApplication {
    type Ask = OwnAsk;
    type Answer = OwnDocument;
    type Subject = u32;
    type Document = OwnDocument;
    fn fits(&self, value: ApplicationValue<'_, Self>, bounds: &Bounds) -> bool {
        match value {
            ApplicationValue::Ask(value) => payload::fits(value, bounds),
            ApplicationValue::Answer(value) | ApplicationValue::Document(value) => payload::fits(value, bounds),
            ApplicationValue::Subject(value) => payload::fits(value, bounds),
            ApplicationValue::State(value) => payload::fits(&value.document, bounds),
        }
    }
    fn decide(&mut self, _: &Truth, _: u64, ask: &OwnAsk, _: Wall) -> ApplicationDecision<OwnDocument> {
        self.document = OwnDocument {
            revision: self.document.revision + 1,
            words: ask.words.clone(),
            rows: if ask.words.bytes().is_empty() { List::with_capacity(33) } else { list(vec![]) },
        };
        ApplicationDecision {
            answer: self.document.clone(),
            changes: if ask.raw {
                vec![Change::Seen { party: 1, results: 3, replies: vec![] }]
            } else {
                ask.core.as_ref().map_or_else(
                    || {
                        vec![Change::Request(Box::new(Ask::Seen {
                            through: SeenThrough::Results { through: self.document.revision },
                        }))]
                    },
                    |asks| asks.iter().cloned().map(|ask| Change::Request(Box::new(ask))).collect(),
                )
            },
        }
    }
    fn document(&self, _: &Truth, _: u64, subject: &u32) -> Option<OwnDocument> {
        (*subject == 0 && self.document.words.bytes() != b"gone").then(|| self.document.clone())
    }
    fn marks(&self, _: &Change) -> Vec<u32> {
        vec![0]
    }
}

fn own() -> OwnApplication {
    OwnApplication { document: OwnDocument { revision: 0, words: text(b"initial"), rows: list(vec![]) } }
}
fn own_script() -> Script {
    let mut setup = live_script();
    let entries: Vec<_> = (1..=3)
        .map(|position| InboxEntry {
            object: ObjectKey::Result(9 + position),
            project: 1,
            group: InboxGroup::Results,
            position,
            at: Wall::EPOCH,
            lineage: list(vec![]),
            goal: None,
            recipient: None,
            stall: None,
            reply: None,
        })
        .collect();
    for entry in &entries {
        let ObjectKey::Result(task_number) = entry.object else { unreachable!() };
        let mut chip = task().chip;
        chip.task = task_number;
        chip.phase = Phase::Done;
        chip.details = Box::new(ChipPhase::Ended { how: Ending::Done, at: Wall::EPOCH });
        setup.truth.objects.insert(
            entry.object,
            ReadValue::Result(Box::new(jig_client_domain::contract::Result {
                task: chip,
                ending: Ending::Done,
                acts: list(vec![]),
                value: None,
                at: Some(Wall::EPOCH),

                details: None,
            })),
        );
        setup.truth.object_projects.insert(entry.object, 1);
    }
    let read = Read::Inbox {
        group: jig_client_domain::contract::InboxReadGroup::Updates,
        project: None,
        kind: None,
        goal: None,
        after: None,
    };
    setup.truth.frames.get_mut(&1).expect("frame").updates.value = 3;
    setup.truth.sources.push(Source { party: 1, read: read.clone(), rows: Rows::Inbox(entries.clone()) });
    for through in 1..=3 {
        let mut latest = frame(1);
        latest.updates.value = u32::try_from(3 - through).expect("three results");
        setup.rules.push(Rule {
            party: Some(1),
            ask: Ask::Seen { through: SeenThrough::Results { through } },
            answer: Answer::Done(Outcome::Changed),
            changes: vec![
                Change::Seen { party: 1, results: through, replies: vec![] },
                Change::Frame(latest),
                Change::Source(Source {
                    party: 1,
                    read: read.clone(),
                    rows: Rows::Inbox(entries.iter().filter(|entry| entry.position > through).cloned().collect()),
                }),
            ],
        });
    }
    setup
}
fn own_ask(transport: &[u8]) -> OwnAsk {
    OwnAsk { words: text(b"changed"), transport: text(transport), core: None, raw: false }
}
#[expect(clippy::too_many_lines, reason = "one scenario follows a shared key through both vocabularies and a restart")]
fn scenario_application_hooks_share_keys_commits_retention_and_restart() {
    let mut engine = Engine::with_application(own_script(), 16, own()).expect("hook kit");
    engine.open_application(Time::ZERO, 11, Token::new(1), 7, 1, 0).expect("own watch");
    drop(engine.poll(Time::ZERO));
    engine
        .send_application(Time::ZERO, Wall::EPOCH, 11, [1; 16], Token::new(2), 7, 1, [9; 32], own_ask(b"one"))
        .expect("own request");
    engine
        .send_application(
            at(1),
            Wall::EPOCH,
            12,
            [1; 16],
            Token::new(3),
            7,
            1,
            [9; 32],
            own_ask(b"different transport"),
        )
        .expect("same digest");
    engine.read_application(at(1), 11, Token::new(4), 7, 1, &0).expect("own read");
    assert!(engine.poll(at(9_999_999)).is_empty());
    engine.taken(Token::new(1));
    let up = engine.poll(at(10_000_000));
    assert_eq!(engine.commits().len(), 1);
    assert_eq!(engine.application().document.revision, 1);
    assert_eq!(engine.truth().results_seen.get(&1), Some(&1));
    let Up::ApplicationAnswer { envelope: Envelope::Final, answer: Some(answer), .. } = &up[0] else {
        panic!("own answer");
    };
    assert_eq!(
        &up[1],
        &Up::ApplicationAnswer { attempt: Token::new(3), envelope: Envelope::Final, answer: Some(answer.clone()) }
    );
    assert!(
        up.contains(&Up::ApplicationRead { token: Token::new(4), terminal: Ok(engine.application().document.clone()) })
    );
    assert!(up.contains(&Up::App { watch: Token::new(1), document: engine.application().document.clone() }));
    assert_eq!(engine.commits()[0].application.as_ref().expect("own ledger").digest, Some([9; 32]));
    engine
        .send(
            at(10_000_001),
            Wall::EPOCH,
            11,
            [1; 16],
            Token::new(5),
            Ask::StartChat { project: 1, words: text(b"collision") },
        )
        .expect("jig collision");
    assert!(engine.poll(at(10_000_001)).contains(&Up::Answer {
        attempt: Token::new(5),
        envelope: Envelope::Final,
        answer: Some(Answer::Refused(Refusal::KeyConflict))
    }));
    engine
        .send_application(
            at(10_000_001),
            Wall::EPOCH,
            11,
            [1; 16],
            Token::new(6),
            8,
            1,
            [9; 32],
            own_ask(b"owner conflict"),
        )
        .expect("owner collision");
    assert_eq!(
        engine.poll(at(10_000_001)),
        vec![Up::ApplicationAnswer {
            attempt: Token::new(6),
            envelope: Envelope::Final,
            answer: Some(Err(Refusal::KeyConflict))
        }]
    );
    engine
        .send_application(at(10_000_001), Wall::EPOCH, 11, [2; 16], Token::new(7), 7, 1, [8; 32], own_ask(b"pending"))
        .expect("pending own request");
    assert_eq!(engine.application().document.revision, 2);
    engine.fault(at(10_000_002), Fault::Restart { offline: Duration::from_millis(1) });
    assert_eq!(engine.application().document.revision, 1);
    assert_eq!(engine.truth().results_seen.get(&1), Some(&1));
    let up = engine.poll(at(10_000_002));
    assert!(up.contains(&Up::ApplicationAnswer { attempt: Token::new(7), envelope: Envelope::Lost, answer: None }));
    assert!(up.contains(&Up::Ended { token: Token::new(1), why: WatchEnd::Dropped }));
    engine
        .send_application(at(11_000_002), Wall::EPOCH, 12, [1; 16], Token::new(8), 7, 1, [9; 32], own_ask(b"retained"))
        .expect("retained own request");
    assert_eq!(
        engine.poll(at(11_000_002)),
        vec![Up::ApplicationAnswer { attempt: Token::new(8), envelope: Envelope::Final, answer: Some(answer.clone()) }]
    );
    engine
        .send_application(at(11_000_002), Wall::EPOCH, 11, [2; 16], Token::new(9), 7, 1, [8; 32], own_ask(b"retry"))
        .expect("remade own request");
    engine.fault(at(11_000_002), Fault::Lose { attempt: Token::new(9) });
    assert_eq!(
        engine.poll(at(21_000_002)),
        vec![Up::ApplicationAnswer { attempt: Token::new(9), envelope: Envelope::Lost, answer: None }]
    );
    engine
        .send_application(
            at(21_000_003),
            Wall::EPOCH,
            11,
            [2; 16],
            Token::new(10),
            7,
            1,
            [8; 32],
            own_ask(b"after loss"),
        )
        .expect("recover own answer");
    assert!(
        matches!(&engine.poll(at(21_000_003))[0],Up::ApplicationAnswer { answer:Some(Ok(doc)),.. } if doc.revision==2)
    );
    assert_eq!(engine.commits().len(), 2);
    let invalid = OwnDocument { revision: 3, words: text(b"bad"), rows: List::with_capacity(33) };
    assert_eq!(
        engine.act_application(at(21_000_003), Wall::EPOCH, 7, 1, OwnApplication { document: invalid }, vec![]),
        Err(Invalid::Payload)
    );
    assert_eq!(engine.application().document.revision, 2);
    let next = OwnDocument { revision: 3, words: text(b"story"), rows: list(vec![]) };
    engine
        .act_application(at(21_000_003), Wall::EPOCH, 7, 1, OwnApplication { document: next.clone() }, vec![])
        .expect("own story");
    engine.read_application(at(21_000_003), 11, Token::new(11), 7, 1, &0).expect("story read");
    assert!(engine.poll(at(31_000_002)).is_empty());
    assert_eq!(engine.poll(at(31_000_003)), vec![Up::ApplicationRead { token: Token::new(11), terminal: Ok(next) }]);
}
fn scenario_hook_limits_visibility_conflicts_and_expired_retention() {
    let mut setup = own_script();
    setup.bounds.watches = 1;
    let mut engine = Engine::with_application(setup, 17, own()).expect("shared limits");
    engine.open_application(Time::ZERO, 11, Token::new(1), 7, 1, 0).expect("own watch");
    engine.open(Time::ZERO, 11, Token::new(2), Subject::Frame).expect("jig over limit");
    assert!(engine.poll(Time::ZERO).contains(&Up::Ended { token: Token::new(2), why: WatchEnd::Busy }));
    engine.close(Time::ZERO, Token::new(1));
    assert_eq!(engine.poll(Time::ZERO), vec![Up::Ended { token: Token::new(1), why: WatchEnd::Closed }]);
    engine.open(Time::ZERO, 11, Token::new(3), Subject::Frame).expect("jig watch");
    engine.open_application(Time::ZERO, 11, Token::new(4), 7, 1, 0).expect("own over limit");
    assert!(engine.poll(Time::ZERO).contains(&Up::Ended { token: Token::new(4), why: WatchEnd::Busy }));
    engine.read_application(Time::ZERO, 11, Token::new(5), 7, 2, &0).expect("hidden project");
    assert_eq!(
        engine.poll(Time::ZERO),
        vec![Up::ApplicationRead { token: Token::new(5), terminal: Err(ReadNone::Unknown) }]
    );
    engine
        .send(
            Time::ZERO,
            Wall::EPOCH,
            11,
            [1; 16],
            Token::new(6),
            Ask::StartChat { project: 1, words: text(b"jig first") },
        )
        .expect("jig key");
    engine
        .send_application(Time::ZERO, Wall::EPOCH, 12, [1; 16], Token::new(7), 7, 1, [9; 32], own_ask(b"own conflict"))
        .expect("jig owns key");
    let up = engine.poll(at(10_000_000));
    assert!(up.contains(&Up::ApplicationAnswer {
        attempt: Token::new(7),
        envelope: Envelope::Final,
        answer: Some(Err(Refusal::KeyConflict))
    }));
    assert_eq!(engine.application().document.revision, 0);
    engine
        .send_application(
            at(1_010_000_001),
            Wall::EPOCH,
            11,
            [1; 16],
            Token::new(8),
            7,
            1,
            [9; 32],
            own_ask(b"expired jig key"),
        )
        .expect("expired key becomes own");
    let up = engine.poll(at(1_020_000_001));
    assert!(up.iter().any(|value| matches!(value,Up::ApplicationAnswer { attempt,answer:Some(Ok(doc)),.. } if *attempt==Token::new(8) && doc.revision==1)));
    engine
        .send_application(
            at(2_020_000_002),
            Wall::EPOCH,
            11,
            [1; 16],
            Token::new(9),
            7,
            1,
            [9; 32],
            own_ask(b"expired own key"),
        )
        .expect("own retention expired");
    let up = engine.poll(at(2_030_000_002));
    assert!(up.iter().any(|value| matches!(value,Up::ApplicationAnswer { attempt,answer:Some(Ok(doc)),.. } if *attempt==Token::new(9) && doc.revision==2)));
    assert_eq!(engine.commits().len(), 3);
}
fn scenario_deployment_parts_dynamic_rules_and_scheduling() {
    let mut setup = live_script();
    setup.truth.frames.get_mut(&1).expect("frame").projects = list(vec![]);
    let mut engine = Engine::with_application(setup, 18, own()).expect("deployment role");
    engine
        .read_scoped_application(Time::ZERO, 11, Token::new(1), 7, ApplicationScope::Deployment, &0)
        .expect("deployment part");
    assert_eq!(
        engine.poll(Time::ZERO),
        vec![Up::ApplicationRead { token: Token::new(1), terminal: Ok(own().document) }]
    );
    engine
        .open_scoped_application(Time::ZERO, 11, Token::new(2), 7, ApplicationScope::Deployment, 0)
        .expect("deployment watch");
    drop(engine.poll(Time::ZERO));
    let mut removed = frame(1);
    removed.deployment_role = None;
    removed.projects = list(vec![]);
    engine.act(Time::ZERO, Wall::EPOCH, Change::Frame(removed)).expect("role removed");
    assert_eq!(engine.poll(at(10_000_000)), vec![Up::Ended { token: Token::new(2), why: WatchEnd::NotAllowed }]);
    let mut engine = Engine::new(live_script(), 19).expect("dynamic rules");
    let ask = Ask::StartChat { project: 1, words: text(b"world generated words") };
    let mut made = task();
    made.chip.task = 9;
    let rule = Rule {
        party: Some(1),
        ask: ask.clone(),
        answer: Answer::Done(Outcome::Made { what: Made::Chat { task: 9 } }),
        changes: vec![Change::Object { project: 1, key: ObjectKey::Task(9), value: ReadValue::Task(Box::new(made)) }],
    };
    engine.script(rule.clone()).expect("world scripts actual words");
    send(&mut engine, 0, 11, 1, 1, ask.clone());
    assert_eq!(engine.pending_commits().next().expect("staged ledger").durable, None);
    assert!(engine.commits().is_empty());
    let first = engine.poll(at(10_000_000));
    let mut invalid = rule.clone();
    invalid.answer = Answer::Refused(Refusal::NotOffered);
    assert_eq!(engine.script(invalid), Err(Invalid::Script));
    let mut replacement = rule;
    replacement.answer = Answer::Refused(Refusal::NotOffered);
    replacement.changes = vec![];
    engine.script(replacement).expect("replace future outcome");
    send(&mut engine, 10_000_001, 11, 1, 2, ask);
    assert_eq!(
        final_answer(&engine.poll(at(10_000_001))[0]),
        final_answer(&first[0]),
        "retained key ignores later script changes"
    );
    engine.fault(at(10_000_001), Fault::Restart { offline: Duration::from_millis(1) });
    drop(engine.poll(at(10_000_001)));
    send(&mut engine, 12_000_000, 11, 2, 3, Ask::StartChat { project: 1, words: text(b"fresh") });
    assert!(
        engine.next_due().is_some_and(|due| due >= at(12_000_000)),
        "late activity after offline cannot move the world's clock back"
    );
    let mut engine = Engine::new(live_script(), 20).expect("run heartbeat");
    engine.open(Time::ZERO, 11, Token::new(1), Subject::Run { task: 8, turn: None, offset: 0 }).expect("run");
    drop(engine.poll(Time::ZERO));
    let turn = TurnName { attempt: 1, number: 0 };
    engine.write(Time::ZERO, 8, turn, Wall::EPOCH, &text(b"first"), None).expect("write");
    drop(engine.poll(Time::ZERO));
    for nanos in [500_000_000, 1_000_000_000] {
        engine.write(at(nanos), 8, turn, Wall::EPOCH, &text(b"more"), None).expect("still writing");
    }
    assert_eq!(
        engine.poll(at(1_000_000_000)),
        vec![Up::Watch { token: Token::new(1), event: WatchEvent::Alive }],
        "writing does not postpone the slow client's heartbeat"
    );
    let mut engine = Engine::new(live_script(), 21).expect("slow heartbeat");
    engine.fault(Time::ZERO, Fault::Slow { span: Duration::from_secs(5) });
    engine.open(Time::ZERO, 11, Token::new(1), Subject::Frame).expect("slow frame");
    for seconds in 1..5 {
        assert!(engine.poll(at(seconds * 1_000_000_000)).is_empty());
    }
    assert_eq!(
        engine.due.iter().filter(|due| matches!(due.up, Up::Watch { event: WatchEvent::Alive, .. })).count(),
        1,
        "slow transport coalesces heartbeat output too"
    );
}
fn scenario_application_jig_requests_cannot_bypass_admission() {
    for (raw, core, expected) in [
        (true, None, Refusal::NotOffered),
        (
            false,
            Some(list(vec![
                Ask::Seen { through: SeenThrough::Results { through: 1 } },
                Ask::StartChat { project: 2, words: text(b"hidden") },
            ])),
            Refusal::Unknown,
        ),
        (
            false,
            Some(list(vec![
                Ask::Seen { through: SeenThrough::Results { through: 1 } },
                Ask::Release { project: 1, task: 8, revision: Revision { value: 1 }, again: None },
            ])),
            Refusal::NotOffered,
        ),
    ] {
        let mut engine = Engine::with_application(own_script(), 22, own()).expect("admission kit");
        let mut ask = own_ask(b"admission");
        ask.raw = raw;
        ask.core = core;
        engine
            .send_application(Time::ZERO, Wall::EPOCH, 11, [1; 16], Token::new(1), 7, 1, [1; 32], ask.clone())
            .expect("judged ask");
        assert_eq!(engine.application(), &own(), "staged own state rolls back with a refused jig act");
        assert_eq!(engine.truth().results_seen.get(&1), None, "earlier admitted jig act rolls back too");
        assert_eq!(engine.truth().frames.get(&1).expect("frame").updates.value, 3);
        assert!(engine.poll(at(9_999_999)).is_empty());
        assert_eq!(
            engine.poll(at(10_000_000)),
            vec![Up::ApplicationAnswer {
                attempt: Token::new(1),
                envelope: Envelope::Final,
                answer: Some(Err(expected.clone()))
            }]
        );
        assert!(engine.commits()[0].changes.is_empty());
        assert!(engine.commits()[0].application.as_ref().expect("hook ledger").requests.is_empty());
        engine
            .send_application(at(10_000_001), Wall::EPOCH, 12, [1; 16], Token::new(2), 7, 1, [1; 32], ask)
            .expect("retained refusal");
        assert_eq!(
            engine.poll(at(10_000_001)),
            vec![Up::ApplicationAnswer {
                attempt: Token::new(2),
                envelope: Envelope::Final,
                answer: Some(Err(expected))
            }]
        );
        assert_eq!(engine.commits().len(), 1);
    }
}
fn scenario_application_values_are_bounded_before_retention() {
    let mut invalid = own();
    invalid.document.rows = List::with_capacity(33);
    assert_eq!(Engine::with_application(own_script(), 23, invalid).err(), Some(Invalid::Payload));
    let mut engine = Engine::with_application(own_script(), 23, own()).expect("bounded kit");
    let mut ask = own_ask(b"input capacity");
    ask.core = Some(List::with_capacity(33));
    assert_eq!(
        engine.send_application(Time::ZERO, Wall::EPOCH, 11, [1; 16], Token::new(1), 7, 1, [1; 32], ask),
        Err(Invalid::Payload)
    );
    let mut ask = own_ask(b"program output");
    ask.words = text(b"");
    assert_eq!(
        engine.send_application(Time::ZERO, Wall::EPOCH, 11, [1; 16], Token::new(1), 7, 1, [1; 32], ask),
        Err(Invalid::Payload)
    );
    assert_eq!(engine.application(), &own());
    assert_eq!(engine.pending_commits().count(), 0);
    assert!(engine.poll(at(10_000_000)).is_empty());
    engine
        .send_application(
            at(10_000_000),
            Wall::EPOCH,
            11,
            [1; 16],
            Token::new(1),
            7,
            1,
            [1; 32],
            own_ask(b"valid retry"),
        )
        .expect("invalid values retained no key or answer");
    assert!(
        matches!(&engine.poll(at(20_000_000))[0],Up::ApplicationAnswer {answer:Some(Ok(doc)),..} if doc.revision==1)
    );
    let mut setup = own_script();
    setup.bounds.document_bytes = 5000;
    let mut engine = Engine::with_application(setup, 24, own()).expect("whole-byte kit");
    let mut ask = own_ask(b"large");
    ask.words = text(&[b'a'; 3000]);
    ask.transport = text(&[b'b'; 3000]);
    assert_eq!(
        engine.send_application(Time::ZERO, Wall::EPOCH, 11, [1; 16], Token::new(1), 7, 1, [1; 32], ask),
        Err(Invalid::Payload),
        "nested fields are priced together"
    );
}
fn scenario_application_watches_coalesce_and_end_truthfully() {
    let mut engine = Engine::with_application(own_script(), 25, own()).expect("own watch kit");
    engine.open_application(Time::ZERO, 11, Token::new(1), 7, 1, 0).expect("watch");
    assert!(engine.poll(Time::ZERO).contains(&Up::App { watch: Token::new(1), document: own().document }));
    for revision in 1..=2 {
        let mut state = own();
        state.document.revision = revision;
        engine.act_application(Time::ZERO, Wall::EPOCH, 7, 1, state, vec![]).expect("own story");
    }
    assert!(engine.poll(at(10_000_000)).is_empty(), "taken is required while the first delivery is held");
    engine.taken(Token::new(1));
    let up = engine.poll(at(10_000_000));
    assert!(matches!(&up[..],[Up::App {document,..}] if document.revision==2));
    let mut gone = own();
    gone.document.words = text(b"gone");
    engine.act_application(at(10_000_000), Wall::EPOCH, 7, 1, gone, vec![]).expect("subject ends");
    engine.taken(Token::new(1));
    assert!(engine.poll(at(19_999_999)).is_empty());
    assert_eq!(engine.poll(at(20_000_000)), vec![Up::Ended { token: Token::new(1), why: WatchEnd::SubjectEnded }]);
    assert!(engine.poll(at(20_000_000)).is_empty());
    engine.open_application(at(20_000_000), 11, Token::new(2), 7, 1, 0).expect("absent subject");
    assert_eq!(engine.poll(at(20_000_000)), vec![Up::Ended { token: Token::new(2), why: WatchEnd::Unknown }]);
}
pub(super) fn run() {
    scenario_application_hooks_share_keys_commits_retention_and_restart();
    scenario_hook_limits_visibility_conflicts_and_expired_retention();
    scenario_deployment_parts_dynamic_rules_and_scheduling();
    scenario_application_jig_requests_cannot_bypass_admission();
    scenario_application_values_are_bounded_before_retention();
    scenario_application_watches_coalesce_and_end_truthfully();
}
