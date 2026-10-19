//! Passive whole historical objects (client/domain/client.md, 5;
//! contract.md, 6 and view.md, 7, 10). Original identities and independent
//! sources are retained without acquiring current offers or manufacturing context.
use crate::JigBindings;
use crate::owner_historical_words::HistoricalPhrase as P;
use crate::words::Words;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use crate::{Borrowed, Limits};
use alloc::boxed::Box;
use jig_client::contract::{
    Chip, ChipPhase, OwnerEscalationDecision, OwnerGoalHolding, OwnerHistorical, OwnerHistoricalValue,
    OwnerMessageProposalKind, OwnerPattern, OwnerPatternEnd, OwnerPersonProposalGoal, OwnerPersonProposalState,
    OwnerProposalChoice, OwnerProposalDecision, OwnerRequester, OwnerTaskLink, Parameter, ParameterDocument, Proposer,
    Result, Specification, Task, TaskContractDocument, TaskResource, TaskResult, Text,
};
use skein_lib::{Id, Wall};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoricalFragment {
    pub nodes: u32,
    pub text_bytes: u32,
    pub text_per_node: u32,
    pub depth: u32,
    pub bindings: u32,
    pub slots: u32,
}
fn saying(words: &Words, phrase: P, holes: &[Hole]) -> Box<[u8]> {
    words.book().say(words.language(), phrase.id(), holes).expect("checked historical sayings and typed holes").text
}
fn text(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    b.text(Run::Plain, &saying(words, phrase, holes));
}
fn label(b: &mut Builder, words: &Words, phrase: P) {
    text(b, words, phrase, &[]);
}
fn number(b: &mut Builder, words: &Words, phrase: P, number: u64) {
    text(b, words, phrase, &[Hole::Number(number)]);
}
fn at(b: &mut Builder, words: &Words, phrase: P, at: Wall) {
    text(b, words, phrase, &[Hole::Wall(words.adjusted_wall(at))]);
}
fn raw(b: &mut Builder, value: &Text) {
    crate::literal_segment::render(b, value.bytes());
}
fn region(b: &mut Builder, words: &Words, phrase: P) {
    b.open(Role::Region);
    b.name(&saying(words, phrase, &[]));
}
fn kind(b: &mut Builder, value: OwnerMessageProposalKind, words: &Words) {
    label(
        b,
        words,
        match value {
            OwnerMessageProposalKind::Batch => P::BatchKind,
            OwnerMessageProposalKind::Effect => P::EffectKind,
            OwnerMessageProposalKind::Amend => P::AmendKind,
            OwnerMessageProposalKind::Widen => P::WidenKind,
            OwnerMessageProposalKind::Release => P::ReleaseKind,
        },
    );
}
fn goal(b: &mut Builder, value: &OwnerPersonProposalGoal, words: &Words, bindings: &mut JigBindings, rows: u32) {
    let text_limit = bindings.text_limit();
    number(b, words, P::Goal, value.number);
    number(b, words, P::Project, u64::from(value.project));
    crate::proposed_task::executor(b, value.executor, words);
    crate::specification::specification(b, &value.spec, words);
    crate::proposed_task::contract(b, &value.contract, words, bindings);
    crate::owner_authority::render_words(b, &value.authority, words, text_limit, rows);
    for (phrase, amount) in [
        (P::Budget, value.numbers.budget),
        (P::Spent, value.numbers.spent),
        (P::SpentBelow, value.numbers.spent_below),
        (P::Reserved, value.numbers.reserved),
    ] {
        text(b, words, phrase, &[Hole::Amount(amount)]);
    }
    crate::proposed_task::funder(b, value.funder, words);
    for task in &value.dependencies {
        number(b, words, P::Dependency, *task);
    }
    for holding in &value.holdings {
        let (connector, path, kind, phrase) = match holding {
            OwnerGoalHolding::Write { connector, path, kind } => (*connector, path, *kind, P::Write),
            OwnerGoalHolding::Slot { connector, path, kind } => (*connector, path, *kind, P::Slot),
        };
        region(b, words, phrase);
        label(b, words, phrase);
        number(b, words, P::Connector, u64::from(connector));
        number(b, words, P::Kind, u64::from(kind));
        crate::literal_segment::path(
            b,
            path.as_slice(),
            words,
            crate::literal_segment::PathDisplay { text: bindings.text_limit(), reserved: 0, keep_last: true },
        );
        b.close();
    }
    crate::owner_wake::policy(b, &value.wake, words.book(), words.language());
    match value.tracked {
        Some(number_) => number(b, words, P::Tracked, u64::from(number_)),
        None => label(b, words, P::TrackedAbsent),
    }
}
fn proposal_decision(b: &mut Builder, value: &OwnerProposalDecision, words: &Words) {
    number(b, words, P::Project, u64::from(value.project));
    match value.proposer {
        Proposer::Task(task) => crate::owner_result::requester(b, OwnerRequester::Task(task), words),
        Proposer::Party(person) => crate::owner_result::requester(b, OwnerRequester::Person(person), words),
    }
    number(b, words, P::Number, value.proposal);
    kind(b, value.kind, words);
    number(b, words, P::DecisionBy, value.by);
    at(b, words, P::DecisionAt, value.at);
    label(
        b,
        words,
        match value.choice {
            OwnerProposalChoice::Accepted => P::AcceptChoice,
            OwnerProposalChoice::Rejected => P::RejectChoice,
            OwnerProposalChoice::Passed => P::Pass,
            OwnerProposalChoice::Withdrawn => P::Withdrawn,
            OwnerProposalChoice::Stale => P::Stale,
        },
    );
    match value.created {
        Some((first, last)) => {
            number(b, words, P::FirstCreated, first);
            number(b, words, P::LastCreated, last);
        }
        None => label(b, words, P::CreatedAbsent),
    }
}
fn task(b: &mut Builder, client: &Borrowed<'_>, value: &Task, words: &Words, bindings: &mut JigBindings) {
    number(b, words, P::Project, u64::from(value.project));
    crate::cards::chip::chip(b, client, &value.chip, words, bindings);
    crate::sections::task::overview(b, client, value, words, bindings);
    crate::sections::task::budget(b, value, words, bindings, client.limits.rows);
}
fn outer(b: &mut Builder, value: &OwnerHistorical, words: &Words) {
    match value.at {
        Some(time) => at(b, words, P::At, time),
        None => label(b, words, P::AtAbsent),
    }
    match value.by {
        Some(by) => {
            label(b, words, P::By);
            crate::owner_result::requester(b, by, words);
        }
        None => label(b, words, P::ByAbsent),
    }
}
/// Render each genuine source separately. No current card controls are added.
pub fn render(
    b: &mut Builder,
    client: &Borrowed<'_>,
    value: &OwnerHistorical,
    words: &Words,
    bindings: &mut JigBindings,
) {
    region(b, words, P::Historical);
    outer(b, value, words);
    match &value.value {
        OwnerHistoricalValue::Task { document, result } => {
            label(b, words, P::Task);
            task(b, client, document, words, bindings);
            crate::cards::result::facts(b, client, result, words, bindings);
        }
        OwnerHistoricalValue::Proposal(value) => {
            label(b, words, P::TaskProposal);
            crate::owner_history::proposal(b, value, words, bindings, client.limits.rows);
        }
        OwnerHistoricalValue::PersonProposal { proposal, decision } => {
            label(b, words, P::PersonProposal);
            number(b, words, P::Number, proposal.number);
            number(b, words, P::Proposer, proposal.proposer);
            number(b, words, P::Project, u64::from(proposal.project));
            goal(b, &proposal.goal, words, bindings, client.limits.rows);
            match &proposal.state {
                OwnerPersonProposalState::Pending { since } => {
                    label(b, words, P::Pending);
                    at(b, words, P::Since, *since);
                }
                OwnerPersonProposalState::Accepted { by } => {
                    label(b, words, P::Accepted);
                    crate::owner_result::requester(b, *by, words);
                }
                OwnerPersonProposalState::Rejected { by, reason, message, at: time } => {
                    label(b, words, P::Rejected);
                    crate::owner_result::requester(b, *by, words);
                    raw(b, reason);
                    number(b, words, P::Message, *message);
                    at(b, words, P::DecisionAt, *time);
                }
            }
            match decision {
                Some(value) => proposal_decision(b, value, words),
                None => label(b, words, P::DecisionAbsent),
            }
        }
        OwnerHistoricalValue::Escalation { task: document, decision, hold } => {
            label(b, words, P::Escalation);
            task(b, client, document, words, bindings);
            number(b, words, P::Project, u64::from(decision.project));
            number(b, words, P::Requester, decision.requester);
            number(b, words, P::Number, decision.task);
            number(b, words, P::Revision, decision.revision);
            number(b, words, P::DecisionBy, decision.by);
            at(b, words, P::DecisionAt, decision.at);
            match &decision.decision {
                OwnerEscalationDecision::Release => label(b, words, P::Release),
                OwnerEscalationDecision::Pass => label(b, words, P::Pass),
                OwnerEscalationDecision::Reject { reason } => {
                    label(b, words, P::Rejected);
                    raw(b, reason);
                }
            }
            match hold {
                Some(value) => crate::owner_milestone::render(b, value, client, words, bindings),
                None => label(b, words, P::HoldAbsent),
            }
        }
        OwnerHistoricalValue::Question { question, answer, ending } => {
            label(b, words, P::Question);
            crate::cards::chip::chip(b, client, &question.task, words, bindings);
            number(b, words, P::Message, question.message);
            if let Some(by) = question.from {
                crate::owner_result::requester(b, by, words);
            }
            at(b, words, P::Since, question.since);
            raw(b, &question.words);
            if let Some(how) = &question.ending {
                words.ending(b, how);
            }
            match answer {
                Some(value) => crate::owner_activity::render(b, value, words),
                None => label(b, words, P::AnswerAbsent),
            }
            match ending {
                Some(value) => crate::cards::result::facts(b, client, value, words, bindings),
                None => label(b, words, P::EndingAbsent),
            }
        }
        OwnerHistoricalValue::PersonTask { document, result } => {
            label(b, words, P::PersonTask);
            crate::owner_person_task::render(b, client, document, words, bindings);
            crate::cards::result::facts(b, client, result, words, bindings);
        }
    }
    b.close();
}
/// Rich departure facts take precedence over later compact/body refreshes.
pub(crate) fn card(
    b: &mut Builder,
    client: &Borrowed<'_>,
    object: Id<crate::objects::Object>,
    place: &crate::cards::Place,
    words: &Words,
    bindings: &mut JigBindings,
) -> bool {
    use crate::objects::{Body, Card, Left};
    let Some(fact) = client.objects().get(object) else {
        return false;
    };
    let value = match &fact.card {
        Card::Leaving { how: Left::Historical { facts }, .. } => Some(facts.as_ref()),
        Card::Open
        | Card::Deciding { .. }
        | Card::NotSent { .. }
        | Card::Refused { .. }
        | Card::Unknown { .. }
        | Card::Leaving {
            how: Left::Looking | Left::Answered { .. } | Left::Ended { .. } | Left::OwnerEnded { .. } | Left::Unknown,
            ..
        } => match &fact.body {
            Some(Body::Historical(value)) => Some(value.as_ref()),
            Some(
                Body::Chip(_)
                | Body::Task(_)
                | Body::Proposal(_)
                | Body::OwnerProposal(_)
                | Body::Escalation(_)
                | Body::OwnerEscalation(_)
                | Body::Question(_)
                | Body::PersonTask(_)
                | Body::OwnerPersonTask(_)
                | Body::Result(_)
                | Body::Note { .. },
            )
            | None => None,
        },
    };
    let Some(value) = value else {
        return false;
    };
    if !crate::cards::open(b, client, object, place, &saying(words, P::Historical, &[]), words, bindings) {
        return false;
    }
    render(b, client, value, words, bindings);
    crate::cards::close(b);
    true
}

/// Checked envelope for each independent source and retained outer context.
/// The Task helper's eight typed working-set presence families are already
/// bounded by the released escalation envelope. Historical embedded sources
/// remain additional allocations and additional displayed fragments.
#[must_use]
pub fn max_nodes(l: &Limits) -> Option<u32> {
    let result = crate::sizes::result_facts(l)?;
    let task =
        crate::sizes::owner_escalation_nodes(l)?.checked_add(result)?.checked_add(result.checked_mul(l.rows)?)?;
    let escalation = task.checked_add(crate::owner_milestone::max_nodes(l)?)?.checked_add(12)?;
    let proposal = crate::owner_history::proposal_max_nodes(l)?;
    let person_goal = crate::specification::nodes(l)?
        .checked_add(crate::proposed_task::contract_nodes(l)?)?
        .checked_add(crate::owner_authority::document_nodes(l.rows, l.segments)?)?
        .checked_add(crate::owner_wake::policy_nodes())?
        .checked_add(l.rows.checked_mul(l.segments.checked_add(5)?)?)?
        .checked_add(40)?;
    let question = crate::chip_nodes(l)?
        .checked_add(skein_ui::markdown::max_nodes(l.text)?)?
        .checked_add(result.checked_mul(2)?)?
        .checked_add(l.rows.checked_mul(4)?)?
        .checked_add(120)?;
    let person = crate::owner_person_task::max_nodes(l)?.checked_add(result)?;
    task.max(escalation).max(proposal).max(person_goal).max(question).max(person).checked_add(12)
}
#[must_use]
pub fn worst_case(l: &Limits, words: &Words) -> Option<HistoricalFragment> {
    let nodes = max_nodes(l)?;
    let text_per_node = words.jig_capacity()?.max(crate::literal_segment::max_bytes(l.text)?).checked_mul(2)?;
    Some(HistoricalFragment {
        nodes,
        text_bytes: nodes.checked_mul(text_per_node)?,
        text_per_node,
        depth: l.rows.checked_add(40)?,
        bindings: nodes,
        slots: nodes,
    })
}
fn spec_fits(value: &Specification, l: &Limits) -> Option<()> {
    if !value.words.fits(l.text) || value.parameters.capacity() > l.rows || value.inputs.capacity() > l.rows {
        return None;
    }
    for parameter in &value.parameters {
        match parameter {
            Parameter::Bytes { value, .. } => {
                if !value.fits(l.text) {
                    return None;
                }
            }
            Parameter::Number { .. } | Parameter::Resource { .. } => {}
        }
    }
    Some(())
}
fn chip_fits(value: &Chip, l: &Limits, words: &Words) -> Option<()> {
    if !value.title.bytes.fits(l.text) {
        return None;
    }
    match value.details.as_ref() {
        ChipPhase::Owner { details, .. } => {
            crate::owner_chip::required(details, l, words)?;
        }
        ChipPhase::Active(_)
        | ChipPhase::Waiting { .. }
        | ChipPhase::Idle { .. }
        | ChipPhase::Starting
        | ChipPhase::Running { .. }
        | ChipPhase::WaitingForWords { .. }
        | ChipPhase::BackingOff { .. }
        | ChipPhase::Procedure { .. }
        | ChipPhase::Person { .. }
        | ChipPhase::Closing(_)
        | ChipPhase::Held { .. }
        | ChipPhase::Ended { .. } => {}
    }
    Some(())
}
fn result_fits(value: &Result, l: &Limits, words: &Words) -> Option<()> {
    chip_fits(&value.task, l, words)?;
    if let Some(details) = &value.details {
        contract_fits(&details.contract, l)?;
        if details.delegates.capacity() > l.rows {
            return None;
        }
        for link in &details.delegates {
            link_fits(link, l, words)?;
        }
        if let Some(links) = &details.followups {
            if links.capacity() > l.rows {
                return None;
            }
            for link in links {
                link_fits(link, l, words)?;
            }
        }
    }
    if let Some(value) = &value.value {
        crate::owner_chip::result_nodes(value, l)?;
    }
    Some(())
}
fn task_fits(value: &Task, l: &Limits, words: &Words) -> Option<()> {
    chip_fits(&value.chip, l, words)?;
    crate::owner_amendment::authority_fits(&value.authority, l)?;
    if value.dependencies.capacity() > l.rows
        || value.references.capacity() > l.rows
        || value.inputs.capacity() > l.rows
        || value.resources.capacity() > l.rows
        || value.subscriptions.entries.capacity() > l.rows
    {
        return None;
    }
    let spec = &value.spec.specification;
    if !spec.words.bytes.fits(l.text) || spec.parameters.capacity() > l.rows || spec.inputs.capacity() > l.rows {
        return None;
    }
    for parameter in &spec.parameters {
        match parameter {
            ParameterDocument::Bytes { value, .. } => {
                if !value.bytes.fits(l.text) {
                    return None;
                }
            }
            ParameterDocument::Number { .. } | ParameterDocument::Resource { .. } => {}
        }
    }
    contract_fits(&value.result_contract, l)?;
    for link in &value.dependencies {
        link_fits(link, l, words)?;
    }
    for link in &value.references {
        link_fits(link, l, words)?;
    }
    for link in &value.inputs {
        link_fits(link, l, words)?;
    }
    if let Some(value) = &value.instructions
        && !value.bytes.fits(l.text)
    {
        return None;
    }
    for resource in &value.resources {
        match resource {
            TaskResource::Owner(value) => crate::owner_chip::part_fits(&value.part, l)?,
            TaskResource::Part(_) => {}
        }
    }
    match &value.result {
        Some(TaskResult::Document(value)) => result_fits(value, l, words)?,
        Some(TaskResult::Ending(_)) | None => {}
    }
    Some(())
}
/// Receiving validation refuses whole values, preserving every nested allocation.
#[must_use]
pub fn required(value: &OwnerHistorical, l: &Limits, words: &Words) -> Option<HistoricalFragment> {
    if !value.matches(value.key) {
        return None;
    }
    match &value.value {
        OwnerHistoricalValue::Task { document, result } => {
            task_fits(document, l, words)?;
            result_fits(result, l, words)?;
        }
        OwnerHistoricalValue::Proposal(value) => {
            crate::owner_history::proposal_required(value, l, words)?;
        }
        OwnerHistoricalValue::PersonProposal { proposal, .. } => {
            let goal = &proposal.goal;
            spec_fits(&goal.spec, l)?;
            contract_fits(&goal.contract, l)?;
            let authority = &goal.authority;
            for grant in &authority.grants {
                pattern_fits(&grant.pattern, l)?;
            }
            for resource in &authority.note_resources {
                pattern_fits(&resource.pattern, l)?;
            }
            if goal.dependencies.capacity() > l.rows || goal.holdings.capacity() > l.rows {
                return None;
            }
            for holding in &goal.holdings {
                let path = match holding {
                    OwnerGoalHolding::Write { path, .. } | OwnerGoalHolding::Slot { path, .. } => path,
                };
                if path.capacity() > l.segments {
                    return None;
                }
                for segment in path {
                    if !segment.fits(l.text) {
                        return None;
                    }
                }
            }
            match &proposal.state {
                OwnerPersonProposalState::Rejected { reason, .. } => {
                    if !reason.fits(l.text) {
                        return None;
                    }
                }
                OwnerPersonProposalState::Pending { .. } | OwnerPersonProposalState::Accepted { .. } => {}
            }
        }
        OwnerHistoricalValue::Escalation { task, decision, hold } => {
            task_fits(task, l, words)?;
            match &decision.decision {
                OwnerEscalationDecision::Reject { reason } => {
                    if !reason.fits(l.text) {
                        return None;
                    }
                }
                OwnerEscalationDecision::Release | OwnerEscalationDecision::Pass => {}
            }
            if let Some(value) = hold {
                crate::owner_milestone::required(value, l, words)?;
            }
        }
        OwnerHistoricalValue::Question { question, answer, ending } => {
            chip_fits(&question.task, l, words)?;
            if !question.words.fits(l.text) {
                return None;
            }
            if let Some(value) = answer {
                crate::owner_activity::required(value, l, words)?;
            }
            if let Some(value) = ending {
                result_fits(value, l, words)?;
            }
        }
        OwnerHistoricalValue::PersonTask { document, result } => {
            crate::owner_person_task::required(document, l, words)?;
            result_fits(result, l, words)?;
        }
    }
    worst_case(l, words)
}
fn link_fits(value: &OwnerTaskLink, l: &Limits, words: &Words) -> Option<()> {
    match value {
        OwnerTaskLink::Live(chip) | OwnerTaskLink::Retained { chip, .. } => chip_fits(chip, l, words),
        OwnerTaskLink::Stored { .. } => Some(()),
    }
}
fn contract_fits(value: &TaskContractDocument, l: &Limits) -> Option<()> {
    match value {
        TaskContractDocument::Approval { state, part } => {
            if u32::try_from(state.len()).ok()? > l.text {
                return None;
            }
            crate::owner_chip::part_fits(part, l)?;
        }
        TaskContractDocument::Verdict { choices, .. } => {
            if choices.capacity() > l.rows {
                return None;
            }
            for choice in choices {
                if choice.fields.capacity() > l.rows {
                    return None;
                }
                for field in &choice.fields {
                    if u32::try_from(field.name.len()).ok()? > l.text {
                        return None;
                    }
                }
            }
        }
        TaskContractDocument::Report { .. } | TaskContractDocument::Change { .. } => {}
    }
    Some(())
}
fn pattern_fits(value: &OwnerPattern, l: &Limits) -> Option<()> {
    if value.segments.capacity() > l.segments {
        return None;
    }
    for segment in &value.segments {
        if u32::try_from(segment.len()).ok()? > l.text {
            return None;
        }
    }
    let last = match &value.last {
        OwnerPatternEnd::Exact(bytes) | OwnerPatternEnd::Open(bytes) => bytes,
    };
    if u32::try_from(last.len()).ok()? > l.text {
        return None;
    }
    Some(())
}
