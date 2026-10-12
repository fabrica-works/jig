//! Closed request fields translate from their supplied owner vocabulary
//! (client/domain/contract.md, 3), preserving absent words and numeric widths.
use jig_domain_client::contract as client;
use jig_domain_people as people;

#[must_use]
pub fn wake(value: client::OwnerWakePolicy) -> people::WakePolicy {
    people::WakePolicy {
        words: wake_rule(value.words),
        notices: wake_rule(value.notices),
        news: wake_rule(value.news),
        results: match value.results {
            client::ResultsWake::Never => people::ResultsWake::Never,
            client::ResultsWake::Each => people::ResultsWake::Each,
            client::ResultsWake::LastOrFailure => people::ResultsWake::LastOrFailure,
        },
        questions: value.questions,
        answers: value.answers,
        timers: value.timers,
    }
}

fn wake_rule(value: client::WakeRule) -> people::WakeRule {
    match value {
        client::WakeRule::Never => people::WakeRule::Never,
        client::WakeRule::Immediate => people::WakeRule::Immediate,
        client::WakeRule::Batch { count, age } => people::WakeRule::Batch { count, age },
    }
}

#[must_use]
pub fn person_answer(value: &client::PersonAnswer) -> people::PersonResult {
    match value {
        client::PersonAnswer::Choice { option, reason } => {
            people::PersonResult::Verdict { code: *option, words: Box::from(reason.bytes()) }
        }
        client::PersonAnswer::Approved { state, words } => people::PersonResult::Approved {
            state: state.clone(),
            words: words.as_ref().map(|value| Box::from(value.bytes())),
        },
        client::PersonAnswer::Words(value) => people::PersonResult::Report { words: Box::from(value.bytes()) },
        client::PersonAnswer::Cannot { reason } => people::PersonResult::Failure { reason: Box::from(reason.bytes()) },
    }
}

#[must_use]
pub fn goal_executor(value: client::GoalExecutor) -> people::GoalExecutor {
    match value {
        client::GoalExecutor::Charter(charter) => people::GoalExecutor::Charter(charter),
        client::GoalExecutor::Procedure { connector, code } => people::GoalExecutor::Procedure { connector, code },
    }
}

#[must_use]
pub fn seen(value: client::Seen) -> people::Seen {
    match value {
        client::Seen::Results { through } => people::Seen::Results { through },
        client::Seen::Replies { chat, through } => people::Seen::Chat { task: chat, through },
    }
}

#[must_use]
pub fn disposal(value: &client::Disposal) -> people::Disposal {
    match value {
        client::Disposal::Cancel { task } => people::Disposal { task: *task, keep: people::Keep::Cancel },
        client::Disposal::TakeOver { task } => people::Disposal { task: *task, keep: people::Keep::MakeMine },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(value: &[u8]) -> client::Text {
        client::Text::new(Box::from(value), 64).expect("the complete supplied fixture is admitted")
    }

    #[test]
    fn person_answer_translation_keeps_opaque_approval_state_absent_or_empty_words_and_failures() {
        for words in [None, Some(text(b"")), Some(text(b"literal/words"))] {
            let value = client::PersonAnswer::Approved { state: Box::from([0, 255, b'/', 1]), words: words.clone() };
            assert_eq!(
                person_answer(&value),
                people::PersonResult::Approved {
                    state: Box::from([0, 255, b'/', 1]),
                    words: words.map(|value| Box::from(value.bytes()))
                }
            );
        }
        let cases = [
            (
                client::PersonAnswer::Choice { option: u32::MAX, reason: text(b"") },
                people::PersonResult::Verdict { code: u32::MAX, words: Box::new([]) },
            ),
            (
                client::PersonAnswer::Words(text(b"report/bytes")),
                people::PersonResult::Report { words: Box::from(b"report/bytes".as_slice()) },
            ),
            (
                client::PersonAnswer::Cannot { reason: text(&[255, b'/', 0]) },
                people::PersonResult::Failure { reason: Box::from([255, b'/', 0]) },
            ),
        ];
        for (given, expected) in cases {
            assert_eq!(person_answer(&given), expected);
        }
    }

    #[test]
    fn goal_executor_disposal_and_seen_translation_preserve_closed_variants_and_full_width_identities() {
        assert_eq!(goal_executor(client::GoalExecutor::Charter(u32::MAX)), people::GoalExecutor::Charter(u32::MAX));
        assert_eq!(
            goal_executor(client::GoalExecutor::Procedure { connector: u16::MAX, code: u32::MAX }),
            people::GoalExecutor::Procedure { connector: u16::MAX, code: u32::MAX }
        );
        assert_eq!(seen(client::Seen::Results { through: u64::MAX }), people::Seen::Results { through: u64::MAX });
        assert_eq!(
            seen(client::Seen::Replies { chat: u64::MAX, through: u64::MAX }),
            people::Seen::Chat { task: u64::MAX, through: u64::MAX }
        );
        assert_eq!(
            disposal(&client::Disposal::Cancel { task: u64::MAX }),
            people::Disposal { task: u64::MAX, keep: people::Keep::Cancel }
        );
        assert_eq!(
            disposal(&client::Disposal::TakeOver { task: u64::MAX }),
            people::Disposal { task: u64::MAX, keep: people::Keep::MakeMine }
        );
    }
}
