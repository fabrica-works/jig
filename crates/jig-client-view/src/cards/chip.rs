//! Live task identity and phase words (client/domain/objects.md, section 3).
//! Every chip carries whole bounded phase facts. Dependency chips are visited
//! once through a bounded work list; neither the carrier nor traversal is recursive.
use crate::{
    Builder, JigBindings, Role,
    phrases::{Hole, jig::JigPhrase},
    words::{Words, name},
};
use jig_client::{
    Domain,
    action::Act,
    address::Address,
    contract::{Chip, ChipPhase, FailureClass},
    objects::Body,
};

/// The task's named link and copied title, with its executor kind in words.
pub fn head(b: &mut Builder, chip: &Chip, words: &Words, bindings: &mut JigBindings) {
    crate::frame::control(
        b,
        words,
        bindings,
        Role::Link,
        JigPhrase::ChipNumber,
        &[Hole::Number(chip.task), name(chip.title.bytes())],
        Act::Go { address: Address::task(chip.task) },
    );
    if chip.title.cut {
        words.text(b, JigPhrase::ChipTitleCut, &[]);
    }
    match chip.details.as_ref() {
        ChipPhase::Owner { details, .. } => crate::proposed_task::executor(b, details.executor, words),
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
        | ChipPhase::Ended { .. } => words.executor(b, chip.executor),
    }
}
/// Build all kept dependency chips with a bounded work list, never recursive traversal.
pub fn chip(b: &mut Builder, client: &Domain, chip: &Chip, words: &Words, bindings: &mut JigBindings) {
    let count = client.objects().len();
    let mut pending = skein_lib::Queue::with_capacity(count);
    let mut seen = skein_lib::Set::with_capacity(count.checked_add(1).expect("one external root chip fits"));
    seen.insert(chip.task).expect("root fits");
    region(b, client, chip, words, bindings, &mut pending, &mut seen);
    for _ in 0..count {
        let Some(task) = pending.pop() else {
            break;
        };
        if let Some(chip) = kept(client, task) {
            region(b, client, chip, words, bindings, &mut pending, &mut seen);
        }
    }
}
fn kept(client: &Domain, task: u64) -> Option<&Chip> {
    let object = client.objects().get(client.objects().find(jig_client::contract::ObjectKey::Task(task))?)?;
    match &object.body {
        Some(Body::Chip(chip)) => Some(chip),
        Some(Body::Task(task)) => Some(&task.chip),
        Some(
            Body::Historical(_)
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
    }
}
fn region(
    b: &mut Builder,
    client: &Domain,
    chip: &Chip,
    words: &Words,
    bindings: &mut JigBindings,
    pending: &mut skein_lib::Queue<u64>,
    seen: &mut skein_lib::Set<u64>,
) {
    b.open(Role::Region);
    b.style(crate::JigStyle::Chip.token());
    b.name(&words.say(JigPhrase::ChipNumber, &[Hole::Number(chip.task), name(chip.title.bytes())]));
    head(b, chip, words, bindings);
    details(b, client, &chip.details, words, bindings, pending, seen);
    b.close();
}
/// Failure classes have one whole table so a new class must acquire words.
pub fn failure(b: &mut Builder, class: FailureClass, words: &Words) {
    let phrase = match class {
        FailureClass::HostLost => JigPhrase::HostLost,
        FailureClass::Temporary => JigPhrase::Temporary,
        FailureClass::Permanent => JigPhrase::Permanent,
        FailureClass::Stall => JigPhrase::Stall,
    };
    words.text(b, phrase, &[]);
}
/// Detailed task runtime, with application phase explanations left as parts.
pub fn details(
    b: &mut Builder,
    client: &Domain,
    phase: &ChipPhase,
    words: &Words,
    bindings: &mut JigBindings,
    pending: &mut skein_lib::Queue<u64>,
    seen: &mut skein_lib::Set<u64>,
) {
    match phase {
        ChipPhase::Owner { phase, details: _ } => {
            crate::chip_phase::brief(b, phase, words);
            if let jig_client::contract::OwnerChipPhase::Waiting { dependencies, .. } = phase {
                dependencies_list(b, client, dependencies, words, bindings, pending, seen);
            }
        }
        ChipPhase::Active(value) => crate::runtime::active(b, value, words),
        ChipPhase::Waiting { dependencies, hold } => {
            words.text(b, JigPhrase::Waiting, &[]);
            dependencies_list(b, client, dependencies, words, bindings, pending, seen);
            if let Some(hold) = hold {
                super::common::part(b, hold, words, bindings);
            }
        }
        ChipPhase::Idle { wake: policy } => {
            words.text(b, JigPhrase::Idle, &[]);
            wake(b, policy, words, bindings);
        }
        ChipPhase::Starting => words.text(b, JigPhrase::TaskStarting, &[]),
        ChipPhase::Running { attempt, since, tool } => {
            words.text(b, JigPhrase::Running, &[]);
            words.text(b, JigPhrase::Attempt, &[Hole::Number(*attempt)]);
            words.text(
                b,
                JigPhrase::ElapsedSpan,
                &[Hole::Span(skein_lib::Duration::from_nanos(
                    client.wall().as_nanos().saturating_sub(since.as_nanos()),
                ))],
            );
            match tool {
                Some(tool) => words.text(b, JigPhrase::Tool, &[name(tool.bytes())]),
                None => words.text(b, JigPhrase::Writing, &[]),
            }
        }
        ChipPhase::WaitingForWords { attempt, since } => {
            words.text(b, JigPhrase::WaitingWords, &[]);
            words.text(b, JigPhrase::Attempt, &[Hole::Number(*attempt)]);
            words.text(
                b,
                JigPhrase::ElapsedSpan,
                &[Hole::Span(skein_lib::Duration::from_nanos(
                    client.wall().as_nanos().saturating_sub(since.as_nanos()),
                ))],
            );
        }
        ChipPhase::BackingOff { at, attempt, tries, class, reason } => {
            words.text(
                b,
                JigPhrase::RetryAt,
                &[Hole::Wall(words.adjusted_wall(*at)), Hole::Number(*attempt), Hole::Number(u64::from(*tries))],
            );
            failure(b, *class, words);
            words.text(b, JigPhrase::Reason, &[name(reason.bytes())]);
        }
        ChipPhase::Procedure { panel } => super::common::part(b, panel, words, bindings),
        ChipPhase::Person { addressed, taken, since } => {
            super::common::recipient(b, addressed, words);
            if let Some(by) = taken {
                super::common::taken_by(b, by, words);
            }
            if let Some(since) = since {
                words.text(b, JigPhrase::Since, &[Hole::Wall(words.adjusted_wall(*since))]);
            }
        }
        ChipPhase::Closing(closing) => super::common::closing(b, closing, words, bindings),
        ChipPhase::Held { reason, since, revision: _, was } => {
            if let Some(was) = was {
                crate::runtime::prior(b, was, words, bindings);
            }
            super::hold::reason(b, reason, words, bindings);
            words.text(b, JigPhrase::Since, &[Hole::Wall(words.adjusted_wall(*since))]);
        }
        ChipPhase::Ended { how, at } => {
            words.ending(b, how);
            words.wall(b, *at);
        }
    }
}
/// Show the wake policy verbatim; which news is enabled comes from the document.
pub fn wake(b: &mut Builder, wake: &jig_client::contract::WakePolicy, words: &Words, bindings: &mut JigBindings) {
    if wake.words {
        words.text(b, JigPhrase::WakeInbox, &[]);
    }
    if wake.results {
        words.text(b, JigPhrase::WakeResult, &[]);
    }
    for topic in &wake.topics {
        words.text(b, JigPhrase::WakeTopic, &[]);
        words.text(b, JigPhrase::Topic, &[Hole::Number(u64::from(*topic))]);
    }
    for part in &wake.subscriptions {
        super::common::part(b, part, words, bindings);
    }
}

fn dependencies_list(
    b: &mut Builder,
    client: &Domain,
    dependencies: &skein_lib::List<u64>,
    words: &Words,
    bindings: &mut JigBindings,
    pending: &mut skein_lib::Queue<u64>,
    seen: &mut skein_lib::Set<u64>,
) {
    for dependency in dependencies {
        crate::frame::control(
            b,
            words,
            bindings,
            Role::Link,
            JigPhrase::TaskPage,
            &[Hole::Number(*dependency)],
            Act::Go { address: Address::task(*dependency) },
        );
        if seen.contains(dependency) {
            // The named chip has already been shown or queued.
        } else if kept(client, *dependency).is_some() {
            seen.insert(*dependency).expect("only kept objects are added");
            pending.push(*dependency);
        } else {
            words.status(b, JigPhrase::LoadingDependency, &[Hole::Number(*dependency)]);
        }
    }
}
