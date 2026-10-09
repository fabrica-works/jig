//! Goal board ordering and its one page form confirmation. Admission is an
//! engine fact; comparing budget to the party pool only explains that a goal
//! would be proposed, never admits it. See client/domain/pages.md, 6.2.
//!
//! | Form | Event | Next | Emits |
//! |---|---|---|---|
//! | edited | submit | confirmation | nothing |
//! | confirmation | confirm | pending | one keyed goal intent |
//! | pending | confirm | pending | nothing |
//! | pending | refused | editable | refusal facts |
//! | pending | made/proposed | editable | clear matching words; navigate/notice |
use crate::{
    Domain, Request,
    address::{Address, Place},
    contract::{
        ActKind, Admission, Answer, Ask, Bounds, Executor, GoalLine, Made, Outcome, Phase, Recipient, Refusal, Text,
    },
    fields::Purpose,
    intents::Origin,
    pages::IntentPurpose,
};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, Token, Wall};
/// Goal options chosen from the project's offered charters and party pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GoalForm {
    pub project: u32,
    pub charter: u32,
    pub budget: u64,
    pub deadline: Option<Wall>,
    pub priority: u32,
}
/// Exact words and factual admission recalled before sending this form.
#[derive(Debug)]
pub struct GoalConfirmation {
    pub form: GoalForm,
    pub words: Text,
    pub pool_left: u64,
    pub proposed: bool,
    pub to: Option<Recipient>,
    pub scope: u64,
    pub generation: u32,
}
/// Snapshot belongs to its request even if the page or field changes.
#[derive(Debug)]
pub struct Submission {
    pub request: Token,
    pub scope: u64,
    pub form: GoalForm,
    pub words: Text,
}
/// Pending form and its durable result survive page context replacement.
#[derive(Debug)]
pub struct FormState {
    /// Domain writes of the typed form, independent of list deliveries.
    pub writes: u64,
    pub form: Option<GoalForm>,
    pub confirmation: Option<GoalConfirmation>,
    pub pending: Option<Submission>,
    pub refusal: Option<Refusal>,
    pub answer: Option<Outcome>,
}
impl FormState {
    pub(crate) const fn new() -> Self {
        Self { writes: 0, form: None, confirmation: None, pending: None, refusal: None, answer: None }
    }
}
pub(crate) fn configure(domain: &mut Domain, form: GoalForm) {
    if super::number(domain) != Some(form.project) {
        return;
    }
    domain.project_page.form.writes = domain.project_page.form.writes.checked_add(1).expect("form writes remain");
    domain.project_page.form.form = Some(form);
    domain.project_page.form.confirmation = None;
    domain.project_page.form.refusal = None;
    domain.changed();
}
/// Closes object confirmation before opening the page's one confirmation.
pub(crate) fn open(domain: &mut Domain) {
    crate::confirm::close(domain);
    if domain.blocked || domain.project_page.form.pending.is_some() {
        return;
    }
    let Some(scope) = domain.confirmed else {
        return;
    };
    let Some(form) = domain.project_page.form.form else {
        return;
    };
    if super::number(domain) != Some(form.project) {
        return;
    }
    let Some(field) = domain.fields.get(Purpose::NewGoal) else {
        return;
    };
    if field.text.bytes().is_empty() {
        return;
    }
    let words = field.text.clone();
    let Some((prediction, pool_left)) = prediction(domain, form, scope) else {
        return;
    };
    let to = match prediction {
        Admission::Allowed => None,
        Admission::Proposed { to } => Some(to),
        Admission::Bounded { .. } => return,
        Admission::Refused(why) => {
            domain.project_page.form.refusal = Some(why);
            domain.changed();
            return;
        }
    };
    domain.project_page.form.confirmation = Some(GoalConfirmation {
        form,
        words,
        pool_left,
        proposed: to.is_some(),
        to,
        scope,
        generation: domain.generation,
    });
    domain.changed();
}
fn prediction(domain: &Domain, form: GoalForm, scope: u64) -> Option<(Admission, u64)> {
    let mut offered = false;
    let mut pool = None;
    if let Some(frame) = &domain.frame.document {
        for project in &frame.projects {
            if project.project == form.project {
                for charter in &project.goals {
                    if charter.number == form.charter {
                        offered = true;
                    }
                }
                pool = Some(project.pool.left);
            }
        }
    }
    let mut admission = Admission::Refused(Refusal::NotOffered);
    if let Some(doc) = &domain.project_page.document {
        for act in &doc.acts {
            if act.act == ActKind::SetGoal {
                admission = act.admission.clone();
            }
        }
        for party in &doc.pools {
            if party.party == scope {
                pool = Some(party.budget.left);
            }
        }
    }
    // Bounded offers own these facts; frame facts remain for fixed admissions.
    match &admission {
        Admission::Bounded { within, .. } => match within {
            Bounds::Goal { pool_left, .. } => pool = Some(*pool_left),
            Bounds::Amend { .. } => {}
        },
        Admission::Refused(why) => return Some((Admission::Refused(why.clone()), pool.unwrap_or(0))),
        Admission::Allowed | Admission::Proposed { .. } => {
            if !offered {
                return Some((Admission::Refused(Refusal::NotOffered), pool.unwrap_or(0)));
            }
        }
    }
    Some((crate::contract::admission::goal(&admission, form.charter, form.budget, form.deadline), pool?))
}
/// Refresh an unsent draft when the supplied project facts move.
pub(crate) fn sync(domain: &mut Domain) {
    let Some(open) = &domain.project_page.form.confirmation else {
        return;
    };
    let Some((prediction, pool_left)) = prediction(domain, open.form, open.scope) else {
        return;
    };
    let to = match prediction {
        Admission::Allowed => None,
        Admission::Proposed { to } => Some(to),
        Admission::Bounded { .. } => return,
        Admission::Refused(why) => {
            domain.project_page.form.confirmation = None;
            domain.project_page.form.refusal = Some(why);
            domain.changed();
            return;
        }
    };
    let open = domain.project_page.form.confirmation.as_mut().expect("open above");
    if open.to != to || open.pool_left != pool_left {
        open.proposed = to.is_some();
        open.to = to;
        open.pool_left = pool_left;
        domain.changed();
    }
}
/// Sending consumes the confirmation, so a second press can only join pending.
pub(crate) fn send(domain: &mut Domain, out: &mut Queue<Request>) {
    sync(domain);
    if domain.blocked || domain.project_page.form.pending.is_some() {
        return;
    }
    let Some(open) = domain.project_page.form.confirmation.take() else {
        return;
    };
    if open.scope != domain.confirmed.unwrap_or(0) || open.generation != domain.generation {
        return;
    }
    let ask = Ask::SetGoal {
        project: open.form.project,
        spec: open.words.clone(),
        executor: Executor::Agent { charter: open.form.charter },
        budget: open.form.budget,
        priority: open.form.priority,
        deadline: open.form.deadline,
    };
    match domain.intents.ask(
        open.scope,
        Origin::Page { generation: domain.generation, purpose: IntentPurpose::SetGoal },
        Box::new(ask),
        out,
    ) {
        Some(id) => {
            domain.project_page.form.pending =
                Some(Submission { request: id.token(), scope: open.scope, form: open.form, words: open.words });
            domain.project_page.form.refusal = None;
        }
        None => domain.project_page.form.refusal = Some(Refusal::Limit { which: crate::contract::Limit::Requests }),
    }
    domain.changed();
}
pub(crate) fn answered(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    request: Token,
    generation: u32,
    answer: Answer,
    out: &mut Queue<Request>,
) {
    let same = match &domain.project_page.form.pending {
        Some(pending) => pending.request == request,
        None => false,
    };
    if !same {
        return;
    }
    let pending = domain.project_page.form.pending.take().expect("matching pending form");
    match answer {
        Answer::Refused(why) => {
            if generation == domain.generation && domain.confirmed == Some(pending.scope) {
                domain.project_page.form.refusal = Some(why);
            } else {
                crate::notices::show(domain, crate::notices::Kind::PageAnswer { answer: Answer::Refused(why) });
            }
        }
        Answer::Done(outcome) => {
            if domain.confirmed == Some(pending.scope) {
                let matching = match domain.fields.get(Purpose::NewGoal) {
                    Some(field) => field.text == pending.words,
                    None => false,
                };
                if matching {
                    let empty = Text::new(Box::from([]), domain.text_limit).expect("empty words fit");
                    if let Err(which) = domain.write_field(Purpose::NewGoal, empty) {
                        crate::pages::routing::refuse(domain, which);
                    }
                }
            }
            domain.project_page.form.answer = Some(outcome.clone());
            match outcome {
                Outcome::Made { what: Made::Goal { task } } => {
                    if generation == domain.generation && domain.confirmed == Some(pending.scope) {
                        crate::pages::routing::navigate(domain, env, Place::Jig(Address::task(task)), false, out);
                    } else {
                        crate::notices::show(
                            domain,
                            crate::notices::Kind::PageAnswer {
                                answer: Answer::Done(Outcome::Made { what: Made::Goal { task } }),
                            },
                        );
                    }
                }
                Outcome::Made {
                    what:
                        Made::Chat { .. }
                        | Made::Words { .. }
                        | Made::Tasks { .. }
                        | Made::Service { .. }
                        | Made::PersonResult { .. }
                        | Made::Note { .. },
                }
                | Outcome::Proposed { .. }
                | Outcome::Decided { .. }
                | Outcome::DecidedFirst { .. }
                | Outcome::Changed => {
                    crate::notices::show(domain, crate::notices::Kind::PageAnswer { answer: Answer::Done(outcome) });
                }
            }
        }
    }
    domain.changed();
}
pub(crate) fn unknown(domain: &mut Domain, request: Token) {
    if let Some(pending) = &domain.project_page.form.pending
        && pending.request == request
    {
        domain.project_page.form.pending = None;
        domain.changed();
    }
}
/// Running goals precede waiting goals; priority descends within each group.
pub(crate) fn order(rows: &mut List<GoalLine>, ended: bool) {
    for first in 0..rows.len() {
        for second in first.checked_add(1).expect("row index fits")..rows.len() {
            let a = rows.get(first).expect("row exists");
            let b = rows.get(second).expect("row exists");
            let before = if ended {
                b.changed > a.changed || (b.changed == a.changed && b.chip.task < a.chip.task)
            } else {
                (a.chip.phase == Phase::Waiting && b.chip.phase != Phase::Waiting)
                    || (a.chip.phase == Phase::Waiting) == (b.chip.phase == Phase::Waiting)
                        && (b.priority > a.priority || (b.priority == a.priority && b.chip.task < a.chip.task))
            };
            if before {
                let b = b.clone();
                let a = core::mem::replace(rows.get_mut(first).expect("row exists"), b);
                *rows.get_mut(second).expect("row exists") = a;
            }
        }
    }
}
