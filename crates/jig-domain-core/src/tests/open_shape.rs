//! The restore copies agree with core's ending-authority check (domain/tasks.md, 5.8).
use super::{fixtures, people, tasks};
use people::{EndedBy, Malformed, TaskExecutor};

const AGENT: TaskExecutor = TaskExecutor::Agent { charter: 1 };
const PERSON: TaskExecutor = TaskExecutor::Person(people::PersonAddress::Person(9));
const PROCEDURE: TaskExecutor = TaskExecutor::Procedure { connector: 1, code: 1 };
const ROWS: u32 = 12; // Two endings, three executors, two conversation flags.

#[test]
fn tasks_people_and_core_agree_on_open_shape() {
    let rows = [
        ("result agent", EndedBy::Result, AGENT, false, true, Ok(())),
        ("result agent conversation", EndedBy::Result, AGENT, true, true, Ok(())),
        ("result person", EndedBy::Result, PERSON, false, true, Ok(())),
        ("result person conversation", EndedBy::Result, PERSON, true, true, Err(Malformed::Conversation)),
        ("result procedure", EndedBy::Result, PROCEDURE, false, true, Ok(())),
        ("result procedure conversation", EndedBy::Result, PROCEDURE, true, true, Err(Malformed::Conversation)),
        ("requester agent", EndedBy::Requester, AGENT, false, false, Err(Malformed::EndedBy)),
        ("requester agent conversation", EndedBy::Requester, AGENT, true, true, Ok(())),
        ("requester person", EndedBy::Requester, PERSON, false, false, Err(Malformed::EndedBy)),
        ("requester person conversation", EndedBy::Requester, PERSON, true, false, Err(Malformed::EndedBy)),
        ("requester procedure", EndedBy::Requester, PROCEDURE, false, false, Err(Malformed::EndedBy)),
        ("requester procedure conversation", EndedBy::Requester, PROCEDURE, true, false, Err(Malformed::EndedBy)),
    ];
    let mut checked = 0_u32;
    for (name, ended_by, executor, conversation, expected_shape, expected_verdict) in rows {
        let mut member = fixtures::member(b"open request", 1);
        member.executor = executor;
        member.flags.ended_by = ended_by;
        member.flags.conversation = conversation;
        let flags = tasks::Flags {
            ended_by: match ended_by {
                EndedBy::Result => tasks::EndedBy::Result,
                EndedBy::Requester => tasks::EndedBy::Requester,
            },
            conversation,
            ..tasks::Flags::PLAIN
        };
        let executor = match executor {
            TaskExecutor::Agent { charter } => tasks::Executor::Agent { charter },
            TaskExecutor::Person(people::PersonAddress::Person(person)) => {
                tasks::Executor::Person(tasks::PersonAddress::Person(person))
            }
            TaskExecutor::Person(people::PersonAddress::Role(role)) => {
                tasks::Executor::Person(tasks::PersonAddress::Role(role))
            }
            TaskExecutor::Procedure { connector, code } => tasks::Executor::Procedure { connector, code },
        };
        let task_shape = tasks::shaped(flags, executor);
        let people_shape = people::shaped(&member);
        let verdict = crate::routing::create::well_formed(core::slice::from_ref(&member));
        let core_shape = verdict != Err(people::Refusal::NotWellFormed { at: 0, why: Malformed::EndedBy });
        let expected = match expected_verdict {
            Ok(()) => Ok(()),
            Err(why) => Err(people::Refusal::NotWellFormed { at: 0, why }),
        };
        assert_eq!(task_shape, expected_shape, "tasks: {name}");
        assert_eq!(people_shape, task_shape, "people: {name}");
        assert_eq!(core_shape, task_shape, "core: {name}");
        assert_eq!(verdict, expected, "full verdict: {name}");
        checked = checked.checked_add(1).expect("twelve rows");
    }
    assert_eq!(checked, ROWS, "every ending, executor and conversation combination");
}
