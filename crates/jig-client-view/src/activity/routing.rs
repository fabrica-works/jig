//! Routes task activity, stream cursors and page folds into the conversation.
//! Reads use the page generation's activity source; stale terminals stay outside
//! this module. The root only carries the resulting reads, names and parts.
//! See client/domain/client.md, 10, and contract.md, 6 and 7.3.
use crate::{Request, Ui, objects::Source};
use jig_client::contract::{ActivityPage, Executor, Limit, Read, Runtime, Task};
use jig_client::thread::window::End;
use skein_lib::{Env, Queue, Token};
/// The activity window binding, independent of a watch's slab token.
pub const ACTIVITY_WINDOW: u64 = 1;
/// The activity read source.
pub(crate) const ACTIVITY_READ: u16 = 1;
fn ask(domain: &mut Ui, what: Option<Read>, out: &mut Queue<Request>) {
    let Some(what) = what else {
        return;
    };
    if domain.handles.pending(&what) {
        return;
    }
    let source = Source::Window { number: ACTIVITY_READ };
    if domain.handles.read(source, what, out).is_none() {
        domain.conversation.failed();
        crate::page::routing::refuse(domain, Limit::Reads);
    }
}
/// A task document moves only the activity belonging to the current conversation.
pub(crate) fn document(domain: &mut Ui, task: &Task, out: &mut Queue<Request>, lent: &jig_client::Client) {
    if !domain.shows(
        crate::page::Section::Task { task: task.chip.task, section: crate::address::TaskSection::Activity },
        lent,
    ) {
        return;
    }
    if domain.conversation.task != task.chip.task {
        return;
    }
    match task.chip.executor {
        Executor::Agent { .. } => {}
        Executor::Procedure { .. } | Executor::Party { .. } | Executor::Role { .. } => return,
    }
    domain.conversation.run_finished = match &task.context {
        Some(context) => match &context.runtime {
            Runtime::Active(value) => active_finished(&value.stage),
            Runtime::Running { .. } | Runtime::WaitingForWords { .. } => false,
            Runtime::Closing(value) => closing_finished(value.stage),
            Runtime::Held { was } => match was {
                Some(jig_client::contract::HeldRuntime::Waiting) => true,
                Some(jig_client::contract::HeldRuntime::Active(value)) => active_finished(&value.stage),
                Some(jig_client::contract::HeldRuntime::Closing(value)) => closing_finished(value.stage),
                None => false,
            },
            Runtime::Waiting { .. }
            | Runtime::Idle
            | Runtime::Starting
            | Runtime::BackingOff { .. }
            | Runtime::Procedure { .. }
            | Runtime::Person { .. }
            | Runtime::Ended => true,
        },
        None => match task.chip.details.as_ref() {
            jig_client::contract::ChipPhase::Owner { phase, details } => owner_finished(phase, details.hold.as_ref()),
            jig_client::contract::ChipPhase::Waiting { .. }
            | jig_client::contract::ChipPhase::Idle { .. }
            | jig_client::contract::ChipPhase::Starting
            | jig_client::contract::ChipPhase::Active(_)
            | jig_client::contract::ChipPhase::Running { .. }
            | jig_client::contract::ChipPhase::WaitingForWords { .. }
            | jig_client::contract::ChipPhase::BackingOff { .. }
            | jig_client::contract::ChipPhase::Procedure { .. }
            | jig_client::contract::ChipPhase::Person { .. }
            | jig_client::contract::ChipPhase::Closing(_)
            | jig_client::contract::ChipPhase::Held { .. }
            | jig_client::contract::ChipPhase::Ended { .. } => false,
        },
    };
    let what = domain.conversation.moved(task.activity);
    ask(domain, what, out);
    finish(domain);
}
/// Takes an activity terminal and names its live cards and connector slots.
pub(crate) fn page(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    page: ActivityPage,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    if domain.conversation.reading.is_none() {
        domain.conversation.reading = Some(jig_client::thread::Reading::Latest { known: domain.conversation.loaded });
    }
    let next = domain.conversation.page(page);
    working_set(domain, env, out, lent);
    ask(domain, next, out);
    finish(domain);
    domain.changed();
}
fn working_set(domain: &mut Ui, env: &Env<crate::Limits>, out: &mut Queue<Request>, lent: &jig_client::Client) {
    let keys = domain.borrow(lent).conversation().names(env.limits.objects);
    let source = Source::Window { number: ACTIVITY_READ };
    crate::page::routing::names(domain, env, source, keys, out, lent);
    // Immutable turns still refer to mutable decisions. A new activity snapshot
    // refreshes those documents without deriving an outcome from the call text.
    let keys = domain.borrow(lent).conversation().names(env.limits.objects);
    for key in &keys {
        match key {
            jig_client::contract::ObjectKey::Task(_) | jig_client::contract::ObjectKey::Result(_) => {
                continue;
            }
            jig_client::contract::ObjectKey::Proposal(_)
            | jig_client::contract::ObjectKey::Escalation(_)
            | jig_client::contract::ObjectKey::Question { .. }
            | jig_client::contract::ObjectKey::PersonTask(_)
            | jig_client::contract::ObjectKey::Note { .. } => {}
        }
        let Some(id) = domain.objects.find(*key) else {
            continue;
        };
        let what = crate::objects::document_read(*key);
        if !domain.handles.pending(&what) && domain.handles.read(Source::Card(id), what, out).is_none() {
            crate::page::routing::refuse(domain, Limit::Reads);
        }
    }
    let parts = domain.borrow(lent).conversation().parts(env.limits.parts);
    crate::page::routing::parts(domain, env, parts.as_slice(), out, lent);
}
fn active_finished(stage: &jig_client::contract::RunStage) -> bool {
    match stage {
        jig_client::contract::RunStage::Claimed { .. } | jig_client::contract::RunStage::Running { .. } => false,
        jig_client::contract::RunStage::Idle
        | jig_client::contract::RunStage::Due
        | jig_client::contract::RunStage::Preparing
        | jig_client::contract::RunStage::BackingOff { .. } => true,
    }
}
// Absence of the joined closing stage is not evidence that the live run ended.
fn closing_finished(stage: Option<jig_client::contract::ClosingStage>) -> bool {
    match stage {
        None | Some(jig_client::contract::ClosingStage::Run { .. }) => false,
        Some(
            jig_client::contract::ClosingStage::Delegates
            | jig_client::contract::ClosingStage::Effects
            | jig_client::contract::ClosingStage::Releases
            | jig_client::contract::ClosingStage::Settled,
        ) => true,
    }
}
// Only the supplied lifecycle and prior stages attest whether the live run finished.
fn owner_finished(
    phase: &jig_client::contract::OwnerChipPhase,
    hold: Option<&jig_client::contract::OwnerChipHold>,
) -> bool {
    match phase {
        jig_client::contract::OwnerChipPhase::Active(stage) => active_finished(stage),
        jig_client::contract::OwnerChipPhase::Closing(stage) => closing_finished(Some(*stage)),
        jig_client::contract::OwnerChipPhase::Held { .. } => match hold {
            Some(hold) => match hold.before {
                jig_client::contract::OwnerPriorPhase::Waiting => true,
                jig_client::contract::OwnerPriorPhase::Active(stage) => active_finished(&stage),
                jig_client::contract::OwnerPriorPhase::Closing(stage) => closing_finished(Some(stage)),
            },
            None => false,
        },
        jig_client::contract::OwnerChipPhase::Waiting { .. }
        | jig_client::contract::OwnerChipPhase::Person { .. }
        | jig_client::contract::OwnerChipPhase::Ended => true,
    }
}
fn finish(domain: &mut Ui) {
    if domain.conversation.run_finished && domain.conversation.loaded >= domain.conversation.committed {
        domain.conversation.end_run();
    }
}
/// The client's text changed; only names, parts and local folds change here.
pub(crate) fn text(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    handle: jig_client::Handle,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    if !domain.handles.watches.contains_key(&handle) {
        return;
    }
    let run = domain.borrow(lent).conversation().current_run.cloned();
    let mut text = lent.text(handle).cloned();
    if let Some(text) = &mut text {
        let bounded = jig_client::thread::live_turn(text.clone(), domain.text_limit);
        text.text = bounded.text;
        text.cut = bounded.cut;
    }
    let run = match run {
        Some(run) => Some(run),
        None => match &text {
            Some(text) => Some(jig_client::contract::RunState {
                task: domain.conversation.task,
                attempt: Some(text.attempt),
                committed_turn: 0,
                writing: None,
                call: None,
            }),
            None => None,
        },
    };
    if let Some(run) = run {
        domain.conversation.run(&run, text.as_ref());
    }
    let projected = domain.borrow(lent).conversation();
    let live = projected.live;
    let run = projected.current_run.cloned();
    domain.conversation.prune_folds(live.as_slice(), run.as_ref());
    working_set(domain, env, out, lent);
    domain.changed();
}
/// The person's scroll uses the supplied opaque cursor at the requested end.
pub(crate) fn more(domain: &mut Ui, window: Token, end: End, out: &mut Queue<Request>) {
    if window != Token::new(ACTIVITY_WINDOW) {
        return;
    }
    let what = match end {
        End::Before => domain.conversation.older(),
        End::After => domain.conversation.newer(),
    };
    ask(domain, what, out);
}

/// A fold changes page presentation without changing committed call facts.
pub(crate) fn fold(domain: &mut Ui, name: jig_client::thread::CallName, open: bool, lent: &jig_client::Client) {
    let fold = if open { jig_client::thread::Fold::Open } else { jig_client::thread::Fold::Folded };
    let live = domain.borrow(lent).conversation().live;
    if domain.conversation.set_fold(name, fold, live.as_slice()) {
        domain.changed();
    }
}

/// Raw provider identities never match a committed transcript call number.
pub(crate) fn fold_current(
    domain: &mut Ui,
    identity: jig_client::contract::CallIdentity,
    open: bool,
    lent: &jig_client::Client,
) {
    let fold = if open { jig_client::thread::Fold::Open } else { jig_client::thread::Fold::Folded };
    let projected = domain.borrow(lent).conversation();
    let live = projected.live;
    let run = projected.current_run.cloned();
    if domain.conversation.set_current_fold(identity, fold, live.as_slice(), run.as_ref()) {
        domain.changed();
    }
}
