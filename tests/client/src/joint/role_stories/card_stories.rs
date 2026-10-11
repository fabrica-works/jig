//! Genuine Question and Result cards keep precisely their supplied acts.
use super::*;

#[test]
fn a_real_engine_question_offers_exactly_its_admitted_answer_act() {
    let mut world = world(8520, vec![Box::new([Script::Wait])]);
    let member = world.client_with(3, crate::stories::conversation(), receiving_limits(), client);
    world.person(member, start(b"ask a question"));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let assignment =
        world.front().session.driver.world.peers.as_ref().expect("actual worker").workers[0].seen[0].clone();
    super::engine_stories::action(
        &mut world,
        &assignment,
        1,
        jig_core::NamedAction::Message {
            target: assignment.task,
            kind: jig_core_tasks::MessageKind::Question,
            words: b"which report should I use?".as_slice().into(),
        },
    );
    let within = Duration::from_secs(1);
    world.person(
        member,
        vec![
            Step::Go { address: crate::stories::inbox() },
            Step::See { find: Find::text(b"which report should I use?"), within },
            Step::See { find: Find::exactly_named(Role::Button, b"Answer"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
    let questions: Vec<_> = world
        .front()
        .session
        .card_offer_keys(member)
        .into_iter()
        .filter(|key| matches!(key, client::ObjectKey::Question { .. }))
        .collect();
    assert_eq!(questions.len(), 1);
    super::super::offered_acts::card(&mut world, member, questions[0], b"Question");
    assert!(world.front().session.driver.world.store.rows.values().any(|row| matches!(row,
        jig_test_domain::Record::Core(jig_core::Record::Tasks(jig_core_tasks::Stored::Live(task)))
        if task.number == assignment.task && task.inbox.iter().any(|word|
            word.kind == jig_core_tasks::MessageKind::Question && word.words.as_ref() == b"which report should I use?"))));
}

#[test]
fn a_real_completed_chat_result_offers_exactly_its_admitted_mark_read_act() {
    let mut world = world(
        8521,
        vec![Box::new([
            Script::Silence { for_: Duration::from_millis(100) },
            Script::Finish { report: b"the complete result".as_slice().into() },
        ])],
    );
    let member = world.client_with(3, crate::stories::conversation(), receiving_limits(), client);
    let mut steps = start(b"finish this chat");
    steps.push(Step::Pause { span: Duration::from_millis(200) });
    world.person(member, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let within = Duration::from_secs(1);
    world.person(
        member,
        vec![
            Step::Go { address: crate::stories::inbox() },
            Step::Press { find: Find::exactly_named(Role::Button, b"Result") },
            Step::See { find: Find::text(b"the complete result"), within },
            Step::See { find: Find::exactly_named(Role::Button, b"Mark read"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
    let results: Vec<_> = world
        .front()
        .session
        .card_offer_keys(member)
        .into_iter()
        .filter(|key| matches!(key, client::ObjectKey::Result(_)))
        .collect();
    assert_eq!(results.len(), 1);
    super::super::offered_acts::card(&mut world, member, results[0], b"Result");
    assert!(world.front().session.driver.world.store.rows.values().any(|row| matches!(row,
        jig_test_domain::Record::Core(jig_core::Record::Tasks(jig_core_tasks::Stored::Ended(task)))
        if matches!(&task.phase, jig_core_tasks::Phase::Ended(jig_core_tasks::Ending::Done(
            jig_core_tasks::TaskResult::Report { words })) if words.as_ref() == b"the complete result"))));
}
