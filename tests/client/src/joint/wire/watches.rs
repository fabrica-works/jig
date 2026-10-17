//! Live watches preserve their exact owner subject, window and run position
//! (client/domain/contract.md, 7). The sign-in supplies the authenticated party;
//! translation does not choose visibility or reinterpret a role's project.
use jig_client_domain::contract as client;
use jig_domain_core::{Event, WatchRefusal, documents};
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use skein_lib::Token;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invalid {
    WrongRoleProject,
}

/// # Errors
/// Refuses a mismatched executor role project or any whole window/subject allocation above the receiving bounds.
pub fn watch(watcher: Token, sign_in: u64, value: &client::Subject, default_rows: u32) -> Result<Event, Invalid> {
    if let client::Subject::Run { task, turn, offset } = value {
        return Ok(Event::WatchRun {
            watcher,
            sign_in,
            task: *task,
            turn: turn.map(|name| views::TurnName { attempt: name.attempt, turn: name.number, retry: name.retry }),
            offset: *offset,
        });
    }
    let (subject, window) = subject(value, default_rows)?;
    Ok(Event::Watch { watcher, sign_in, subject, window })
}

fn subject(value: &client::Subject, default_rows: u32) -> Result<(documents::Watched, documents::Window), Invalid> {
    let (subject, rows) = match value {
        client::Subject::Me => (documents::Watched::Frame, default_rows),
        client::Subject::Task { task } => (documents::Watched::Task { task: *task }, default_rows),
        client::Subject::Tree { task, window } => (documents::Watched::Tree { task: *task }, *window),
        client::Subject::Chips { tasks } => (documents::Watched::Chips { tasks: tasks.clone() }, default_rows),
        client::Subject::Inbox { window } => (documents::Watched::Inbox, *window),
        client::Subject::Goals { project } => (documents::Watched::Goals { project: *project }, default_rows),
        client::Subject::Tasks { project, filter, window } => {
            (documents::Watched::Tasks { project: *project, filter: task_filter(*project, filter)? }, *window)
        }
        client::Subject::Conversations => (documents::Watched::Conversations, default_rows),
        client::Subject::Run { task, turn: _, offset: _ } => (documents::Watched::Run { task: *task }, default_rows),
        client::Subject::Deployment => (documents::Watched::Deployment, default_rows),
    };
    Ok((subject, documents::Window { rows, after: None }))
}

/// # Errors
/// Refuses a mismatched executor role project or any whole window/subject allocation above the receiving bounds.
pub fn task_filter(project: u32, value: &client::TaskFilter) -> Result<documents::TaskFilter, Invalid> {
    Ok(documents::TaskFilter {
        phase: value.phase.map(phase),
        executor: value.executor.map(|value| executor(project, value)).transpose()?,
        goal: value.goal,
        asked_by_me: value.asked_by_me,
    })
}

/// # Errors
/// Refuses a mismatched executor role project or any whole window/subject allocation above the receiving bounds.
pub fn executor(project: u32, value: client::Executor) -> Result<tasks::Executor, Invalid> {
    Ok(match value {
        client::Executor::Agent { charter } => tasks::Executor::Agent { charter },
        client::Executor::Procedure { connector, kind } => tasks::Executor::Procedure { connector, code: kind },
        client::Executor::Party { party } => tasks::Executor::Person(tasks::PersonAddress::Person(party)),
        client::Executor::Role { project: actual, role } => {
            if actual != project {
                return Err(Invalid::WrongRoleProject);
            }
            tasks::Executor::Person(tasks::PersonAddress::Role(role))
        }
    })
}

#[must_use]
pub fn phase(value: client::Phase) -> documents::TaskPhase {
    match value {
        client::Phase::Waiting => documents::TaskPhase::Waiting,
        client::Phase::Idle => documents::TaskPhase::Idle,
        client::Phase::Due => documents::TaskPhase::Due,
        client::Phase::Preparing => documents::TaskPhase::Preparing,
        client::Phase::Claimed => documents::TaskPhase::Claimed,
        client::Phase::Running => documents::TaskPhase::Running,
        client::Phase::BackingOff => documents::TaskPhase::BackingOff,
        client::Phase::Active => documents::TaskPhase::Active,
        client::Phase::Closing => documents::TaskPhase::Closing,
        client::Phase::Held => documents::TaskPhase::Held,
        client::Phase::Done => documents::TaskPhase::Done,
        client::Phase::Failed => documents::TaskPhase::Failed,
        client::Phase::Cancelled => documents::TaskPhase::Cancelled,
        client::Phase::Ended => documents::TaskPhase::Ended,
    }
}

#[must_use]
pub fn ended(value: views::End) -> client::WatchEnd {
    match value {
        views::End::Unwatched => client::WatchEnd::Closed,
        views::End::Gone => client::WatchEnd::SubjectEnded,
        views::End::NotAllowed => client::WatchEnd::NotAllowed,
        views::End::SignedOut => client::WatchEnd::SignedOut,
        views::End::Stopping => client::WatchEnd::Stopping,
    }
}

#[must_use]
pub fn refused(value: WatchRefusal) -> client::WatchEnd {
    match value {
        WatchRefusal::SignedOut => client::WatchEnd::SignedOut,
        WatchRefusal::NotAllowed => client::WatchEnd::NotAllowed,
        WatchRefusal::Unknown => client::WatchEnd::Unknown,
        WatchRefusal::Busy => client::WatchEnd::Busy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_ended_watch_selector_keeps_its_exact_live_query_identity_without_a_terminal_outcome() {
        let filter = client::TaskFilter {
            phase: Some(client::Phase::Ended),
            executor: None,
            goal: Some(u64::MAX),
            asked_by_me: true,
        };
        let actual = task_filter(u32::MAX, &filter).expect("actual generic ended selector is legal");
        assert_eq!(actual.phase, Some(documents::TaskPhase::Ended));
        assert_eq!(actual.goal, Some(u64::MAX));
        assert!(actual.asked_by_me);
        assert_eq!(actual.executor, None);
        assert_ne!(actual.phase, Some(documents::TaskPhase::Done));
        assert_ne!(actual.phase, Some(documents::TaskPhase::Failed));
        assert_ne!(actual.phase, Some(documents::TaskPhase::Cancelled));
    }
    use skein_lib::List;

    #[test]
    fn run_watch_translation_keeps_absent_and_full_width_turn_positions_on_the_actual_producer() {
        for turn in [None, Some(client::ProvisionalTurnName { attempt: u64::MAX, number: u32::MAX, retry: u32::MAX })] {
            let result = watch(Token::new(71), 19, &client::Subject::Run { task: u64::MAX, turn, offset: u32::MAX }, 2)
                .expect("the complete supplied fixture is admitted");
            let Event::WatchRun { watcher, sign_in, task, turn: actual, offset } = result else {
                panic!("run positions require the named producer")
            };
            assert_eq!(watcher, Token::new(71));
            assert_eq!(sign_in, 19);
            assert_eq!(task, u64::MAX);
            assert_eq!(offset, u32::MAX);
            assert_eq!(
                actual,
                turn.map(|name| views::TurnName { attempt: name.attempt, turn: name.number, retry: name.retry })
            );
        }
    }

    #[test]
    fn watch_subject_translation_keeps_allocated_chip_identities_authentication_and_explicit_windows() {
        let mut chips = List::with_capacity(4);
        chips.push(u64::MAX).expect("the complete supplied fixture is admitted");
        chips.push(1).expect("the complete supplied fixture is admitted");
        let values = [
            ("Me", client::Subject::Me, documents::Watched::Frame, 5),
            ("Task", client::Subject::Task { task: u64::MAX }, documents::Watched::Task { task: u64::MAX }, 5),
            (
                "Tree",
                client::Subject::Tree { task: 9, window: u32::MAX },
                documents::Watched::Tree { task: 9 },
                u32::MAX,
            ),
            ("Chips", client::Subject::Chips { tasks: chips.clone() }, documents::Watched::Chips { tasks: chips }, 5),
            ("Inbox", client::Subject::Inbox { window: 0 }, documents::Watched::Inbox, 0),
            ("Goals", client::Subject::Goals { project: u32::MAX }, documents::Watched::Goals { project: u32::MAX }, 5),
            ("Conversations", client::Subject::Conversations, documents::Watched::Conversations, 5),
            ("Deployment", client::Subject::Deployment, documents::Watched::Deployment, 5),
        ];
        for (label, value, expected, rows) in values {
            assert_eq!(
                subject(&value, 5).expect("the complete supplied fixture is admitted"),
                (expected, documents::Window { rows, after: None }),
                "{label}"
            );
        }
    }

    #[test]
    fn task_watch_filters_keep_full_executor_identity_and_refuse_a_different_role_project() {
        let filter = client::TaskFilter {
            phase: Some(client::Phase::BackingOff),
            executor: Some(client::Executor::Procedure { connector: u16::MAX, kind: u32::MAX }),
            goal: Some(u64::MAX),
            asked_by_me: true,
        };
        assert_eq!(
            task_filter(8, &filter).expect("the complete supplied fixture is admitted"),
            documents::TaskFilter {
                phase: Some(documents::TaskPhase::BackingOff),
                executor: Some(tasks::Executor::Procedure { connector: u16::MAX, code: u32::MAX }),
                goal: Some(u64::MAX),
                asked_by_me: true,
            }
        );
        let mut filter = filter;
        filter.executor = Some(client::Executor::Role { project: 9, role: u32::MAX });
        assert_eq!(task_filter(8, &filter), Err(Invalid::WrongRoleProject));
        filter.executor = Some(client::Executor::Role { project: 8, role: u32::MAX });
        assert_eq!(
            task_filter(8, &filter).expect("the complete supplied fixture is admitted").executor,
            Some(tasks::Executor::Person(tasks::PersonAddress::Role(u32::MAX)))
        );
    }
}
