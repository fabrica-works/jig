//! Goal board, factual funding and typed form controls (pages.md, 6.2).
use crate::{
    Builder, JigBindings, JigMeaning, Role, State, cards,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_client::{
    Domain,
    action::Act,
    contract::{GoalLine, ObjectKey, Project},
    fields::Purpose,
    pages::project::goals::GoalForm,
};
/// The one typed form property changed by an input's complete value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    /// Whole funding amount.
    Budget,
    /// Optional wall deadline as UTC year-month-day and hour-minute.
    Deadline,
    /// Lower numbers place the goal earlier.
    Priority,
}
/// Decode one form field without deriving admission or changing other choices.
#[must_use]
pub fn edit(form: GoalForm, field: Field, value: &[u8]) -> Option<Act> {
    let mut form = form;
    match field {
        Field::Budget => form.budget = crate::number(value)?,
        Field::Priority => form.priority = u32::try_from(crate::number(value)?).ok()?,
        Field::Deadline => {
            form.deadline = if value.is_empty() { None } else { Some(crate::confirmation::amendment::deadline(value)?) }
        }
    }
    Some(Act::GoalForm { form })
}
pub(super) fn goals(b: &mut Builder, client: &Domain, doc: &Project, words: &Words, bindings: &mut JigBindings) {
    words.text(b, JigPhrase::PriorityMeaning, &[]);
    b.open(Role::List);
    for goal in &client.project_page().listed_goals {
        line(b, client, goal, words, bindings);
    }
    b.close();
    if client.project_page().goals_more {
        words.status(b, JigPhrase::MoreRows, &[]);
    }
    super::super::work::cursors(
        b,
        words,
        bindings,
        client.project_page().ended_before.as_ref(),
        client.project_page().ended_after.as_ref(),
        jig_client::pages::project::GOALS_WINDOW,
    );
    form(b, client, doc, words, bindings);
}
fn form(b: &mut Builder, client: &Domain, doc: &Project, words: &Words, bindings: &mut JigBindings) {
    let state = &client.project_page().form;
    if let Some(frame) = &client.frame().document {
        for role in &frame.projects {
            if role.project != doc.number || role.goals.is_empty() {
                continue;
            }
            let mut pool_left = role.pool.left;
            for pool in &doc.pools {
                if pool.party == frame.party.number {
                    pool_left = pool.budget.left;
                }
            }
            let current = match state.form {
                Some(form) if form.project == doc.number => Some(form),
                Some(_) | None => None,
            };
            words.text(b, JigPhrase::PoolLeft, &[Hole::Amount(pool_left)]);
            cards::common::field(b, client, Purpose::NewGoal, JigPhrase::GoalWords, words, bindings);
            for charter in &role.goals {
                let form = current.unwrap_or(GoalForm {
                    project: doc.number,
                    charter: charter.number,
                    budget: pool_left,
                    deadline: None,
                    priority: 0,
                });
                let form = GoalForm { charter: charter.number, ..form };
                crate::frame::control(
                    b,
                    words,
                    bindings,
                    Role::Button,
                    JigPhrase::Charter,
                    &[Hole::Number(u64::from(charter.number))],
                    Act::GoalForm { form },
                );
                b.text(crate::Run::Plain, charter.name.bytes());
            }
            if let Some(form) = current {
                for (field, phrase) in [
                    (Field::Budget, JigPhrase::GoalBudget),
                    (Field::Deadline, JigPhrase::GoalDeadline),
                    (Field::Priority, JigPhrase::GoalPriority),
                ] {
                    b.open(Role::TextField);
                    b.name(&words.say(phrase, &[]));
                    match field {
                        Field::Budget => b.value(skein_lib::Decimal::of(form.budget).as_bytes(), state.writes),
                        Field::Priority => {
                            b.value(skein_lib::Decimal::of(u64::from(form.priority)).as_bytes(), state.writes);
                        }
                        Field::Deadline => match form.deadline {
                            Some(at) => b.value(&date(at), state.writes),
                            None => b.value(&[], state.writes),
                        },
                    }
                    b.bind(bindings.insert(JigMeaning::GoalField { form, field }));
                    b.close();
                }
                words.text(b, JigPhrase::Budget, &[Hole::Amount(form.budget)]);
                words.text(b, JigPhrase::Priority, &[Hole::Number(u64::from(form.priority))]);
                if let Some(deadline) = form.deadline {
                    words.deadline(b, deadline);
                }
                if form.budget > pool_left {
                    words.text(b, JigPhrase::GoalPropose, &[]);
                }
                super::super::work::control(
                    b,
                    words,
                    bindings,
                    JigPhrase::ActSetGoal,
                    Act::Submit { field: Purpose::NewGoal },
                    false,
                );
            }
        }
    }
    if let Some(submission) = &state.pending {
        b.open(Role::Status);
        b.name(&words.say(JigPhrase::ActSetGoal, &[]));
        b.state(State::Busy);
        if let Some(intent) = client.intents().get(skein_lib::Id::from_token(submission.request)) {
            words.progress(b, intent.progress);
        }
        b.close();
    }
    if let Some(why) = &state.refusal {
        words.refusal(b, why);
    }
    if let Some(answer) = &state.answer {
        words.outcome(b, answer);
    }
}
pub(super) fn line(b: &mut Builder, client: &Domain, goal: &GoalLine, words: &Words, bindings: &mut JigBindings) {
    b.open(Role::Item);
    b.key(super::super::object_key(ObjectKey::Task(goal.chip.task)));
    cards::chip::chip(b, client, &goal.chip, words, bindings);
    if let Some(id) = client.objects().find(ObjectKey::Task(goal.chip.task)) {
        cards::card(b, client, id, &cards::Place::Elsewhere, words, bindings);
    }
    words.text(b, JigPhrase::Priority, &[Hole::Number(u64::from(goal.priority))]);
    super::super::work::counts(b, goal.progress, words);
    words.text(b, JigPhrase::HeldBelow, &[Hole::Number(u64::from(goal.held_below))]);
    super::super::work::budget(b, &goal.budget, words);
    if let Some(deadline) = goal.deadline {
        words.deadline(b, deadline);
    }
    b.close();
}

fn date(at: skein_lib::Wall) -> [u8; 16] {
    let seconds = at.as_secs();
    let mut days = seconds / 86400;
    let mut year = 1970_u64;
    // Wall's nanosecond range ends in 2554, so the traversal is bounded.
    for candidate in 1970_u64..=2554 {
        year = candidate;
        let leap = candidate.is_multiple_of(4) && (!candidate.is_multiple_of(100) || candidate.is_multiple_of(400));
        let span = if leap { 366 } else { 365 };
        if days < span {
            break;
        }
        days = days.checked_sub(span).expect("complete year");
    }
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let months = [31_u64, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1_u64;
    for span in months {
        if days < span {
            break;
        }
        days = days.checked_sub(span).expect("complete month");
        month = month.checked_add(1).expect("at most twelve months");
    }
    let mut bytes = *b"0000-00-00 00:00";
    for (number, start, width) in [
        (year, 0_usize, 4_usize),
        (month, 5, 2),
        (days.checked_add(1).expect("day exists"), 8, 2),
        ((seconds % 86400) / 3600, 11, 2),
        ((seconds % 3600) / 60, 14, 2),
    ] {
        let decimal = skein_lib::Decimal::of(number);
        let value = decimal.as_bytes();
        let end = start.checked_add(width).expect("fixed field");
        let begin = end.checked_sub(value.len()).expect("date field fits");
        for (offset, byte) in value.iter().enumerate() {
            let index = begin.checked_add(offset).expect("fixed date offset");
            *bytes.get_mut(index).expect("fixed date byte") = *byte;
        }
    }
    bytes
}
