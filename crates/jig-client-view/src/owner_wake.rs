//! Standalone exact wake fragments for owner document and editor consumers.
//! See client/domain/objects.md, 3 and 11; view.md, section 7.
use crate::owner_wake_words::WakePhrase;
use skein_ui::phrases::{Hole, Phrasebook};
use skein_ui::{Builder, Role, Run};

use jig_client_domain::contract::{OwnerWakePolicy, ResultsWake, SubscriptionKind, Subscriptions, WakeRule};

fn text(b: &mut Builder, book: &Phrasebook, language: u16, phrase: WakePhrase, holes: &[Hole]) {
    let saying = book.say(language, phrase.id(), holes).expect("wake phrases and typed holes were checked");
    b.text(Run::Plain, &saying.text);
}
fn region(b: &mut Builder, book: &Phrasebook, language: u16, phrase: WakePhrase) {
    let saying = book.say(language, phrase.id(), &[]).expect("wake region phrase was checked");
    b.open(Role::Region);
    b.name(&saying.text);
}
fn rule(b: &mut Builder, value: WakeRule, book: &Phrasebook, language: u16) {
    match value {
        WakeRule::Never => text(b, book, language, WakePhrase::Never, &[]),
        WakeRule::Immediate => text(b, book, language, WakePhrase::Immediate, &[]),
        WakeRule::Batch { count, age } => {
            text(b, book, language, WakePhrase::Batch, &[Hole::Number(u64::from(count)), Hole::Number(age.as_nanos())]);
        }
    }
}
/// Render every channel choice, including channels that do not wake.
pub fn policy(b: &mut Builder, value: &OwnerWakePolicy, book: &Phrasebook, language: u16) {
    region(b, book, language, WakePhrase::Policy);
    text(b, book, language, WakePhrase::Words, &[]);
    rule(b, value.words, book, language);
    text(b, book, language, WakePhrase::Notices, &[]);
    rule(b, value.notices, book, language);
    text(b, book, language, WakePhrase::News, &[]);
    rule(b, value.news, book, language);
    let results = match value.results {
        ResultsWake::Never => WakePhrase::ResultsNever,
        ResultsWake::Each => WakePhrase::ResultsEach,
        ResultsWake::LastOrFailure => WakePhrase::ResultsLastOrFailure,
    };
    text(b, book, language, results, &[]);
    let questions = if value.questions { WakePhrase::QuestionsYes } else { WakePhrase::QuestionsNo };
    text(b, book, language, questions, &[]);
    let answers = if value.answers { WakePhrase::AnswersYes } else { WakePhrase::AnswersNo };
    text(b, book, language, answers, &[]);
    let timers = if value.timers { WakePhrase::TimersYes } else { WakePhrase::TimersNo };
    text(b, book, language, timers, &[]);
    b.close();
}
/// Render separate numbered standing interests without interpreting topic identities.
pub fn subscriptions(b: &mut Builder, value: &Subscriptions, book: &Phrasebook, language: u16) {
    region(b, book, language, WakePhrase::Subscriptions);
    for entry in &value.entries {
        b.open(Role::Item);
        text(b, book, language, WakePhrase::Number, &[Hole::Number(entry.number)]);
        match entry.kind {
            SubscriptionKind::Task { target, held, result } => {
                text(b, book, language, WakePhrase::Task, &[Hole::Number(target)]);
                let held = if held { WakePhrase::HeldYes } else { WakePhrase::HeldNo };
                text(b, book, language, held, &[]);
                let result = if result { WakePhrase::ResultYes } else { WakePhrase::ResultNo };
                text(b, book, language, result, &[]);
            }
            SubscriptionKind::Timer { at, period } => {
                text(b, book, language, WakePhrase::Timer, &[Hole::Number(at.as_nanos())]);
                match period {
                    Some(period) => text(b, book, language, WakePhrase::Period, &[Hole::Number(period.as_nanos())]),
                    None => text(b, book, language, WakePhrase::Once, &[]),
                }
            }
            SubscriptionKind::Topic { connector, topic } => {
                text(b, book, language, WakePhrase::Topic, &[Hole::Number(u64::from(connector)), Hole::Number(topic)]);
            }
        }
        b.close();
    }
    b.close();
}
/// Policy region and ten text leaves.
#[must_use]
pub const fn policy_nodes() -> u32 {
    11
}
/// Region plus an item and at most four text leaves per subscription.
#[must_use]
pub fn subscription_nodes(rows: u32) -> Option<u32> {
    rows.checked_mul(5)?.checked_add(1)
}
/// Largest selected-language saying, including each region name.
#[must_use]
pub fn phrase_bytes(book: &Phrasebook) -> Option<u32> {
    let mut largest = 0;
    for phrase in crate::owner_wake_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    Some(largest)
}
