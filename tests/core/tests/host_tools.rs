use jig_core_world::effects::World;
const CHILD: u64 = 2;
const EFFECT: u64 = 1;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_test_domain as root;
use skein_lib::{Duration, Token};

fn relay(world: &mut World, name: &[u8], tool: &[u8], input: &[u8]) {
    let assignment = world.assigned.as_ref().expect("assigned chat");
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Relay {
        channel: Token::new(7),
        run: Token::new(assignment.task),
        attempt: Token::new(assignment.attempt),
        call: fleet::Call {
            name: name.into(),
            tool: tool.into(),
            input: input.into(),
            writes: true,
            deadline: Duration::from_secs(2),
        },
    })));
}

fn answer(world: &World) -> &core::SettledAnswer {
    &world.host_answers.last().expect("the host received a rendered answer").1.answer
}

#[test]
fn a_message_call_is_decoded_decided_and_answered_in_words() {
    let mut world = World::new(170, false);
    world.hold_procedures = true;
    world.delegate();
    let task = world.procedure.expect("delegated child");
    let input = format!(r#"{{"target":{task},"form":"words","words":"is the build green?"}}"#);
    world.tool_call(b"message", input.as_bytes(), true);
    assert_eq!(answer(&world), &core::SettledAnswer::Host { error: false, body: b"Sent message 1".as_slice().into() });
    let Some(root::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Live(child)))) =
        world.store.rows.get(&root::Key::Core(core::Key::Tasks(jig_domain_tasks::Key::Live(task))))
    else {
        panic!("child");
    };
    assert_eq!(child.inbox.last().expect("message delivered").words.as_ref(), b"is the build green?");
}

#[test]
fn a_call_whose_input_does_not_decode_is_answered_an_error_and_commits_nothing() {
    let mut world = World::new(171, false);
    let commits = world.store.applied;
    world.tool_call(b"message", br#"{"form":"words","words":"is the build green?"}"#, true);
    assert_eq!(world.store.applied, commits, "a call that does not decode decides nothing");
    assert_eq!(
        answer(&world),
        &core::SettledAnswer::Host { error: true, body: b"message: the field target is missing".as_slice().into() }
    );
}

#[test]
fn a_call_asked_again_after_an_error_is_decoded_again() {
    let mut world = World::new(172, false);
    world.hold_procedures = true;
    world.delegate();
    world.tool_call(b"message", b"{}", true);
    let name = world.host_answers.last().expect("error").0.clone();
    let task = world.procedure.expect("delegated child");
    let input = format!(r#"{{"target":{task},"form":"words","words":"try again"}}"#);
    relay(&mut world, &name, b"message", input.as_bytes());
    assert_eq!(answer(&world), &core::SettledAnswer::Host { error: false, body: b"Sent message 1".as_slice().into() });
}

#[test]
fn a_call_whose_name_does_not_decode_is_answered_an_error() {
    let mut world = World::new(173, false);
    let commits = world.store.applied;
    relay(&mut world, b"bad-name", b"message", br#"{"target":1,"form":"words","words":"hello"}"#);
    assert_eq!(world.store.applied, commits);
    assert_eq!(
        answer(&world),
        &core::SettledAnswer::Host { error: true, body: b"message: the field name is malformed".as_slice().into() }
    );
}

#[test]
fn a_delegate_call_makes_its_batch_and_answers_their_numbers() {
    let mut world = World::new(174, false);
    world.hold_procedures = true;
    world.tool_call(b"delegate", br#"{"batch":[{"executor":{"kind":"procedure","connector":1,"code":1},"spec":{"words":"check"},"contract":{"kind":"report","words":64},"authority":{"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":20},"notes":0}}]}"#, true);
    assert_eq!(
        answer(&world),
        &core::SettledAnswer::Host { error: false, body: b"Delegated tasks: 2".as_slice().into() }
    );
    assert_eq!(world.procedure, Some(2));
}

#[test]
fn an_amend_naming_an_old_spec_revision_is_answered_moved() {
    let mut world = World::new(176, false);
    world.hold_procedures = true;
    world.delegate();
    let task = world.procedure.expect("delegated child");
    let amend = format!(
        r#"{{"target":{task},"revision":0,"amendment":{{"spec":{{"words":"check again"}},"reason":"new evidence"}}}}"#
    );
    world.tool_call(b"amend", amend.as_bytes(), true);
    world.turn(1, 0, None, b"delegate and amendment answered");
    world.turn(2, 0, None, b"record the answers");
    world.tool_call(b"amend", amend.as_bytes(), true);
    assert_eq!(
        answer(&world),
        &core::SettledAnswer::Host { error: true, body: b"Moved: task 2 is at spec revision 1 now".as_slice().into() }
    );
}

#[test]
fn an_amend_naming_its_spec_revision_is_done_with_the_next() {
    let mut world = World::new(177, false);
    world.hold_procedures = true;
    world.delegate();
    let task = world.procedure.expect("delegated child");
    let amend = format!(
        r#"{{"target":{task},"revision":0,"amendment":{{"spec":{{"words":"check again"}},"reason":"new evidence"}}}}"#
    );
    world.tool_call(b"amend", amend.as_bytes(), true);
    assert_eq!(
        answer(&world),
        &core::SettledAnswer::Host { error: false, body: b"Done: task 2 is at spec revision 1".as_slice().into() }
    );
}

#[test]
fn agents_tools_scenario_family() {
    use jig_core_world::{agents_tools::STORIES, effects::Cut};
    for story in STORIES {
        let commits = (story.run)(&mut World::new(190, false), None);
        for cut in [Cut::Submitted(commits), Cut::Durable(commits)] {
            let mut world = World::new(190, false);
            (story.run)(&mut world, Some(cut));
            assert_eq!(world.reached_cut, Some(cut), "{} reaches its cut", story.name);
        }
    }
}

#[test]
fn a_release_naming_an_old_hold_revision_is_answered_moved() {
    let story = jig_core_world::agents_tools::STORIES
        .iter()
        .find(|story| story.name == "release moved")
        .expect("release story");
    (story.run)(&mut World::new(178, false), None);
}
#[test]
fn a_release_naming_its_hold_revision_lifts_the_hold() {
    let mut world = World::new(179, false);
    world.hold_procedures = true;
    world.delegate();
    world.send(root::Event::Core(core::Event::Tasks(jig_domain_tasks::Event::Hold {
        task: CHILD,
        why: jig_domain_tasks::Hold::Budget,
    })));
    world.tool_call(b"release", format!(r#"{{"target":{CHILD},"revision":1}}"#).as_bytes(), true);
    assert_eq!(
        answer(&world),
        &core::SettledAnswer::Host {
            error: false,
            body: format!("Done: task {CHILD} is at spec revision 0").as_bytes().into()
        }
    );
    assert_ne!(jig_core_world::fixtures::live_row(&world, CHILD).revisions.hold, 0);
    assert!(!matches!(jig_core_world::fixtures::live_row(&world, CHILD).phase, jig_domain_tasks::Phase::Held { .. }));
}
#[test]
fn a_release_of_an_uncertain_effect_without_again_is_refused() {
    let mut world = World::new(180, false);
    world.hold_procedures = true;
    world.delegate();
    world.send(root::Event::Core(core::Event::Tasks(jig_domain_tasks::Event::Hold {
        task: CHILD,
        why: jig_domain_tasks::Hold::Uncertain { entry: EFFECT },
    })));
    world.tool_call(b"release", format!(r#"{{"target":{CHILD},"revision":1}}"#).as_bytes(), true);
    assert_eq!(
        answer(&world),
        &core::SettledAnswer::Host {
            error: true,
            body: format!("Refused: task {CHILD} holds an effect whose outcome is uncertain; say again: true or false")
                .as_bytes()
                .into()
        }
    );
    assert_eq!(jig_core_world::fixtures::live_row(&world, CHILD).revisions.hold, 1);
}
#[test]
fn a_decide_naming_an_old_escalation_revision_is_answered_moved() {
    let mut world = World::new(181, false);
    world.hold_procedures = true;
    world.delegate();
    for why in [jig_domain_tasks::Hold::Stopped, jig_domain_tasks::Hold::Budget] {
        world.send(root::Event::Core(core::Event::Tasks(jig_domain_tasks::Event::Hold { task: CHILD, why })));
    }
    world.tool_call(
        b"decide",
        format!(r#"{{"kind":"escalation","task":{CHILD},"revision":1,"decision":"release"}}"#).as_bytes(),
        true,
    );
    assert_eq!(
        answer(&world),
        &core::SettledAnswer::Host {
            error: true,
            body: format!("Moved: task {CHILD} is held at hold revision 2 now, for its budget").as_bytes().into()
        }
    );
}
