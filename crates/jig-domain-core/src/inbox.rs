//! Volatile party inbox projections from the task child's durable rows.
//! The core routes these sibling facts on save and restore (domain/engine.md, 4.4).

use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::List;

use crate::{Core, Limits};

/// Inbox words, role-routed proposals and the four scalar references: a
/// person-held proposal, escalation, answerable person task and stall update.
pub(crate) fn waiting_capacity(limits: &Limits) -> Option<u32> {
    limits.tasks.inbox_messages.checked_mul(2)?.checked_add(limits.authority.roles)?.checked_add(4)
}

fn add(
    entries: &mut List<people::Entry>,
    row: &tasks::TaskRecord,
    whom: people::Whom,
    kind: people::EntryKind,
    at: skein_lib::Wall,
) {
    entries
        .push(people::Entry { task: row.number, project: row.project, whom, kind, at })
        .expect("one bounded projection per task state");
}

fn proposal_kind(kind: tasks::ProposalKind) -> authority::ProposalKind {
    match kind {
        tasks::ProposalKind::Batch => authority::ProposalKind::Batch,
        tasks::ProposalKind::Effect => authority::ProposalKind::Effect,
        tasks::ProposalKind::Amend => authority::ProposalKind::Amend,
        tasks::ProposalKind::Widen => authority::ProposalKind::Widen,
        tasks::ProposalKind::Release => authority::ProposalKind::Escalation,
    }
}

fn question_answered(row: &tasks::TaskRecord, question: u64) -> bool {
    for word in &row.inbox {
        match word.kind {
            tasks::MessageKind::Answer { question: answered } if answered == question => return true,
            tasks::MessageKind::Answer { .. }
            | tasks::MessageKind::Question
            | tasks::MessageKind::EffectNotMade { .. }
            | tasks::MessageKind::Words
            | tasks::MessageKind::Amendment { .. }
            | tasks::MessageKind::Proposal { .. }
            | tasks::MessageKind::ProposalDecision { .. }
            | tasks::MessageKind::Escalation { .. }
            | tasks::MessageKind::Notice { .. }
            | tasks::MessageKind::Timer { .. }
            | tasks::MessageKind::News { .. }
            | tasks::MessageKind::PersonStalled { .. }
            | tasks::MessageKind::Result(_) => {}
        }
    }
    false
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct ArchivedChat {
    pub(crate) party: u64,
    pub(crate) project: u32,
    pub(crate) at: Option<skein_lib::Wall>,
    pub(crate) chip: crate::documents::Chip,
    pub(crate) line: Option<crate::documents::ConversationLine>,
}

pub(crate) fn reply_refs(limits: &Limits, row: &tasks::TaskRecord, party: u64) -> Box<[people::ReplyRef]> {
    let mut refs = List::with_capacity(limits.tasks.inbox_messages);
    for reply in &row.replies {
        refs.push(people::ReplyRef {
            entry: people::Entry {
                task: row.number,
                project: row.project,
                whom: people::Whom::Person(party),
                kind: people::EntryKind::ChatReply { message: reply.number },
                at: reply.at,
            },
            activity: reply.activity,
        })
        .expect("bounded owner hints");
    }
    refs.into_boxed()
}

pub(crate) fn worst_case(limits: &Limits) -> Option<u64> {
    let headers = limits.people.people.checked_mul(limits.people.inbox_entries)?.checked_add(1)?;
    skein_lib::Map::<u64, ArchivedChat>::worst_case(headers)?
        .checked_add(
            u64::from(headers)
                .checked_mul(crate::documents::chip_heap_worst_case(&limits.documents)?.checked_mul(2)?)?,
        )?
        .checked_add(List::<u64>::worst_case(headers)?)?
        .checked_add(List::<people::ReplyRef>::worst_case(limits.tasks.inbox_messages)?)?
        .checked_add(List::<people::Update>::worst_case(limits.people.inbox_entries)?)
}

impl Core {
    /// The authenticated requester of a chat, also usable for a loaded ended task.
    #[must_use]
    pub fn chat_requester(&self, row: &tasks::TaskRecord) -> Option<u64> {
        if row.tracked.is_some() || row.recurring.is_some() {
            return None;
        }
        match row.executor {
            tasks::Executor::Agent { charter } if charter == self.settings.chat => {}
            tasks::Executor::Agent { .. } | tasks::Executor::Person(_) | tasks::Executor::Procedure { .. } => {
                return None;
            }
        }
        match row.requester {
            tasks::Party::Person(party) => Some(party),
            tasks::Party::Task(_) | tasks::Party::Deployment { .. } => None,
        }
    }
    /// Keep whole committed archive facts only while the shared recent prefix may deliver them.
    pub(crate) fn cache_archived_chat(&mut self, limits: &Limits, row: &tasks::TaskRecord) {
        let party = match row.requester {
            tasks::Party::Person(party) => party,
            tasks::Party::Task(_) | tasks::Party::Deployment { .. } => return,
        };
        if self.people.role(party, row.project).is_none() {
            return;
        }
        let line = if self.chat_requester(row) == Some(party) && row.last_reply > 0 {
            Some(crate::documents::conversation_from_row(self, limits, party, row).expect("authorized archive owner"))
        } else {
            None
        };
        let chip = crate::documents::chip_from_row(limits, row).expect("ended chip has no whole resource paths");
        assert!(
            self.archived_chats
                .insert(row.number, ArchivedChat { party, project: row.project, at: row.ended_at, chip, line })
                .is_ok(),
            "one temporary archive slot beyond the bounded prefix"
        );
        let mut names = List::with_capacity(self.archived_chats.capacity());
        for (number, _) in &self.archived_chats {
            names.push(*number).expect("archive snapshot bound");
        }
        for number in names.into_boxed() {
            let owner = self.archived_chats.get(&number).expect("snapshot archive").party;
            let mut needed = false;
            if let Some(updates) = self.people.cached_updates(&limits.people, owner) {
                for update in updates {
                    needed |= match update {
                        people::Update::ChatReply(reply) => reply.entry.task == number,
                        people::Update::Result(result) => result.task == number,
                        people::Update::Reply(entry) => entry.task == number,
                    };
                }
            }
            if !needed {
                drop(self.archived_chats.remove(&number));
            }
        }
    }

    /// Project the currently waiting person-facing references of one task.
    #[must_use]
    #[expect(clippy::too_many_lines, reason = "one exhaustive task projection covers all person-facing waiting kinds")]
    pub fn waiting_entries(&self, limits: &Limits, row: &tasks::TaskRecord) -> Box<[people::Entry]> {
        let capacity = waiting_capacity(limits).expect("validated task inbox projection bound");
        let mut entries = List::with_capacity(capacity);
        match &row.phase {
            tasks::Phase::Ended(_) | tasks::Phase::Closing(_) => return entries.into_boxed(),
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => {}
        }
        match row.requester {
            tasks::Party::Person(person) => {
                for word in &row.inbox {
                    match word.kind {
                        tasks::MessageKind::Question => {
                            if !question_answered(row, word.number) {
                                add(
                                    &mut entries,
                                    row,
                                    people::Whom::Person(person),
                                    people::EntryKind::Question { message: word.number },
                                    word.at,
                                );
                            }
                        }
                        tasks::MessageKind::Answer { .. }
                        | tasks::MessageKind::ProposalDecision { .. }
                        | tasks::MessageKind::Proposal { .. }
                        | tasks::MessageKind::EffectNotMade { .. }
                        | tasks::MessageKind::Words
                        | tasks::MessageKind::Amendment { .. }
                        | tasks::MessageKind::Notice { .. }
                        | tasks::MessageKind::Timer { .. }
                        | tasks::MessageKind::News { .. }
                        | tasks::MessageKind::PersonStalled { .. }
                        | tasks::MessageKind::Result(_)
                        | tasks::MessageKind::Escalation { .. } => {}
                    }
                }
            }
            tasks::Party::Task(_) | tasks::Party::Deployment { .. } => {}
        }
        if let Some(person) = self.chat_requester(row) {
            for reply in &row.replies {
                add(
                    &mut entries,
                    row,
                    people::Whom::Person(person),
                    people::EntryKind::ChatReply { message: reply.number },
                    reply.at,
                );
            }
        }
        if let Some(proposal) = &row.proposal {
            match proposal.state {
                tasks::ProposalState::Pending { holder, since } => match holder {
                    tasks::ProposalHolder::Person(person) => add(
                        &mut entries,
                        row,
                        people::Whom::Person(person),
                        people::EntryKind::Proposal { number: proposal.number },
                        since,
                    ),
                    tasks::ProposalHolder::Policy { project, kind } => {
                        if let Some(project_policy) = self.authority.policy(project) {
                            for policy in &project_policy.roles {
                                if !policy.decides.allows(proposal_kind(kind)) {
                                    continue;
                                }
                                add(
                                    &mut entries,
                                    row,
                                    people::Whom::Role { project, role: policy.number },
                                    people::EntryKind::Proposal { number: proposal.number },
                                    since,
                                );
                            }
                        }
                    }
                    tasks::ProposalHolder::Task(_) => {}
                },
                tasks::ProposalState::Accepted { .. }
                | tasks::ProposalState::Rejected { .. }
                | tasks::ProposalState::Withdrawn => {}
            }
        }
        match row.escalation {
            tasks::Escalation::Waiting { holder, revision, since, .. } => match holder {
                tasks::EscalationHolder::Person(person) => {
                    add(
                        &mut entries,
                        row,
                        people::Whom::Person(person),
                        people::EntryKind::Escalation { revision },
                        since,
                    );
                }
                tasks::EscalationHolder::Role { project, role } => {
                    add(
                        &mut entries,
                        row,
                        people::Whom::Role { project, role },
                        people::EntryKind::Escalation { revision },
                        since,
                    );
                }
                tasks::EscalationHolder::Task(_) => {}
            },
            tasks::Escalation::Unheld { .. }
            | tasks::Escalation::Routing { .. }
            | tasks::Escalation::Rejected { .. } => {}
        }
        if let Some(wait) = row.person_wait
            && let Some(notice) = wait.notice
            && let tasks::Party::Person(person) = row.requester
        {
            add(
                &mut entries,
                row,
                people::Whom::Person(person),
                people::EntryKind::Reply { message: notice.message },
                notice.at,
            );
        }
        let answerable = match row.phase {
            tasks::Phase::Active(tasks::Active::Due | tasks::Active::Idle) => true,
            tasks::Phase::Waiting
            | tasks::Phase::Active(_)
            | tasks::Phase::Held { .. }
            | tasks::Phase::Closing(_)
            | tasks::Phase::Ended(_) => false,
        };
        match row.executor {
            tasks::Executor::Person(_) if !answerable => {}
            tasks::Executor::Person(tasks::PersonAddress::Person(person)) => {
                if row.taken_by.is_none() {
                    let since = row.person_wait.expect("active person wait").since;
                    add(&mut entries, row, people::Whom::Person(person), people::EntryKind::PersonTask, since);
                }
            }
            tasks::Executor::Person(tasks::PersonAddress::Role(role)) => {
                let since = row.person_wait.expect("active person wait").since;
                let whom = match row.taken_by {
                    Some(person) => people::Whom::Person(person),
                    None => people::Whom::Role { project: row.project, role },
                };
                add(&mut entries, row, whom, people::EntryKind::PersonTask, since);
            }
            tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } => {}
        }
        entries.into_boxed()
    }
}

/// Validate a bounded owner header before using its committed reply positions.
pub(crate) fn valid_header(core: &Core, limits: &Limits, row: &tasks::TaskRecord) -> bool {
    let deployment = core.counters.deployment();
    if row.number == 0
        || row.number > deployment.tasks
        || row.project == 0
        || row.last_reply > row.activity
        || row.replies.len() > usize::try_from(limits.tasks.inbox_messages).expect("hint bound fits")
    {
        return false;
    }
    let mut previous = None;
    for (index, hint) in row.replies.iter().enumerate() {
        if hint.number == 0
            || hint.number > row.last_reply
            || hint.activity == 0
            || hint.activity > row.activity
            || hint.attempt == 0
            || hint.attempt > deployment.runs
            || hint.attempt > row.attempt
            || (hint.attempt == row.attempt && hint.turn > row.turn)
            || hint.turn == 0
        {
            return false;
        }
        if let Some(key) = previous
            && key <= (hint.at, hint.number)
        {
            return false;
        }
        for old in row.replies.iter().take(index) {
            if old.number == hint.number || old.activity == hint.activity {
                return false;
            }
        }
        previous = Some((hint.at, hint.number));
    }
    row.last_reply == 0 || !row.replies.is_empty()
}

pub(crate) fn seen_chat_loaded(
    core: &mut Core,
    env: &skein_lib::Env<Limits>,
    work: &mut skein_lib::Queue<crate::Event>,
    request: skein_lib::Token,
    row: Option<Box<tasks::TaskRecord>>,
    busy: bool,
) -> crate::Requests {
    let Some((party, people::Seen::Chat { task, .. })) = core.people.seen_context(request) else {
        return crate::Requests::Out(skein_lib::Queue::with_capacity(0));
    };
    let event = if busy {
        people::Event::Decided { request, outcome: people::Outcome::Again(people::Again::Busy) }
    } else if let Some(row) = row {
        if row.number != task || !valid_header(core, &env.limits, &row) || !matches_ended(&row.phase) {
            people::Event::Decided { request, outcome: people::Outcome::Refused(people::Refusal::Unknown) }
        } else if core.chat_requester(&row) != Some(party) || core.people.role(party, row.project).is_none() {
            people::Event::Decided {
                request,
                outcome: people::Outcome::Refused(people::Refusal::Role { held: core.people.role(party, row.project) }),
            }
        } else {
            work.push(crate::Event::People(people::Event::ChatSeen {
                request,
                project: row.project,
                last_reply: row.last_reply,
            }));
            work.push(crate::Event::ForgetChat { row });
            let mut out = skein_lib::Queue::with_capacity(1);
            out.push(crate::Request::Decided);
            return crate::Requests::Out(out);
        }
    } else {
        people::Event::Decided { request, outcome: people::Outcome::Refused(people::Refusal::Unknown) }
    };
    work.push(crate::Event::People(event));
    let mut out = skein_lib::Queue::with_capacity(1);
    out.push(crate::Request::Decided);
    crate::Requests::Out(out)
}
fn matches_ended(phase: &tasks::Phase) -> bool {
    match phase {
        tasks::Phase::Ended(_) => true,
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => false,
    }
}

/// Owner-defined startup census range, loaded after people's positions restore.
#[must_use]
pub fn restore_key(key: &crate::Key) -> bool {
    match key {
        crate::Key::Tasks(tasks::Key::Live(_) | tasks::Key::Ended(_) | tasks::Key::PersonProposal(_)) => true,
        crate::Key::Tasks(
            tasks::Key::Activity { .. }
            | tasks::Key::History { .. }
            | tasks::Key::Milestone { .. }
            | tasks::Key::Stub(_)
            | tasks::Key::Ledger(_)
            | tasks::Key::Writer(_)
            | tasks::Key::ConversationNewest { .. }
            | tasks::Key::ConversationOldest { .. }
            | tasks::Key::Update { .. }
            | tasks::Key::Pool(_)
            | tasks::Key::ClosedLedger(_)
            | tasks::Key::DecidedProposal(_),
        )
        | crate::Key::Core(_)
        | crate::Key::People(_)
        | crate::Key::Notes(_) => false,
    }
}
impl Core {
    /// Rebuild bounded inbox references and exact chat totals from their owner rows.
    /// This census is a separate startup range so all read positions already exist.
    #[must_use]
    pub fn restore_inbox_row(&mut self, limits: &Limits, row: &tasks::Stored) -> bool {
        if self.restart_step() != Some(crate::RestartStep::LoadInbox) || !self.people.ready() {
            return false;
        }
        let mut out = skein_lib::Queue::with_capacity(people::max_out(&limits.people));
        let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: limits.people };
        let (task, entries) = match row {
            tasks::Stored::Live(row) | tasks::Stored::Ended(row) => {
                if !valid_header(self, limits, row) {
                    return false;
                }
                let live = !matches_ended(&row.phase);
                if let tasks::Party::Person(party) = row.requester {
                    if self.people.identity(party).is_none() {
                        return false;
                    }
                    if !live {
                        let Some(at) = row.ended_at else {
                            return false;
                        };
                        if row.result_position == 0 || row.result_position > self.counters.deployment().messages {
                            return false;
                        }
                        self.people.remember_result(
                            &limits.people,
                            party,
                            people::ResultRef {
                                task: row.number,
                                project: row.project,
                                at,
                                position: row.result_position,
                            },
                        );
                    }
                    if self.chat_requester(row).is_some() {
                        people::step(
                            &mut self.people,
                            &env,
                            people::Event::ChatState {
                                party,
                                project: row.project,
                                task: row.number,
                                last_reply: row.last_reply,
                                live,
                                replies: reply_refs(limits, row, party),
                            },
                            &mut out,
                        );
                    }
                }
                if !live {
                    self.cache_archived_chat(limits, row);
                }
                (row.number, self.waiting_entries(limits, row))
            }
            tasks::Stored::PersonProposal(row) => (row.goal.number, self.person_proposal_entries(row)),
            tasks::Stored::Activity(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Milestone(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Ledger(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::ConversationNewest(_)
            | tasks::Stored::ConversationOldest(_)
            | tasks::Stored::Update(_)
            | tasks::Stored::Pool(_)
            | tasks::Stored::ClosedLedger(_)
            | tasks::Stored::DecidedProposal(_) => return true,
        };
        people::step(&mut self.people, &env, people::Event::Waiting { task, entries }, &mut out);
        out.is_empty()
    }
}

/// Forget the ended owner only after every update has been read. The marker erase
/// and owner erase share the root's journal decision; linked archives without an
/// owner become unreadable and can be collected by the store's retention pass.
pub(crate) fn forget_chat(
    core: &mut Core,
    env: &skein_lib::Env<Limits>,
    work: &mut skein_lib::Queue<crate::Event>,
    row: &tasks::TaskRecord,
) -> crate::Requests {
    let mut out = skein_lib::Queue::with_capacity(5);
    if matches_ended(&row.phase)
        && core.people.ready()
        && valid_header(core, &env.limits, row)
        && row.ended_at.is_some()
        && row.result_position > 0
        && row.result_position <= core.counters.deployment().messages
        && let Some(party) = core.chat_requester(row)
        && core.people.chat_seen(party, row.number).unwrap_or(0) >= row.last_reply
        && core.people.results_seen(party).unwrap_or(0) >= row.result_position
    {
        work.push(crate::Event::People(people::Event::ForgetChat {
            project: row.project,
            task: row.number,
            party: Some(party),
            last_reply: row.last_reply,
        }));
        out.push(crate::Request::Write(crate::Write::Erase(crate::Key::Tasks(tasks::Key::Ended(row.number)))));
        let update = tasks::Update {
            party,
            at: row.ended_at.expect("validated end time"),
            task: row.number,
            object: tasks::UpdateObject::Result,
        };
        out.push(crate::Request::Write(crate::Write::Erase(crate::Key::Tasks(update.key()))));
        for key in [
            tasks::Key::ConversationNewest { party, task: tasks::Newest::of(row.number) },
            tasks::Key::ConversationOldest { party, task: row.number },
        ] {
            out.push(crate::Request::Write(crate::Write::Erase(crate::Key::Tasks(key))));
        }
    }
    out.push(crate::Request::Decided);
    crate::Requests::Out(out)
}
