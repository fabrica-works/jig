//! Independent check of immediately used delivered offers (contract.md, 5.3).
//! This observer receives only boundary events, durable commit numbers and
//! documents/replies. It never reads a domain or calls an admission judge.
mod facts;
use jig_core::{
    self as core,
    acts::{Act, Admission, Offered},
    documents::{Document, Proposer, Subject, Task},
};
use jig_core_people as people;
use jig_test_domain as root;
use skein_lib::List;
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug)]
struct Received {
    party: u64,
    subject: Subject,
    at: u64,
    offers: List<Offered>,
    task: Option<Task>,
}
#[derive(Debug, Default)]
pub struct Referee {
    made: u64,
    people: BTreeMap<u64, u64>,
    watches: BTreeMap<u64, (u64, Subject)>,
    documents: BTreeMap<u64, Received>,
    pending: BTreeMap<u64, Admission>,
    reading: VecDeque<(u64, Option<Subject>)>,
    next_read: u64,
    pub checked: u32,
}
impl Referee {
    pub fn register(&mut self, sign_in: u64, person: u64, made: u64) {
        self.people.insert(sign_in, person);
        self.made = made;
    }
    pub fn committed(&mut self, made: u64) {
        self.made = made;
    }
    pub fn inbound(&mut self, event: &root::Event) {
        if let root::Event::Core(core::Event::Watch { watcher, sign_in, subject, .. }) = event
            && let Some(party) = self.people.get(sign_in)
        {
            self.watches.insert(watcher.raw(), (*party, subject.clone()));
        }
        if let root::Event::Core(core::Event::Read { sign_in, read, .. }) = event
            && let Some(party) = self.people.get(sign_in)
        {
            self.reading.push_back((*party, read_subject(read)));
        }
        if let root::Event::Core(core::Event::People(people::Event::Ask { sign_in, ask, .. })) = event {
            let Some(party) = self.people.get(sign_in) else {
                return;
            };
            let Some((act, subject)) = request(ask) else {
                return;
            };
            for received in self.documents.values() {
                if received.at != self.made || received.party != *party || !fits(&received.subject, &subject) {
                    continue;
                }
                if let Some(offer) = received.offers.iter().find(|value| value.act == act)
                    && let Some(expected) = facts::predict(&offer.admission, ask, received.task.as_ref())
                {
                    self.pending.insert(*sign_in, expected);
                }
            }
        }
    }
    fn remember(&mut self, number: u64, party: u64, subject: Subject, document: &Document) {
        let Some(offers) = offers(document) else {
            return;
        };
        let task = match document {
            Document::Task(value) => Some(*value.clone()),
            Document::Escalation(value) => Some(*value.task.clone()),
            Document::Frame(_)
            | Document::Chips(_)
            | Document::Run(_)
            | Document::Tree(_)
            | Document::Goals(_)
            | Document::Tasks(_)
            | Document::Proposal(_)
            | Document::Question(_)
            | Document::PersonTask(_)
            | Document::Result(_)
            | Document::Project(_)
            | Document::Note(_)
            | Document::Inbox(_)
            | Document::Conversations(_)
            | Document::Deployment(_) => None,
        };
        self.documents.insert(number, Received { party, subject, at: self.made, offers: offers.clone(), task });
    }
    pub fn delivered(&mut self, value: &root::Delivery) {
        match value {
            root::Delivery::Core(core::Held::Delivery { watcher, document }) => {
                if let Some((party, subject)) = self.watches.get(&watcher.raw()).cloned() {
                    self.remember(watcher.raw(), party, subject, document);
                }
            }
            root::Delivery::Core(core::Held::Read { answer, .. }) => {
                if let Some((party, Some(subject))) = self.reading.pop_front()
                    && let core::reads::Answer::Document(document) = answer
                {
                    self.next_read += 1;
                    self.remember(u64::MAX - self.next_read, party, subject, document);
                }
            }
            root::Delivery::Core(core::Held::PeopleReply { sign_in: Some(sign_in), reply, .. }) => {
                if let Some(expected) = self.pending.remove(sign_in) {
                    check(&expected, reply).expect("delivered offer matches admission");
                    self.checked += 1;
                }
            }
            root::Delivery::ApplicationDocument { .. }
            | root::Delivery::ApplicationRead { .. }
            | root::Delivery::Application { .. }
            | root::Delivery::Projection { .. }
            | root::Delivery::Message { .. }
            | root::Delivery::Restart(_)
            | root::Delivery::Core(_)
            | root::Delivery::CallAnswer { .. }
            | root::Delivery::Assigned { .. }
            | root::Delivery::Fleet(_)
            | root::Delivery::System { .. }
            | root::Delivery::LostRead { .. }
            | root::Delivery::Procedure { .. } => {}
        }
    }
}
/// The negative fake and the world's real transport use this exact rule.
/// # Errors
/// An immediate semantic answer contradicts its delivered admission.
pub fn check(expected: &Admission, actual: &people::Reply) -> Result<(), &'static str> {
    let answer = match actual {
        people::Reply::Outcome(value) => value,
        people::Reply::Refused(why) => {
            return if *expected == Admission::Refused(*why) { Ok(()) } else { Err("offered act was refused") };
        }
        people::Reply::Again(_) | people::Reply::SignedOut | people::Reply::SignedIn { .. } => {
            return Err("offer received no semantic admission");
        }
    };
    let matches = match expected {
        Admission::Refused(why) => *answer == people::Outcome::Refused(*why),
        Admission::Proposed { .. } => {
            matches!(answer, people::Outcome::GoalProposed { .. } | people::Outcome::AmendProposed { .. })
        }
        Admission::Allowed => !matches!(
            answer,
            people::Outcome::Refused(_)
                | people::Outcome::Again(_)
                | people::Outcome::SignedOut
                | people::Outcome::GoalProposed { .. }
                | people::Outcome::AmendProposed { .. }
        ),
        Admission::Bounded { .. } => false,
    };
    if matches { Ok(()) } else { Err("delivered offer contradicts admission") }
}
fn offers(value: &Document) -> Option<&List<Offered>> {
    Some(match value {
        Document::Task(v) => &v.acts,
        Document::Proposal(v) => &v.acts,
        Document::Escalation(v) => &v.acts,
        Document::Question(v) => &v.acts,
        Document::PersonTask(v) => &v.acts,
        Document::Result(v) => &v.acts,
        Document::Project(v) => &v.acts,
        Document::Note(v) => &v.acts,
        Document::Frame(_)
        | Document::Chips(_)
        | Document::Run(_)
        | Document::Tree(_)
        | Document::Goals(_)
        | Document::Tasks(_)
        | Document::Inbox(_)
        | Document::Conversations(_)
        | Document::Deployment(_) => return None,
    })
}

fn fits(received: &Subject, request: &Subject) -> bool {
    received == request
        || matches!((received, request),
        (Subject::Project { project: a }, Subject::Goals { project: b }) if a == b)
        || matches!((received, request), (Subject::Proposal { proposer: Proposer::Party(a), number: n }, Subject::Proposal { proposer: Proposer::Task(b), number: m }) if a == b && n == m)
}
fn request(ask: &people::Ask) -> Option<(Act, Subject)> {
    Some(match ask {
        people::Ask::SetGoal { project, .. } => (Act::SetGoal, Subject::Project { project: *project }),
        people::Ask::Say { task, .. } => (Act::Write, Subject::Task { task: *task }),
        people::Ask::AnswerQuestion { task, question, .. } => {
            (Act::Answer, Subject::Question { task: *task, message: *question })
        }
        people::Ask::TakePerson { task, .. } => (Act::Take, Subject::PersonTask { task: *task }),
        people::Ask::HandBackPerson { task, .. } => (Act::HandBack, Subject::PersonTask { task: *task }),
        people::Ask::AnswerPerson { task, .. } => (Act::Answer, Subject::PersonTask { task: *task }),
        people::Ask::DecideProposal { proposer, proposal, decision, .. } => (
            match decision {
                people::ProposalDecision::Accept => Act::Accept,
                people::ProposalDecision::Reject { .. } => Act::Reject,
                people::ProposalDecision::Pass => Act::PassUp,
            },
            Subject::Proposal { proposer: Proposer::Task(*proposer), number: *proposal },
        ),
        people::Ask::DecideEscalation { task, decision, .. } => (
            match decision {
                people::EscalationDecision::Release => Act::Release,
                people::EscalationDecision::Reject { .. } => Act::LeaveHeld,
                people::EscalationDecision::Pass => Act::PassUp,
            },
            Subject::Escalation { task: *task },
        ),
        people::Ask::Release { task, .. } => (Act::Release, Subject::Task { task: *task }),
        people::Ask::Amend { task, release, .. } => {
            (if release.is_some() { Act::AmendRelease } else { Act::Amend }, Subject::Task { task: *task })
        }
        people::Ask::Stop { task, .. } => (Act::Stop, Subject::Task { task: *task }),
        people::Ask::Cancel { task, .. } => (Act::Cancel, Subject::Task { task: *task }),
        people::Ask::Close { chat, .. } => (Act::Close, Subject::Task { task: *chat }),
        people::Ask::Move { task, .. } => (Act::TakeOver, Subject::Task { task: *task }),
        people::Ask::Prioritise { project, .. } => (Act::Prioritise, Subject::Goals { project: *project }),
        people::Ask::MarkSeen { seen: people::Seen::Chat { task, .. } } => {
            (Act::MarkSeen, Subject::Task { task: *task })
        }
        people::Ask::EditNote { name, change, .. } => (
            match &**change {
                people::NoteChange::Correct { .. } => Act::EditNote,
                people::NoteChange::Delete { .. } => Act::DeleteNote,
            },
            Subject::Note { note: *name },
        ),
        people::Ask::Adopt { project, .. } => (Act::Adopt, Subject::Project { project: *project }),
        people::Ask::ChangePolicy { project, .. } => (Act::ChangePolicy, Subject::Project { project: *project }),
        people::Ask::SetRoles { project, .. } => (Act::SetRoles, Subject::Project { project: *project }),
        people::Ask::MakeService { project, .. } => (Act::MakeService, Subject::Project { project: *project }),
        people::Ask::SetPool { project, .. } => (Act::SetPool, Subject::Project { project: *project }),
        people::Ask::Application { .. }
        | people::Ask::StartChat { .. }
        | people::Ask::MarkSeen { seen: people::Seen::Results { .. } } => return None,
    })
}

fn read_subject(read: &core::reads::Read) -> Option<Subject> {
    match read {
        core::reads::Read::Task { task } => Some(Subject::Task { task: *task }),
        core::reads::Read::Project { project } => Some(Subject::Project { project: *project }),
        core::reads::Read::Proposal { proposer, proposal } => {
            Some(Subject::Proposal { proposer: *proposer, number: *proposal })
        }
        core::reads::Read::Escalation { task } => Some(Subject::Escalation { task: *task }),
        core::reads::Read::Question { task, message } => Some(Subject::Question { task: *task, message: *message }),
        core::reads::Read::PersonTask { task } => Some(Subject::PersonTask { task: *task }),
        core::reads::Read::Result { task } => Some(Subject::Result { task: *task }),
        core::reads::Read::Frame
        | core::reads::Read::Deployment
        | core::reads::Read::Application { .. }
        | core::reads::Read::Reply { .. }
        | core::reads::Read::Turn { .. }
        | core::reads::Read::EndedDelegates { .. }
        | core::reads::Read::History { .. }
        | core::reads::Read::Activity { .. }
        | core::reads::Read::Inbox { .. }
        | core::reads::Read::Conversations { .. }
        | core::reads::Read::Goals { .. }
        | core::reads::Read::Notes { .. }
        | core::reads::Read::ScopedNotes { .. }
        | core::reads::Read::Spend { .. } => None,
    }
}
