use super::Borrowed;
use crate::page::project::ProjectPage;
use jig_client::{
    Latest,
    contract::{Delivery, TaskLine},
};
use skein_lib::List;
#[derive(Debug)]
pub struct Project<'a> {
    pub state: &'a ProjectPage,
    pub goals: List<TaskLine>,
    pub goals_more: bool,
    pub listed_goals: List<TaskLine>,
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
        for (handle, watch) in &self.ui.handles.watches {
            let Some(document) = self.client.document(*handle) else {
                continue;
            };
            match document.latest {
                Some(Latest::Jig(Delivery::Tasks(goals)))
                    if goals_watch(&watch.subject, self.ui.shown_project(), self.ui.limits.rows) =>
                {
                    page.goals_more = goals.more;
                    for row in &goals.rows {
                        if page.goals.push(row.clone()).is_err() {
                            break;
                        }
                    }
                }
                Some(Latest::Jig(Delivery::Tasks(tasks)))
                    if watch.subject
                        == jig_client::contract::Subject::Tasks {
                            query: jig_client::contract::TaskQuery {
                                project: self.ui.shown_project(),
                                ..page.task_filter
                            },
                            window: self.ui.limits.rows,
                        } =>
                {
                    page.tasks_more = tasks.more;
                    for row in &tasks.rows {
                        if page.tasks.push(row.clone()).is_err() {
                            break;
                        }
                    }
                }
                Some(Latest::Jig(_) | Latest::Family(_)) | None => {}
            }
        }
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
        page
    }
}

fn goals_watch(subject: &jig_client::contract::Subject, project: Option<u32>, window: u32) -> bool {
    let Some(project) = project else { return false };
    *subject == jig_client::contract::Subject::Tasks { query: crate::queries::goals(project), window }
}
