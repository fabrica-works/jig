use super::Borrowed;
use crate::page::project::ProjectPage;
use jig_client::{
    Latest,
    contract::{Delivery, GoalLine, TaskLine},
};
use skein_lib::List;
#[derive(Debug)]
pub struct Project<'a> {
    pub state: &'a ProjectPage,
    pub goals: List<GoalLine>,
    pub goals_more: bool,
    pub listed_goals: List<GoalLine>,
    pub tasks: List<TaskLine>,
    pub tasks_more: bool,
}
impl core::ops::Deref for Project<'_> {
    type Target = ProjectPage;
    fn deref(&self) -> &ProjectPage {
        self.state
    }
}
impl<'a> Borrowed<'a> {
    #[must_use]
    pub fn project_page(&self) -> Project<'a> {
        let mut page = Project {
            state: &self.ui.project_page,
            goals: List::with_capacity(self.ui.limits.rows),
            goals_more: false,
            listed_goals: List::with_capacity(self.ui.limits.rows.checked_mul(2).expect("two goal windows fit")),
            tasks: List::with_capacity(self.ui.limits.rows),
            tasks_more: false,
        };
        for (handle, _) in &self.ui.handles.watches {
            let Some(document) = self.client.document(*handle) else {
                continue;
            };
            match document.latest {
                Some(Latest::Jig(Delivery::Goals(goals))) if Some(goals.project) == self.ui.shown_project() => {
                    page.goals_more = goals.more;
                    for row in &goals.rows {
                        if page.goals.push(row.clone()).is_err() {
                            break;
                        }
                    }
                }
                Some(Latest::Jig(Delivery::Tasks(tasks)))
                    if Some(tasks.project) == self.ui.shown_project() && tasks.filter == page.task_filter =>
                {
                    page.tasks_more = tasks.more;
                    for row in &tasks.rows {
                        if crate::page::project::matches_filter(row, tasks.filter)
                            && page.tasks.push(row.clone()).is_err()
                        {
                            break;
                        }
                    }
                }
                Some(Latest::Jig(_) | Latest::Family(_)) | None => {}
            }
        }
        crate::page::project::goals::order(&mut page.goals);
        crate::page::project::order_tasks(&mut page.tasks);
        for rows in [&page.goals, &page.state.ended] {
            for row in rows {
                let mut named = false;
                for current in &page.listed_goals {
                    named |= current.chip.task == row.chip.task;
                }
                if !named {
                    page.listed_goals.push(row.clone()).expect("two goal sources fit");
                }
            }
        }
        crate::page::project::goals::order(&mut page.listed_goals);
        page
    }
}
