//! Requests are decided once in the party's key space and released after durability.
//! Revision and decided-first checks precede scripted outcomes; contract.md, 3–4.
use crate::{Engine, Envelope, Invalid, Keyed, RequestIdentity, Retained, Up, payload};
use jig_client::contract::{
    ActKind, Actor, Admission, Answer, Ask, ChipPhase, CurrentHold, Decision, Ending, Moved, ObjectKey, Offered,
    Outcome, Phase, ReadValue, Recipient, Refusal, Revision, Runtime,
};
use skein_lib::{List, Time, Token, Wall};

impl<A: crate::Application> Engine<A> {
    /// Runs one attempt; the world's root supplies the table's attempt token.
    /// Invalid typed payloads are rejected before anything is decided or retained.
    pub fn send(
        &mut self,
        now: Time,
        wall: Wall,
        sign_in: u64,
        key: [u8; 16],
        attempt: Token,
        ask: Ask,
    ) -> core::result::Result<(), Invalid> {
        if !payload::fits(&ask, &self.bounds) {
            return Err(Invalid::Payload);
        }
        self.attempts.insert(attempt, sign_in);
        if !self.online(now) || self.request_fault == Some(Envelope::Again) {
            self.request_fault = None;
            self.queue(now, 0, Up::Answer { attempt, envelope: Envelope::Again, answer: None });
            return Ok(());
        }
        let Some(party) = self.truth.sign_ins.get(&sign_in).copied() else {
            self.queue(now, 0, Up::Answer { attempt, envelope: Envelope::SignedOut, answer: None });
            return Ok(());
        };
        self.keys.retain(|_, keyed| keyed.retained_until.is_none_or(|until| now <= until));
        if let Some(old) = self.keys.get(&(party, key)) {
            let answer = if let (RequestIdentity::Jig(previous), Retained::Jig(answer)) = (&old.ask, &old.answer) {
                if **previous == ask { answer.clone() } else { Answer::Refused(Refusal::KeyConflict) }
            } else {
                Answer::Refused(Refusal::KeyConflict)
            };
            let commit = old.commit;
            self.answer(now, commit, attempt, answer);
            return Ok(());
        }
        let (answer, changes) = match self.decide(party, &ask, wall) {
            Ok(decision) => decision,
            Err(error) => {
                self.attempts.remove(&attempt);
                return Err(error);
            }
        };
        let commit = self.commit(now, wall, Some(party), Some(key), Some(ask.clone()), Some(answer.clone()), changes);
        self.keys.insert(
            (party, key),
            Keyed {
                ask: RequestIdentity::Jig(Box::new(ask)),
                answer: Retained::Jig(answer.clone()),
                commit,
                retained_until: None,
            },
        );
        self.answer(now, commit, attempt, answer);
        Ok(())
    }
    fn answer(&mut self, now: Time, commit: u64, attempt: Token, answer: Answer) {
        let envelope = self.request_fault.take().unwrap_or(Envelope::Final);
        self.queue(
            now,
            commit,
            Up::Answer { attempt, envelope, answer: if envelope == Envelope::Final { Some(answer) } else { None } },
        );
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive decision checks all revision and offered-act variants before changing truth"
    )]
    pub(crate) fn decide(
        &mut self,
        party: u64,
        ask: &Ask,
        wall: Wall,
    ) -> core::result::Result<(Answer, Vec<crate::Change>), Invalid> {
        let (project, object, kind, revision) = target(ask);
        if project.is_some_and(|project| !self.truth.visible(party, project)) {
            return Ok((Answer::Refused(Refusal::Unknown), Vec::new()));
        }
        if let Ask::EditNote { note, revision, .. } | Ask::DeleteNote { note, revision, .. } = ask {
            let Some(current) = self.truth.notes.get(note) else {
                return Ok((Answer::Refused(Refusal::Unknown), Vec::new()));
            };
            if *revision != current.revision {
                return Ok((Answer::Refused(Refusal::Moved { now: current.revision, what: Moved::Note }), Vec::new()));
            }
        }
        if let Some(key) = object {
            if project != self.truth.project(key) {
                return Ok((Answer::Refused(Refusal::Unknown), Vec::new()));
            }
            let Some(value) = self.truth.objects.get(&key) else {
                return Ok((Answer::Refused(Refusal::Unknown), Vec::new()));
            };
            if let Some(Ending::Decided { party: Some(by), how: Some(how), at, .. }) = ending(value) {
                return Ok((
                    Answer::Done(Outcome::DecidedFirst { by: by.clone(), how: how.clone(), at: *at }),
                    Vec::new(),
                ));
            }
            if let Some(how) = ending(value) {
                return Ok((Answer::Refused(Refusal::Ended { how: how.clone() }), Vec::new()));
            }
            if let Some((seen, what)) = revision {
                let current = match (value, &what) {
                    (ReadValue::Task(value), Moved::Spec) => Some(value.spec_revision),
                    (ReadValue::Task(value), Moved::Hold { .. }) => value.hold.as_ref().map(|hold| hold.revision),
                    (ReadValue::Escalation(value), Moved::Hold { .. }) => Some(value.hold.revision),
                    (ReadValue::PersonTask(value), Moved::PersonTask) => Some(value.revision),
                    (
                        ReadValue::Frame(_)
                        | ReadValue::Task(_)
                        | ReadValue::Proposal(_)
                        | ReadValue::Escalation(_)
                        | ReadValue::Question(_)
                        | ReadValue::PersonTask(_)
                        | ReadValue::Result(_)
                        | ReadValue::Ended(_)
                        | ReadValue::History(_)
                        | ReadValue::Activity(_)
                        | ReadValue::Inbox(_)
                        | ReadValue::Conversations(_)
                        | ReadValue::Goals(_)
                        | ReadValue::Spend(_)
                        | ReadValue::Project(_)
                        | ReadValue::Notes(_)
                        | ReadValue::Deployment(_),
                        Moved::Hold { .. } | Moved::Spec | Moved::PersonTask | Moved::Note | Moved::Waiting,
                    ) => None,
                };
                if let Some(now) = current {
                    if seen != now {
                        let what = match what {
                            Moved::Hold { .. } => {
                                let hold = match value {
                                    ReadValue::Task(task) => task.hold.as_ref(),
                                    ReadValue::Escalation(escalation) => Some(&escalation.hold),
                                    ReadValue::Frame(_)
                                    | ReadValue::Proposal(_)
                                    | ReadValue::Question(_)
                                    | ReadValue::PersonTask(_)
                                    | ReadValue::Result(_)
                                    | ReadValue::Ended(_)
                                    | ReadValue::Inbox(_)
                                    | ReadValue::Conversations(_)
                                    | ReadValue::Goals(_)
                                    | ReadValue::Activity(_)
                                    | ReadValue::History(_)
                                    | ReadValue::Spend(_)
                                    | ReadValue::Project(_)
                                    | ReadValue::Notes(_)
                                    | ReadValue::Deployment(_) => None,
                                };
                                Moved::Hold {
                                    reason: hold.map(|hold| CurrentHold::Document(Box::new(hold.reason.clone()))),
                                }
                            }
                            Moved::Spec | Moved::PersonTask | Moved::Note | Moved::Waiting => what,
                        };
                        return Ok((Answer::Refused(Refusal::Moved { now, what }), Vec::new()));
                    }
                } else {
                    return Ok((Answer::Refused(Refusal::NotOffered), Vec::new()));
                }
            }
            if let Ask::Amend { release: Some(seen), .. } = ask
                && let ReadValue::Task(task) = value
                && let Some(hold) = &task.hold
                && *seen != hold.revision
            {
                return Ok((
                    Answer::Refused(Refusal::Moved {
                        now: hold.revision,
                        what: Moved::Hold { reason: Some(CurrentHold::Document(Box::new(hold.reason.clone()))) },
                    }),
                    Vec::new(),
                ));
            }
            let waiting = match value {
                ReadValue::Proposal(value) => Some(&value.waiting),
                ReadValue::Escalation(value) => Some(&value.waiting),
                ReadValue::Frame(_)
                | ReadValue::Task(_)
                | ReadValue::Question(_)
                | ReadValue::PersonTask(_)
                | ReadValue::Result(_)
                | ReadValue::Ended(_)
                | ReadValue::History(_)
                | ReadValue::Activity(_)
                | ReadValue::Inbox(_)
                | ReadValue::Conversations(_)
                | ReadValue::Goals(_)
                | ReadValue::Spend(_)
                | ReadValue::Project(_)
                | ReadValue::Notes(_)
                | ReadValue::Deployment(_) => None,
            };
            if waiting.is_some_and(|waiting| !recipient(&self.truth, party, waiting)) {
                return Ok((
                    Answer::Refused(Refusal::Moved { now: Revision { value: 0 }, what: Moved::Waiting }),
                    Vec::new(),
                ));
            }
        }
        let note_offers = match ask {
            Ask::EditNote { note, .. } | Ask::DeleteNote { note, .. } => {
                self.truth.notes.get(note).map(|note| &note.acts)
            }
            Ask::StartChat { .. }
            | Ask::Write { .. }
            | Ask::AnswerQuestion { .. }
            | Ask::AnswerPerson { .. }
            | Ask::Take { .. }
            | Ask::HandBack { .. }
            | Ask::DecideProposal { .. }
            | Ask::DecideEscalation { .. }
            | Ask::Release { .. }
            | Ask::Amend { .. }
            | Ask::Stop { .. }
            | Ask::Cancel { .. }
            | Ask::Close { .. }
            | Ask::TakeOver { .. }
            | Ask::SetGoal { .. }
            | Ask::Prioritise { .. }
            | Ask::MarkSeen { .. }
            | Ask::ChangePolicy { .. }
            | Ask::SetRoles { .. }
            | Ask::MakeService { .. }
            | Ask::SetPool { .. } => None,
        };
        let offer = note_offers.or_else(|| {
            object
                .and_then(|key| self.truth.objects.get(&key))
                .and_then(offers)
                .or_else(|| project.and_then(|p| self.truth.projects.get(&p).map(|p| &p.acts)))
        });
        let admission =
            offer.and_then(|acts| acts.iter().find(|offered| offered.act == kind).map(|offered| &offered.admission));
        let bounded = matches!(admission, Some(Admission::Bounded { .. }));
        let prediction = admission.map(|offer| match offer {
            Admission::Bounded { .. } => match ask {
                Ask::SetGoal {
                    executor: jig_client::contract::Executor::Agent { charter }, budget, deadline, ..
                } => jig_client::contract::admission::goal(offer, *charter, *budget, *deadline),
                Ask::Amend { task, change, .. } => match self.truth.objects.get(&ObjectKey::Task(*task)) {
                    Some(ReadValue::Task(before)) => {
                        jig_client::contract::admission::amendment(offer, change, &before.authority)
                    }
                    _ => match self.truth.objects.get(&ObjectKey::Escalation(*task)) {
                        Some(ReadValue::Escalation(before)) => {
                            jig_client::contract::admission::amendment(offer, change, &before.steering.authority)
                        }
                        _ => Admission::Refused(Refusal::Unknown),
                    },
                },
                Ask::StartChat { .. }
                | Ask::Write { .. }
                | Ask::AnswerQuestion { .. }
                | Ask::AnswerPerson { .. }
                | Ask::Take { .. }
                | Ask::HandBack { .. }
                | Ask::DecideProposal { .. }
                | Ask::DecideEscalation { .. }
                | Ask::Release { .. }
                | Ask::Stop { .. }
                | Ask::Cancel { .. }
                | Ask::Close { .. }
                | Ask::TakeOver { .. }
                | Ask::SetGoal { .. }
                | Ask::Prioritise { .. }
                | Ask::MarkSeen { .. }
                | Ask::EditNote { .. }
                | Ask::DeleteNote { .. }
                | Ask::ChangePolicy { .. }
                | Ask::SetRoles { .. }
                | Ask::MakeService { .. }
                | Ask::SetPool { .. } => Admission::Refused(Refusal::NotOffered),
            },
            Admission::Allowed | Admission::Proposed { .. } | Admission::Refused(_) => offer.clone(),
        });
        let admission = prediction.as_ref();
        if let Some(Admission::Refused(refusal)) = admission {
            return Ok((Answer::Refused(refusal.clone()), Vec::new()));
        }
        if let Ask::StartChat { project, .. } = ask
            && !self.truth.frames.get(&party).is_some_and(|frame| {
                frame.projects.iter().any(|role| role.project == *project && !role.chats.is_empty())
            })
        {
            return Ok((Answer::Refused(Refusal::NotOffered), Vec::new()));
        }
        if let Some(rule) = self
            .rules
            .iter()
            .find(|rule| rule.ask == *ask && rule.party == Some(party))
            .or_else(|| self.rules.iter().find(|rule| rule.ask == *ask && rule.party.is_none()))
            .cloned()
        {
            if bounded && matches!(rule.answer, Answer::Refused(_)) {
                return Err(Invalid::Script);
            }
            if matches!(rule.answer, Answer::Done(_)) {
                if matches!(admission, Some(Admission::Proposed { .. }))
                    && !matches!(rule.answer, Answer::Done(Outcome::Proposed { .. }))
                {
                    return Err(Invalid::Script);
                }
                if bounded
                    && matches!(admission, Some(Admission::Allowed))
                    && matches!(rule.answer, Answer::Done(Outcome::Proposed { .. }))
                {
                    return Err(Invalid::Script);
                }
                if admission.is_none() && !matches!(ask, Ask::StartChat { .. } | Ask::MarkSeen { .. }) {
                    return Ok((Answer::Refused(Refusal::NotOffered), Vec::new()));
                }
            }
            let mut next = self.truth.clone();
            for change in &rule.changes {
                next.apply(change)?;
            }
            self.truth = next;
            return Ok((rule.answer, rule.changes));
        }
        if admission.is_none() {
            return Ok((Answer::Refused(Refusal::NotOffered), Vec::new()));
        }
        if let Some(Admission::Proposed { .. }) = admission {
            return Ok((Answer::Refused(Refusal::NotOffered), Vec::new()));
        }
        let how = match ask {
            Ask::DecideProposal { decision, .. } | Ask::DecideEscalation { decision, .. } => Some(decision.clone()),
            Ask::Release { again, .. } => Some(Decision::Released { again: *again }),
            Ask::StartChat { .. }
            | Ask::Write { .. }
            | Ask::AnswerQuestion { .. }
            | Ask::AnswerPerson { .. }
            | Ask::Take { .. }
            | Ask::HandBack { .. }
            | Ask::Amend { .. }
            | Ask::Stop { .. }
            | Ask::Cancel { .. }
            | Ask::Close { .. }
            | Ask::TakeOver { .. }
            | Ask::SetGoal { .. }
            | Ask::Prioritise { .. }
            | Ask::MarkSeen { .. }
            | Ask::EditNote { .. }
            | Ask::DeleteNote { .. }
            | Ask::ChangePolicy { .. }
            | Ask::SetRoles { .. }
            | Ask::MakeService { .. }
            | Ask::SetPool { .. } => None,
        };
        if let (Some(key), Some(how)) = (object, how) {
            if matches!(how, Decision::PassedUp | Decision::LeftHeld { .. }) {
                return Ok((Answer::Refused(Refusal::NotOffered), Vec::new()));
            }
            let by = self.truth.frames.get(&party).expect("signed-in party has a frame").party.clone();
            let mut value = self.truth.objects.get(&key).expect("target checked").clone();
            let terminal = Ending::Decided { by: by.number, party: Some(by), how: Some(how.clone()), at: wall };
            match &mut value {
                ReadValue::Proposal(v) => v.ending = Some(terminal),
                ReadValue::Escalation(v) => v.ending = Some(terminal),
                ReadValue::Task(v) => {
                    v.hold = None;
                    v.escalation = None;
                    v.chip.phase = Phase::Due;
                    *v.chip.details = ChipPhase::Starting;
                    v.runtime = Runtime::Starting;
                }
                ReadValue::Frame(_)
                | ReadValue::Question(_)
                | ReadValue::PersonTask(_)
                | ReadValue::Result(_)
                | ReadValue::Ended(_)
                | ReadValue::History(_)
                | ReadValue::Activity(_)
                | ReadValue::Inbox(_)
                | ReadValue::Conversations(_)
                | ReadValue::Goals(_)
                | ReadValue::Spend(_)
                | ReadValue::Project(_)
                | ReadValue::Notes(_)
                | ReadValue::Deployment(_) => return Ok((Answer::Refused(Refusal::NotOffered), Vec::new())),
            }
            let change =
                crate::Change::Object { project: project.expect("object requests name a project"), key, value };
            change.validate(&self.bounds)?;
            self.truth.apply(&change)?;
            return Ok((Answer::Done(Outcome::Decided { by: Actor::Party(party), how, at: wall }), vec![change]));
        }
        Ok((Answer::Refused(Refusal::NotOffered), Vec::new()))
    }
}
pub(crate) fn recipient(truth: &crate::Truth, party: u64, to: &Recipient) -> bool {
    match to {
        Recipient::Party(number) => *number == party,
        Recipient::Role { project, role } => truth.frames.get(&party).is_some_and(|frame| {
            frame.projects.iter().any(|p| p.project == *project && p.roles.iter().any(|r| u32::from(*r) == *role))
        }),
        Recipient::Roles { project, roles } => truth.frames.get(&party).is_some_and(|frame| {
            frame.projects.iter().any(|p| {
                p.project == *project && roles.iter().any(|role| p.roles.iter().any(|r| u32::from(*r) == *role))
            })
        }),
        Recipient::Task(task) => truth.objects.get(&ObjectKey::Task(*task)).is_some_and(|value| {
            if let ReadValue::Task(v) = value { v.requester == Actor::Party(party) } else { false }
        }),
    }
}
fn offers(value: &ReadValue) -> Option<&List<Offered>> {
    match value {
        ReadValue::Task(v) => Some(&v.acts),
        ReadValue::Proposal(v) => Some(&v.acts),
        ReadValue::Escalation(v) => Some(&v.acts),
        ReadValue::Question(v) => Some(&v.acts),
        ReadValue::PersonTask(v) => Some(&v.acts),
        ReadValue::Result(v) => Some(&v.acts),
        ReadValue::Frame(_)
        | ReadValue::Ended(_)
        | ReadValue::History(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Conversations(_)
        | ReadValue::Goals(_)
        | ReadValue::Spend(_)
        | ReadValue::Project(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_) => None,
    }
}
fn ending(value: &ReadValue) -> Option<&Ending> {
    match value {
        ReadValue::Task(v) => v.result.as_ref().map(|result| &result.ending),
        ReadValue::Proposal(v) => v.ending.as_ref(),
        ReadValue::Escalation(v) => v.ending.as_ref(),
        ReadValue::Question(v) => v.ending.as_ref(),
        ReadValue::PersonTask(v) => v.ending.as_ref(),
        ReadValue::Result(v) => Some(&v.ending),
        ReadValue::Frame(_)
        | ReadValue::Ended(_)
        | ReadValue::History(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Conversations(_)
        | ReadValue::Goals(_)
        | ReadValue::Spend(_)
        | ReadValue::Project(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_) => None,
    }
}
fn decision_kind(decision: &Decision) -> ActKind {
    match decision {
        Decision::Accepted => ActKind::Accept,
        Decision::Rejected { .. } => ActKind::Reject,
        Decision::PassedUp => ActKind::PassUp,
        Decision::Released { .. } => ActKind::Release,
        Decision::LeftHeld { .. } => ActKind::LeaveHeld,
    }
}
type Target = (Option<u32>, Option<ObjectKey>, ActKind, Option<(Revision, Moved)>);
fn target(ask: &Ask) -> Target {
    match ask {
        Ask::StartChat { project, .. } => (Some(*project), None, ActKind::Write, None),
        Ask::Write { project, task, .. } => (Some(*project), Some(ObjectKey::Task(*task)), ActKind::Write, None),
        Ask::AnswerQuestion { project, task, message, .. } => {
            (Some(*project), Some(ObjectKey::Question { task: *task, message: *message }), ActKind::Answer, None)
        }
        Ask::AnswerPerson { project, task, revision, .. } => {
            (Some(*project), Some(ObjectKey::PersonTask(*task)), ActKind::Answer, Some((*revision, Moved::PersonTask)))
        }
        Ask::Take { project, task } => (Some(*project), Some(ObjectKey::PersonTask(*task)), ActKind::Take, None),
        Ask::HandBack { project, task } => {
            (Some(*project), Some(ObjectKey::PersonTask(*task)), ActKind::HandBack, None)
        }
        Ask::DecideProposal { project, proposal, decision } => {
            (Some(*project), Some(ObjectKey::Proposal(*proposal)), decision_kind(decision), None)
        }
        Ask::DecideEscalation { project, task, revision, decision } => (
            Some(*project),
            Some(ObjectKey::Escalation(*task)),
            decision_kind(decision),
            Some((*revision, Moved::Hold { reason: None })),
        ),
        Ask::Release { project, task, revision, .. } => (
            Some(*project),
            Some(ObjectKey::Task(*task)),
            ActKind::Release,
            Some((*revision, Moved::Hold { reason: None })),
        ),
        Ask::Amend { project, task, revision, release, .. } => (
            Some(*project),
            Some(ObjectKey::Task(*task)),
            if release.is_some() { ActKind::AmendRelease } else { ActKind::Amend },
            Some((*revision, Moved::Spec)),
        ),
        Ask::Stop { project, task } => (Some(*project), Some(ObjectKey::Task(*task)), ActKind::Stop, None),
        Ask::Cancel { project, task, .. } => (Some(*project), Some(ObjectKey::Task(*task)), ActKind::Cancel, None),
        Ask::Close { project, chat, .. } => (Some(*project), Some(ObjectKey::Task(*chat)), ActKind::Close, None),
        Ask::TakeOver { project, task, .. } => (Some(*project), Some(ObjectKey::Task(*task)), ActKind::TakeOver, None),
        Ask::SetGoal { project, .. } => (Some(*project), None, ActKind::SetGoal, None),
        Ask::Prioritise { project, .. } => (Some(*project), None, ActKind::Prioritise, None),
        Ask::MarkSeen { .. } => (None, None, ActKind::MarkSeen, None),
        Ask::EditNote { project, .. } => (Some(*project), None, ActKind::EditNote, None),
        Ask::DeleteNote { project, .. } => (Some(*project), None, ActKind::DeleteNote, None),
        Ask::ChangePolicy { project, .. } => (Some(*project), None, ActKind::ChangePolicy, None),
        Ask::SetRoles { project, .. } => (Some(*project), None, ActKind::SetRoles, None),
        Ask::MakeService { project, .. } => (Some(*project), None, ActKind::MakeService, None),
        Ask::SetPool { project, .. } => (Some(*project), None, ActKind::SetPool, None),
    }
}
