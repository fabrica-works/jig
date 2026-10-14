//! Shared page sections, counts and accounting (client/domain/pages.md, 5–7).
use crate::{
    Builder, JigBindings, Role, Run, State,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_domain_client::{
    action::Act,
    contract::{Budget, PhaseCounts},
};
pub(crate) fn section(b: &mut Builder, words: &Words, phrase: JigPhrase, region: crate::regions::JigRegion) {
    let title = words.say(phrase, &[]);
    b.open(Role::Region);
    b.region(region.region());
    b.name(&title);
    b.open(Role::Heading { level: 2 });
    b.name(&title);
    b.text(Run::Plain, &title);
    b.close();
}
pub(crate) fn tab(
    b: &mut Builder,
    words: &Words,
    bindings: &mut JigBindings,
    phrase: JigPhrase,
    action: Act,
    current: bool,
) {
    let label = words.say(phrase, &[]);
    b.open(Role::Tab);
    b.name(&label);
    b.text(Run::Plain, &label);
    if current {
        b.state(State::Current);
    }
    b.bind(bindings.insert(crate::JigMeaning::Action(action)));
    b.close();
}
pub(crate) fn control(
    b: &mut Builder,
    words: &Words,
    bindings: &mut JigBindings,
    phrase: JigPhrase,
    action: Act,
    selected: bool,
) {
    let label = words.say(phrase, &[]);
    b.open(Role::Button);
    b.name(&label);
    b.text(Run::Plain, &label);
    if selected {
        b.state(State::Selected);
    }
    b.bind(bindings.insert(crate::JigMeaning::Action(action)));
    b.close();
}
pub(crate) fn budget(b: &mut Builder, value: &Budget, words: &Words) {
    for (phrase, amount) in [
        (JigPhrase::TotalBudget, value.total),
        (JigPhrase::OwnBudget, value.own),
        (JigPhrase::EndedBudget, value.ended),
        (JigPhrase::Reserved, value.reserved),
        (JigPhrase::Left, value.left),
    ] {
        words.text(b, phrase, &[Hole::Amount(amount)]);
    }
    words.actor(b, value.funder);
}
pub(crate) fn counts(b: &mut Builder, counts: PhaseCounts, words: &Words) {
    for (phrase, count) in [
        (JigPhrase::TasksWaiting, counts.waiting),
        (JigPhrase::TasksRunning, counts.running),
        (JigPhrase::TasksHeld, counts.held),
        (JigPhrase::TasksClosing, counts.closing),
        (JigPhrase::TasksEnded, counts.ended),
        (JigPhrase::TasksDone, counts.done),
        (JigPhrase::TasksFailed, counts.failed),
        (JigPhrase::TasksCancelled, counts.cancelled),
    ] {
        if count > 0 {
            words.text(b, phrase, &[Hole::Number(u64::from(count))]);
        }
    }
}
pub(crate) fn cursors(
    b: &mut Builder,
    words: &Words,
    bindings: &mut JigBindings,
    before: Option<&jig_domain_client::contract::Cursor>,
    after: Option<&jig_domain_client::contract::Cursor>,
    window: u64,
) {
    use jig_domain_client::windows::{Edge, Edges};
    super::edges(
        b,
        words,
        bindings,
        &Edges {
            before: match before {
                Some(c) => Edge::More(c.clone()),
                None => Edge::End,
            },
            after: match after {
                Some(c) => Edge::More(c.clone()),
                None => Edge::End,
            },
        },
        window,
    );
}

// Supplied stage counts stay distinct from the locally folded PhaseCounts.
pub(crate) fn owner_counts(b: &mut Builder, counts: jig_domain_client::contract::OwnerPhaseCounts, words: &Words) {
    for (phrase, count) in [
        (JigPhrase::TasksWaiting, counts.waiting),
        (JigPhrase::TasksIdle, counts.idle),
        (JigPhrase::TasksStarting, counts.starting),
        (JigPhrase::TasksRunning, counts.running),
        (JigPhrase::TasksBackingOff, counts.backing_off),
        (JigPhrase::TasksClosing, counts.closing),
        (JigPhrase::TasksHeld, counts.held),
        (JigPhrase::TasksDone, counts.done),
        (JigPhrase::TasksFailed, counts.failed),
        (JigPhrase::TasksCancelled, counts.cancelled),
    ] {
        if count > 0 {
            words.text(b, phrase, &[Hole::Number(u64::from(count))]);
        }
    }
}
