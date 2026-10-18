//! Say the facts a person decides on, in the application's checked phrasebook.
use super::{DecisionWord as P, Word};
use crate::{
    contract::{Effect, Funder, ReleaseEffect},
    facts::DecisionFacts,
};
use alloc::boxed::Box;
use skein_lib::List;
use skein_ui::phrases::{Hole, Phrasebook};
/// Say each required decision fact without choosing a presentation tree.
#[must_use]
pub fn decision(value: &DecisionFacts, book: &Phrasebook, language: u16) -> List<Box<[u8]>> {
    let mut out = List::with_capacity(value.funding.len().saturating_mul(6).saturating_add(14));
    if value.dangerous {
        push(&mut out, say(book, language, P::Dangerous, &[]));
    }
    if let Some(effect) = value.effect {
        effect_words(effect, book, language, &mut out);
    }
    for offer in &value.funding {
        push(&mut out, funder_words(offer.source, book, language));
        if offer.left.is_none() {
            push(&mut out, say(book, language, P::LeftAbsent, &[]));
        }
        if offer.after.is_none() {
            push(&mut out, say(book, language, P::AfterAbsent, &[]));
        }
        for (phrase, number) in [(P::Needed, Some(offer.needed)), (P::Left, offer.left), (P::AfterFunding, offer.after)]
        {
            if let Some(number) = number {
                push(&mut out, say(book, language, phrase, &[Hole::Number(number)]));
            }
        }
    }
    if let Some(funds) = value.funds {
        push(
            &mut out,
            say(
                book,
                language,
                P::Funds,
                &[Hole::Amount(funds.needed), Hole::Amount(funds.left), Hole::Amount(funds.after)],
            ),
        );
    }
    out
}
fn say(book: &Phrasebook, language: u16, phrase: P, holes: &[Hole]) -> Box<[u8]> {
    book.say(language, Word::Decision(phrase).id(), holes).expect("checked decision phrase").text
}
fn push(out: &mut List<Box<[u8]>>, text: Box<[u8]>) {
    out.push(text).expect("bounded decision facts");
}
fn effect_words(effect: Effect, book: &Phrasebook, language: u16, out: &mut List<Box<[u8]>>) {
    match effect {
        Effect::Release(effect) => push(
            out,
            say(
                book,
                language,
                match effect {
                    ReleaseEffect::Judged => P::ReleaseFresh,
                    ReleaseEffect::RunAgain => P::ResumeRelease,
                    ReleaseEffect::Uncertain => P::EngineFollowsChoice,
                },
                &[],
            ),
        ),
        Effect::Move { reservation } => push(out, say(book, language, P::TakeoverFunds, &[Hole::Amount(reservation)])),
        Effect::Cancel(impact) => {
            for (phrase, number) in [
                (P::TasksWaiting, impact.tasks.waiting),
                (P::TasksRunning, impact.tasks.running),
                (P::TasksHeld, impact.tasks.held),
                (P::TasksClosing, impact.tasks.closing),
                (P::TasksDone, impact.tasks.done),
                (P::TasksFailed, impact.tasks.failed),
                (P::TasksCancelled, impact.tasks.cancelled),
                (P::LiveRuns, impact.live_runs),
                (P::PendingProposals, impact.proposals),
            ] {
                push(out, say(book, language, phrase, &[Hole::Number(u64::from(number))]));
            }
            if impact.tasks.ended > 0 {
                push(out, say(book, language, P::TasksEnded, &[Hole::Number(u64::from(impact.tasks.ended))]));
            }
            push(out, say(book, language, P::EffectsSettle, &[]));
        }
    }
}
fn funder_words(source: Funder, book: &Phrasebook, language: u16) -> Box<[u8]> {
    match source {
        Funder::Task(task) => say(book, language, P::FunderTask, &[Hole::Number(task)]),
        Funder::Pool { project, person, period } => say(
            book,
            language,
            P::FunderPool,
            &[Hole::Number(u64::from(project)), Hole::Number(person), Hole::Number(period)],
        ),
        Funder::Recurring { project, task, period } => say(
            book,
            language,
            P::FunderRecurring,
            &[Hole::Number(u64::from(project)), Hole::Number(task), Hole::Number(period)],
        ),
        Funder::Period { project, period } => {
            say(book, language, P::FunderPeriod, &[Hole::Number(u64::from(project)), Hole::Number(period)])
        }
    }
}
