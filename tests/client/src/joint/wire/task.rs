//! Whole current Task facts (client/domain/contract.md, 5.1).
//! The source owner supplies current effective instructions and no separate
//! `TaskContext`. Repeated specification facts must agree before admission.
use super::{chips, documents::Invalid, offers, owner, results};
use jig_client_domain::contract as client;
use jig_client_view::Limits;
use jig_domain_core::documents;

/// # Errors
/// Rejects inconsistent repeated owner facts or any whole nested allocation
/// above the receiving limits, without clipping a list or inventing context.
pub fn document(source: &documents::Task, limits: &Limits) -> Result<client::Task, Invalid> {
    if source.project != source.chip.details.project
        || source.executor != source.chip.details.executor
        || source.words != source.spec.words
        || source.parameters != source.spec.parameters
        || source.parameters.capacity() != source.spec.parameters.capacity()
    {
        return Err(Invalid::Identity);
    }
    copy(source, limits).map_err(Invalid::Owner)
}

fn copy(source: &documents::Task, limits: &Limits) -> Result<client::Task, owner::Invalid> {
    Ok(client::Task {
        label: source.label,
        flags: super::queries::flags(source.flags),
        last_activity: source.last_activity,
        waits_on: super::queries::waits(source.waits_on),
        chip: chips::chip(&source.chip, limits)?,
        goal: source.goal,
        priority: source.priority,
        lineage_chips: owner::list(&source.lineage, limits, |value| chips::chip(value, limits))?,
        inputs: owner::list(&source.inputs, limits, |value| results::link(value, limits))?,
        project: source.project,
        requester: results::requester(source.requester),
        lineage: owner::list(&source.lineage, limits, |value| Ok(value.number))?,
        spec: owner::specification(&source.spec, limits)?,
        spec_revision: client::Revision { value: source.spec_revision },
        person_revision: source.person_revision.map(|value| client::Revision { value }),
        taken_by: source.taken_by,
        instructions: source.instructions.as_ref().map(|value| owner::text(value, limits)).transpose()?,
        wake: owner::wake(source.wake),
        subscriptions: owner::subscriptions(&source.subscriptions, limits)?,
        result_contract: owner::contract(&source.contract, limits)?,
        dependencies: owner::list(&source.dependencies, limits, |value| results::link(value, limits))?,
        references: owner::list(&source.references, limits, |value| results::link(value, limits))?,
        cut: source.cut,
        resources: owner::list(&source.resources, limits, |value| {
            Ok(client::TaskResource::Owner(Box::new(chips::resource(value, limits)?)))
        })?,
        budget: owner::budget(&source.budget),
        authority: owner::authority(&source.authority, limits)?,
        hold: source
            .hold
            .as_ref()
            .map(|value| chips::held(value, limits).map(|value| client::TaskHold::Owner(Box::new(value))))
            .transpose()?,
        proposals: owner::list(&source.proposals, limits, |value| {
            Ok(client::ProposalKey {
                proposer: match value.proposer {
                    documents::Proposer::Task(number) => client::Proposer::Task(number),
                    documents::Proposer::Party(number) => client::Proposer::Party(number),
                },
                number: value.number,
            })
        })?,
        activity: source.activity,
        changed: source.changed,
        acts: offers::offered(&source.acts, limits)?,
        result: source
            .result
            .as_ref()
            .map(|value| results::ending(value, limits).map(|value| client::TaskResult::Ending(Box::new(value))))
            .transpose()?,
        delegates: owner::list(&source.delegates, limits, |value| {
            Ok(client::LiveDelegate { chip: chips::chip(&value.chip, limits)?, takeover_budget: value.takeover_budget })
        })?,
        context: None,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    const UTF8_TEXTS: [&[u8]; 1] = [b"~\0"];

    use super::*;
    use jig_domain_tasks as tasks;
    use skein_lib::{List, Wall};

    pub(crate) fn source() -> documents::Task {
        let chip = chips::tests::source_chip();
        let mut state = owner::tests::amendment_source();
        state.spec.parameters = List::with_capacity(4);
        state
            .spec
            .parameters
            .push(documents::Parameter::Number { name: u32::MAX, value: u64::MAX })
            .expect("source room");
        let mut inputs = List::with_capacity(4);
        inputs.push(documents::TaskLink::Stored { number: 0, status: None }).expect("source room");
        let mut resources = List::with_capacity(4);
        resources
            .push(documents::ResourceLine {
                part: tasks::Part {
                    connector: u16::MAX,
                    kind: tasks::PartKind::Effect,
                    key: tasks::PartKey::Number(u64::MAX),
                },
                kind: u16::MAX,
                taken: true,
                holder: None,
                writer: Some(tasks::Writer::Effect { entry: 0 }),
                writer_lost: true,
            })
            .expect("source room");
        let mut delegates = List::with_capacity(4);
        delegates.push(documents::LiveDelegate { chip: chip.clone(), takeover_budget: u64::MAX }).expect("source room");
        documents::Task {
            waits_on: documents::WaitsOn::Nothing,
            label: 0,
            flags: tasks::Flags::PLAIN,
            last_activity: skein_lib::Wall::EPOCH,
            project: chip.details.project,
            executor: chip.details.executor,
            requester: tasks::Party::Deployment { project: 0 },
            goal: Some(0),
            priority: Some(u32::MAX),
            lineage: List::with_capacity(4),
            delegates,
            changed: Wall::EPOCH,
            spec_revision: u64::MAX,
            person_revision: Some(0),
            taken_by: Some(u64::MAX),
            activity: u64::MAX,
            words: state.spec.words.clone(),
            parameters: state.spec.parameters.clone(),
            spec: state.spec,
            instructions: Some(documents::Text { bytes: Box::new([]), cut: false }),
            resources,
            inputs,
            dependencies: List::with_capacity(4),
            references: List::with_capacity(4),
            subscriptions: List::with_capacity(4),
            wake: state.wake,
            contract: documents::ContractLine::Change { connector: u16::MAX, kind: 0, words: u32::MAX },
            result: Some(documents::Ending::Failed(documents::Text { bytes: Box::from(UTF8_TEXTS[0]), cut: true })),
            budget: chip.details.budget,
            authority: state.authority,
            hold: Some(documents::HoldLine {
                account: None,
                source: None,
                reason: tasks::Hold::Uncertain { entry: u64::MAX },
                revision: 17,
                before: documents::PriorPhase::Active(tasks::Active::BackingOff { until: Wall::EPOCH }),
                escalation: None,
                tries: tasks::Tries { transient: 2, permanent: 3, run: 5, agent: 7, lost: 11, invalid: u32::MAX },
                hold_wait_since: None,
            }),
            proposals: List::with_capacity(4),
            cut: true,
            acts: List::with_capacity(4),
            chip,
        }
    }

    #[test]
    fn whole_current_task_keeps_owner_fields_absent_context_and_independent_revisions_without_fabricating_a_result() {
        let source = source();
        let original = source.clone();
        let actual = document(&source, &super::super::rows::tests::limits()).expect("whole Task");
        assert!(actual.context.is_none());
        assert_eq!(actual.project, u32::MAX);
        assert_eq!(actual.requester, client::OwnerRequester::Deployment { project: 0 });
        assert_eq!(actual.goal, Some(0));
        assert_eq!(actual.person_revision, Some(client::Revision { value: 0 }));
        assert_eq!(actual.spec_revision.value, u64::MAX);
        assert_eq!(actual.hold.as_ref().expect("whole owner hold").revision().value, 17);
        assert!(actual.hold.as_ref().expect("actual uncertainty").uncertain());
        assert_eq!(actual.taken_by, Some(u64::MAX));
        assert_eq!(actual.changed, Wall::EPOCH);
        assert_eq!(actual.inputs.as_slice(), &[client::OwnerTaskLink::Stored { number: 0, status: None }]);
        assert_eq!(actual.dependencies.capacity(), 4);
        assert_eq!(actual.references.capacity(), 4);
        assert_eq!(actual.lineage.capacity(), 4);
        assert_eq!(actual.lineage_chips.capacity(), 4);
        assert_eq!(actual.budget, owner::budget(&source.budget));
        assert_eq!(actual.wake, owner::wake(source.wake));
        assert_eq!(
            actual.authority,
            owner::authority(&source.authority, &super::super::rows::tests::limits()).expect("whole source authority")
        );
        assert_eq!(
            actual.result_contract,
            client::TaskContractDocument::Change { connector: u16::MAX, kind: 0, words: u32::MAX }
        );
        assert!(actual.cut);
        assert!(actual.instructions.as_ref().expect("explicit empty").bytes.bytes().is_empty());
        let client::TaskResource::Owner(resource) = actual.resources.get(0).expect("whole resource") else {
            panic!("raw owner reference stays whole")
        };
        assert_eq!(resource.part.kind, client::OwnerPartKind::Effect);
        assert_eq!(resource.writer, Some(client::OwnerResourceWriter::Effect { entry: 0 }));
        assert_eq!(resource.holder, None);
        assert!(resource.writer_lost);
        let client::TaskResult::Ending(ending) = actual.result.expect("actual source ending") else {
            panic!("no full Result metadata is supplied")
        };
        let client::OwnerChipEnding::Failed(words) = *ending else { panic!("source failure") };
        assert_eq!(words.bytes.bytes(), UTF8_TEXTS[0]);
        assert!(words.cut);
        assert_eq!(actual.delegates.get(0).expect("whole delegate").takeover_budget, u64::MAX);
        assert_eq!(source, original);
    }

    #[test]
    fn task_envelopes_admit_every_reserved_allocation_and_reject_contradictory_repeated_facts() {
        let source = source();
        let original = source.clone();
        let limits = super::super::rows::tests::limits();
        assert!(super::super::documents::task_read(&source, &limits).is_ok());
        assert!(super::super::documents::task_delivery(&source, &limits).is_ok());
        assert_eq!(
            super::super::documents::task_delivery(&source, &Limits { document_bytes: 0, ..limits }),
            Err(Invalid::Payload)
        );
        assert_eq!(document(&source, &Limits { rows: 3, ..limits }), Err(Invalid::Owner(owner::Invalid::TooLarge)));
        let mut changed = source.clone();
        changed.project = 0;
        assert_eq!(document(&changed, &limits), Err(Invalid::Identity));
        changed = source.clone();
        changed.words.cut = !changed.words.cut;
        assert_eq!(document(&changed, &limits), Err(Invalid::Identity));
        changed = source.clone();
        changed.parameters = List::with_capacity(0);
        assert_eq!(document(&changed, &limits), Err(Invalid::Identity));
        let mut lower = 0;
        let mut upper = limits.document_bytes;
        while lower < upper {
            let middle = lower + (upper - lower) / 2;
            if super::super::documents::task_delivery(&source, &Limits { document_bytes: middle, ..limits }).is_ok() {
                upper = middle;
            } else {
                lower = middle + 1;
            }
        }
        assert!(super::super::documents::task_delivery(&source, &Limits { document_bytes: lower, ..limits }).is_ok());
        assert_eq!(
            super::super::documents::task_delivery(&source, &Limits { document_bytes: lower - 1, ..limits }),
            Err(Invalid::Payload)
        );
        assert_eq!(source, original);
    }
}
