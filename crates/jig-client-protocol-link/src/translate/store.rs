//! The small store's owner records (protocol/client.md, 10).
use super::{answers, documents, pages, too_large};
use crate::{Ask, Family, Limits};
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::{self as wire, v7 as w};
use jig_domain_client::address::{Address, ProjectSection, TaskSection};
use skein_lib::{List, Reader, RequestRecord, Wall, Writer, bytes};

fn project_section(value: ProjectSection) -> w::ProjectSection {
    match value {
        ProjectSection::Overview => w::ProjectSection::Overview,
        ProjectSection::Goals => w::ProjectSection::Goals,
        ProjectSection::Tasks => w::ProjectSection::Tasks,
        ProjectSection::Notes => w::ProjectSection::Notes,
        ProjectSection::Settings => w::ProjectSection::Settings,
        ProjectSection::Resources => w::ProjectSection::Resources,
        ProjectSection::Policy => w::ProjectSection::Policy,
        ProjectSection::Roles => w::ProjectSection::Roles,
        ProjectSection::Spend => w::ProjectSection::Spend,
        ProjectSection::Recurring => w::ProjectSection::Recurring,
    }
}
fn task_section(value: TaskSection) -> w::TaskSection {
    match value {
        TaskSection::Overview => w::TaskSection::Overview,
        TaskSection::Activity => w::TaskSection::Activity,
        TaskSection::Plan => w::TaskSection::Plan,
        TaskSection::History => w::TaskSection::History,
        TaskSection::Budget => w::TaskSection::Budget,
        TaskSection::Panels => w::TaskSection::Panels,
    }
}
fn restore_project_section(value: &w::ProjectSection) -> ProjectSection {
    match value {
        w::ProjectSection::Overview => ProjectSection::Overview,
        w::ProjectSection::Goals => ProjectSection::Goals,
        w::ProjectSection::Tasks => ProjectSection::Tasks,
        w::ProjectSection::Notes => ProjectSection::Notes,
        w::ProjectSection::Settings => ProjectSection::Settings,
        w::ProjectSection::Resources => ProjectSection::Resources,
        w::ProjectSection::Policy => ProjectSection::Policy,
        w::ProjectSection::Roles => ProjectSection::Roles,
        w::ProjectSection::Spend => ProjectSection::Spend,
        w::ProjectSection::Recurring => ProjectSection::Recurring,
    }
}
fn restore_task_section(value: &w::TaskSection) -> TaskSection {
    match value {
        w::TaskSection::Overview => TaskSection::Overview,
        w::TaskSection::Activity => TaskSection::Activity,
        w::TaskSection::Plan => TaskSection::Plan,
        w::TaskSection::History => TaskSection::History,
        w::TaskSection::Budget => TaskSection::Budget,
        w::TaskSection::Panels => TaskSection::Panels,
    }
}
fn address(value: Address, limits: &w::Limits) -> Result<w::Address, w::Problem> {
    Ok(match value {
        Address::Inbox { project } => {
            w::Address::Inbox(w::AddressInbox::new(limits, w::AddressInboxParts { project })?)
        }
        Address::Conversations { project } => {
            w::Address::Conversations(w::AddressConversations::new(limits, w::AddressConversationsParts { project })?)
        }
        Address::Task { task, section } => {
            w::Address::Task(w::AddressTask::new(limits, w::AddressTaskParts { task, section: task_section(section) })?)
        }
        Address::Project { project, section } => w::Address::Project(w::AddressProject::new(
            limits,
            w::AddressProjectParts { project, section: project_section(section) },
        )?),
        Address::Deployment => w::Address::Deployment,
    })
}
fn restore_address(value: &w::Address) -> Address {
    match value {
        w::Address::Inbox(value) => Address::Inbox { project: *value.project() },
        w::Address::Conversations(value) => Address::Conversations { project: *value.project() },
        w::Address::Task(value) => Address::Task { task: value.task(), section: restore_task_section(value.section()) },
        w::Address::Project(value) => {
            Address::Project { project: value.project(), section: restore_project_section(value.section()) }
        }
        w::Address::Deployment => Address::Deployment,
    }
}
fn text(value: &[u8], limits: &Limits) -> Result<c::Text, w::Problem> {
    match c::Text::new(bytes::copy_of(value), limits.client.text) {
        Ok(value) => Ok(value),
        Err(_) => Err(too_large()),
    }
}

fn person_answer(value: &w::PersonAnswer, limits: &Limits) -> Result<c::PersonAnswer, w::Problem> {
    Ok(match value {
        w::PersonAnswer::Choice(value) => {
            c::PersonAnswer::Choice { option: value.option(), reason: text(value.reason(), limits)? }
        }
        w::PersonAnswer::Approved(value) => {
            let words = match value.words() {
                Some(value) => Some(text(value, limits)?),
                None => None,
            };
            c::PersonAnswer::Approved { state: bytes::copy_of(value.state()), words }
        }
        w::PersonAnswer::Words(value) => c::PersonAnswer::Words(text(value.value(), limits)?),
        w::PersonAnswer::Cannot(value) => c::PersonAnswer::Cannot { reason: text(value.reason(), limits)? },
    })
}
fn disposal(value: &w::Disposal) -> c::Disposal {
    match value {
        w::Disposal::Cancel(value) => c::Disposal::Cancel { task: value.task() },
        w::Disposal::TakeOver(value) => c::Disposal::TakeOver { task: value.task() },
    }
}
fn goal_executor(value: &w::GoalExecutor) -> c::GoalExecutor {
    match value {
        w::GoalExecutor::Charter(value) => c::GoalExecutor::Charter(value.value()),
        w::GoalExecutor::Procedure(value) => {
            c::GoalExecutor::Procedure { connector: value.connector(), code: value.code() }
        }
    }
}
fn policy_edit(value: &w::PolicyEdit, limits: &Limits) -> Result<c::PolicyEdit, w::Problem> {
    Ok(match value {
        w::PolicyEdit::ProjectSpend(value) => c::PolicyEdit::ProjectSpend { period_spend: value.period_spend() },
        w::PolicyEdit::Role(value) => {
            let value = value.value();
            c::PolicyEdit::Role(c::PolicyEditRole {
                number: value.number(),
                authority: documents::owner_authority_fields(value.authority(), limits)?,
                period_spend: value.period_spend(),
                requests: value.requests(),
                decides: value.decides(),
            })
        }
        w::PolicyEdit::Requirements(value) => c::PolicyEdit::Requirements {
            requirements: documents::policy_requirement_piece(value.requirements(), limits, limits.client.rows)?,
        },
        w::PolicyEdit::Projections(value) => c::PolicyEdit::Projections {
            grants: documents::owner_grant_piece(value.grants(), limits, limits.client.rows)?,
        },
        w::PolicyEdit::Permissions(value) => {
            let mut mappings = List::with_capacity(value.mappings().len());
            for row in value.mappings() {
                mappings.push(documents::policy_permission_fields(row, limits)).expect("reserved decoded mappings");
            }
            c::PolicyEdit::Permissions { mappings }
        }
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
#[expect(clippy::too_many_lines, reason = "one exhaustive translation arm per kept client ask")]
fn restore_ask(value: w::Ask, limits: &Limits) -> Result<c::Ask, w::Problem> {
    Ok(match value {
        w::Ask::StartChat(value) => c::Ask::StartChat { project: value.project(), words: text(value.words(), limits)? },
        w::Ask::Write(value) => c::Ask::Write {
            project: value.project(),
            task: value.task(),
            words: text(value.words(), limits)?,
            release: value.release(),
        },
        w::Ask::AnswerQuestion(value) => c::Ask::AnswerQuestion {
            project: value.project(),
            task: value.task(),
            message: value.message(),
            words: text(value.words(), limits)?,
        },
        w::Ask::AnswerPerson(value) => c::Ask::AnswerPerson {
            project: value.project(),
            task: value.task(),
            revision: answers::revision(value.revision(), limits),
            answer: person_answer(value.answer(), limits)?,
        },
        w::Ask::Take(value) => c::Ask::Take { project: value.project(), task: value.task() },
        w::Ask::HandBack(value) => c::Ask::HandBack { project: value.project(), task: value.task() },
        w::Ask::DecideProposal(value) => c::Ask::DecideProposal {
            project: value.project(),
            proposal: answers::proposal_key(value.proposal(), limits),
            decision: answers::decision(value.decision(), limits)?,
        },
        w::Ask::DecideEscalation(value) => c::Ask::DecideEscalation {
            project: value.project(),
            task: value.task(),
            revision: answers::revision(value.revision(), limits),
            decision: answers::decision(value.decision(), limits)?,
        },
        w::Ask::Release(value) => c::Ask::Release {
            project: value.project(),
            task: value.task(),
            revision: answers::revision(value.revision(), limits),
            again: *value.again(),
        },
        w::Ask::Amend(value) => {
            let release = match value.release() {
                Some(value) => Some(answers::revision(value, limits)),
                None => None,
            };
            c::Ask::Amend {
                project: value.project(),
                task: value.task(),
                revision: answers::revision(value.revision(), limits),
                change: pages::amendment_fields(value.change(), limits)?,
                release,
                reason: text(value.reason(), limits)?,
            }
        }
        w::Ask::Stop(value) => c::Ask::Stop { project: value.project(), task: value.task() },
        w::Ask::Cancel(value) => {
            c::Ask::Cancel { project: value.project(), task: value.task(), reason: text(value.reason(), limits)? }
        }
        w::Ask::Close(value) => {
            let mut disposals = List::with_capacity(value.disposals().len());
            for row in value.disposals() {
                disposals.push(disposal(row)).expect("reserved decoded disposals");
            }
            c::Ask::Close { project: value.project(), chat: value.chat(), disposals }
        }
        w::Ask::TakeOver(value) => {
            c::Ask::TakeOver { project: value.project(), task: value.task(), reason: text(value.reason(), limits)? }
        }
        w::Ask::SetGoal(value) => {
            let deadline = match value.deadline() {
                Some(value) => Some(Wall::from_nanos(*value)),
                None => None,
            };
            c::Ask::SetGoal {
                project: value.project(),
                spec: text(value.spec(), limits)?,
                executor: goal_executor(value.executor()),
                budget: value.budget(),
                priority: value.priority(),
                deadline,
            }
        }
        w::Ask::Prioritise(value) => {
            let mut goals = List::with_capacity(value.goals().len());
            for row in value.goals() {
                goals.push(c::Priority { task: row.task(), priority: row.priority() }).expect("reserved decoded goals");
            }
            c::Ask::Prioritise { project: value.project(), goals }
        }
        w::Ask::MarkSeen(value) => c::Ask::MarkSeen { seen: answers::seen(value.seen(), limits) },
        w::Ask::EditNote(value) => c::Ask::EditNote {
            project: value.project(),
            note: value.note(),
            scope: answers::note_scope(value.scope(), limits),
            revision: answers::revision(value.revision(), limits),
            description: text(value.description(), limits)?,
            body: text(value.body(), limits)?,
            references: value.references().clone(),
        },
        w::Ask::DeleteNote(value) => c::Ask::DeleteNote {
            project: value.project(),
            note: value.note(),
            scope: answers::note_scope(value.scope(), limits),
            revision: answers::revision(value.revision(), limits),
        },
        w::Ask::ChangePolicy(value) => {
            c::Ask::ChangePolicy { project: value.project(), edit: policy_edit(value.edit(), limits)? }
        }
        w::Ask::SetRoles(value) => {
            let mut holdings = List::with_capacity(value.holdings().len());
            for row in value.holdings() {
                holdings
                    .push(c::Holding { party: row.party(), role: answers::held_role(row.role(), limits) })
                    .expect("reserved decoded holdings");
            }
            c::Ask::SetRoles { project: value.project(), holdings }
        }
        w::Ask::MakeService(value) => c::Ask::MakeService {
            project: value.project(),
            name: text(value.name(), limits)?,
            role: answers::held_role(value.role(), limits),
        },
        w::Ask::SetPool(value) => {
            c::Ask::SetPool { project: value.project(), party: value.party(), budget: value.budget() }
        }
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub fn table(value: &RequestRecord<Ask>, limits: &Limits) -> Result<w::TableEntry, w::Problem> {
    let (owner, version, request) = match &value.request {
        Ask::Jig(value) => {
            let record = super::asks::ask(value, &limits.wire)?;
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("u32 fits usize"));
            record.encode(&mut writer).expect("measured ask");
            (w::Owner::Jig, wire::VERSION, writer.finish())
        }
        Ask::Family(value) => (
            w::Owner::Family(w::Family::new(&limits.wire, w::FamilyParts { number: value.family })?),
            value.version,
            bytes::copy_of(&value.bytes),
        ),
    };
    let first_sent = match value.first_sent {
        Some(value) => Some(value.as_nanos()),
        None => None,
    };
    w::TableEntry::new(
        &limits.wire,
        w::TableEntryParts { key: value.key, scope: value.scope, first_sent, owner, version, request },
    )
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub fn restore_table(value: w::TableEntry, limits: &Limits) -> Result<RequestRecord<Ask>, w::Problem> {
    let request = match value.owner() {
        w::Owner::Jig => {
            if value.version() != wire::VERSION {
                return Err(super::piece_problem(skein_codec::Reason::Version));
            }
            let mut reader = Reader::new(value.request());
            let ask = w::Ask::decode(&limits.wire, &mut reader)?;
            if reader.remaining() != 0 {
                return Err(super::piece_problem(skein_codec::Reason::Malformed));
            }
            Ask::Jig(Box::new(restore_ask(ask, limits)?))
        }
        w::Owner::Family(owner) => Ask::Family(Family {
            family: owner.number(),
            version: value.version(),
            bytes: bytes::copy_of(value.request()),
        }),
    };
    let first_sent = match value.first_sent() {
        Some(value) => Some(Wall::from_nanos(*value)),
        None => None,
    };
    Ok(RequestRecord { key: *value.key(), scope: value.scope(), first_sent, request })
}

pub fn place(party: u64, at: Address, limits: &Limits) -> Result<w::Place, w::Problem> {
    w::Place::new(&limits.wire, w::PlaceParts { party, address: address(at, &limits.wire)? })
}
#[must_use]
pub fn restore_place(value: w::Place, version: u16) -> jig_domain_client::store::Record {
    jig_domain_client::store::Record::Place { version, party: value.party(), address: restore_address(value.address()) }
}
pub fn draft(party: u64, task: u64, words: &c::Text, touched: u64, limits: &Limits) -> Result<w::Draft, w::Problem> {
    w::Draft::new(&limits.wire, w::DraftParts { party, task, words: bytes::copy_of(words.bytes()), touched })
}
pub fn restore_draft(
    value: w::Draft,
    version: u16,
    limits: &Limits,
) -> Result<jig_domain_client::store::Record, w::Problem> {
    Ok(jig_domain_client::store::Record::Draft {
        version,
        party: value.party(),
        task: value.task(),
        words: text(value.words(), limits)?,
        touched: value.touched(),
    })
}

pub fn jig(value: &jig_domain_client::store::Record, limits: &Limits) -> Result<w::JigRecord, w::Problem> {
    Ok(match value {
        jig_domain_client::store::Record::Place { party, address, .. } => {
            w::JigRecord::Place(place(*party, *address, limits)?)
        }
        jig_domain_client::store::Record::Draft { party, task, words, touched, .. } => {
            w::JigRecord::Draft(draft(*party, *task, words, *touched, limits)?)
        }
    })
}
pub fn restore_jig(
    value: w::JigRecord,
    version: u16,
    limits: &Limits,
) -> Result<jig_domain_client::store::Record, w::Problem> {
    match value {
        w::JigRecord::Place(value) => Ok(restore_place(value, version)),
        w::JigRecord::Draft(value) => restore_draft(value, version, limits),
    }
}
