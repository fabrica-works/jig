//! The one named review dialog, from recalled facts (view.md, 18.5; objects.md, 2).
//! Adoption belongs to the application slot rather than this jig form (client/domain/pages.md, 6.5).
use crate::cards::{common, person, proposal};
use crate::phrases::jig::JigPhrase;
use crate::words::Words;
use crate::{JigBindings, JigMeaning, JigStyle};
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run, State};

use crate::{
    Borrowed,
    action::Act,
    confirm::{Confirmation, Missing},
    fields::Purpose,
    objects::{Body, Object},
};
use jig_client::contract::{
    ActKind, Effect, Hold, HoldReason, Impact, ObjectKey, OwnerChipHold, ReleaseEffect, TaskHold,
};

pub mod amendment;
/// Build the domain's one object or goal confirmation, or nothing when closed.
pub fn confirmation(b: &mut Builder, client: &Borrowed<'_>, words: &Words, bindings: &mut JigBindings) {
    let text_limit = bindings.text_limit();
    if let Some(open) = client.confirmation() {
        let title = match open.form {
            crate::confirm::Form::Act => common::act_phrase(open.act),
            crate::confirm::Form::CloseChat => JigPhrase::ActClose,
        };
        begin(b, words, title);
        if let Some(object) = client.objects().get(open.object) {
            context(b, client, &object, words, bindings);
            action(b, client, &object, open, words, bindings);
        }
        if open.proposed.is_some() {
            words.text(b, JigPhrase::WillPropose, &[]);
            if let Some(to) = &open.proposed {
                common::recipient(b, to, words);
            }
        }
        if let Some(missing) = &open.missing {
            missing_words(b, missing, words, text_limit);
        }
        if open.changed {
            words.status(b, JigPhrase::ChangedConfirmation, &[]);
        }
        end(b, words, bindings, open.dangerous, open.missing.is_none() && !open.changed);
    } else if let Some(open) = &client.project_page().form.confirmation {
        begin(b, words, JigPhrase::ActSetGoal);
        words.text(b, JigPhrase::WillGoal, &[]);
        skein_ui::markdown::read(open.words.bytes(), 2, b);
        words.text(b, JigPhrase::ContextProject, &[Hole::Number(u64::from(open.form.project))]);
        words.goal_executor(b, open.form.executor);
        words.text(b, JigPhrase::Priority, &[Hole::Number(u64::from(open.form.priority))]);
        words.text(b, JigPhrase::Budget, &[Hole::Amount(open.form.budget)]);
        if let Some(deadline) = open.form.deadline {
            words.deadline(b, deadline);
        }
        match open.pool_left.checked_sub(open.form.budget) {
            Some(after) => words.text(
                b,
                JigPhrase::Funds,
                &[Hole::Amount(open.form.budget), Hole::Amount(open.pool_left), Hole::Amount(after)],
            ),
            None => {
                words.text(
                    b,
                    JigPhrase::BudgetMissing,
                    &[Hole::Amount(open.pool_left), Hole::Amount(open.form.budget)],
                );
            }
        }
        if open.proposed {
            words.text(b, JigPhrase::WillPropose, &[]);
        }
        if let Some(to) = &open.to {
            common::recipient(b, to, words);
        }
        end(b, words, bindings, false, true);
    }
}
fn begin(b: &mut Builder, words: &Words, act: JigPhrase) {
    b.open(Role::Dialog);
    b.region(crate::regions::JigRegion::Confirmation.region());
    b.name(&words.say(JigPhrase::Confirmation, &[]));
    words.text(b, act, &[]);
}
fn end(b: &mut Builder, words: &Words, bindings: &mut JigBindings, dangerous: bool, complete: bool) {
    if dangerous {
        words.status(b, JigPhrase::Dangerous, &[]);
    }
    b.open(Role::Button);
    let label = words.say(JigPhrase::Confirm, &[]);
    b.name(&label);
    b.style(JigStyle::ButtonWeight.token());
    if dangerous {
        b.style(JigStyle::Danger.token());
    }
    if complete {
        b.bind(bindings.insert(JigMeaning::Action(Act::Confirm)));
    } else {
        b.state(State::Disabled);
    }
    b.close();
    crate::frame::control(b, words, bindings, Role::Button, JigPhrase::Dismiss, &[], Act::Dismiss);
    b.close();
}
fn context(b: &mut Builder, client: &Borrowed<'_>, object: &Object, words: &Words, bindings: &mut JigBindings) {
    match &object.body {
        Some(Body::Historical(_)) => b.text(
            Run::Plain,
            &words
                .book
                .say(words.language, crate::owner_historical_words::HistoricalPhrase::Historical.id(), &[])
                .expect("checked passive historical caption")
                .text,
        ),
        Some(Body::Task(task)) => crate::cards::chip::chip(b, client, &task.chip, words, bindings),
        Some(Body::Chip(chip)) => crate::cards::chip::chip(b, client, chip, words, bindings),
        Some(Body::Escalation(doc)) => crate::cards::chip::chip(b, client, &doc.task, words, bindings),
        Some(Body::OwnerEscalation(doc)) => crate::cards::chip::chip(b, client, &doc.task.chip, words, bindings),
        Some(Body::Question(doc)) => crate::cards::chip::chip(b, client, &doc.task, words, bindings),
        Some(Body::PersonTask(doc)) => crate::cards::chip::chip(b, client, &doc.task, words, bindings),
        Some(Body::OwnerPersonTask(doc)) => crate::cards::chip::chip(b, client, &doc.task, words, bindings),
        Some(Body::Result(doc)) => crate::cards::chip::chip(b, client, &doc.task, words, bindings),
        Some(Body::Proposal(doc)) => words.proposer(b, doc.proposer),
        Some(Body::OwnerProposal(doc)) => words.proposer(b, doc.key.proposer),
        Some(Body::Note { document, .. }) => b.text(Run::Plain, document.description.bytes()),
        None => words.status(b, JigPhrase::LoadingObject, &[]),
    }
}
/// Consequences are a whole act table, never inferred from authority or styles.
#[expect(clippy::too_many_lines, reason = "the closed act vocabulary has one exhaustive consequence table")]
fn action(
    b: &mut Builder,
    client: &Borrowed<'_>,
    object: &Object,
    open: &Confirmation,
    words: &Words,
    bindings: &mut JigBindings,
) {
    let text_limit = bindings.text_limit();
    if open.form == crate::confirm::Form::CloseChat {
        words.text(b, JigPhrase::WillClose, &[]);
        if let Some(impact) = impact(object) {
            impact_words(b, impact, words);
        }
        delegates(b, client, object, open, words, bindings);
        common::field(b, client, Purpose::Reason, JigPhrase::ReasonField, words, bindings);
        return;
    }
    match open.act {
        ActKind::Accept => {
            words.text(b, JigPhrase::WillAccept, &[]);
            match &object.body {
                Some(Body::OwnerProposal(doc)) => crate::owner_proposal::render(b, client, doc, words, bindings),
                Some(Body::Proposal(doc)) => {
                    proposal::action(b, &doc.action, words, bindings, client.limits.rows);
                    common::authority(b, &doc.needs, words, bindings);
                    common::funding(b, &doc.funding, doc.needs.budget, words);
                }
                Some(
                    Body::Task(_)
                    | Body::Historical(_)
                    | Body::Chip(_)
                    | Body::Escalation(_)
                    | Body::OwnerEscalation(_)
                    | Body::Question(_)
                    | Body::PersonTask(_)
                    | Body::OwnerPersonTask(_)
                    | Body::Result(_)
                    | Body::Note { .. },
                )
                | None => {}
            }
        }
        ActKind::Release => {
            release_object(b, client, object, words, bindings);
            if let Some(again) = open.choices.again {
                words.text(b, if again { JigPhrase::RepeatEffect } else { JigPhrase::SkipEffect }, &[]);
            }
        }
        ActKind::Amend => {
            words.text(b, if open.choices.release { JigPhrase::WillAmendRelease } else { JigPhrase::WillAmend }, &[]);
            amendment::form(b, client, object, open, words, bindings);
            common::field(b, client, Purpose::Reason, JigPhrase::ReasonField, words, bindings);
            if let Some(revision) = open.release_revision {
                words.text(b, JigPhrase::HoldRevision, &[Hole::Number(revision.value)]);
            }
            if open.choices.release {
                release_object(b, client, object, words, bindings);
            }
        }
        ActKind::Answer => {
            words.text(b, JigPhrase::WillAnswer, &[]);
            match &object.body {
                Some(Body::Question(doc)) => {
                    skein_ui::markdown::read(doc.words.bytes(), 2, b);
                    common::field(b, client, Purpose::Answer, JigPhrase::AnswerField, words, bindings);
                }
                Some(Body::PersonTask(doc)) => person::form(b, client, open.object, &doc.contract, words, bindings),
                Some(Body::OwnerPersonTask(doc)) => {
                    crate::owner_person_task::form(b, client, open.object, &doc.contract, words, bindings);
                }
                Some(
                    Body::Task(_)
                    | Body::Historical(_)
                    | Body::Chip(_)
                    | Body::Proposal(_)
                    | Body::OwnerProposal(_)
                    | Body::Escalation(_)
                    | Body::OwnerEscalation(_)
                    | Body::Result(_)
                    | Body::Note { .. },
                )
                | None => {}
            }
        }
        ActKind::Cancel => {
            words.text(b, JigPhrase::WillCancel, &[]);
            if let Some(offers) = crate::acts::offers(object)
                && let Some(Effect::Cancel(impact)) = jig_client::facts::effect(offers.as_slice(), open.act)
            {
                impact_words(b, impact, words);
            }
            common::field(b, client, Purpose::Reason, JigPhrase::ReasonField, words, bindings);
        }
        ActKind::Move => {
            words.text(b, JigPhrase::WillTakeOver, &[]);
            if let Some(offers) = crate::acts::offers(object)
                && let Some(Effect::Move { reservation }) = jig_client::facts::effect(offers.as_slice(), open.act)
            {
                words.text(b, JigPhrase::TakeoverFunds, &[Hole::Amount(*reservation)]);
            }
            common::field(b, client, Purpose::Reason, JigPhrase::ReasonField, words, bindings);
        }
        ActKind::Message => {
            words.text(b, JigPhrase::WillWrite, &[]);
            match object.key {
                ObjectKey::Task(task) => {
                    b.open(Role::Region);
                    b.region(crate::regions::JigRegion::Composer.region());
                    b.name(&words.say(JigPhrase::AnswerField, &[]));
                    common::field(b, client, Purpose::Composer { task }, JigPhrase::AnswerField, words, bindings);
                    b.close();
                }
                ObjectKey::Proposal(_)
                | ObjectKey::Escalation(_)
                | ObjectKey::PersonTask(_)
                | ObjectKey::Question { .. }
                | ObjectKey::Result(_)
                | ObjectKey::Note { .. } => {}
            }
            if held(object) {
                crate::frame::control(
                    b,
                    words,
                    bindings,
                    Role::Button,
                    JigPhrase::WriteAndRelease,
                    &[],
                    Act::WriteRelease { release: true },
                );
                crate::frame::control(
                    b,
                    words,
                    bindings,
                    Role::Button,
                    JigPhrase::WriteOnly,
                    &[],
                    Act::WriteRelease { release: false },
                );
                words.text(
                    b,
                    if open.choices.release { JigPhrase::WriteAndRelease } else { JigPhrase::WriteOnly },
                    &[],
                );
            }
        }
        ActKind::Reorder => {
            words.text(b, JigPhrase::WillPrioritise, &[]);
            b.open(Role::TextField);
            b.name(&words.say(JigPhrase::PriorityField, &[]));
            b.value(&[], open.writes);
            b.bind(bindings.insert(JigMeaning::Priority));
            b.close();
            if let Some(priority) = open.choices.priority {
                words.text(b, JigPhrase::Priority, &[Hole::Number(u64::from(priority))]);
            }
        }
        ActKind::EditNote => {
            words.text(b, JigPhrase::WillEditNote, &[]);
            if let Some(Body::Note { document, .. }) = &object.body {
                crate::document_text::loss(b, &document.description, words);
                crate::document_text::loss(b, &document.body, words);
                if document.cut {
                    words.text(b, JigPhrase::NoteReferencesCut, &[]);
                }
                if !(document.cut || document.description.cut || document.body.cut) {
                    common::field(b, client, Purpose::NoteDescription, JigPhrase::NoteDescription, words, bindings);
                    common::field(b, client, Purpose::NoteBody, JigPhrase::NoteBody, words, bindings);
                }
            }
        }
        ActKind::Reject | ActKind::LeaveHeld => {
            words.text(
                b,
                if open.act == ActKind::Reject { JigPhrase::WillReject } else { JigPhrase::WillLeaveHeld },
                &[],
            );
            common::field(b, client, Purpose::Reason, JigPhrase::ReasonField, words, bindings);
        }
        ActKind::Withdraw => words.text(b, JigPhrase::ActWithdraw, &[]),
        ActKind::PassUp => words.text(b, JigPhrase::WillPassUp, &[]),
        ActKind::Take => words.text(b, JigPhrase::WillTake, &[]),
        ActKind::HandBack => words.text(b, JigPhrase::WillHandBack, &[]),
        ActKind::Stop => words.text(b, JigPhrase::WillStop, &[]),
        ActKind::DeleteNote => words.text(b, JigPhrase::WillDeleteNote, &[]),
        ActKind::Seen
        | ActKind::Adopt
        | ActKind::ChangePolicy
        | ActKind::SetRoles
        | ActKind::MakeService
        | ActKind::SetPool => words.refusal(b, &jig_client::contract::Refusal::NotOffered, text_limit),
    }
}
fn held(object: &Object) -> bool {
    match &object.body {
        Some(Body::Task(doc)) => doc.hold.is_some(),
        Some(Body::OwnerEscalation(doc)) => doc.task.hold.is_some(),
        Some(Body::Escalation(_)) => true,
        Some(
            Body::Historical(_)
            | Body::Chip(_)
            | Body::Proposal(_)
            | Body::OwnerProposal(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => false,
    }
}
fn release_object(b: &mut Builder, client: &Borrowed<'_>, object: &Object, words: &Words, bindings: &mut JigBindings) {
    if let Some(offers) = crate::acts::offers(object)
        && let Some(Effect::Release(effect)) = jig_client::facts::effect(offers.as_slice(), ActKind::Release)
    {
        match effect {
            ReleaseEffect::Judged => words.text(b, JigPhrase::ReleaseFresh, &[]),
            ReleaseEffect::RunAgain => words.text(b, JigPhrase::ResumeRelease, &[]),
            ReleaseEffect::Uncertain => words.text(b, JigPhrase::EngineFollowsChoice, &[]),
        }
    }
    match &object.body {
        Some(Body::Task(doc)) => match &doc.hold {
            Some(TaskHold::Document(hold)) => release(b, client, hold, words, bindings),
            Some(TaskHold::Owner(hold)) => release_owner(b, hold, words, bindings),
            None => {}
        },
        Some(Body::Escalation(doc)) => release(b, client, &doc.hold, words, bindings),
        Some(Body::OwnerEscalation(doc)) => match &doc.task.hold {
            Some(TaskHold::Document(hold)) => release(b, client, hold, words, bindings),
            Some(TaskHold::Owner(hold)) => release_owner(b, hold, words, bindings),
            None => {}
        },
        Some(
            Body::Historical(_)
            | Body::Chip(_)
            | Body::Proposal(_)
            | Body::OwnerProposal(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => {}
    }
}
/// Release follows the supplied prior lifecycle and resets all failure counts
/// (tasks.md, 9); only an uncertain entry needs the person's repeat choice.
fn release_owner(b: &mut Builder, facts: &OwnerChipHold, words: &Words, bindings: &mut JigBindings) {
    crate::owner_chip::hold(b, facts, words, bindings);
    words.text(b, JigPhrase::OwnerHoldRelease, &[]);
    if let jig_client::contract::HoldCategory::Uncertain { .. } = facts.reason {
        words.text(b, JigPhrase::EngineFollowsChoice, &[]);
        crate::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            JigPhrase::RepeatEffect,
            &[],
            Act::Again { again: true },
        );
        crate::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            JigPhrase::SkipEffect,
            &[],
            Act::Again { again: false },
        );
    }
}
fn impact(object: &Object) -> Option<&Impact> {
    match &object.body {
        Some(Body::Task(doc)) => match &doc.context {
            Some(context) => Some(&context.impact),
            None => None,
        },
        Some(Body::Escalation(doc)) => Some(&doc.impact),
        Some(Body::OwnerEscalation(doc)) => Some(&doc.task.context.as_ref()?.impact),
        Some(
            Body::Historical(_)
            | Body::Chip(_)
            | Body::Proposal(_)
            | Body::OwnerProposal(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => None,
    }
}
/// Say each authoritative cancellation count and the reservation a takeover needs.
pub fn impact_words(b: &mut Builder, impact: &Impact, words: &Words) {
    words.text(b, JigPhrase::TasksWaiting, &[Hole::Number(u64::from(impact.tasks.waiting))]);
    words.text(b, JigPhrase::TasksRunning, &[Hole::Number(u64::from(impact.tasks.running))]);
    words.text(b, JigPhrase::TasksHeld, &[Hole::Number(u64::from(impact.tasks.held))]);
    words.text(b, JigPhrase::TasksClosing, &[Hole::Number(u64::from(impact.tasks.closing))]);
    if impact.tasks.ended > 0 {
        words.text(b, JigPhrase::TasksEnded, &[Hole::Number(u64::from(impact.tasks.ended))]);
    }
    words.text(b, JigPhrase::TasksDone, &[Hole::Number(u64::from(impact.tasks.done))]);
    words.text(b, JigPhrase::TasksFailed, &[Hole::Number(u64::from(impact.tasks.failed))]);
    words.text(b, JigPhrase::TasksCancelled, &[Hole::Number(u64::from(impact.tasks.cancelled))]);
    words.text(b, JigPhrase::LiveRuns, &[Hole::Number(u64::from(impact.live_runs))]);
    words.text(b, JigPhrase::PendingProposals, &[Hole::Number(u64::from(impact.proposals))]);
    words.text(b, JigPhrase::EffectsSettle, &[]);
}
fn release(b: &mut Builder, client: &Borrowed<'_>, facts: &Hold, words: &Words, bindings: &mut JigBindings) {
    match &facts.reason {
        HoldReason::Procedure { part } | HoldReason::Drift { part } | HoldReason::EffectFailed { part } => {
            common::part(b, part, words, bindings);
        }
        HoldReason::EffectUncertain { part } => {
            common::part(b, part, words, bindings);
            crate::frame::control(
                b,
                words,
                bindings,
                Role::Button,
                JigPhrase::RepeatEffect,
                &[],
                Act::Again { again: true },
            );
            crate::frame::control(
                b,
                words,
                bindings,
                Role::Button,
                JigPhrase::SkipEffect,
                &[],
                Act::Again { again: false },
            );
        }
        HoldReason::Tries { .. }
        | HoldReason::Stopped { .. }
        | HoldReason::Stalled { .. }
        | HoldReason::Dependency { .. }
        | HoldReason::Budget { .. }
        | HoldReason::Deadline { .. }
        | HoldReason::Resource { .. }
        | HoldReason::Pool { .. } => {}
    }
    if let Some(result) = &facts.result {
        crate::cards::result::value_nodes(b, client, result, words, bindings);
    }
}
fn delegates(
    b: &mut Builder,
    client: &Borrowed<'_>,
    object: &Object,
    open: &Confirmation,
    words: &Words,
    bindings: &mut JigBindings,
) {
    let doc = match &object.body {
        Some(Body::Task(doc)) => doc,
        Some(
            Body::Historical(_)
            | Body::Chip(_)
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
        | None => return,
    };
    words.text(b, JigPhrase::DelegateChoices, &[]);
    for delegate in &doc.delegates {
        crate::cards::chip::chip(b, client, &delegate.chip, words, bindings);
        crate::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            JigPhrase::DelegateCancel,
            &[Hole::Number(delegate.chip.task)],
            Act::Dispose { task: delegate.chip.task, take_over: false },
        );
        crate::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            JigPhrase::DelegateTake,
            &[Hole::Number(delegate.chip.task), Hole::Amount(delegate.takeover_budget)],
            Act::Dispose { task: delegate.chip.task, take_over: true },
        );
        for disposal in &open.choices.disposals {
            if disposal.task == delegate.chip.task {
                match disposal.fate {
                    jig_client::shapes::Fate::Cancel => {
                        words.text(b, JigPhrase::DelegateCancel, &[Hole::Number(disposal.task)]);
                    }
                    jig_client::shapes::Fate::Move => words.text(
                        b,
                        JigPhrase::DelegateTake,
                        &[Hole::Number(disposal.task), Hole::Amount(delegate.takeover_budget)],
                    ),
                }
            }
        }
    }
}
/// Every incomplete/changed state is semantic text; style never carries its meaning.
pub fn missing_words(b: &mut Builder, missing: &Missing, words: &Words, text_limit: u32) {
    b.open(Role::Status);
    b.name(&words.say(JigPhrase::Confirmation, &[]));
    b.style(JigStyle::Warning.token());
    match missing {
        Missing::Words { field: _ } => words.text(b, JigPhrase::MissingWords, &[]),
        Missing::Choice => words.text(b, JigPhrase::MissingChoice, &[]),
        Missing::Again => words.text(b, JigPhrase::MissingAgain, &[]),
        Missing::Change => words.text(b, JigPhrase::MissingChange, &[]),
        Missing::Delegates => words.text(b, JigPhrase::MissingDelegates, &[]),
        Missing::Priority => words.text(b, JigPhrase::MissingPriority, &[]),
        Missing::Context => words.text(b, JigPhrase::MissingContext, &[]),
        Missing::Changed { now } => {
            words.text(b, JigPhrase::ChangedConfirmation, &[]);
            if let Some(now) = now {
                words.text(b, JigPhrase::Revision, &[Hole::Number(now.value)]);
            }
        }
        Missing::ChangedHold { now } => {
            words.text(b, JigPhrase::ChangedConfirmation, &[]);
            if let Some(now) = now {
                words.text(b, JigPhrase::HoldRevision, &[Hole::Number(now.value)]);
            }
        }
        Missing::Refused { why } => words.refusal(b, why, text_limit),
    }
    b.close();
}
