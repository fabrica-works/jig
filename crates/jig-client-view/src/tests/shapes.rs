use super::cards::{contains, document, drive, text, tree};
use crate::tests::fixtures::{shape_goal_fixture, unoffered_chat};
use crate::{Event, action::Act, fields::Purpose, page::project::goals::GoalForm};
use jig_client::{contract as c, shapes};
use skein_lib::{Duration, Wall};
const PROJECT: u32 = 4;
// The bound and one nanosecond beyond it expose both sides of the named deadline comparison.
const BOUND: Wall = Wall::from_nanos(100);
const LATER: Wall = Wall::from_nanos(101);
const SPAN: Duration = Duration::from_secs(5);
#[test]
fn a_chat_without_creating_sends_nothing() {
    let mut fixture = unoffered_chat();
    assert!(drive(&mut fixture, Event::Act { action: Act::BeginConversation { project: PROJECT } }).is_empty());
}
#[test]
fn a_chat_without_creating_shows_the_refusal() {
    let mut fixture = unoffered_chat();
    drive(&mut fixture, Event::Act { action: Act::BeginConversation { project: PROJECT } });
    assert_eq!(fixture.chats.refusal, Some(c::Refusal::NotOffered));
    assert!(contains(&tree(&fixture, None, None), b"This act is not offered"));
}
#[test]
fn the_goal_prediction_agrees_with_the_shaped_ask() {
    let cases = [
        ("span", c::TemplateDeadline::Span(SPAN), Some(LATER), true),
        ("given at bound", c::TemplateDeadline::Given, Some(BOUND), true),
        ("given beyond bound", c::TemplateDeadline::Given, Some(LATER), false),
        ("none beyond bound", c::TemplateDeadline::None, Some(BOUND), false),
    ];
    for (name, deadline, wall, offered) in cases {
        let mut me = document();
        let project = me.projects.get_mut(0).unwrap();
        project.templates.get_mut(1).unwrap().deadline = deadline;
        let creating = project.creating.as_mut().unwrap();
        creating.within.latest = Some(BOUND);
        creating.within.maximum_deadline = Some(BOUND);
        let mut fixture = shape_goal_fixture(me.clone());
        drive(&mut fixture, Event::Act { action: Act::Edit { field: Purpose::NewGoal, text: text(b"goal") } });
        let form = GoalForm {
            project: PROJECT,
            executor: c::OwnerExecutor::Agent { charter: 1 },
            budget: 10,
            deadline: wall,
            priority: 0,
        };
        drive(&mut fixture, Event::Act { action: Act::GoalForm { form } });
        assert_eq!(contains(&tree(&fixture, None, None), b"This act is not offered"), !offered, "{name}");
        drive(&mut fixture, Event::Act { action: Act::Submit { field: Purpose::NewGoal } });
        assert_eq!(fixture.project_page.form.confirmation.is_some(), offered, "{name}");
        let output = drive(&mut fixture, Event::Act { action: Act::Confirm });
        let judged = shapes::set_goal(&me, PROJECT, crate::page::project::goals::draft(form, text(b"goal")), text(b""));
        match judged {
            shapes::Judged::Create(expected) => {
                let [
                    crate::Request::Client {
                        asked: jig_client::Event::Send { request: jig_client::Ask::Jig(ask), .. },
                    },
                ] = output.as_slice()
                else {
                    panic!("one goal: {name}")
                };
                assert_eq!(**ask, expected, "{name}");
            }
            shapes::Judged::NotOffered(why) => {
                assert_eq!(why, shapes::NotOffered::Ceiling, "{name}");
                assert!(output.is_empty(), "{name}");
            }
            other @ (shapes::Judged::Propose { .. } | shapes::Judged::Refused(_)) => {
                panic!("unexpected judgment: {name}: {other:?}")
            }
        }
    }
}
