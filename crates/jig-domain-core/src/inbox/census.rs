//! Rebuild live projections once, then name the indexed ended owners (store.md 7.2).
use crate::{Census, Core, Limits, ReadResults};
use jig_domain_people as people;
use skein_lib::{Env, List, Map, Queue, Time, Wall};

#[derive(Debug)]
pub(crate) struct Progress {
    unread_result: u64,
    read_conversation: u64,
}

pub(crate) fn worst_case(limits: &Limits) -> Option<u64> {
    Map::<u64, Progress>::worst_case(limits.people.people)?
        .checked_add(List::<u64>::worst_case(limits.people.people)?)?
        .checked_add(List::<ReadResults>::worst_case(limits.people.people)?)
}
impl Core {
    pub fn census(&mut self, limits: &Limits) -> Census {
        assert!(
            self.restart_step() == Some(crate::RestartStep::LoadInbox) && self.people.ready(),
            "people restored before census"
        );
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
        let mut out = Queue::with_capacity(people::max_out(&limits.people));
        for task in self.tasks.view_tasks() {
            let row = self.tasks.task(task.number).expect("live view names its owner");
            if let Some(party) = self.chat_requester(row) {
                people::step(
                    &mut self.people,
                    &env,
                    people::Event::ChatState {
                        party,
                        project: row.project,
                        task: row.number,
                        last_reply: row.last_reply,
                        live: true,
                        replies: super::reply_refs(limits, row, party),
                    },
                    &mut out,
                );
            }
            let entries = self.waiting_entries(limits, row);
            people::step(&mut self.people, &env, people::Event::Waiting { task: row.number, entries }, &mut out);
            assert!(out.is_empty(), "restored live projections decide nothing");
        }
        for (_, proposal) in self.tasks.person_proposals() {
            let entries = self.person_proposal_entries(proposal);
            people::step(
                &mut self.people,
                &env,
                people::Event::Waiting { task: proposal.goal.number, entries },
                &mut out,
            );
            assert!(out.is_empty(), "restored pending proposals decide nothing");
        }
        let mut results = List::with_capacity(limits.people.people);
        for party in &self.people.parties() {
            self.census_progress
                .insert(*party, Progress { unread_result: 0, read_conversation: 0 })
                .expect("bounded census parties");
            results
                .push(ReadResults {
                    party: *party,
                    after: self.people.results_seen(*party).expect("known restored party"),
                })
                .expect("bounded restored parties");
        }
        Census { results: results.into_boxed() }
    }
}

impl Core {
    #[must_use]
    pub fn restore_inbox_row(&mut self, limits: &Limits, record: &jig_domain_tasks::Stored) -> bool {
        use jig_domain_tasks as tasks;
        if self.restart_step() != Some(crate::RestartStep::LoadInbox) || !self.people.ready() {
            return false;
        }
        let row = match record {
            tasks::Stored::Ended(row) => row,
            tasks::Stored::Live(_)
            | tasks::Stored::PersonProposal(_)
            | tasks::Stored::Activity(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Milestone(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Ledger(_)
            | tasks::Stored::ClosedLedger(_)
            | tasks::Stored::DecidedProposal(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::Pool(_)
            | tasks::Stored::Update(_)
            | tasks::Stored::Result(_)
            | tasks::Stored::ConversationNewest(_)
            | tasks::Stored::Goal(_)
            | tasks::Stored::EndedByParty(_)
            | tasks::Stored::EndedDelegate(_)
            | tasks::Stored::Delegate(_)
            | tasks::Stored::ConversationOldest(_) => return false,
        };
        if !super::matches_ended(&row.phase) || !super::valid_header(self, limits, row) {
            return false;
        }
        let mut out = Queue::with_capacity(people::max_out(&limits.people));
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
        match row.requester {
            tasks::Party::Person(party) => {
                if self.people.identity(party).is_none() {
                    return false;
                }
                let Some(at) = row.ended_at else { return false };
                if row.result_position == 0 || row.result_position > self.counters.deployment().messages {
                    return false;
                }
                let after = self.people.results_seen(party).expect("known restored party");
                let progress = self.census_progress.get_mut(&party).expect("party census initialised");
                let mark = if row.result_position > after {
                    &mut progress.unread_result
                } else {
                    &mut progress.read_conversation
                };
                let position = if row.result_position > after { row.result_position } else { row.number };
                if position <= *mark {
                    return true;
                }
                *mark = position;
                self.people.remember_result(
                    &limits.people,
                    party,
                    people::ResultRef { task: row.number, project: row.project, at, position: row.result_position },
                );
                if self.chat_requester(row).is_some() {
                    people::step(
                        &mut self.people,
                        &env,
                        people::Event::ChatState {
                            party,
                            project: row.project,
                            task: row.number,
                            last_reply: row.last_reply,
                            live: false,
                            replies: super::reply_refs(limits, row, party),
                        },
                        &mut out,
                    );
                }
            }
            tasks::Party::Task(_) | tasks::Party::Deployment { .. } => {}
        }
        self.cache_archived_chat(limits, row);
        out.is_empty()
    }
}
