//! Exact batch member contracts, funding and connector references (objects.md, 4).
use crate::{
    Builder, JigBindings, JigMeaning, Owner, Role, SlotRef,
    phrases::{Hole, jig::JigPhrase},
    proposed_task_words::ProposedPhrase,
    words::Words,
};
use jig_domain_client::{
    Limits,
    contract::{Funder, OwnerExecutor, OwnerPart, OwnerPartKey, OwnerPartKind, PersonAddress, TaskContractDocument},
};

pub fn text(b: &mut Builder, words: &Words, phrase: ProposedPhrase, holes: &[Hole]) {
    let saying = words.book.say(words.language, phrase.id(), holes).expect("exact member words were checked");
    b.text(crate::Run::Plain, &saying.text);
}
pub fn executor(b: &mut Builder, value: OwnerExecutor, words: &Words) {
    match value {
        OwnerExecutor::Agent { charter } => {
            text(b, words, ProposedPhrase::ExecutorCharter, &[Hole::Number(u64::from(charter))]);
        }
        OwnerExecutor::Procedure { connector, code } => text(
            b,
            words,
            ProposedPhrase::ExecutorProcedure,
            &[Hole::Number(u64::from(connector)), Hole::Number(u64::from(code))],
        ),
        OwnerExecutor::Person(address) => match address {
            PersonAddress::Party(person) => text(b, words, ProposedPhrase::ExecutorParty, &[Hole::Number(person)]),
            PersonAddress::Role(role) => text(b, words, ProposedPhrase::ExecutorRole, &[Hole::Number(u64::from(role))]),
        },
    }
}
pub fn funder(b: &mut Builder, value: Funder, words: &Words) {
    match value {
        Funder::Task(task) => text(b, words, ProposedPhrase::FunderTask, &[Hole::Number(task)]),
        Funder::Pool { project, person, period } => text(
            b,
            words,
            ProposedPhrase::FunderPool,
            &[Hole::Number(u64::from(project)), Hole::Number(person), Hole::Number(period)],
        ),
        Funder::Recurring { project, task, period } => text(
            b,
            words,
            ProposedPhrase::FunderRecurring,
            &[Hole::Number(u64::from(project)), Hole::Number(task), Hole::Number(period)],
        ),
        Funder::Period { project, period } => {
            text(b, words, ProposedPhrase::FunderPeriod, &[Hole::Number(u64::from(project)), Hole::Number(period)]);
        }
    }
}
/// Every actual funder identity is one checked saying leaf.
#[must_use]
pub const fn funder_nodes() -> u32 {
    1
}
/// Maximum funder saying across every loaded language and owner variant.
#[must_use]
pub fn funder_phrase_bytes(words: &Words) -> Option<u32> {
    let mut largest = 0;
    for phrase in [
        ProposedPhrase::FunderTask,
        ProposedPhrase::FunderPool,
        ProposedPhrase::FunderRecurring,
        ProposedPhrase::FunderPeriod,
    ] {
        largest = largest.max(words.book.capacity(phrase.id())?);
    }
    Some(largest)
}
pub fn part(b: &mut Builder, value: &OwnerPart, words: &Words, bindings: &mut JigBindings) {
    b.open(Role::Region);
    text(b, words, ProposedPhrase::PartConnector, &[Hole::Number(u64::from(value.connector))]);
    let kind = match value.kind {
        OwnerPartKind::Resource => ProposedPhrase::PartResource,
        OwnerPartKind::Pool => ProposedPhrase::PartPool,
        OwnerPartKind::Result => ProposedPhrase::PartResult,
        OwnerPartKind::Procedure => ProposedPhrase::PartProcedure,
        OwnerPartKind::Effect => ProposedPhrase::PartEffect,
        OwnerPartKind::Outbox => ProposedPhrase::PartOutbox,
        OwnerPartKind::Hold => ProposedPhrase::PartHold,
    };
    let name = words.book.say(words.language, kind.id(), &[]).expect("exact part kind words were checked");
    b.name(&name.text);
    text(b, words, kind, &[]);
    match &value.key {
        OwnerPartKey::Number(number) => text(b, words, ProposedPhrase::PartNumber, &[Hole::Number(*number)]),
        OwnerPartKey::Path(segments) => {
            for segment in segments {
                crate::literal_segment::render(b, segment);
            }
        }
    }
    let token = bindings.insert(JigMeaning::OwnerPart(value.clone())).token;
    b.slot(
        SlotRef { owner: Owner::Application { number: value.connector }, token },
        &words.say(JigPhrase::LoadingPart, &[]),
    );
    b.close();
}
pub fn contract(b: &mut Builder, value: &TaskContractDocument, words: &Words, bindings: &mut JigBindings) {
    match value {
        TaskContractDocument::Approval { state, part: reference } => {
            text(b, words, ProposedPhrase::ContractApproval, &[]);
            crate::literal_segment::render(b, state);
            part(b, reference, words, bindings);
        }
        TaskContractDocument::Report { words: cap } => {
            text(b, words, ProposedPhrase::ContractReport, &[Hole::Number(u64::from(*cap))]);
        }
        TaskContractDocument::Verdict { choices, cut } => {
            text(b, words, ProposedPhrase::ContractVerdict, &[]);
            for choice in choices {
                text(
                    b,
                    words,
                    ProposedPhrase::VerdictChoice,
                    &[
                        Hole::Number(u64::from(choice.code)),
                        Hole::Number(u64::from(choice.words)),
                        Hole::Number(u64::from(choice.followups)),
                    ],
                );
                for field in &choice.fields {
                    crate::literal_segment::render(b, &field.name);
                    text(b, words, ProposedPhrase::FieldCap, &[Hole::Number(u64::from(field.max))]);
                }
            }
            if *cut {
                text(b, words, ProposedPhrase::ContractCut, &[]);
            }
        }
        TaskContractDocument::Change { connector, kind, words: cap } => text(
            b,
            words,
            ProposedPhrase::ContractChange,
            &[Hole::Number(u64::from(*connector)), Hole::Number(u64::from(*kind)), Hole::Number(u64::from(*cap))],
        ),
    }
}
#[must_use]
pub fn part_nodes(limits: &Limits) -> Option<u32> {
    limits.rows.max(1).checked_add(6)
}
#[must_use]
pub fn contract_nodes(limits: &Limits) -> Option<u32> {
    let approval = part_nodes(limits)?.checked_add(2)?;
    let verdict = limits.rows.checked_mul(2)?.checked_add(1)?.checked_mul(limits.rows)?.checked_add(2)?;
    Some(approval.max(verdict).max(1))
}
