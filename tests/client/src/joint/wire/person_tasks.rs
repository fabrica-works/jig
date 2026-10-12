//! Whole pending person work (client/domain/objects.md, 7; contract.md, 5).
//! Shown links and optional complete results remain independent source facts.
use super::{chips, offers, owner, results};
use jig_client::{Limits, contract as client};
use jig_domain_core::documents;

/// # Errors
/// Refuses any complete nested allocation above the receiving bounds.
pub fn document(source: &documents::PersonTask, limits: &Limits) -> Result<client::OwnerPersonTask, owner::Invalid> {
    Ok(client::OwnerPersonTask {
        task: chips::chip(&source.task, limits)?,
        revision: client::Revision { value: source.revision },
        requester: results::requester(source.requester),
        spec: owner::specification(&source.spec, limits)?,
        address: chips::address(source.address),
        taken_by: source.taken_by,
        since: source.since,
        stall: source.stall,
        deadline: source.deadline,
        contract: owner::contract(&source.contract, limits)?,
        results: owner::list(&source.results, limits, |value| {
            Ok(client::OwnerShownResult {
                task: results::link(&value.task, limits)?,
                result: value
                    .result
                    .as_ref()
                    .map(|value| results::document(value, limits).map(Box::new))
                    .transpose()?,
            })
        })?,
        cut: source.cut,
        acts: offers::offered(&source.acts, limits)?,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use jig_domain_tasks as tasks;
    use skein_lib::{Duration, List, Wall};

    pub(crate) fn source() -> documents::PersonTask {
        let mut shown = List::with_capacity(4);
        shown
            .push(documents::ShownResult {
                task: documents::TaskLink::Stored { number: 0, status: None },
                result: None,
            })
            .expect("whole missing reference");
        shown
            .push(documents::ShownResult {
                task: documents::TaskLink::Retained { chip: chips::tests::source_chip(), at: None },
                result: Some(results::tests::document()),
            })
            .expect("whole supplied result");
        shown
            .push(documents::ShownResult {
                task: documents::TaskLink::Retained { chip: chips::tests::source_chip(), at: Some(Wall::EPOCH) },
                result: None,
            })
            .expect("whole retained reference");
        documents::PersonTask {
            acts: List::with_capacity(4),
            task: chips::tests::source_chip(),
            revision: u64::MAX,
            requester: tasks::Party::Deployment { project: 0 },
            spec: owner::tests::amendment_source().spec,
            address: tasks::PersonAddress::Role(u32::MAX),
            taken_by: Some(0),
            since: None,
            stall: Some(Duration::ZERO),
            deadline: Some(Wall::EPOCH),
            contract: documents::ContractLine::Change { connector: u16::MAX, kind: 0, words: u32::MAX },
            results: shown,
            cut: true,
        }
    }

    #[test]
    fn person_work_keeps_missing_and_whole_shown_results_in_source_order_without_clock_or_scope_inference() {
        let source = source();
        let original = source.clone();
        let limits = Limits { rows: 8, ..super::super::rows::tests::limits() };
        let actual = document(&source, &limits).expect("whole pending person work");
        assert_eq!(actual.revision.value, u64::MAX);
        assert_eq!(actual.project(), Some(u32::MAX));
        assert_eq!(actual.requester, client::OwnerRequester::Deployment { project: 0 });
        assert_eq!(actual.address, client::PersonAddress::Role(u32::MAX));
        assert_eq!(actual.taken_by, Some(0));
        assert_eq!((actual.since, actual.stall, actual.deadline), (None, Some(Duration::ZERO), Some(Wall::EPOCH)));
        assert_eq!((actual.results.len(), actual.results.capacity()), (3, 4));
        assert_eq!(
            actual.results.get(0).expect("first original row").task,
            client::OwnerTaskLink::Stored { number: 0, status: None }
        );
        assert!(actual.results.get(0).expect("missing result").result.is_none());
        let row = actual.results.get(1).expect("second original row");
        let client::OwnerTaskLink::Retained { at, .. } = row.task else { panic!("actual retained link") };
        assert_eq!(at, None);
        let result = row.result.as_ref().expect("complete source result");
        assert_eq!(result.at, None);
        assert!(result.details.as_ref().expect("whole metadata").followups.is_none());
        assert_eq!(result.details.as_ref().expect("reserved metadata").delegates.capacity(), 8);
        let client::OwnerTaskLink::Retained { at, .. } = actual.results.get(2).expect("third original row").task else {
            panic!("actual retained link")
        };
        assert_eq!(at, Some(Wall::EPOCH));
        assert!(actual.results.get(2).expect("missing result independent of clock").result.is_none());
        assert!(actual.cut);
        assert_eq!(source, original);
    }

    #[test]
    fn person_work_preserves_absent_and_present_zero_wait_facts_and_all_requester_address_kinds() {
        let limits = Limits { rows: 8, ..super::super::rows::tests::limits() };
        for requester in
            [tasks::Party::Task(u64::MAX), tasks::Party::Person(0), tasks::Party::Deployment { project: u32::MAX }]
        {
            for address in [tasks::PersonAddress::Person(u64::MAX), tasks::PersonAddress::Role(0)] {
                for present in [false, true] {
                    let mut source = source();
                    source.requester = requester;
                    source.address = address;
                    source.taken_by = present.then_some(u64::MAX);
                    source.since = present.then_some(Wall::EPOCH);
                    source.stall = present.then_some(Duration::ZERO);
                    source.deadline = present.then_some(Wall::EPOCH);
                    source.cut = false;
                    let actual = document(&source, &limits).expect("whole independent optional facts");
                    assert_eq!(actual.requester, results::requester(requester));
                    assert_eq!(actual.address, chips::address(address));
                    assert_eq!(
                        (actual.taken_by, actual.since, actual.stall, actual.deadline),
                        (source.taken_by, source.since, source.stall, source.deadline)
                    );
                    assert!(!actual.cut);
                }
            }
        }
    }

    #[test]
    fn person_work_read_prices_whole_reserved_results_and_refuses_one_under_without_shortening() {
        let source = source();
        let original = source.clone();
        let limits = Limits { rows: 8, ..super::super::rows::tests::limits() };
        let value = super::super::documents::person_task_read(&source, &limits).expect("whole read");
        let bounds = jig_scripted_engine::Bounds {
            text: limits.text,
            rows: limits.rows,
            document_bytes: limits.document_bytes,
            ..jig_scripted_engine::Bounds::default()
        };
        let client::ReadValue::OwnerPersonTask(actual) = &value else { panic!("genuine person work") };
        let bytes = jig_scripted_engine::owner_person_task::bytes(actual, &bounds).expect("whole reserved heap")
            + u64::try_from(core::mem::size_of::<client::ReadValue>()).expect("inline envelope");
        assert_eq!(
            super::super::documents::person_task_read(&source, &Limits { document_bytes: bytes, ..limits }),
            Ok(value)
        );
        assert_eq!(
            super::super::documents::person_task_read(&source, &Limits { document_bytes: bytes - 1, ..limits }),
            Err(super::super::documents::Invalid::Payload)
        );
        assert_eq!(document(&source, &Limits { rows: 7, ..limits }), Err(owner::Invalid::TooLarge));
        assert_eq!(document(&source, &Limits { text: 1, ..limits }), Err(owner::Invalid::TooLarge));
        assert_eq!(source, original);
    }
}
