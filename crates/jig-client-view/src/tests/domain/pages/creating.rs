use super::*;
use crate::{action::Act, fields::Purpose, page::project::goals::GoalForm};
const PROJECT: u32 = 1;
const CHARTER: u32 = 1;
const BUDGET: u64 = 3;
const PRIORITY: u32 = 0;
#[test]
fn withdrawing_creating_closes_the_goal_confirmation_without_a_send() {
    let (mut domain, opened) = project_open(ProjectSection::Goals);
    let read = project_read(&opened);
    drive(
        &mut domain,
        Time::ZERO,
        ReadPage { read, result: ReadResult::Value(ReadValue::Project(Box::new(project_document()))) },
    );
    drive(&mut domain, Time::ZERO, Event::Act { action: Act::Edit { field: Purpose::NewGoal, text: text(b"a goal") } });
    drive(
        &mut domain,
        Time::ZERO,
        Event::Act {
            action: Act::GoalForm {
                form: GoalForm {
                    project: PROJECT,
                    executor: OwnerExecutor::Agent { charter: CHARTER },
                    budget: BUDGET,
                    deadline: None,
                    priority: PRIORITY,
                },
            },
        },
    );
    drive(&mut domain, Time::ZERO, Event::Act { action: Act::Submit { field: Purpose::NewGoal } });
    assert!(domain.project_page().form.confirmation.is_some());
    crate::tests::fixtures::set_creating(&mut domain, PROJECT, &Admission::Allowed);
    assert!(domain.project_page().form.confirmation.is_none());
    assert_eq!(domain.project_page().form.refusal, Some(Refusal::NotOffered));
    assert!(drive(&mut domain, Time::ZERO, Event::Act { action: Act::Confirm }).is_empty());
}
