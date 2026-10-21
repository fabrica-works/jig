//! Live task identity and phase words (client/domain/objects.md, section 3).
//! Every chip carries whole bounded phase facts. Dependency chips are visited
//! once through a bounded work list; neither the carrier nor traversal is recursive.
use crate::JigBindings;
use crate::phrases::jig::JigPhrase;
use crate::words::{Words, name};
use jig_client::contract::Phase;
use jig_client::words::{StatusWord, Word};
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role};

use crate::{Borrowed, action::Act, address::Address, objects::Body};
use jig_client::contract::{Chip, ChipPhase, FailureClass};

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
pub fn chip(b: &mut Builder, client: &Borrowed<'_>, chip: &Chip, words: &Words, bindings: &mut JigBindings) {
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
            region(b, client, &chip, words, bindings, &mut pending, &mut seen);
        }
    }
}
fn kept(client: &Borrowed<'_>, task: u64) -> Option<Chip> {
    let objects = client.objects();
    let object = objects.get(objects.find(jig_client::contract::ObjectKey::Task(task))?)?;
    match &object.body {
        Some(Body::Chip(chip)) => Some(chip.clone()),
        Some(Body::Task(task)) => Some(task.chip.clone()),
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
    client: &Borrowed<'_>,
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
    client: &Borrowed<'_>,
    phase: &ChipPhase,
    words: &Words,
    bindings: &mut JigBindings,
    pending: &mut skein_lib::Queue<u64>,
    seen: &mut skein_lib::Set<u64>,
) {
    match phase {
        ChipPhase::Owner { phase, details } => {
            crate::chip_phase::brief(b, phase, words);
            let mut kept = skein_lib::List::with_capacity(details.dependencies.len());
            for dependency in &details.dependencies {
                if dependency.keeps && dependency.pending {
                    dependency_words(b, dependency, words);
                    kept.push(dependency.number).expect("bounded kept dependencies");
                }
            }
            dependencies_list(b, client, &kept, words, bindings, pending, seen);
            if let Some(hold) = &details.hold
                && let Some(circle) = &hold.circle
            {
                words.text(b, JigPhrase::CircleHold, &[]);
                dependencies_list(b, client, &circle.tasks, words, bindings, pending, seen);
                words.text(b, JigPhrase::CircleLifts, &[]);
                if circle.cut {
                    words.text(b, JigPhrase::TaskListsCut, &[]);
                }
            }
            if let jig_client::contract::OwnerChipPhase::Waiting { dependencies, .. } = phase {
                dependencies_list(b, client, dependencies, words, bindings, pending, seen);
            }
        }
        ChipPhase::Active(value) => crate::runtime::active(b, value, words),
        ChipPhase::Waiting { dependencies, hold } => {
            words.text(b, JigPhrase::Word(Word::Phase(Phase::Waiting)), &[]);
            dependencies_list(b, client, dependencies, words, bindings, pending, seen);
            if let Some(hold) = hold {
                super::common::part(b, hold, words, bindings);
            }
        }
        ChipPhase::Idle { wake: policy } => {
            words.text(b, JigPhrase::Word(Word::Phase(Phase::Idle)), &[]);
            wake(b, policy, words, bindings);
        }
        ChipPhase::Starting => words.text(b, JigPhrase::Word(Word::Status(StatusWord::Starting)), &[]),
        ChipPhase::Running { attempt, since, tool } => {
            words.text(b, JigPhrase::Word(Word::Phase(Phase::Running)), &[]);
            words.text(b, JigPhrase::Attempt, &[Hole::Number(*attempt)]);
            words.text(
                b,
                JigPhrase::ElapsedSpan,
                &[Hole::Span(skein_lib::Duration::from_nanos(
                    client.wall().as_nanos().saturating_sub(since.as_nanos()),
                ))],
            );
            match tool {
                Some(tool) => words.text(b, JigPhrase::Word(Word::Status(StatusWord::Tool)), &[name(tool.bytes())]),
                None => words.text(b, JigPhrase::Word(Word::Status(StatusWord::Writing)), &[]),
            }
        }
        ChipPhase::WaitingForWords { attempt, since } => {
            words.text(b, JigPhrase::Word(Word::Status(StatusWord::WaitingForYou)), &[]);
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
    client: &Borrowed<'_>,
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

/// Dependency provenance uses the same words on a chip and its task page.
pub(crate) fn dependency_words(b: &mut Builder, dependency: &jig_client::contract::Dependency, words: &Words) {
    let phrase = match dependency.status {
        Some(jig_client::contract::OwnerTaskStatus::Failed) => JigPhrase::DependencyFailed,
        Some(jig_client::contract::OwnerTaskStatus::Cancelled) => JigPhrase::DependencyCancelled,
        None | Some(jig_client::contract::OwnerTaskStatus::Done) => {
            if dependency.keeps && dependency.pending {
                JigPhrase::KeptUntil
            } else if dependency.pending {
                JigPhrase::DependencyPending
            } else {
                JigPhrase::DependencyDone
            }
        }
    };
    words.text(b, phrase, &[Hole::Number(dependency.number)]);
    if let Some(added) = dependency.added {
        let (phrase, number) = match added.by {
            jig_client::contract::OwnerRequester::Task(task) => (JigPhrase::DependencyByTask, task),
            jig_client::contract::OwnerRequester::Person(party) => (JigPhrase::DependencyByParty, party),
            jig_client::contract::OwnerRequester::Deployment { project } => {
                (JigPhrase::DependencyByDeployment, u64::from(project))
            }
        };
        words.text(b, phrase, &[Hole::Number(number), Hole::Wall(words.adjusted_wall(added.at))]);
    }
}

/// Show a numbered task in a refusal without inventing a phase when its chip is absent.
pub(crate) fn named(b: &mut Builder, client: &Borrowed<'_>, number: u64, words: &Words, bindings: &mut JigBindings) {
    match kept(client, number) {
        Some(chip) => {
            head(b, &chip, words, bindings);
            if let ChipPhase::Owner { phase, .. } = chip.details.as_ref() {
                crate::chip_phase::brief(b, phase, words);
            }
        }
        None => crate::frame::control(
            b,
            words,
            bindings,
            Role::Link,
            JigPhrase::TaskPage,
            &[Hole::Number(number)],
            Act::Go { address: Address::task(number) },
        ),
    }
}
