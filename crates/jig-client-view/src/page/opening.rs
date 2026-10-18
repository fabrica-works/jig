//! Sources opened once for each section's owner (client/domain/client.md, 5.1).
use super::routing::{open_once, read};
use crate::{
    Request, Ui,
    address::{ProjectSection, TaskSection},
    handles::Owner,
    page::{Section, project},
};
use jig_client::contract::{self, Read, Subject};
use skein_lib::{Env, Queue};

pub(crate) fn section(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    section: Section,
    owner: Owner,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    match section {
        Section::Inbox { group, project } => {
            domain.inbox.filter = super::inbox::Filter { project, kind: None, goal: None };
            open_once(domain, Subject::Inbox { window: env.limits.rows }, owner, out);
            super::conversations::start_group(domain, group, out);
        }
        Section::Conversations { project } => {
            if let Some(me) = domain.frame(lent).document
                && let Some(query) = crate::queries::chats(me, project)
            {
                domain.chats.more = true;
                open_once(domain, Subject::Tasks { query, window: env.limits.rows }, owner, out);
            }
        }
        Section::Task { task, section } => {
            domain.conversation.task = task;
            open_once(domain, Subject::Task { task }, owner, out);
            match section {
                TaskSection::History => {
                    read(domain, super::task::HISTORY_READ, Read::History { task, subtree: false, after: None }, out);
                }
                TaskSection::Overview
                | TaskSection::Activity
                | TaskSection::Plan
                | TaskSection::Budget
                | TaskSection::Panels => {}
            }
        }
        Section::Project { project, section } => {
            match section {
                ProjectSection::Overview => {
                    if let Some(me) = domain.frame(lent).document
                        && crate::queries::goal_template(me, project).is_some()
                    {
                        open_goals(domain, project, env.limits.rows, owner, out);
                    }
                    open_once(domain, Subject::Inbox { window: env.limits.rows }, owner, out);
                    domain.inbox.filter = super::inbox::Filter { project: Some(project), kind: None, goal: None };
                }
                ProjectSection::Goals => {
                    if let Some(me) = domain.frame(lent).document
                        && let Some(template) = crate::queries::goal_template(me, project)
                    {
                        open_goals(domain, project, env.limits.rows, owner, out);
                        project::ask(
                            domain,
                            project::GOALS_READ,
                            Read::Tasks {
                                query: crate::queries::ended_goals(project, template.label),
                                live: None,
                                ended: None,
                            },
                            out,
                        );
                    }
                }
                ProjectSection::Tasks => {
                    let subject = Subject::Tasks {
                        query: contract::TaskQuery { project: Some(project), ..domain.project_page.task_filter },
                        window: env.limits.rows,
                    };
                    open_once(domain, subject, owner, out);
                    domain.changed();
                }
                ProjectSection::Notes => project::notes::filter(
                    domain,
                    project::NoteFilter { scope: contract::NoteScope::Project(project), description: None },
                    out,
                ),
                ProjectSection::Spend => {
                    project::ask(domain, project::SPEND_READ, Read::Spend { project, after: None }, out);
                }
                ProjectSection::Settings
                | ProjectSection::Resources
                | ProjectSection::Policy
                | ProjectSection::Roles
                | ProjectSection::Recurring => {}
            }
            project::ask(domain, project::PROJECT_READ, Read::Project { project }, out);
        }
        Section::Deployment => {
            open_once(domain, Subject::Deployment, owner, out);
        }
    }
}

fn open_goals(domain: &mut Ui, project: u32, rows: u32, owner: Owner, out: &mut Queue<Request>) {
    open_once(domain, Subject::Tasks { query: crate::queries::goals(project), window: rows }, owner, out);
}
