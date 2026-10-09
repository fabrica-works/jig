use super::*;
use jig_client::contract::*;
use skein_lib::List;
fn text(bytes: &[u8]) -> Text {
    Text::new(bytes.into(), 4096).expect("fixture text fits")
}
fn list<T>(items: Vec<T>) -> List<T> {
    let mut out = List::with_capacity(u32::try_from(items.len()).expect("fixture list fits"));
    for item in items {
        assert!(out.push(item).is_ok());
    }
    out
}
fn party(number: u64) -> Party {
    Party { number, kind: PartyKind::Person, name: text(b"party") }
}
fn budget() -> Budget {
    Budget { total: 100, own: 0, ended: 0, reserved: 0, left: 100, funder: Actor::Deployment }
}
fn frame(number: u64) -> Frame {
    Frame {
        party: party(number),
        projects: list(vec![ProjectRole {
            project: 1,
            name: text(b"project"),
            roles: list(vec![1]),
            chats: list(vec![Charter { number: 1, name: text(b"chat") }]),
            goals: list(vec![]),
            pool: budget(),
            chat_budget: 100,
        }]),
        needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        updates: Count { value: 0, limited: false },
        versions: VersionRange { first: 1, last: 1 },
        retention: Duration::from_secs(1),
        deployment_role: Some(1),
    }
}
fn authority() -> Authority {
    Authority {
        budget: 100,
        grants: list(vec![]),
        task_kinds: list(vec![]),
        depth: 10,
        tasks: 10,
        deadline: None,
        tools: list(vec![]),
        note_scopes: list(vec![]),
    }
}
fn script() -> Script {
    let mut truth = Truth::default();
    truth.frames.insert(1, frame(1));
    truth.frames.insert(2, frame(2));
    truth.sign_ins.insert(11, 1);
    truth.sign_ins.insert(12, 1);
    truth.sign_ins.insert(22, 2);
    Script { truth, ..Script::default() }
}
fn send(engine: &mut Engine, now: u64, sign_in: u64, key: u8, token: u64, ask: Ask) {
    engine
        .send(Time::from_nanos(now), Wall::from_nanos(now), sign_in, [key; 16], Token::new(token), ask)
        .expect("typed request fits");
}
fn final_answer(up: &Up) -> &Answer {
    match up {
        Up::Answer { envelope: Envelope::Final, answer: Some(answer), .. } => answer,
        Up::Read { .. } | Up::Watch { .. } | Up::Ended { .. } | Up::Answer { .. } => {
            panic!("expected a final answer: {up:?}")
        }
    }
}
fn proposal() -> Proposal {
    Proposal {
        key: ProposalKey { proposer: 8, number: 1 },
        project: 1,
        proposer: Actor::Task(8),
        why: text(b"wider"),
        action: ProposalAction::Batch { tasks: list(vec![]), own: false },
        needs: authority(),
        funding: budget(),
        waiting: Recipient::Role { project: 1, role: 1 },
        trail: list(vec![]),
        since: Wall::EPOCH,
        acts: list(vec![Offered { act: ActKind::Accept, admission: Admission::Allowed }]),
        ending: None,
    }
}
#[test]
fn keyed_contract_durability_sign_ins_conflict_and_decided_first() {
    let mut setup = script();
    let p = proposal();
    setup.truth.objects.insert(ObjectKey::Proposal(p.key), ReadValue::Proposal(Box::new(p.clone())));
    let ask = Ask::DecideProposal { project: 1, proposal: p.key, decision: Decision::Accepted };
    let mut engine = Engine::new(setup, 1).expect("valid script");
    send(&mut engine, 0, 11, 1, 1, ask.clone());
    send(&mut engine, 1, 12, 1, 2, ask.clone());
    send(&mut engine, 2, 22, 2, 3, ask.clone());
    assert_eq!(engine.next_due(), Some(Time::from_nanos(10_000_000)));
    assert!(engine.poll(Time::from_nanos(9_999_999)).is_empty());
    assert!(engine.commits().is_empty());
    let up = engine.poll(Time::from_nanos(10_000_002));
    assert_eq!(up.len(), 3);
    assert_eq!(final_answer(&up[0]), final_answer(&up[1]));
    assert_eq!(
        final_answer(&up[2]),
        &Answer::Done(Outcome::DecidedFirst { by: party(1), how: Decision::Accepted, at: Wall::EPOCH })
    );
    assert_eq!(engine.commits().len(), 2);
    assert_eq!(engine.commits()[0].durable, Some(Time::from_nanos(10_000_000)));
    send(
        &mut engine,
        20_000_000,
        11,
        1,
        4,
        Ask::DecideProposal { project: 1, proposal: p.key, decision: Decision::PassedUp },
    );
    assert_eq!(final_answer(&engine.poll(Time::from_nanos(20_000_000))[0]), &Answer::Refused(Refusal::KeyConflict));
    send(&mut engine, 20_000_000, 11, 1, 5, ask);
    assert_eq!(final_answer(&engine.poll(Time::from_nanos(20_000_000))[0]), final_answer(&up[0]));
    assert_eq!(engine.commits().len(), 2);
}
#[test]
fn revision_refusals_scripts_retention_and_whole_payload_admission() {
    let mut setup = script();
    let chip = Chip { task: 8, title: text(b"held"), executor: Executor::Agent { charter: 1 }, phase: Phase::Held };
    let escalation = Escalation {
        ending: None,
        task: chip,
        hold: Hold {
            revision: Revision { value: 2 },
            reason: HoldReason::Budget { spent: 100, limit: 100 },
            before: Phase::Running,
            since: Wall::EPOCH,
            result: None,
        },
        waiting: Recipient::Party(1),
        trail: list(vec![]),
        acts: list(vec![Offered { act: ActKind::Release, admission: Admission::Allowed }]),
    };
    setup.truth.object_projects.insert(ObjectKey::Escalation(8), 1);
    setup.truth.objects.insert(ObjectKey::Escalation(8), ReadValue::Escalation(Box::new(escalation)));
    let mut engine = Engine::new(setup, 2).expect("valid script");
    let ask = Ask::DecideEscalation {
        project: 1,
        task: 8,
        revision: Revision { value: 1 },
        decision: Decision::Released { again: None },
    };
    send(&mut engine, 0, 11, 1, 1, ask);
    assert_eq!(
        final_answer(&engine.poll(Time::from_nanos(10_000_000))[0]),
        &Answer::Refused(Refusal::Moved { now: Revision { value: 2 }, what: Moved::Hold })
    );
    let mut setup = script();
    let ask = Ask::StartChat { project: 1, words: text(b"hello") };
    setup.rules.push(Rule {
        party: None,
        ask: ask.clone(),
        answer: Answer::Refused(Refusal::Authority { lacking: Lacking::Budget { needed: 100, left: 5 } }),
        changes: vec![],
    });
    let mut engine = Engine::new(setup, 3).expect("valid script");
    send(&mut engine, 0, 11, 1, 1, ask.clone());
    assert!(matches!(
        final_answer(&engine.poll(Time::from_nanos(10_000_000))[0]),
        Answer::Refused(Refusal::Authority { .. })
    ));
    send(&mut engine, 2_000_000_000, 11, 1, 2, ask);
    drop(engine.poll(Time::from_nanos(2_010_000_000)));
    assert_eq!(engine.commits().len(), 2, "retention expired before reuse");
    let mut setup = script();
    setup.bounds.document_bytes = 1;
    assert_eq!(Engine::new(setup, 1).err(), Some(Invalid::Payload));
    let mut setup = script();
    setup.truth.frames.get_mut(&1).expect("frame").projects = List::with_capacity(33);
    assert_eq!(Engine::new(setup, 1).err(), Some(Invalid::Payload), "unused list capacity is part of a whole payload");
    let mut engine = Engine::new(script(), 1).expect("valid script");
    let ask = Ask::StartChat {
        project: 1,
        words: Text::new(vec![0; 4097].into_boxed_slice(), 5000).expect("larger producer permits text"),
    };
    assert_eq!(engine.send(Time::ZERO, Wall::EPOCH, 11, [1; 16], Token::new(1), ask), Err(Invalid::Payload));
    assert!(engine.commits().is_empty());
}
