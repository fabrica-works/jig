use jig_client::{Ask, Asker, Key, contract as c, shapes};
use jig_client_protocol_world::{client::Story, fixtures};
use skein_lib::{Duration, Time, Wall};
// Five seconds exceeds the finite first bound while remaining inside the policy ceiling.
const SPAN: Duration = Duration::from_secs(5);
const WAIT: Duration = Duration::from_secs(1);
const PROJECT: u32 = 1;
const FIRST_TASK: u64 = 1;
const CREATE: Key = Key { asker: Asker::JIG, number: 85 };
const PROPOSE: Key = Key { asker: Asker::JIG, number: 86 };
fn send(story: &mut Story, key: Key, ask: c::Ask) {
    story.send(key, Ask::Jig(Box::new(ask)));
    assert!(story.run_until(|s| !s.answered(key).is_empty(), WAIT));
}
fn deadline_after(story: &Story, before: Wall) {
    let at = story.task_deadline(FIRST_TASK).expect("made with deadline").as_nanos();
    assert!((before.as_nanos() + SPAN.as_nanos()..=story.wall().as_nanos() + SPAN.as_nanos()).contains(&at));
}
#[test]
fn a_span_crosses_the_client_wire_to_task_admission() {
    let mut story = Story::live_batch(fixtures::SEED, true);
    let mut tasks = fixtures::create_tasks();
    tasks.get_mut(0).expect("first task").deadline = c::NewDeadline::Span(SPAN);
    let before = story.wall();
    send(&mut story, CREATE, c::Ask::Create { project: PROJECT, tasks });
    deadline_after(&story, before);
}
#[test]
fn a_shaped_chat_span_reaches_the_engine() {
    let mut story = Story::live_batch(fixtures::SEED, true);
    let mut me = story.shapes_me();
    me.projects.get_mut(0).expect("project").templates.get_mut(0).expect("chat template").deadline =
        c::TemplateDeadline::Span(SPAN);
    let ask = shapes::chat(&me, PROJECT, jig_client_world::stories::text(b"Hello")).expect("chat offered");
    let before = story.wall();
    send(&mut story, CREATE, ask);
    deadline_after(&story, before);
}
#[test]
fn a_shaped_goal_span_beyond_authority_is_not_automatically_proposed() {
    let first = Wall::from_nanos(fixtures::wall(Time::ZERO).as_nanos() + WAIT.as_nanos());
    let mut story = Story::live_span_goal(fixtures::SEED, first);
    let mut me = story.shapes_me();
    let project = me.projects.get_mut(0).expect("project");
    assert_eq!(project.creating.as_ref().expect("creating").within.latest, Some(first));
    project.templates.get_mut(1).expect("goal template").deadline = c::TemplateDeadline::Span(SPAN);
    let words = jig_client_world::stories::text(b"goal");
    let draft = shapes::GoalDraft {
        words: words.clone(),
        executor: c::OwnerExecutor::Agent { charter: 1 },
        budget: 10,
        deadline: None,
        priority: Some(7),
    };
    let shapes::Judged::Create(ask) = shapes::set_goal(&me, PROJECT, draft, words.clone()) else {
        panic!("span draft within first bounds")
    };
    let c::Ask::Create { tasks, .. } = &ask else { panic!("create") };
    assert_eq!(tasks.get(0).expect("goal").deadline, c::NewDeadline::Span(SPAN));
    let tasks = tasks.clone();
    send(&mut story, CREATE, ask);
    assert_eq!(
        story.answered(CREATE),
        vec![jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::Authority {
                lacking: c::Lacking::Deadline { asked: None, latest: None },
                proposable: Some(true)
            },
            at: None,
        }))]
    );
    assert_eq!(story.party_proposal(1), None);
    assert_eq!(story.task_deadline(FIRST_TASK), None);
    send(
        &mut story,
        PROPOSE,
        c::Ask::Propose { project: PROJECT, action: c::ProposeAction::Batch { tasks }, reason: words },
    );
    assert_eq!(
        story.answered(PROPOSE),
        vec![jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Owner(
            c::OwnerOutcome::Proposed { proposal: 1 }
        ))))]
    );
}
