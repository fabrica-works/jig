//! Goal board, factual funding and typed form controls (pages.md, 6.2).
use crate::{
    Builder, JigBindings, JigMeaning, Role, State, cards,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_domain_client::{
    Domain,
    action::Act,
    contract::{ActKind, Admission, Bounds, GoalExecutor, GoalLine, ObjectKey, Project},
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
    super::super::cut(b, words, client.project_page().ended_cut);
    super::super::work::cursors(
        b,
        words,
        bindings,
        client.project_page().ended_before.as_ref(),
        client.project_page().ended_after.as_ref(),
        jig_domain_client::pages::project::GOALS_WINDOW,
    );
    form(b, client, doc, words, bindings);
}
fn form(b: &mut Builder, client: &Domain, doc: &Project, words: &Words, bindings: &mut JigBindings) {
    let state = &client.project_page().form;
    let mut pool_left = None;
    let mut bounds = None;
    let mut offered = false;
    if let Some(frame) = &client.frame().document {
        for role in &frame.projects {
            if role.project == doc.number
                && let Some(ledger) = &role.pool
            {
                pool_left = Some(ledger.left);
            }
        }
        for pool in &doc.pools {
            if pool.party == frame.party.number {
                pool_left = Some(pool.budget.left);
            }
        }
        for offers in &frame.charters {
            if offers.project == doc.number && !offers.goals.is_empty() {
                offered = true;
            }
        }
    }
    for offer in &doc.acts {
        if offer.act == ActKind::SetGoal {
            match &offer.admission {
                Admission::Bounded { within: Bounds::Goal { pool_left: left, .. }, .. } => {
                    pool_left = Some(*left);
                    bounds = Some(&offer.admission);
                }
                Admission::Allowed
                | Admission::Proposed { .. }
                | Admission::Refused(_)
                | Admission::Bounded { within: Bounds::Amend { .. }, .. } => {}
            }
        }
    }
    if bounds.is_some() || offered {
        match pool_left {
            Some(left) => {
                words.text(b, JigPhrase::PoolLeft, &[Hole::Amount(left)]);
                cards::common::field(b, client, Purpose::NewGoal, JigPhrase::GoalWords, words, bindings);
                choices(b, client, doc, words, bindings, left, bounds);
                inputs(b, client, doc, words, bindings, left);
            }
            None => words.text(b, JigPhrase::FramePoolUnopened, &[]),
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
/// Bounds supply the whole choice list; Frame supplies only actual charter names.
fn choices(
    b: &mut Builder,
    client: &Domain,
    doc: &Project,
    words: &Words,
    bindings: &mut JigBindings,
    pool_left: u64,
    offer: Option<&Admission>,
) {
    match offer {
        Some(Admission::Bounded { within: Bounds::Goal { charters, procedures, .. }, .. }) => {
            for charter in charters {
                choice(b, client, words, bindings, doc.number, GoalExecutor::Charter(*charter), pool_left);
            }
            for procedure in procedures {
                choice(
                    b,
                    client,
                    words,
                    bindings,
                    doc.number,
                    GoalExecutor::Procedure { connector: procedure.connector, code: procedure.code },
                    pool_left,
                );
            }
        }
        None
        | Some(
            Admission::Allowed
            | Admission::Proposed { .. }
            | Admission::Refused(_)
            | Admission::Bounded { within: Bounds::Amend { .. }, .. },
        ) => {
            if let Some(frame) = &client.frame().document {
                for role in &frame.charters {
                    if role.project == doc.number {
                        for charter in &role.goals {
                            choice(
                                b,
                                client,
                                words,
                                bindings,
                                doc.number,
                                GoalExecutor::Charter(charter.number),
                                pool_left,
                            );
                        }
                    }
                }
            }
        }
    }
}
pub(super) fn choice(
    b: &mut Builder,
    client: &Domain,
    words: &Words,
    bindings: &mut JigBindings,
    project: u32,
    executor: GoalExecutor,
    pool_left: u64,
) {
    let form = match client.project_page().form.form {
        Some(form) if form.project == project => GoalForm { executor, ..form },
        Some(_) | None => GoalForm { project, executor, budget: pool_left, deadline: None, priority: 0 },
    };
    match executor {
        GoalExecutor::Charter(number) => crate::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            JigPhrase::Charter,
            &[Hole::Number(u64::from(number))],
            Act::GoalForm { form },
        ),
        GoalExecutor::Procedure { connector, code } => crate::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            JigPhrase::GoalProcedure,
            &[Hole::Number(u64::from(connector)), Hole::Number(u64::from(code))],
            Act::GoalForm { form },
        ),
    }
    match executor {
        GoalExecutor::Charter(number) => {
            if let Some(frame) = &client.frame().document {
                for role in &frame.charters {
                    if role.project == project {
                        for charter in &role.goals {
                            if charter.number == number {
                                crate::frame_facts::charter_name(b, words, charter);
                                return;
                            }
                        }
                    }
                }
            }
        }
        GoalExecutor::Procedure { .. } => {}
    }
}
fn inputs(b: &mut Builder, client: &Domain, doc: &Project, words: &Words, bindings: &mut JigBindings, pool_left: u64) {
    let state = &client.project_page().form;
    if let Some(form) = state.form {
        if form.project != doc.number {
            return;
        }
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
        let mut offered = None;
        for offer in &doc.acts {
            if offer.act == ActKind::SetGoal {
                offered = Some(&offer.admission);
            }
        }
        match offered {
            Some(Admission::Bounded { .. }) => {
                match jig_domain_client::contract::admission::goal(
                    offered.expect("bounded offer above"),
                    form.executor,
                    form.budget,
                    form.deadline,
                ) {
                    Admission::Proposed { to } => {
                        let phrase =
                            if form.budget > pool_left { JigPhrase::GoalPropose } else { JigPhrase::WillPropose };
                        words.text(b, phrase, &[]);
                        cards::common::recipient(b, &to, words);
                    }
                    Admission::Refused(why) => words.refusal(b, &why),
                    Admission::Allowed | Admission::Bounded { .. } => {}
                }
            }
            None | Some(Admission::Allowed | Admission::Proposed { .. } | Admission::Refused(_)) => {
                if form.budget > pool_left {
                    words.text(b, JigPhrase::GoalPropose, &[]);
                }
            }
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
pub(super) fn line(b: &mut Builder, client: &Domain, goal: &GoalLine, words: &Words, bindings: &mut JigBindings) {
    b.open(Role::Item);
    b.key(super::super::object_key(ObjectKey::Task(goal.chip.task)));
    cards::chip::chip(b, client, &goal.chip, words, bindings);
    if let Some(id) = client.objects().find(ObjectKey::Task(goal.chip.task)) {
        cards::card(b, client, id, &cards::Place::Elsewhere, words, bindings);
    }
    words.text(b, JigPhrase::Priority, &[Hole::Number(u64::from(goal.priority))]);
    super::super::work::owner_counts(b, goal.progress, words);
    words.text(b, JigPhrase::HeldBelow, &[Hole::Number(u64::from(goal.held_below))]);
    crate::owner_budget::render(b, &goal.budget, words);
    b.close();
}

pub(super) fn date(at: skein_lib::Wall) -> [u8; 16] {
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
