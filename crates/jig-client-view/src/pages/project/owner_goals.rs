//! Genuine whole Project goal offers and the original typed goal form; pages.md, 6.2.
use crate::{
    Builder, JigBindings, JigMeaning, Role, State, cards,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_client::{
    Domain,
    action::Act,
    contract::{ActKind, Admission, Bounds, Funder, GoalExecutor, OwnerProject},
    fields::Purpose,
};

/// The goal renderer traverses bounded choices and recipients; both have independent row bounds.
pub(super) fn offer_fits(value: &Admission, limits: &jig_client::Limits) -> Option<()> {
    use jig_client::contract::{Beyond, Recipient};
    let recipient = match value {
        Admission::Bounded { within, beyond } => {
            match within {
                Bounds::Goal { charters, procedures, .. } => {
                    if charters.capacity() > limits.rows || procedures.capacity() > limits.rows {
                        return None;
                    }
                }
                Bounds::Amend { .. } => {}
            }
            match beyond {
                Beyond::Proposed { to } => Some(to),
                Beyond::Refused(_) => None,
            }
        }
        Admission::Proposed { to } => Some(to),
        Admission::Allowed | Admission::Refused(_) => None,
    };
    match recipient {
        Some(Recipient::Roles { roles, .. }) if roles.capacity() > limits.rows => None,
        None | Some(Recipient::Task(_) | Recipient::Party(_) | Recipient::Role { .. } | Recipient::Roles { .. }) => {
            Some(())
        }
    }
}

pub(super) fn form(b: &mut Builder, client: &Domain, doc: &OwnerProject, words: &Words, bindings: &mut JigBindings) {
    let mut offer = None;
    for row in &doc.acts {
        if row.act == ActKind::SetGoal {
            offer = Some(&row.admission);
        }
    }
    let mut pool = None;
    if let Some(frame) = &client.frame().document {
        for project in &frame.projects {
            if project.project == doc.number {
                if let Some(ledger) = &project.pool {
                    pool = Some(ledger.left);
                }
            }
        }
        if !doc.configured_pools_cut {
            for ledger in &doc.configured_pools {
                match ledger.source {
                    Funder::Pool { project, person, period }
                        if project == doc.number && person == frame.party.number && period == doc.configured_period =>
                    {
                        pool = Some(ledger.left)
                    }
                    Funder::Task(_) | Funder::Pool { .. } | Funder::Period { .. } | Funder::Recurring { .. } => {}
                }
            }
        }
    }
    match offer {
        Some(Admission::Bounded { within: Bounds::Goal { pool_left, .. }, .. }) => pool = Some(*pool_left),
        Some(Admission::Refused(why)) => words.refusal(b, why),
        None
        | Some(
            Admission::Allowed | Admission::Proposed { .. } | Admission::Bounded { within: Bounds::Amend { .. }, .. },
        ) => {}
    }
    let eligible = match offer {
        Some(admission) => match admission {
            Admission::Allowed
            | Admission::Proposed { .. }
            | Admission::Bounded { within: Bounds::Goal { .. }, .. } => Some(admission),
            Admission::Refused(_) | Admission::Bounded { within: Bounds::Amend { .. }, .. } => None,
        },
        None => None,
    };
    if let Some(admission) = eligible {
        match pool {
            Some(left) => {
                words.text(b, JigPhrase::PoolLeft, &[Hole::Amount(left)]);
                cards::common::field(b, client, Purpose::NewGoal, JigPhrase::GoalWords, words, bindings);
                match admission {
                    Admission::Bounded { within: Bounds::Goal { charters, procedures, .. }, .. } => {
                        for charter in charters {
                            super::goals::choice(
                                b,
                                client,
                                words,
                                bindings,
                                doc.number,
                                GoalExecutor::Charter(*charter),
                                left,
                            );
                        }
                        for procedure in procedures {
                            super::goals::choice(
                                b,
                                client,
                                words,
                                bindings,
                                doc.number,
                                GoalExecutor::Procedure { connector: procedure.connector, code: procedure.code },
                                left,
                            );
                        }
                    }
                    Admission::Allowed | Admission::Proposed { .. } => {
                        if let Some(frame) = &client.frame().document {
                            for project in &frame.charters {
                                if project.project == doc.number {
                                    for charter in &project.goals {
                                        super::goals::choice(
                                            b,
                                            client,
                                            words,
                                            bindings,
                                            doc.number,
                                            GoalExecutor::Charter(charter.number),
                                            left,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Admission::Refused(_) | Admission::Bounded { within: Bounds::Amend { .. }, .. } => {}
                }
                inputs(b, client, doc.number, admission, left, words, bindings);
            }
            None => words.text(b, JigPhrase::FramePoolUnopened, &[]),
        }
    }
    let state = &client.project_page().form;
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

fn inputs(
    b: &mut Builder,
    client: &Domain,
    project: u32,
    admission: &Admission,
    pool_left: u64,
    words: &Words,
    bindings: &mut JigBindings,
) {
    use super::goals::Field;
    let state = &client.project_page().form;
    let Some(form) = state.form else {
        return;
    };
    if form.project != project {
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
            Field::Priority => b.value(skein_lib::Decimal::of(u64::from(form.priority)).as_bytes(), state.writes),
            Field::Deadline => match form.deadline {
                Some(at) => b.value(&super::goals::date(at), state.writes),
                None => b.value(&[], state.writes),
            },
        }
        b.bind(bindings.insert(JigMeaning::GoalField { form, field }));
        b.close();
    }
    words.text(b, JigPhrase::Budget, &[Hole::Amount(form.budget)]);
    words.text(b, JigPhrase::Priority, &[Hole::Number(u64::from(form.priority))]);
    if let Some(at) = form.deadline {
        words.deadline(b, at);
    }
    match jig_client::contract::admission::goal(admission, form.executor, form.budget, form.deadline) {
        Admission::Proposed { to } => {
            words.text(b, if form.budget > pool_left { JigPhrase::GoalPropose } else { JigPhrase::WillPropose }, &[]);
            cards::common::recipient(b, &to, words);
        }
        Admission::Refused(why) => words.refusal(b, &why),
        Admission::Allowed | Admission::Bounded { .. } => {}
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
