//! Whole held-task escalation with its independent decisions (contract.md, 5.1–5.3).
use super::{documents::Invalid, offers, task};
use jig_client::DomainLimits as Limits;
use jig_client_domain::contract as client;
use jig_domain_core::documents;

/// # Errors
/// Refuses inconsistent whole Task facts or any nested reserved allocation;
/// the escalation's acts never replace its Task's independent acts or result.
pub fn document(
    source: &documents::Escalation,
    limits: &Limits,
    models: &[jig_domain_core::Model],
) -> Result<client::OwnerEscalation, Invalid> {
    Ok(client::OwnerEscalation {
        acts: offers::offered(&source.acts, limits, models).map_err(Invalid::Owner)?,
        task: Box::new(task::document(&source.task, limits, models)?),
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::super::owner;
    use super::*;
    use jig_domain_core::acts;
    use skein_lib::List;

    pub(crate) fn source() -> documents::Escalation {
        let mut task = task::tests::source();
        task.acts = List::with_capacity(client::MAX_OFFERED_ACTS);
        task.acts
            .push(acts::Offered { effect: None, act: acts::Act::Stop, admission: acts::Admission::Allowed })
            .expect("one act");
        let mut acts = List::with_capacity(client::MAX_OFFERED_ACTS);
        for act in [acts::Act::Release, acts::Act::PassUp, acts::Act::Release] {
            acts.push(acts::Offered { effect: None, act, admission: acts::Admission::Allowed })
                .expect("three outer acts");
        }
        documents::Escalation { acts, task: Box::new(task) }
    }

    #[test]
    fn whole_escalation_keeps_independent_offers_hold_revisions_optional_result_and_all_task_facts() {
        let source = source();
        let original = source.clone();
        let limits = super::super::rows::tests::limits();
        let actual = document(&source, &limits, &[]).expect("whole supplied escalation");
        assert_eq!(actual.acts.capacity(), client::MAX_OFFERED_ACTS);
        assert_eq!(
            actual.acts.iter().map(|value| value.act).collect::<Vec<_>>(),
            [client::ActKind::Release, client::ActKind::PassUp, client::ActKind::Release]
        );
        assert!(actual.acts.iter().all(|value| value.admission == client::Admission::Allowed));
        assert_eq!(actual.task.acts.capacity(), client::MAX_OFFERED_ACTS);
        assert_eq!(actual.task.acts.get(0).expect("independent task act").act, client::ActKind::Stop);
        assert_eq!(actual.task.chip.task, u64::MAX);
        assert_eq!(actual.task.project, u32::MAX);
        assert_eq!(actual.task.spec_revision.value, u64::MAX);
        assert_eq!(actual.task.person_revision, Some(client::Revision { value: 0 }));
        assert_eq!(actual.task.hold.as_ref().expect("supplied hold").revision().value, 17);
        assert!(actual.task.hold.as_ref().expect("source uncertainty").uncertain());
        assert_eq!(actual.task.inputs.as_slice(), &[client::OwnerTaskLink::Stored { number: 0, status: None }]);
        assert_eq!(actual.task.delegates.get(0).expect("whole delegate").takeover_budget, u64::MAX);
        assert!(actual.task.result.is_some());
        assert!(actual.task.context.is_none());
        assert!(actual.task.cut);
        assert_eq!(
            actual.task.as_ref(),
            &task::document(&source.task, &limits, &[]).expect("same whole task contract")
        );
        let body = jig_client_view::objects::Body::OwnerEscalation(Box::new(actual));
        assert_eq!(body.ending(), None);
        assert_eq!(body.terminal(), None);
        assert_eq!(source, original);
    }

    #[test]
    fn escalation_read_admits_the_entire_envelope_and_both_reserved_offers_without_clipping() {
        let source = source();
        let original = source.clone();
        let limits = super::super::rows::tests::limits();
        let read = super::super::documents::escalation_read(&source, &limits, &[]).expect("whole read");
        let mut lower = 0;
        let mut upper = limits.document_bytes;
        while lower < upper {
            let middle = lower + (upper - lower) / 2;
            if super::super::documents::escalation_read(
                &source,
                &Limits { document_bytes: middle, turn_text: limits.turn_text, ..limits },
                &[],
            )
            .is_ok()
            {
                upper = middle;
            } else {
                lower = middle + 1;
            }
        }
        let exact = lower;
        assert!(exact > 0);
        assert_eq!(
            super::super::documents::escalation_read(
                &source,
                &Limits { document_bytes: exact, turn_text: limits.turn_text, ..limits },
                &[]
            ),
            Ok(read.clone())
        );
        assert_eq!(
            super::super::documents::escalation_read(
                &source,
                &Limits { document_bytes: exact - 1, turn_text: limits.turn_text, ..limits },
                &[]
            ),
            Err(Invalid::Payload)
        );
        assert_eq!(
            super::super::documents::escalation_read(
                &source,
                &Limits { rows: 3, turn_text: limits.turn_text, ..limits },
                &[]
            ),
            Err(Invalid::Owner(owner::Invalid::TooLarge))
        );
        for outer in [false, true] {
            let mut oversized = source.clone();
            if outer {
                oversized.acts = List::with_capacity(client::MAX_OFFERED_ACTS + 1);
            } else {
                oversized.task.acts = List::with_capacity(client::MAX_OFFERED_ACTS + 1);
            }
            let untouched = oversized.clone();
            assert_eq!(
                document(&oversized, &limits, &[]),
                Err(Invalid::Owner(owner::Invalid::OfferCapacity {
                    allocated: client::MAX_OFFERED_ACTS + 1,
                    maximum: client::MAX_OFFERED_ACTS,
                }))
            );
            assert_eq!(oversized, untouched);
        }
        let mut inconsistent = source.clone();
        inconsistent.task.project = 0;
        assert_eq!(document(&inconsistent, &limits, &[]), Err(Invalid::Identity));
        assert_eq!(source, original);
    }
}
