//! Authenticated person-task standing and task handoff
//! (domain/engine.md, section 4.4; domain/people.md, section 5).

use alloc::boxed::Box;
use jig_core_people as people;
use jig_core_tasks as tasks;
use skein_lib::{ReplyTo, Token};

use crate::{Core, PersonTaskRoute};

pub(crate) fn number(ask: &people::Ask) -> u64 {
    match ask {
        people::Ask::TakePerson { task, .. }
        | people::Ask::HandBackPerson { task, .. }
        | people::Ask::AnswerPerson { task, .. } => *task,
        people::Ask::MarkSeen { .. }
        | people::Ask::Application { .. }
        | people::Ask::Move { .. }
        | people::Ask::DecideProposal { .. }
        | people::Ask::Say { .. }
        | people::Ask::AnswerQuestion { .. }
        | people::Ask::Prioritise { .. }
        | people::Ask::Amend { .. }
        | people::Ask::EditNote { .. }
        | people::Ask::MakeService { .. }
        | people::Ask::SetRoles { .. }
        | people::Ask::Adopt { .. }
        | people::Ask::ChangePolicy { .. }
        | people::Ask::SetPool { .. }
        | people::Ask::DecideEscalation { .. }
        | people::Ask::StartChat { .. }
        | people::Ask::Stop { .. }
        | people::Ask::Cancel { .. }
        | people::Ask::Release { .. }
        | people::Ask::SetGoal { .. } => unreachable!("person-task route owns its ask"),
    }
}

fn person_result(result: people::PersonResult) -> tasks::TaskResult {
    match result {
        people::PersonResult::Report { words } => tasks::TaskResult::Report { words },
        people::PersonResult::Verdict { code, words } => tasks::TaskResult::Verdict { code, words },
        people::PersonResult::Failure { reason } => tasks::TaskResult::Failure { reason },
    }
}

impl Core {
    /// Recheck the stored owner and current address before answering an act on an ended task.
    pub(crate) fn person_task_ended(
        &self,
        person: u64,
        project: u32,
        task: u64,
        row: Option<Box<tasks::TaskRecord>>,
        busy: bool,
    ) -> people::Outcome {
        if busy {
            return people::Outcome::Again(people::Again::Busy);
        }
        let Some(row) = row else { return people::Outcome::Refused(people::Refusal::Unknown) };
        let role = self.people.role(person, project);
        let allowed = row.number == task
            && task != 0
            && task <= self.counters.deployment().tasks
            && row.project == project
            && role.is_some()
            && match row.executor {
                tasks::Executor::Person(tasks::PersonAddress::Person(address)) => address == person,
                tasks::Executor::Person(tasks::PersonAddress::Role(address)) => {
                    role.expect("membership").number() == address
                }
                tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } => false,
            };
        if !allowed {
            return people::Outcome::Refused(people::Refusal::Unknown);
        }
        let ending = match &row.phase {
            tasks::Phase::Ended(ending) => ending,
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
                return people::Outcome::Refused(people::Refusal::Unknown);
            }
        };
        let refusal = match row.person_answer {
            Some(answer) if answer.by != 0 && answer.by <= self.counters.deployment().people => {
                people::Refusal::Ended { how: people::Ending::Decided { by: answer.by, at: answer.at } }
            }
            Some(_) => people::Refusal::Unknown,
            None => crate::routing::refusals::ended(ending),
        };
        people::Outcome::Refused(refusal)
    }

    #[expect(clippy::too_many_lines, reason = "one exhaustive route handles all person task requests")]
    pub fn person_task(
        &mut self,
        request: Token,
        person: u64,
        project: u32,
        role: Option<people::Role>,
        ask: people::Ask,
    ) -> Result<tasks::Event, people::Refusal> {
        let task = number(&ask);
        let Some(context) = self.tasks.delegation(task) else {
            return Err(people::Refusal::Unknown);
        };
        let allowed = context.project == project
            && match self.tasks.executor(task).expect("live task has executor") {
                tasks::Executor::Person(tasks::PersonAddress::Person(address)) => {
                    address == person
                        && match &ask {
                            people::Ask::AnswerPerson { .. } => true,
                            people::Ask::TakePerson { .. } | people::Ask::HandBackPerson { .. } => false,
                            people::Ask::MarkSeen { .. }
                            | people::Ask::Application { .. }
                            | people::Ask::Move { .. }
                            | people::Ask::DecideProposal { .. }
                            | people::Ask::Say { .. }
                            | people::Ask::AnswerQuestion { .. }
                            | people::Ask::Prioritise { .. }
                            | people::Ask::Amend { .. }
                            | people::Ask::EditNote { .. }
                            | people::Ask::MakeService { .. }
                            | people::Ask::SetRoles { .. }
                            | people::Ask::Adopt { .. }
                            | people::Ask::ChangePolicy { .. }
                            | people::Ask::SetPool { .. }
                            | people::Ask::DecideEscalation { .. }
                            | people::Ask::StartChat { .. }
                            | people::Ask::Stop { .. }
                            | people::Ask::Cancel { .. }
                            | people::Ask::Release { .. }
                            | people::Ask::SetGoal { .. } => unreachable!("person-task route owns its ask"),
                        }
                }
                tasks::Executor::Person(tasks::PersonAddress::Role(address)) => {
                    let handing_back = match &ask {
                        people::Ask::HandBackPerson { .. } => true,
                        people::Ask::TakePerson { .. } | people::Ask::AnswerPerson { .. } => false,
                        people::Ask::MarkSeen { .. }
                        | people::Ask::Application { .. }
                        | people::Ask::Move { .. }
                        | people::Ask::DecideProposal { .. }
                        | people::Ask::Say { .. }
                        | people::Ask::AnswerQuestion { .. }
                        | people::Ask::Prioritise { .. }
                        | people::Ask::Amend { .. }
                        | people::Ask::EditNote { .. }
                        | people::Ask::MakeService { .. }
                        | people::Ask::SetRoles { .. }
                        | people::Ask::Adopt { .. }
                        | people::Ask::ChangePolicy { .. }
                        | people::Ask::SetPool { .. }
                        | people::Ask::DecideEscalation { .. }
                        | people::Ask::StartChat { .. }
                        | people::Ask::Stop { .. }
                        | people::Ask::Cancel { .. }
                        | people::Ask::Release { .. }
                        | people::Ask::SetGoal { .. } => unreachable!("person-task route owns its ask"),
                    };
                    handing_back
                        || match role {
                            Some(holding) => holding.number() == address,
                            None => false,
                        }
                }
                tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } => false,
            };
        if !allowed {
            return Err(people::Refusal::Unknown);
        }
        let row = self.tasks.task(task).expect("live person task");
        if let Some(answer) = row.person_answer {
            return Err(people::Refusal::Ended { how: people::Ending::Decided { by: answer.by, at: answer.at } });
        }
        if let Some(by) = row.taken_by
            && by != person
        {
            return Err(people::Refusal::Taken { by });
        }
        let (route, event) = match ask {
            people::Ask::TakePerson { .. } => (
                PersonTaskRoute::Take(task),
                tasks::Event::TakePerson { reply_to: ReplyTo::new(request), task, person },
            ),
            people::Ask::HandBackPerson { .. } => (
                PersonTaskRoute::HandBack(task),
                tasks::Event::HandBackPerson { reply_to: ReplyTo::new(request), task, person },
            ),
            people::Ask::AnswerPerson { revision, result, .. } => {
                self.person_revision(task, revision)?;
                (
                    PersonTaskRoute::Answer(task),
                    tasks::Event::AnswerPerson {
                        reply_to: ReplyTo::new(request),
                        task,
                        person,
                        result: person_result(result),
                    },
                )
            }
            people::Ask::MarkSeen { .. }
            | people::Ask::Application { .. }
            | people::Ask::Move { .. }
            | people::Ask::DecideProposal { .. }
            | people::Ask::Say { .. }
            | people::Ask::AnswerQuestion { .. }
            | people::Ask::Prioritise { .. }
            | people::Ask::Amend { .. }
            | people::Ask::EditNote { .. }
            | people::Ask::MakeService { .. }
            | people::Ask::SetRoles { .. }
            | people::Ask::Adopt { .. }
            | people::Ask::ChangePolicy { .. }
            | people::Ask::SetPool { .. }
            | people::Ask::DecideEscalation { .. }
            | people::Ask::StartChat { .. }
            | people::Ask::Stop { .. }
            | people::Ask::Cancel { .. }
            | people::Ask::Release { .. }
            | people::Ask::SetGoal { .. } => unreachable!("person-task route owns its ask"),
        };
        assert!(self.person_tasks.insert(request, route) == Ok(None), "one routed person task per keyed flight");
        Ok(event)
    }
}
