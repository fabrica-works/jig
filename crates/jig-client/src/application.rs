//! Jig objects shown by an application's page remain jig's working-set objects.
//! One window names them; their chips and deciding documents remain live through
//! jig's watches and reads. No application value is held here (root.md, section 7).
use crate::{
    Domain, Request,
    contract::{self, Delivery, ObjectKey, Part, Read, ReadResult, ReadValue, Subject},
    objects::{Body, Source},
    watches::Owner,
};
use skein_lib::{Env, List, Queue};
pub(crate) const WINDOW: u16 = u16::MAX;
fn source(domain: &Domain) -> Source {
    Source::Window { generation: domain.generation, number: WINDOW }
}
pub(crate) fn show(domain: &mut Domain, env: &Env<crate::Limits>, objects: List<ObjectKey>, out: &mut Queue<Request>) {
    if domain.confirmed.is_none() || domain.link != crate::link::Link::Live {
        return;
    }
    let source = source(domain);
    let mut unchanged = true;
    let mut count = 0_u32;
    for (key, _) in domain.objects.ids() {
        if domain.objects.named(*key, source) {
            count = count.checked_add(1).expect("named objects fit their capacity");
            if !objects.as_slice().contains(key) {
                unchanged = false;
            }
        }
    }
    if unchanged && count == objects.len() {
        return;
    }
    close(domain, out);
    let missed = domain.objects.replace(source, objects.as_slice());
    if missed > 0 {
        crate::pages::routing::refuse(domain, contract::Limit::Objects);
    }
    let mut tasks = List::with_capacity(env.limits.rows);
    let mut deciding = false;
    for key in &objects {
        if !domain.objects.named(*key, source) {
            continue;
        }
        read(domain, *key, out);
        match key {
            ObjectKey::Task(task) => {
                if tasks.push(*task).is_err() {
                    crate::pages::routing::refuse(domain, contract::Limit::Objects);
                }
            }
            ObjectKey::Proposal(_)
            | ObjectKey::Escalation(_)
            | ObjectKey::Question { .. }
            | ObjectKey::PersonTask(_)
            | ObjectKey::Result(_)
            | ObjectKey::Note { .. } => deciding = true,
        }
    }
    if !tasks.is_empty() {
        open(domain, env, Subject::Chips { tasks }, out);
    }
    if deciding {
        open(domain, env, Subject::Inbox { window: env.limits.rows }, out);
    }
    domain.changed();
}
fn open(domain: &mut Domain, env: &Env<crate::Limits>, subject: Subject, out: &mut Queue<Request>) {
    if domain.open(env, subject, Owner::Application { generation: domain.generation }, out).is_none() {
        crate::pages::routing::refuse(domain, contract::Limit::Watches);
    }
}
fn close(domain: &mut Domain, out: &mut Queue<Request>) {
    for (id, ()) in &domain.watch_ids {
        let watch = domain.watches.get_mut(*id).expect("registered watch exists");
        match watch.owner {
            Owner::Application { .. } => crate::domain::close_watch(watch, *id, &mut domain.timers, out),
            Owner::Page { .. } | Owner::Frame => {}
        }
    }
}
pub(crate) fn left(domain: &mut Domain, out: &mut Queue<Request>) {
    close(domain, out);
    let missed = domain.objects.replace(source(domain), &[]);
    assert!(missed == 0, "empty source cannot refuse");
    domain.changed();
}
fn read(domain: &mut Domain, key: ObjectKey, out: &mut Queue<Request>) {
    let what = crate::pages::routing::document_read(key);
    if !domain.reads.pending(&what) && domain.reads.ask(domain.generation, source(domain), what, out).is_none() {
        crate::pages::routing::refuse(domain, contract::Limit::Reads);
    }
}
pub(crate) fn delivery(domain: &mut Domain, env: &Env<crate::Limits>, delivery: Delivery, out: &mut Queue<Request>) {
    match delivery {
        Delivery::Chips(chips) => {
            for chip in chips.into_boxed() {
                let key = ObjectKey::Task(chip.task);
                if domain.objects.named(key, source(domain)) {
                    put(domain, env, key, Body::Chip(chip), out);
                }
            }
        }
        Delivery::Inbox(_) => {
            let mut keys = List::with_capacity(env.limits.objects);
            for (key, _) in domain.objects.ids() {
                assert!(keys.push(*key).is_ok(), "objects fit their registry");
            }
            for key in &keys {
                if domain.objects.named(*key, source(domain)) {
                    read(domain, *key, out);
                }
            }
        }
        Delivery::Frame(_)
        | Delivery::Task(_)
        | Delivery::Tree(_)
        | Delivery::Goals(_)
        | Delivery::Tasks(_)
        | Delivery::Conversations(_)
        | Delivery::Run(_)
        | Delivery::RunState(_)
        | Delivery::Deployment(_) => {}
    }
}
pub(crate) fn read_result(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    what: Read,
    result: ReadResult,
    out: &mut Queue<Request>,
) {
    let Some(key) = key(&what) else {
        return;
    };
    if !domain.objects.named(key, source(domain)) {
        return;
    }
    match result {
        ReadResult::None(why) => match why {
            contract::ReadNone::SignedOut => crate::domain::signed_out(domain, out),
            contract::ReadNone::Unknown
            | contract::ReadNone::Gone
            | contract::ReadNone::Refused
            | contract::ReadNone::Busy
            | contract::ReadNone::Unreachable => crate::pages::routing::refuse(domain, contract::Limit::Reads),
        },
        ReadResult::Value(value) => match value {
            ReadValue::Task(document) => put(domain, env, key, Body::Task(document), out),
            ReadValue::Proposal(document) => put(domain, env, key, Body::Proposal(document), out),
            ReadValue::Escalation(document) => put(domain, env, key, Body::Escalation(document), out),
            ReadValue::Question(document) => put(domain, env, key, Body::Question(document), out),
            ReadValue::PersonTask(document) => put(domain, env, key, Body::PersonTask(document), out),
            ReadValue::Result(document) => put(domain, env, key, Body::Result(document), out),
            ReadValue::Frame(_)
            | ReadValue::Ended(_)
            | ReadValue::History(_)
            | ReadValue::Activity(_)
            | ReadValue::Inbox(_)
            | ReadValue::Conversations(_)
            | ReadValue::Goals(_)
            | ReadValue::Spend(_)
            | ReadValue::Project(_)
            | ReadValue::Notes(_)
            | ReadValue::Deployment(_) => {}
        },
    }
}
fn key(what: &Read) -> Option<ObjectKey> {
    match what {
        Read::Task { task } => Some(ObjectKey::Task(*task)),
        Read::Proposal { proposal } => Some(ObjectKey::Proposal(*proposal)),
        Read::Escalation { task } => Some(ObjectKey::Escalation(*task)),
        Read::Question { task, message } => Some(ObjectKey::Question { task: *task, message: *message }),
        Read::PersonTask { task } => Some(ObjectKey::PersonTask(*task)),
        Read::Result { task } => Some(ObjectKey::Result(*task)),
        Read::Frame
        | Read::Ended { .. }
        | Read::History { .. }
        | Read::Activity { .. }
        | Read::Inbox { .. }
        | Read::Conversations { .. }
        | Read::Goals { .. }
        | Read::Spend { .. }
        | Read::Project { .. }
        | Read::Notes { .. }
        | Read::Deployment => None,
    }
}
fn put(domain: &mut Domain, env: &Env<crate::Limits>, key: ObjectKey, body: Body, out: &mut Queue<Request>) {
    let parts = parts(&body, env.limits.parts);
    if parts.full {
        crate::pages::routing::refuse(domain, contract::Limit::Objects);
    }
    let project = match &body {
        Body::Task(task) => Some(task.project),
        Body::Proposal(proposal) => Some(proposal.project),
        Body::Note { project, .. } => Some(*project),
        Body::Chip(_) | Body::Escalation(_) | Body::Question(_) | Body::PersonTask(_) | Body::Result(_) => {
            object_project(domain, key)
        }
    };
    if domain.objects.put(key, body) {
        for part in parts.values.into_boxed() {
            if let Some(project) = project {
                let scope = owner_scope(&part, contract::PartScope::Project(project));
                out.push(Request::ShowPart { part, generation: domain.generation, scope });
            } else if let Some(scope) = scope(domain, &part) {
                out.push(Request::ShowPart { part, generation: domain.generation, scope });
            }
        }

        domain.changed();
    }
}
struct Parts {
    values: List<Part>,
    full: bool,
}
fn parts(body: &Body, limit: u32) -> Parts {
    let mut parts = Parts { values: List::with_capacity(limit), full: false };
    match body {
        Body::Chip(chip) => chip_parts(&mut parts, chip),
        Body::Task(task) => {
            chip_parts(&mut parts, &task.chip);
            append(&mut parts, task.resources.as_slice());
            append(&mut parts, task.parts.as_slice());
        }
        Body::Proposal(proposal) => match &proposal.action {
            contract::ProposalAction::Effect { part } => push(&mut parts, part.clone()),
            contract::ProposalAction::Batch { .. }
            | contract::ProposalAction::Widen { .. }
            | contract::ProposalAction::Amend { .. } => {}
        },
        Body::Escalation(escalation) => {
            chip_parts(&mut parts, &escalation.task);
            hold_parts(&mut parts, &escalation.hold.reason);
        }
        Body::Question(question) => chip_parts(&mut parts, &question.task),
        Body::PersonTask(person) => chip_parts(&mut parts, &person.task),
        Body::Result(result) => {
            chip_parts(&mut parts, &result.task);
            match &result.value {
                Some(contract::ResultValue::Connector { part, .. }) => push(&mut parts, part.clone()),
                Some(
                    contract::ResultValue::Report(_)
                    | contract::ResultValue::Verdict { .. }
                    | contract::ResultValue::Change { .. }
                    | contract::ResultValue::Choice { .. }
                    | contract::ResultValue::Approved { .. }
                    | contract::ResultValue::Failure(_)
                    | contract::ResultValue::Words(_),
                )
                | None => {}
            }
        }
        Body::Note { .. } => {}
    }
    parts
}
fn chip_parts(parts: &mut Parts, chip: &contract::Chip) {
    match chip.details.as_ref() {
        contract::ChipPhase::Waiting { hold, .. } => {
            if let Some(part) = hold {
                push(parts, part.clone());
            }
        }
        contract::ChipPhase::Procedure { panel } => push(parts, panel.clone()),
        contract::ChipPhase::Held { reason, was, .. } => {
            hold_parts(parts, reason);
            if let Some(contract::HeldRuntime::Closing(closing)) = was {
                closing_parts(parts, closing);
            }
        }
        contract::ChipPhase::Closing(closing) => closing_parts(parts, closing),
        contract::ChipPhase::Active(_)
        | contract::ChipPhase::Idle { .. }
        | contract::ChipPhase::Starting
        | contract::ChipPhase::Running { .. }
        | contract::ChipPhase::WaitingForWords { .. }
        | contract::ChipPhase::BackingOff { .. }
        | contract::ChipPhase::Person { .. }
        | contract::ChipPhase::Ended { .. } => {}
    }
}
fn closing_parts(parts: &mut Parts, closing: &contract::Closing) {
    if let Some(connectors) = &closing.connectors {
        for connector in connectors {
            push(parts, Part::Connector { connector: *connector });
        }
    }
}
fn hold_parts(parts: &mut Parts, reason: &contract::HoldReason) {
    match reason {
        contract::HoldReason::Drift { part }
        | contract::HoldReason::EffectFailed { part }
        | contract::HoldReason::EffectUncertain { part }
        | contract::HoldReason::Pool { part }
        | contract::HoldReason::Procedure { part } => push(parts, part.clone()),
        contract::HoldReason::Resource { resource, .. } => push(parts, resource.clone()),
        contract::HoldReason::Tries { .. }
        | contract::HoldReason::Stalled { .. }
        | contract::HoldReason::Dependency { .. }
        | contract::HoldReason::Budget { .. }
        | contract::HoldReason::Deadline { .. }
        | contract::HoldReason::Stopped { .. } => {}
    }
}
fn push(parts: &mut Parts, part: Part) {
    if parts.values.push(part).is_err() {
        parts.full = true;
    }
}
fn append(parts: &mut Parts, values: &[Part]) {
    for part in values {
        push(parts, part.clone());
    }
}

fn object_project(domain: &Domain, key: ObjectKey) -> Option<u32> {
    domain.objects.get(domain.objects.find(key)?)?.project
}
/// An outbox reference names its owner; other parts retain their enclosing document scope.
pub(crate) fn owner_scope(part: &Part, enclosing: contract::PartScope) -> contract::PartScope {
    match part {
        Part::Outbox { scope, .. } => *scope,
        Part::Connector { .. }
        | Part::Resource { .. }
        | Part::Kind { .. }
        | Part::Panel { .. }
        | Part::Result { .. }
        | Part::Effect { .. }
        | Part::Hold { .. }
        | Part::Project { .. }
        | Part::Deployment { .. } => enclosing,
    }
}
#[expect(clippy::manual_map, reason = "step code matches optional values explicitly")]
pub(crate) fn scope(domain: &Domain, part: &Part) -> Option<contract::PartScope> {
    match part {
        Part::Outbox { scope, .. } => Some(*scope),
        Part::Project { project, .. } => Some(contract::PartScope::Project(*project)),
        Part::Deployment { .. } | Part::Connector { .. } | Part::Resource { .. } | Part::Kind { .. } => {
            Some(contract::PartScope::Deployment)
        }
        Part::Panel { task, .. } | Part::Result { task, .. } | Part::Hold { task, .. } => {
            let project = match object_project(domain, ObjectKey::Task(*task)) {
                Some(project) => Some(project),
                None => current_project(domain),
            };
            match project {
                Some(project) => Some(contract::PartScope::Project(project)),
                None => None,
            }
        }
        Part::Effect { proposal, .. } => {
            let project = match object_project(domain, ObjectKey::Proposal(*proposal)) {
                Some(project) => Some(project),
                None => current_project(domain),
            };
            match project {
                Some(project) => Some(contract::PartScope::Project(project)),
                None => None,
            }
        }
    }
}
fn current_project(domain: &Domain) -> Option<u32> {
    match domain.address {
        crate::address::Address::Project { project, .. } => Some(project),
        crate::address::Address::Task { task, .. } => object_project(domain, ObjectKey::Task(task)),
        crate::address::Address::Inbox { project } | crate::address::Address::Conversations { project } => project,
        crate::address::Address::Deployment => None,
    }
}

pub(crate) fn worst_case(limits: &crate::Limits) -> Option<u64> {
    List::<ObjectKey>::worst_case(limits.objects)?
        .checked_add(List::<Part>::worst_case(limits.parts)?)?
        .checked_add(u64::from(limits.parts).checked_mul(u64::from(limits.text))?)?
        .checked_add(List::<u64>::worst_case(limits.rows)?)
}
