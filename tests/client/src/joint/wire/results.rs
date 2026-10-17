//! Exact result payload and closing facts (client/domain/objects.md, 8).
//! References and chooser/requester metadata belong to the independent details.
use super::{chips, offers, owner};
use jig_client_domain::contract as client;
use jig_client_view::Limits;
use jig_domain_core::documents;
use jig_domain_tasks as tasks;
use skein_lib::List;

#[must_use]
pub fn requester(value: tasks::Party) -> client::OwnerRequester {
    match value {
        tasks::Party::Task(number) => client::OwnerRequester::Task(number),
        tasks::Party::Person(number) => client::OwnerRequester::Person(number),
        tasks::Party::Deployment { project } => client::OwnerRequester::Deployment { project },
    }
}

/// # Errors
/// Refuses any complete result, opaque machine field or linked owner allocation above the receiving bounds.
pub fn value(value: &documents::ResultValue, limits: &Limits) -> Result<client::ResultValue, owner::Invalid> {
    Ok(match value {
        documents::ResultValue::Approved { state, words } => client::ResultValue::Approved {
            state: owner::bytes(state, limits)?,
            words: words.as_ref().map(|words| owner::text(words, limits)).transpose()?,
        },
        documents::ResultValue::Report(words) => client::ResultValue::Report(owner::text(words, limits)?),
        documents::ResultValue::Verdict { code, fields, words } => {
            let capacity = u32::try_from(fields.len()).map_err(|_| owner::Invalid::TooLarge)?;
            if capacity > limits.rows {
                return Err(owner::Invalid::TooLarge);
            }
            let mut copied = List::with_capacity(capacity);
            for field in fields {
                copied
                    .push(client::ResultField {
                        name: owner::bytes(&field.name, limits)?,
                        value: owner::bytes(&field.value, limits)?,
                    })
                    .map_err(|_| owner::Invalid::TooLarge)?;
            }
            client::ResultValue::OwnerVerdict { verdict: *code, words: owner::text(words, limits)?, fields: copied }
        }
        documents::ResultValue::Change { kind, part, words } => client::ResultValue::Change {
            kind: *kind,
            part: owner::part(part, limits)?,
            words: owner::text(words, limits)?,
        },
        documents::ResultValue::Failure(words) => client::ResultValue::Failure(owner::text(words, limits)?),
    })
}

/// # Errors
/// Refuses any complete result, opaque machine field or linked owner allocation above the receiving bounds.
pub fn ending(value: &documents::Ending, limits: &Limits) -> Result<client::OwnerChipEnding, owner::Invalid> {
    Ok(match value {
        documents::Ending::Done(result) => client::OwnerChipEnding::Done(self::value(result, limits)?),
        documents::Ending::Failed(words) => client::OwnerChipEnding::Failed(owner::text(words, limits)?),
        documents::Ending::Cancelled { reason, result, by, at } => client::OwnerChipEnding::Cancelled {
            reason: owner::text(reason, limits)?,
            result: result.as_ref().map(|result| self::value(result, limits)).transpose()?,
            by: requester(*by),
            at: *at,
        },
    })
}

/// # Errors
/// Refuses any complete result, opaque machine field or linked owner allocation above the receiving bounds.
pub fn link(value: &documents::TaskLink, limits: &Limits) -> Result<client::OwnerTaskLink, owner::Invalid> {
    Ok(match value {
        documents::TaskLink::Live(value) => client::OwnerTaskLink::Live(chips::chip(value, limits)?),
        documents::TaskLink::Retained { chip, at } => {
            client::OwnerTaskLink::Retained { chip: chips::chip(chip, limits)?, at: *at }
        }
        documents::TaskLink::Stored { number, status } => client::OwnerTaskLink::Stored {
            number: *number,
            status: status.map(|value| match value {
                tasks::Status::Done => client::OwnerTaskStatus::Done,
                tasks::Status::Failed => client::OwnerTaskStatus::Failed,
                tasks::Status::Cancelled => client::OwnerTaskStatus::Cancelled,
            }),
        },
    })
}

/// Copy only the independent metadata supplied by this whole Result document.
/// Its ending, wall time and offered actions belong to the surrounding envelope.
/// # Errors
/// Refuses any complete result, opaque machine field or linked owner allocation above the receiving bounds.
pub fn details(value: &documents::ResultDoc, limits: &Limits) -> Result<client::ResultDetails, owner::Invalid> {
    Ok(client::ResultDetails {
        contract: owner::contract(&value.contract, limits)?,
        requester: requester(value.requester),
        executor: owner::executor(value.executor),
        chosen_by: value.chosen_by,
        delegates: owner::list(&value.delegates, limits, |value| link(value, limits))?,
        followups: value
            .followups
            .as_ref()
            .map(|value| owner::list(value, limits, |value| link(value, limits)))
            .transpose()?,
        proposal: value.proposal.as_ref().map(|value| client::ProposalKey {
            proposer: match value.proposer {
                documents::Proposer::Task(number) => client::Proposer::Task(number),
                documents::Proposer::Party(number) => client::Proposer::Party(number),
            },
            number: value.number,
        }),
        cut: value.cut,
    })
}

/// The whole result document supplies its terminal facts independently of its
/// completion wall, offers and metadata. The native passive value field is an
/// exact copy of the result carried by Done or Cancelled, never a synthetic one.
/// # Errors
/// Refuses any nested source allocation above the receiving bounds.
pub fn document(source: &documents::ResultDoc, limits: &Limits) -> Result<client::Result, owner::Invalid> {
    let result = match &source.ending {
        documents::Ending::Done(value) => Some(self::value(value, limits)?),
        documents::Ending::Failed(_) => None,
        documents::Ending::Cancelled { result, .. } => {
            result.as_ref().map(|value| self::value(value, limits)).transpose()?
        }
    };
    Ok(client::Result {
        task: chips::chip(&source.task, limits)?,
        ending: client::Ending::Task(Box::new(ending(&source.ending, limits)?)),
        value: result,
        at: source.at,
        details: Some(Box::new(details(source, limits)?)),
        acts: offers::offered(&source.acts, limits)?,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    const UTF8_TEXTS: [&[u8]; 3] = [b"\0~", b"~\0", b"~"];

    use super::*;
    use skein_lib::Wall;

    fn text(bytes: &[u8], cut: bool) -> documents::Text {
        documents::Text { bytes: bytes.into(), cut }
    }

    #[test]
    fn exact_verdict_payload_keeps_whole_opaque_fields_and_loss_without_fabricating_reference_arrays() {
        let source = documents::ResultValue::Verdict {
            code: u32::MAX,
            fields: Box::new([tasks::ResultField { name: Box::from([255, 0]), value: Box::from([b'/', 255]) }]),
            words: text(UTF8_TEXTS[0], true),
        };
        let before = source.clone();
        let limits = Limits { text: 2, rows: 1, ..crate::test::limits().jig };
        let actual = value(&source, &limits).expect("whole exact payload fits");
        let client::ResultValue::OwnerVerdict { verdict, words, fields } = actual else {
            panic!("the actual payload has no chip or proposal arrays")
        };
        assert_eq!(verdict, u32::MAX);
        assert_eq!(words.bytes.bytes(), UTF8_TEXTS[0]);
        assert!(words.cut);
        assert_eq!(fields.capacity(), 1);
        assert_eq!(&*fields.get(0).expect("supplied machine field").name, &[255, 0]);
        assert_eq!(&*fields.get(0).expect("supplied machine field").value, &[b'/', 255]);
        assert_eq!(value(&source, &Limits { text: 1, ..limits }), Err(owner::Invalid::TooLarge));
        assert_eq!(value(&source, &Limits { rows: 0, ..limits }), Err(owner::Invalid::TooLarge));
        assert_eq!(source, before);
    }

    #[test]
    fn approved_result_state_and_optional_empty_explanation_are_copied_independently_from_failure_words() {
        let limits = Limits { text: 2, ..crate::test::limits().jig };
        for words in [None, Some(text(b"", false)), Some(text(UTF8_TEXTS[1], true))] {
            let source = documents::ResultValue::Approved { state: Box::from([0, 255]), words: words.clone() };
            let actual = value(&source, &limits).expect("whole approval");
            let client::ResultValue::Approved { state, words: actual } = actual else { panic!("exact approved state") };
            assert_eq!(&*state, &[0, 255]);
            assert_eq!(actual.as_ref().map(|w| (w.bytes.bytes(), w.cut)), words.as_ref().map(|w| (&*w.bytes, w.cut)));
            assert_eq!(value(&source, &Limits { text: 1, ..limits }), Err(owner::Invalid::TooLarge));
        }
        assert_eq!(
            value(&documents::ResultValue::Failure(text(b"", true)), &limits),
            Ok(client::ResultValue::Failure(client::DocumentText {
                bytes: client::Text::new(Box::new([]), limits.text).expect("empty text fits"),
                cut: true
            }))
        );
    }

    #[test]
    fn pending_cancellation_preserves_exact_requester_reason_loss_and_prior_result_without_inventing_a_completion_wall()
    {
        let limits = crate::test::limits().jig;
        for by in [
            tasks::Party::Task(u64::MAX),
            tasks::Party::Person(u64::MAX),
            tasks::Party::Deployment { project: u32::MAX },
        ] {
            for result in [None, Some(documents::ResultValue::Approved { state: Box::new([]), words: None })] {
                let source = documents::Ending::Cancelled {
                    reason: text(UTF8_TEXTS[2], true),
                    result: result.clone(),
                    by,
                    at: Wall::EPOCH,
                };
                let before = source.clone();
                let actual = ending(&source, &limits).expect("whole pending ending");
                let client::OwnerChipEnding::Cancelled { reason, result: actual, by: actor, at } = actual else {
                    panic!("whole cancellation")
                };
                assert_eq!(reason.bytes.bytes(), UTF8_TEXTS[2]);
                assert!(reason.cut);
                assert_eq!(actor, requester(by));
                assert_eq!(at, Wall::EPOCH);
                assert_eq!(actual.is_some(), result.is_some());
                if let Some(result) = actual {
                    assert_eq!(result, client::ResultValue::Approved { state: Box::new([]), words: None });
                }
                assert_eq!(source, before);
            }
        }
    }
    pub(crate) fn document() -> documents::ResultDoc {
        documents::ResultDoc {
            acts: List::with_capacity(4),
            task: super::super::chips::tests::source_chip(),
            contract: documents::ContractLine::Change { connector: u16::MAX, kind: u16::MAX, words: u32::MAX },
            ending: documents::Ending::Failed(text(UTF8_TEXTS[1], true)),
            at: None,
            requester: tasks::Party::Deployment { project: u32::MAX },
            executor: tasks::Executor::Procedure { connector: u16::MAX, code: u32::MAX },
            chosen_by: None,
            followups: None,
            delegates: List::with_capacity(8),
            proposal: None,
            cut: true,
        }
    }

    #[test]
    fn result_metadata_keeps_exact_links_optional_clock_status_chooser_and_closed_proposal_maker_independent() {
        let limits = Limits { rows: 8, ..crate::test::limits().jig };
        for chooser in [None, Some(0), Some(u64::MAX)] {
            for proposer in
                [None, Some(documents::Proposer::Task(u64::MAX)), Some(documents::Proposer::Party(u64::MAX))]
            {
                let mut source = document();
                source.chosen_by = chooser;
                source.proposal = proposer.map(|proposer| documents::ProposalRef { proposer, number: u64::MAX });
                source.followups = Some(List::with_capacity(4));
                source.delegates.push(documents::TaskLink::Live(source.task.clone())).expect("live reference");
                for at in [None, Some(Wall::EPOCH)] {
                    source
                        .delegates
                        .push(documents::TaskLink::Retained { chip: source.task.clone(), at })
                        .expect("retained reference");
                }
                for status in
                    [None, Some(tasks::Status::Done), Some(tasks::Status::Failed), Some(tasks::Status::Cancelled)]
                {
                    source
                        .delegates
                        .push(documents::TaskLink::Stored { number: u64::MAX, status })
                        .expect("stored reference");
                }
                let before = source.clone();
                let result = details(&source, &limits).expect("whole metadata");
                assert_eq!(
                    result.contract,
                    client::TaskContractDocument::Change { connector: u16::MAX, kind: u16::MAX, words: u32::MAX }
                );
                assert_eq!(result.requester, client::OwnerRequester::Deployment { project: u32::MAX });
                assert_eq!(result.executor, client::OwnerExecutor::Procedure { connector: u16::MAX, code: u32::MAX });
                assert_eq!(result.chosen_by, chooser);
                assert_eq!(result.followups.as_ref().map(List::capacity), Some(4));
                assert_eq!(result.followups.as_ref().map(List::len), Some(0));
                assert_eq!(result.delegates.capacity(), 8);
                assert_eq!(result.delegates.len(), 7);
                let linked = result.delegates.as_slice();
                let client::OwnerTaskLink::Live(live) = &linked[0] else { panic!("live identity") };
                assert_eq!(live.task, u64::MAX);
                assert!(live.title.cut);
                for (index, clock) in [(1, None), (2, Some(Wall::EPOCH))] {
                    let client::OwnerTaskLink::Retained { chip, at } = &linked[index] else {
                        panic!("retained identity")
                    };
                    assert_eq!(chip.task, u64::MAX);
                    assert_eq!(*at, clock);
                    assert_eq!(chip.phase, client::Phase::Ended);
                }
                for (index, status) in [
                    (3, None),
                    (4, Some(client::OwnerTaskStatus::Done)),
                    (5, Some(client::OwnerTaskStatus::Failed)),
                    (6, Some(client::OwnerTaskStatus::Cancelled)),
                ] {
                    assert_eq!(linked[index], client::OwnerTaskLink::Stored { number: u64::MAX, status });
                }
                assert_eq!(
                    result.proposal.map(|value| (value.proposer, value.number)),
                    proposer.map(|value| (
                        match value {
                            documents::Proposer::Task(number) => client::Proposer::Task(number),
                            documents::Proposer::Party(number) => client::Proposer::Party(number),
                        },
                        u64::MAX
                    ))
                );
                assert!(result.cut);
                assert_eq!(source, before);
            }
        }
    }

    #[test]
    fn result_metadata_refuses_reserved_rows_and_whole_linked_text_without_reconstructing_missing_followups() {
        let source = document();
        let before = source.clone();
        let limits = Limits { rows: 8, ..crate::test::limits().jig };
        let copied = details(&source, &limits).expect("supplied metadata fits");
        assert_eq!(copied.followups, None);
        assert_eq!(copied.chosen_by, None);
        assert_eq!(copied.proposal, None);
        assert_eq!(details(&source, &Limits { rows: 7, ..limits }), Err(owner::Invalid::TooLarge));
        let mut with_link = source.clone();
        with_link.delegates.push(documents::TaskLink::Live(source.task.clone())).expect("whole linked chip");
        let retained = with_link.clone();
        assert_eq!(details(&with_link, &Limits { text: 1, ..limits }), Err(owner::Invalid::TooLarge));
        assert_eq!(with_link, retained);
        assert_eq!(source, before);
    }
}
