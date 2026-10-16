//! Field-for-field client contract translation (protocol/client.md, 3 and 5).
use super::{
    Child, document_text, owner_chip_hold, owner_executor, owner_numbers, owner_part, owner_person_wait,
    owner_wake_policy, part, party, proposal_key, retry_notice, take_chip, take_owner_chip_ending, take_result_value,
};
use crate::Limits;
use crate::translate::{piece_problem, too_large};
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v8 as w;
use skein_lib::Map;
use skein_lib::{List, Reader, Wall, bytes};
pub(crate) fn funder(value: &w::Funder, _limits: &Limits) -> c::Funder {
    match value {
        w::Funder::Task(row) => c::Funder::Task(row.value()),
        w::Funder::Pool(row) => c::Funder::Pool { project: row.project(), person: row.person(), period: row.period() },
        w::Funder::Recurring(row) => {
            c::Funder::Recurring { project: row.project(), task: row.task(), period: row.period() }
        }
        w::Funder::Period(row) => c::Funder::Period { project: row.project(), period: row.period() },
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_budget(value: &w::OwnerBudget, limits: &Limits) -> c::OwnerBudget {
    let opt0 = match value.deadline() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    c::OwnerBudget {
        numbers: owner_numbers(value.numbers(), limits),
        left: value.left(),
        funder: funder(value.funder(), limits),
        deadline: opt0,
    }
}

pub(crate) fn owner_resource_writer(value: &w::OwnerResourceWriter, _limits: &Limits) -> c::OwnerResourceWriter {
    match value {
        w::OwnerResourceWriter::Run(row) => c::OwnerResourceWriter::Run { task: row.task(), attempt: row.attempt() },
        w::OwnerResourceWriter::Effect(row) => c::OwnerResourceWriter::Effect { entry: row.entry() },
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_resource_line(value: &w::OwnerResourceLine, limits: &Limits) -> c::OwnerResourceLine {
    let opt0 = match value.holder() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt1 = match &value.writer() {
        Some(item) => Some(owner_resource_writer(item, limits)),
        None => None,
    };
    c::OwnerResourceLine {
        part: owner_part(value.part(), limits),
        kind: value.kind(),
        taken: value.taken(),
        holder: opt0,
        writer: opt1,
        writer_lost: value.writer_lost(),
    }
}

pub(crate) fn owner_resource_line_piece(
    bytes: &[u8],
    limits: &Limits,
) -> Result<List<c::OwnerResourceLine>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(limits.client.rows);
    for _ in 0..=limits.client.rows {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(problem) => return Err(piece_problem(problem)),
        };
        let Some(item) = item else { return Ok(rows) };
        let row = w::OwnerResourceLine::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = owner_resource_line(&row, limits);
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn owner_chip_work(value: &w::OwnerChipWork, _limits: &Limits) -> c::OwnerChipWork {
    match value {
        w::OwnerChipWork::Preparing => c::OwnerChipWork::Preparing,
        w::OwnerChipWork::Starting => c::OwnerChipWork::Starting,
        w::OwnerChipWork::Active => c::OwnerChipWork::Active,
        w::OwnerChipWork::Writing => c::OwnerChipWork::Writing,
        w::OwnerChipWork::Tool(row) => {
            c::OwnerChipWork::Tool { call: bytes::copy_of(row.call()), tool: bytes::copy_of(row.tool()) }
        }
        w::OwnerChipWork::Waiting => c::OwnerChipWork::Waiting,
        w::OwnerChipWork::Ending => c::OwnerChipWork::Ending,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_chip_observation(value: &w::OwnerChipObservation, limits: &Limits) -> c::OwnerChipObservation {
    let opt0 = match value.started() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    c::OwnerChipObservation {
        channel: skein_lib::Token::new(value.channel()),
        work: owner_chip_work(value.work(), limits),
        since: Wall::from_nanos(value.since()),
        reported_at: Wall::from_nanos(value.reported_at()),
        started: opt0,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_chip_details_fields(
    value: &w::OwnerChipDetailsFields,
    limits: &Limits,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::OwnerChipDetails, w::Problem> {
    let opt0 = match value.run_since() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt1 = match value.attempt() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt2 = match &value.retry() {
        Some(item) => Some(retry_notice(item, limits)),
        None => None,
    };
    let opt3 = match &value.person_wait() {
        Some(item) => Some(owner_person_wait(item, limits)),
        None => None,
    };
    let opt4 = match &value.hold() {
        Some(item) => Some(owner_chip_hold(item, limits)?),
        None => None,
    };
    let opt5 = match &value.procedure() {
        Some(item) => Some(owner_part(item, limits)),
        None => None,
    };
    let opt6 = match &value.closing_connectors() {
        Some(item) => {
            let mut list0 = List::with_capacity((item).len());
            for item in item {
                list0.push(*item).expect("reserved supplied list");
            }
            Some(list0)
        }
        None => None,
    };
    let opt7 = match &value.ending() {
        Some(item) => Some(take_owner_chip_ending(*item, index, children)?),
        None => None,
    };
    let opt8 = match value.ended_at() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt9 = match &value.observation() {
        Some(item) => Some(owner_chip_observation(item, limits)),
        None => None,
    };
    Ok(c::OwnerChipDetails {
        project: value.project(),
        executor: owner_executor(value.executor(), limits),
        phase_since: Wall::from_nanos(value.phase_since()),
        run_since: opt0,
        attempt: opt1,
        retry: opt2,
        wake: owner_wake_policy(value.wake(), limits),
        person_wait: opt3,
        hold: opt4,
        budget: owner_budget(value.budget(), limits),
        activity: value.activity(),
        resources: owner_resource_line_piece(value.resources(), limits)?,
        resources_cut: value.resources_cut(),
        resources_observed: value.resources_observed(),
        procedure: opt5,
        live_delegates: value.live_delegates(),
        closing_connectors: opt6,
        ending: opt7,
        ended_at: opt8,
        observation: opt9,
    })
}

pub(crate) fn owner_requester(value: &w::OwnerRequester, _limits: &Limits) -> c::OwnerRequester {
    match value {
        w::OwnerRequester::Task(row) => c::OwnerRequester::Task(row.value()),
        w::OwnerRequester::Person(row) => c::OwnerRequester::Person(row.value()),
        w::OwnerRequester::Deployment(row) => c::OwnerRequester::Deployment { project: row.project() },
    }
}

pub(crate) fn owner_chip_ending_fields(
    value: &w::OwnerChipEndingFields,
    limits: &Limits,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::OwnerChipEnding, w::Problem> {
    Ok(match value {
        w::OwnerChipEndingFields::Done(row) => {
            c::OwnerChipEnding::Done(take_result_value(row.value(), index, children)?)
        }
        w::OwnerChipEndingFields::Failed(row) => c::OwnerChipEnding::Failed(document_text(row.value(), limits)?),
        w::OwnerChipEndingFields::Cancelled(row) => {
            let opt0 = match &row.result() {
                Some(item) => Some(take_result_value(*item, index, children)?),
                None => None,
            };
            c::OwnerChipEnding::Cancelled {
                reason: document_text(row.reason(), limits)?,
                result: opt0,
                by: owner_requester(row.by(), limits),
                at: Wall::from_nanos(row.at()),
            }
        }
    })
}

pub(crate) fn result_field(value: &w::ResultField, _limits: &Limits) -> c::ResultField {
    c::ResultField { name: bytes::copy_of(value.name()), value: bytes::copy_of(value.value()) }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn result_value_fields(
    value: &w::ResultValueFields,
    limits: &Limits,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::ResultValue, w::Problem> {
    Ok(match value {
        w::ResultValueFields::Report(row) => c::ResultValue::Report(document_text(row.value(), limits)?),
        w::ResultValueFields::OwnerVerdict(row) => {
            let mut list0 = List::with_capacity(row.fields().len());
            for item in row.fields() {
                list0.push(result_field(item, limits)).expect("reserved supplied list");
            }
            c::ResultValue::OwnerVerdict {
                verdict: row.verdict(),
                words: document_text(row.words(), limits)?,
                fields: list0,
            }
        }
        w::ResultValueFields::Verdict(row) => {
            let mut list0 = List::with_capacity(row.fields().len());
            for item in row.fields() {
                list0.push(result_field(item, limits)).expect("reserved supplied list");
            }
            let mut list1 = List::with_capacity(row.tasks().len());
            for item in row.tasks() {
                list1.push(take_chip(*item, index, children)?).expect("reserved supplied list");
            }
            let opt2 = match &row.proposal() {
                Some(item) => Some(proposal_key(item, limits)),
                None => None,
            };
            c::ResultValue::Verdict {
                verdict: row.verdict(),
                words: document_text(row.words(), limits)?,
                fields: list0,
                tasks: list1,
                proposal: opt2,
            }
        }
        w::ResultValueFields::Connector(row) => {
            c::ResultValue::Connector { summary: document_text(row.summary(), limits)?, part: part(row.part(), limits) }
        }
        w::ResultValueFields::Change(row) => c::ResultValue::Change {
            kind: row.kind(),
            part: owner_part(row.part(), limits),
            words: document_text(row.words(), limits)?,
        },
        w::ResultValueFields::Choice(row) => c::ResultValue::Choice {
            option: row.option(),
            by: party(row.by(), limits)?,
            reason: document_text(row.reason(), limits)?,
        },
        w::ResultValueFields::Approved(row) => {
            let opt0 = match &row.words() {
                Some(item) => Some(document_text(item, limits)?),
                None => None,
            };
            c::ResultValue::Approved { state: bytes::copy_of(row.state()), words: opt0 }
        }
        w::ResultValueFields::Failure(row) => c::ResultValue::Failure(document_text(row.value(), limits)?),
        w::ResultValueFields::Words(row) => c::ResultValue::Words(document_text(row.value(), limits)?),
    })
}
