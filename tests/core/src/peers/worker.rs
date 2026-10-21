use super::{Peers, authority, core, delegate, end_value, fleet, root, tasks, workers};
use skein_lib::{Duration, ReplyTo, Token};
impl Peers {
    pub fn worker_event(&mut self, up: workers::Up) -> root::Event {
        self.upstream.push(up.clone());
        match up {
            workers::Up::Hello { channel, slots, stop_bound, hosting } => {
                root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
                    worker: u32::try_from(
                        self.workers
                            .iter()
                            .filter(|worker| worker.channel != 0)
                            .position(|worker| worker.channel == channel)
                            .expect("configured peer worker"),
                    )
                    .expect("bounded peer membership"),
                    channel: Token::new(channel),
                    hello: fleet::Hello {
                        bounds: crate::fixtures::bounds(),
                        slots,
                        stop_bound,
                        workstreams: Box::new([]),
                        hosting: hosting
                            .into_iter()
                            .map(|(task, attempt, answered)| fleet::Hosted {
                                run: Token::new(task),
                                attempt: Token::new(attempt),
                                phase: if answered { fleet::Phase::Answered } else { fleet::Phase::Active },
                            })
                            .collect(),
                    },
                }))
            }
            workers::Up::Lost { channel } => {
                root::Event::Core(core::Event::Fleet(fleet::Event::Lost { channel: Token::new(channel) }))
            }
            workers::Up::Writing { channel, task, attempt } => {
                root::Event::Core(core::Event::Fleet(fleet::Event::Observe {
                    channel: Token::new(channel),
                    run: Token::new(task),
                    attempt: Token::new(attempt),
                    report: fleet::Report::Activity(fleet::Activity::Writing),
                }))
            }
            workers::Up::Text { task, attempt, turn, offset, text, .. } => {
                root::Event::Text { task, attempt, turn, offset, text }
            }
            workers::Up::Turn { channel, task, attempt, turn, spent, read, body, opens_window, refused } => {
                root::Event::Turn {
                    channel: Token::new(channel),
                    task,
                    attempt,
                    turn,
                    cumulative: spent,
                    read,
                    transcript: body,
                    opens_window,
                    refused,
                }
            }
            workers::Up::Answer { channel, task, attempt, spent, end, read, refused } => root::Event::Answer {
                channel: Token::new(channel),
                task,
                attempt,
                cumulative: spent,
                end: end_value(end),
                read,
                refused,
            },
            workers::Up::Call { channel, task, attempt, number, call } => self.worker_call(
                core::CallKey { task, attempt, completion: number, position: 0 },
                channel,
                number,
                call,
            ),
        }
    }
    fn worker_call(&mut self, key: core::CallKey, channel: u64, number: u32, call: workers::Call) -> root::Event {
        self.call_hosts.insert(key, (channel, number));
        let to = ReplyTo::new(Token::new(channel * 100_000 + key.task * 100 + u64::from(number)));
        match call {
            workers::Call::Question { target, words } => root::Event::Core(core::Event::NamedAction {
                to,
                key,
                action: core::NamedAction::Message { target, kind: tasks::MessageKind::Question, words },
            }),
            workers::Call::Delegate { words, budget } => root::Event::Core(core::Event::DelegateValidated {
                to,
                key,
                batch: Box::new([delegate(words, tasks::Executor::Agent { charter: 1 }, budget)]),
                stubs: Box::new([]),
            }),
            workers::Call::Opaque { name, tool, input, writes } => {
                self.opaque.insert((key.task, key.attempt, name.clone()), (channel, number));
                root::Event::Core(core::Event::Fleet(fleet::Event::Relay {
                    channel: Token::new(channel),
                    run: Token::new(key.task),
                    attempt: Token::new(key.attempt),
                    call: fleet::Call { name, tool, input, writes, deadline: Duration::from_secs(1) },
                }))
            }
            workers::Call::ProposeGoal { words, budget, priority } => root::Event::Core(core::Event::NamedAction {
                to,
                key,
                action: core::NamedAction::ProposeGoal {
                    goal: Box::new(delegate(words, tasks::Executor::Agent { charter: 1 }, budget)),
                    priority,
                    reason: b"this goal needs a holder".as_slice().into(),
                },
            }),
            workers::Call::Choose { role, first, second } => {
                let mut words = first.into_vec();
                words.extend_from_slice(b" or ");
                words.extend_from_slice(&second);
                let mut member =
                    delegate(words.into_boxed_slice(), tasks::Executor::Person(tasks::PersonAddress::Role(role)), 0);
                member.authority = authority(0, false);
                member.contract = tasks::Contract::Verdict {
                    choices: Box::new([
                        tasks::Verdict { fields: Box::new([]), code: 1, words: 64, followups: 0 },
                        tasks::Verdict { fields: Box::new([]), code: 2, words: 64, followups: 0 },
                    ]),
                };
                root::Event::Core(core::Event::DelegateValidated {
                    to,
                    key,
                    batch: Box::new([member]),
                    stubs: Box::new([]),
                })
            }
        }
    }
}
