//! The restart census loads index intervals and joins their ended owners (store.md 7.2).
use super::{Neighbours, Restore};
use jig_conformance::Input;
use jig_core_world::Store;
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use jig_test_domain as root;

impl Neighbours {
    pub(super) fn restart(
        &mut self,
        store: &mut Store,
        step: core::RestartStep,
        census: Option<&core::Census>,
    ) -> Vec<Input<root::Event, root::Record>> {
        self.restart_steps.push(step);
        match step {
            core::RestartStep::LoadCore | core::RestartStep::LoadInbox | core::RestartStep::RestoreConnector { .. } => {
                let mut intervals = std::collections::VecDeque::new();
                if step == core::RestartStep::LoadCore {
                    for range in core::RestoreRange::LIVE {
                        let mut keys = store.rows.keys().filter(|key| match key {
                            root::Key::Core(key) => range.admits(key),
                            root::Key::Connector { .. } => false,
                        });
                        if let Some(first) = keys.next() {
                            let last = keys.next_back().unwrap_or(first);
                            intervals.push_back((first.clone(), last.clone()));
                        }
                    }
                } else {
                    let mut interval = None;
                    for key in store.rows.keys() {
                        let admitted = match (step, key) {
                            (core::RestartStep::LoadInbox, root::Key::Core(key)) => {
                                census.expect("inbox census selected").admits(key)
                            }
                            (
                                core::RestartStep::RestoreConnector { connector },
                                root::Key::Connector { number, .. },
                            ) => connector == *number,
                            _ => false,
                        };
                        if admitted {
                            match &mut interval {
                                Some((_, last)) => *last = key.clone(),
                                None => interval = Some((key.clone(), key.clone())),
                            }
                        } else if let Some(bounds) = interval.take() {
                            intervals.push_back(bounds);
                        }
                    }
                    if let Some(bounds) = interval {
                        intervals.push_back(bounds);
                    }
                }
                if let Some((first, last)) = intervals.pop_front() {
                    store.load(1, &first, &last, None, 2);
                    self.restoring = Some(Restore { step, first, last, rows: Vec::new(), remaining: intervals });
                    Vec::new()
                } else {
                    Self::restored(step, &[])
                }
            }
            core::RestartStep::ReadAfresh { connector: number } => {
                self.restart_reads.insert(number, step);
                vec![Input::Event(root::Event::Connector {
                    number,
                    event: connector::Event::Restart(connector::RestartStep::FreshRead {
                        resource: jig_test_connector_world::path(1, 1),
                    }),
                })]
            }
            core::RestartStep::SettleOutbox { connector: number } => vec![Input::Event(root::Event::Connector {
                number,
                event: connector::Event::Restart(connector::RestartStep::SettleOutbox),
            })],
            core::RestartStep::AdoptRuns | core::RestartStep::Open => panic!("root performs synchronous restart step"),
        }
    }
}

pub(super) fn owner(store: &Store, row: &root::Record) -> Option<root::Record> {
    let task = match row {
        root::Record::Core(core::Record::Tasks(tasks::Stored::Result(row))) => row.task,
        root::Record::Core(core::Record::Tasks(tasks::Stored::ConversationOldest(row))) => row.task,
        root::Record::Core(_) | root::Record::Connector { .. } => return None,
    };
    store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))).cloned()
}
