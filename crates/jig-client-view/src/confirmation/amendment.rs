//! Bounded typed amendment fields; no authority judgment (client.md, 9).
use crate::{
    Builder, JigBindings, JigMeaning, Role,
    cards::{common, proposal},
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use alloc::boxed::Box;
use jig_client::{
    Domain,
    action::Act,
    confirm::Confirmation,
    contract::{Amendment, Text},
    objects::{Body, Object},
};
/// The one property replaced by an input's whole value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Words,
    Instructions,
    Budget,
    Deadline,
    Party,
    Task,
    Role { project: u32 },
}
/// Decode one field against the exact amendment snapshot that named its token.
#[must_use]
pub fn edit(before: &Amendment, field: Field, value: Box<[u8]>, text_limit: u32) -> Option<Act> {
    let mut change = before.clone();
    match field {
        Field::Words => change.spec.as_mut()?.words = Text::new(value, text_limit).ok()?,
        Field::Instructions => change.instructions = Some(Text::new(value, text_limit).ok()?),
        Field::Budget => change.authority.as_mut()?.budget = crate::number(&value)?,
        Field::Deadline => change.authority.as_mut()?.deadline = Some(deadline(&value)?),
        Field::Party => change.addressed = Some(jig_client::contract::Recipient::Party(crate::number(&value)?)),
        Field::Task => change.addressed = Some(jig_client::contract::Recipient::Task(crate::number(&value)?)),
        Field::Role { project } => {
            change.addressed = Some(jig_client::contract::Recipient::Role {
                project,
                role: u16::try_from(crate::number(&value)?).ok()?,
            });
        }
    }
    Some(Act::Change { change: Box::new(change) })
}
/// ISO calendar minute in UTC, checked against the wall clock's representable range.
#[must_use]
pub fn deadline(value: &[u8]) -> Option<skein_lib::Wall> {
    if value.len() != 16
        || *value.get(4)? != b'-'
        || *value.get(7)? != b'-'
        || *value.get(10)? != b' '
        || *value.get(13)? != b':'
    {
        return None;
    }
    let year = crate::number(value.get(0..4)?)?;
    let month = crate::number(value.get(5..7)?)?;
    let day = crate::number(value.get(8..10)?)?;
    let hour = crate::number(value.get(11..13)?)?;
    let minute = crate::number(value.get(14..16)?)?;
    if year < 1970 || !(1..=12).contains(&month) || hour > 23 || minute > 59 {
        return None;
    }
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let months = [31_u64, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let index = usize::try_from(month.checked_sub(1)?).ok()?;
    if day == 0 || day > *months.get(index)? {
        return None;
    }
    let previous = year.checked_sub(1)?;
    let leaps =
        previous.checked_div(4)?.checked_sub(previous.checked_div(100)?)?.checked_add(previous.checked_div(400)?)?;
    let mut days = year.checked_sub(1970)?.checked_mul(365)?.checked_add(leaps.checked_sub(477)?)?;
    for days_in_month in months.get(..index)? {
        days = days.checked_add(*days_in_month)?;
    }
    days = days.checked_add(day.checked_sub(1)?)?;
    let seconds =
        days.checked_mul(86400)?.checked_add(hour.checked_mul(3600)?)?.checked_add(minute.checked_mul(60)?)?;
    Some(skein_lib::Wall::from_nanos(seconds.checked_mul(1_000_000_000)?))
}
/// Show all before facts and form fields; each input preserves the other selected changes.
pub fn form(
    b: &mut Builder,
    _client: &Domain,
    object: &Object,
    open: &Confirmation,
    words: &Words,
    bindings: &mut JigBindings,
) {
    let Some(before) = before(object) else {
        match &object.body {
            Some(Body::Task(doc)) if doc.spec.cut => words.text(b, JigPhrase::SpecificationCut, &[]),
            Some(Body::Escalation(doc)) if doc.steering.spec.cut => words.text(b, JigPhrase::SpecificationCut, &[]),
            _ => {}
        }
        return;
    };
    words.text(b, JigPhrase::Before, &[]);
    proposal::amendment(b, &before, words, bindings);
    let change = open.choices.change.as_ref().unwrap_or(&before);
    words.text(b, JigPhrase::After, &[]);
    proposal::amendment(b, change, words, bindings);
    input(b, change, Field::Words, JigPhrase::NewSpec, open.writes, words, bindings);
    input(b, change, Field::Instructions, JigPhrase::NewInstructions, open.writes, words, bindings);
    input(b, change, Field::Budget, JigPhrase::NewBudget, open.writes, words, bindings);
    input(b, change, Field::Deadline, JigPhrase::NewDeadline, open.writes, words, bindings);
    if let Some(to) = &change.addressed {
        common::recipient(b, to, words);
        input(b, change, Field::Party, JigPhrase::AddressParty, open.writes, words, bindings);
        input(b, change, Field::Task, JigPhrase::AddressTask, open.writes, words, bindings);
        if let Some(project) = object.project {
            input(b, change, Field::Role { project }, JigPhrase::AddressRole, open.writes, words, bindings);
        }
    }
    if let Some(dependencies) = &change.dependencies {
        for task in dependencies {
            let mut without = skein_lib::List::with_capacity(dependencies.capacity());
            for other in dependencies {
                if other != task {
                    without.push(*other).expect("filtered dependencies fit");
                }
            }
            let mut next = change.clone();
            next.dependencies = Some(without);
            crate::frame::control(
                b,
                words,
                bindings,
                Role::Button,
                JigPhrase::DropDependency,
                &[Hole::Number(*task)],
                Act::Change { change: Box::new(next) },
            );
        }
    }
    if let Some(wake) = &change.wake {
        let mut next = change.clone();
        next.wake.as_mut().expect("wake present").words = !wake.words;
        crate::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            JigPhrase::WakeInbox,
            &[],
            Act::Change { change: Box::new(next) },
        );
        let mut next = change.clone();
        next.wake.as_mut().expect("wake present").results = !wake.results;
        crate::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            JigPhrase::WakeResult,
            &[],
            Act::Change { change: Box::new(next) },
        );
    }
    // Funding is kept with the document and may change independently of its spec revision.
    let funding = match &object.body {
        Some(Body::Task(doc)) => Some(&doc.budget),
        Some(Body::Escalation(doc)) => Some(&doc.steering.budget),
        Some(
            Body::Chip(_)
            | Body::Proposal(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => None,
    };
    if let Some(funding) = funding {
        words.actor(b, funding.funder);
    }
}
fn numbers(chips: &skein_lib::List<jig_client::contract::Chip>) -> skein_lib::List<u64> {
    let mut numbers = skein_lib::List::with_capacity(chips.capacity());
    for chip in chips {
        numbers.push(chip.task).expect("one number per chip fits");
    }
    numbers
}
fn input(
    b: &mut Builder,
    change: &Amendment,
    field: Field,
    phrase: JigPhrase,
    written: u64,
    words: &Words,
    bindings: &mut JigBindings,
) {
    b.open(match field {
        Field::Words | Field::Instructions => Role::TextArea,
        Field::Budget | Field::Deadline | Field::Party | Field::Task | Field::Role { .. } => Role::TextField,
    });
    b.name(&words.say(phrase, &[]));
    match field {
        Field::Words => {
            if let Some(value) = &change.spec {
                b.value(value.words.bytes(), written);
            }
        }
        Field::Instructions => {
            if let Some(value) = &change.instructions {
                b.value(value.bytes(), written);
            }
        }
        Field::Budget => {
            if let Some(authority) = &change.authority {
                b.value(&words.say(JigPhrase::Amount, &[Hole::Amount(authority.budget)]), written);
            }
        }
        Field::Deadline | Field::Party | Field::Task | Field::Role { .. } => b.value(&[], written),
    }
    b.bind(bindings.insert(JigMeaning::Amendment { change: Box::new(change.clone()), field }));
    b.close();
}

fn before(object: &Object) -> Option<Amendment> {
    match &object.body {
        Some(Body::Task(doc)) if doc.spec.cut => return None,
        Some(Body::Escalation(doc)) if doc.steering.spec.cut => return None,
        _ => {}
    }
    Some(match &object.body {
        Some(Body::Task(doc)) => Amendment {
            addressed: match &*doc.chip.details {
                jig_client::contract::ChipPhase::Person { addressed, .. } => Some(addressed.clone()),
                jig_client::contract::ChipPhase::Waiting { .. }
                | jig_client::contract::ChipPhase::Idle { .. }
                | jig_client::contract::ChipPhase::Starting
                | jig_client::contract::ChipPhase::Running { .. }
                | jig_client::contract::ChipPhase::WaitingForWords { .. }
                | jig_client::contract::ChipPhase::BackingOff { .. }
                | jig_client::contract::ChipPhase::Procedure { .. }
                | jig_client::contract::ChipPhase::Closing(_)
                | jig_client::contract::ChipPhase::Held { .. }
                | jig_client::contract::ChipPhase::Ended { .. } => None,
            },
            spec: Some(doc.spec.specification.clone()),
            instructions: Some(doc.instructions.clone()),
            wake: Some(doc.wake.clone()),
            dependencies: Some(numbers(&doc.dependencies)),
            authority: Some(doc.authority.clone()),
        },
        Some(Body::Escalation(doc)) => Amendment {
            addressed: doc.steering.addressed.clone(),
            spec: Some(doc.steering.spec.specification.clone()),
            instructions: Some(doc.steering.instructions.clone()),
            wake: Some(doc.steering.wake.clone()),
            dependencies: Some(doc.steering.dependencies.clone()),
            authority: Some(doc.steering.authority.clone()),
        },
        Some(
            Body::Chip(_)
            | Body::Proposal(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => return None,
    })
}
