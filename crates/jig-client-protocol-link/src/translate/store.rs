//! The small store's owner records (protocol/client.md, 10).
use super::{answers, documents, pages, too_large};
use crate::Limits;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_domain::{Ask, Family};
use jig_client_protocol_wire::{self as wire, v8 as w};
use skein_lib::{List, Reader, RequestRecord, Wall, Writer, bytes};

pub(super) fn text(value: &[u8], limits: &Limits) -> Result<c::Text, w::Problem> {
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
        w::PolicyEdit::Templates(value) => super::operations::restore_templates(value.overrides()),
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
pub(crate) fn restore_ask(value: w::Ask, limits: &Limits) -> Result<c::Ask, w::Problem> {
    Ok(match value {
        w::Ask::Propose(value) => super::operations::restore_propose(&value, limits)?,
        w::Ask::Withdraw(value) => c::Ask::Withdraw { project: value.project(), proposal: value.proposal() },
        w::Ask::Create(value) => super::operations::restore_create(&value, limits)?,
        w::Ask::Compound(value) => super::compound::restore(value, limits)?,
        w::Ask::StartChat(value) => c::Ask::StartChat { project: value.project(), words: text(value.words(), limits)? },
        w::Ask::Message(value) => {
            c::Ask::Message { project: value.project(), task: value.task(), words: text(value.words(), limits)? }
        }
        w::Ask::Answer(value) => c::Ask::Answer {
            project: value.project(),
            on: match value.on() {
                w::AnswerOn::Question(v) => {
                    c::AnswerOn::Question { task: v.task(), message: v.message(), words: text(v.words(), limits)? }
                }
                w::AnswerOn::Person(v) => c::AnswerOn::Person {
                    task: v.task(),
                    revision: answers::revision(v.revision(), limits),
                    answer: person_answer(v.answer(), limits)?,
                },
            },
        },
        w::Ask::Take(value) => c::Ask::Take { project: value.project(), task: value.task() },
        w::Ask::HandBack(value) => c::Ask::HandBack { project: value.project(), task: value.task() },
        w::Ask::Decide(value) => c::Ask::Decide {
            project: value.project(),
            on: match value.on() {
                w::DecideOn::Proposal(v) => c::DecideOn::Proposal {
                    proposal: answers::proposal_key(v.proposal(), limits),
                    decision: answers::decision(v.decision(), limits)?,
                },
                w::DecideOn::Escalation(v) => c::DecideOn::Escalation {
                    task: v.task(),
                    revision: answers::revision(v.revision(), limits),
                    decision: answers::decision(v.decision(), limits)?,
                },
            },
        },
        w::Ask::Release(value) => c::Ask::Release {
            project: value.project(),
            task: value.task(),
            revision: answers::revision(value.revision(), limits),
            again: *value.again(),
        },
        w::Ask::Amend(value) => c::Ask::Amend {
            project: value.project(),
            task: value.task(),
            revision: match value.revision() {
                Some(value) => Some(answers::revision(value, limits)),
                None => None,
            },
            change: Box::new(pages::amendment_fields(value.change(), limits)?),
            reason: text(value.reason(), limits)?,
        },
        w::Ask::Stop(value) => c::Ask::Stop { project: value.project(), task: value.task() },
        w::Ask::Cancel(value) => c::Ask::Cancel {
            project: value.project(),
            task: value.task(),
            reason: text(value.reason(), limits)?,
            grace: *value.grace(),
        },
        w::Ask::Close(value) => {
            let mut disposals = List::with_capacity(value.disposals().len());
            for row in value.disposals() {
                disposals.push(disposal(row)).expect("reserved decoded disposals");
            }
            c::Ask::Close { project: value.project(), chat: value.chat(), disposals }
        }
        w::Ask::MoveTask(value) => {
            c::Ask::Move { project: value.project(), task: value.task(), reason: text(value.reason(), limits)? }
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
        w::Ask::Seen(value) => c::Ask::Seen { through: answers::seen(value.through(), limits) },
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
pub fn table(value: &RequestRecord<jig_client_domain::Held>, limits: &Limits) -> Result<w::TableEntry, w::Problem> {
    let (owner, version, request) = match &value.request.request {
        Ask::Jig(value) => {
            let record = super::asks::ask(value, &limits.wire)?;
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("u32 fits usize"));
            record.encode(&mut writer).expect("measured ask");
            (w::Owner::Jig, wire::VERSION, writer.finish())
        }
        Ask::Family { scope, record: value } => (
            w::Owner::Family(w::Family::new(&limits.wire, w::FamilyParts { number: value.family })?),
            value.version,
            super::envelope::request(*scope, value, &limits.wire)?,
        ),
    };
    let first_sent = match value.first_sent {
        Some(value) => Some(value.as_nanos()),
        None => None,
    };
    w::TableEntry::new(
        &limits.wire,
        w::TableEntryParts {
            asker: value.request.key.asker.0,
            number: value.request.key.number,
            key: value.key,
            scope: value.scope,
            first_sent,
            owner,
            version,
            request,
        },
    )
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub fn restore_table(
    value: w::TableEntry,
    limits: &Limits,
) -> Result<RequestRecord<jig_client_domain::Held>, w::Problem> {
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
        w::Owner::Family(owner) => {
            let (scope, bytes) = super::envelope::restore(value.request(), &limits.wire)?;
            Ask::Family { scope, record: Family { family: owner.number(), version: value.version(), bytes } }
        }
    };
    let first_sent = match value.first_sent() {
        Some(value) => Some(Wall::from_nanos(*value)),
        None => None,
    };
    Ok(RequestRecord {
        key: *value.key(),
        scope: value.scope(),
        first_sent,
        request: jig_client_domain::Held {
            key: jig_client_domain::Key { asker: jig_client_domain::Asker(value.asker()), number: value.number() },
            request,
        },
    })
}
