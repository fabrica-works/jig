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
    Request, Ui,
    address::{Address, Place},
    fields::Purpose,
    intents::Origin,
    page::IntentPurpose,
};
use alloc::boxed::Box;
use jig_client::contract::{Admission, Answer, Outcome, OwnerExecutor, Recipient, Refusal, Text};
use skein_lib::{Env, Queue, Token, Wall};
/// Goal options chosen from the project's offered executors and party pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GoalForm {
    pub project: u32,
    pub executor: OwnerExecutor,
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
    pub handle: Option<jig_client::Handle>,
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
    /// Ui writes of the typed form, independent of list deliveries.
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
pub(crate) fn configure(domain: &mut Ui, form: GoalForm) {
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
pub(crate) fn open(domain: &mut Ui, lent: &jig_client::Client) {
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
    let Some((prediction, pool_left)) = prediction(domain, form, scope, lent) else {
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
        handle: domain.page_handle(),
    });
    domain.changed();
}
pub(crate) fn draft(form: GoalForm, words: Text) -> jig_client::shapes::GoalDraft {
    jig_client::shapes::GoalDraft {
        words,
        executor: form.executor,
        budget: form.budget,
        deadline: form.deadline,
        priority: Some(form.priority),
    }
}
fn prediction(domain: &Ui, form: GoalForm, _scope: u64, lent: &jig_client::Client) -> Option<(Admission, u64)> {
    let Some(me) = domain.frame(lent).document else {
        return Some((Admission::Refused(Refusal::NotOffered), 0));
    };
    if crate::queries::goal_template(me, form.project).is_none() {
        return Some((Admission::Refused(Refusal::NotOffered), 0));
    }
    let mut creating = None;
    for project in &me.projects {
        if project.project == form.project {
            creating = project.creating.as_ref();
        }
    }
    let Some(creating) = creating else {
        return Some((Admission::Refused(Refusal::NotOffered), 0));
    };
    let words = domain.fields.get(Purpose::NewGoal)?.text.clone();
    let admission = match jig_client::shapes::fall(creating, &draft(form, words)) {
        jig_client::shapes::Fall::Within => Admission::Allowed,
        jig_client::shapes::Fall::Proposable(to) => Admission::Proposed { to },
        jig_client::shapes::Fall::Refused(why) => Admission::Refused(why),
        jig_client::shapes::Fall::NotOffered(_) => Admission::Refused(Refusal::NotOffered),
    };
    Some((admission, creating.within.pool_left))
}
/// Refresh an unsent draft when the supplied project facts move.
pub(crate) fn sync(domain: &mut Ui, lent: &jig_client::Client) {
    let Some(open) = &domain.project_page.form.confirmation else {
        return;
    };
    let Some((prediction, pool_left)) = prediction(domain, open.form, open.scope, lent) else {
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
pub(crate) fn send(domain: &mut Ui, out: &mut Queue<Request>, lent: &jig_client::Client) {
    sync(domain, lent);
    if domain.blocked || domain.project_page.form.pending.is_some() {
        return;
    }
    let Some(open) = domain.project_page.form.confirmation.take() else {
        return;
    };
    if open.scope != domain.confirmed.unwrap_or(0) || open.handle != domain.page_handle() {
        return;
    }
    let Some(me) = domain.frame(lent).document else { return };
    let reason = match crate::acts::words(&domain.fields, Purpose::Reason) {
        Some(reason) => reason,
        None => Text::new(Box::from([]), domain.text_limit).expect("empty reason fits"),
    };
    let ask = match jig_client::shapes::set_goal(
        me,
        open.form.project,
        draft(open.form, open.words.clone()),
        reason,
        domain.borrow(lent).wall(),
    ) {
        jig_client::shapes::Judged::Create(ask) | jig_client::shapes::Judged::Propose { ask, .. } => ask,
        jig_client::shapes::Judged::Refused(why) => {
            domain.project_page.form.refusal = Some(why);
            domain.changed();
            return;
        }
        jig_client::shapes::Judged::NotOffered(_) => {
            domain.project_page.form.refusal = Some(Refusal::NotOffered);
            domain.changed();
            return;
        }
    };
    match domain.intents.ask(
        open.scope,
        Origin::Page { handle: domain.page_handle(), purpose: IntentPurpose::SetGoal },
        Box::new(ask),
        out,
    ) {
        Some(id) => {
            domain.project_page.form.pending =
                Some(Submission { request: id.token(), scope: open.scope, form: open.form, words: open.words });
            domain.project_page.form.refusal = None;
        }
        None => {
            domain.project_page.form.refusal = Some(Refusal::Limit { which: jig_client::contract::Limit::Requests });
        }
    }
    domain.changed();
}
pub(crate) fn answered(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    request: Token,
    handle: Option<jig_client::Handle>,
    answer: Answer,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
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
        Answer::Refused { refusal: why, .. } => {
            if handle == domain.page_handle() && domain.confirmed == Some(pending.scope) {
                domain.project_page.form.refusal = Some(why);
            } else {
                crate::notices::kept::show(
                    domain,
                    crate::notices::kept::Kind::PageAnswer { answer: Answer::Refused { refusal: why, at: None } },
                );
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
                        crate::page::routing::refuse(domain, which);
                    }
                }
            }
            domain.project_page.form.answer = Some(outcome.clone());
            match crate::outcome::route(&outcome) {
                crate::outcome::Route::Created { task } | crate::outcome::Route::Goal { task } => {
                    if handle == domain.page_handle() && domain.confirmed == Some(pending.scope) {
                        crate::page::routing::navigate(domain, env, Place::Jig(Address::task(task)), false, out, lent);
                    } else {
                        crate::notices::kept::show(
                            domain,
                            crate::notices::kept::Kind::PageAnswer { answer: Answer::Done(outcome) },
                        );
                    }
                }
                crate::outcome::Route::Chat { .. }
                | crate::outcome::Route::Words { .. }
                | crate::outcome::Route::Proposed
                | crate::outcome::Route::Other => {
                    crate::notices::kept::show(
                        domain,
                        crate::notices::kept::Kind::PageAnswer { answer: Answer::Done(outcome) },
                    );
                }
            }
        }
    }
    domain.changed();
}
pub(crate) fn unknown(domain: &mut Ui, request: Token) {
    if let Some(pending) = &domain.project_page.form.pending
        && pending.request == request
    {
        domain.project_page.form.pending = None;
        domain.changed();
    }
}
