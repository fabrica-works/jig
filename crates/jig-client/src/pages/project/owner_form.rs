//! Whole supplied settings baselines and one keyed edit; pages.md, 6.5.
//! This form never computes policy. Only explicit offered requests are eligible.
use crate::{
    Domain, Request,
    action::Act,
    contract::{ActKind, Admission, Ask, OwnerProject, PolicyEdit, Refusal},
    intents::Origin,
    pages::IntentPurpose,
};
use alloc::boxed::Box;
use skein_lib::{Queue, Token};
/// One unsent review and the independent pending keyed request.
#[derive(Debug)]
pub struct Form {
    pub review: Option<Box<Ask>>,
    pub before: Option<Box<Ask>>,
    pub pending: Option<Token>,
    pub refused: Option<Refusal>,
}
/// The complete offered source baseline needed for one typed replacement.
#[must_use]
pub fn supported(doc: &OwnerProject, ask: &Ask) -> bool {
    let (project, kind, complete) = match ask {
        Ask::ChangePolicy { project, edit } => (*project, ActKind::ChangePolicy, policy_complete(doc, edit)),
        Ask::SetRoles { project, .. } => (*project, ActKind::SetRoles, !doc.cut),
        Ask::MakeService { project, .. } => (*project, ActKind::MakeService, true),
        Ask::SetPool { project, party, .. } => {
            let mut supplied = false;
            for pool in &doc.pools {
                match pool.source {
                    crate::contract::Funder::Pool { project: source, person: owner, .. } => {
                        supplied |= source == *project && owner == *party
                    }
                    crate::contract::Funder::Period { .. }
                    | crate::contract::Funder::Task(_)
                    | crate::contract::Funder::Recurring { .. } => {}
                }
            }
            (*project, ActKind::SetPool, supplied)
        }
        Ask::StartChat { .. }
        | Ask::Write { .. }
        | Ask::AnswerQuestion { .. }
        | Ask::AnswerPerson { .. }
        | Ask::Take { .. }
        | Ask::HandBack { .. }
        | Ask::DecideProposal { .. }
        | Ask::DecideEscalation { .. }
        | Ask::Release { .. }
        | Ask::Amend { .. }
        | Ask::Stop { .. }
        | Ask::Cancel { .. }
        | Ask::Close { .. }
        | Ask::TakeOver { .. }
        | Ask::SetGoal { .. }
        | Ask::Prioritise { .. }
        | Ask::MarkSeen { .. }
        | Ask::EditNote { .. }
        | Ask::DeleteNote { .. } => return false,
    };
    if project != doc.number || !complete {
        return false;
    }
    for offer in &doc.acts {
        if offer.act == kind {
            return match &offer.admission {
                Admission::Allowed | Admission::Proposed { .. } => true,
                Admission::Refused(_) | Admission::Bounded { .. } => false,
            };
        }
    }
    false
}
fn policy_complete(doc: &OwnerProject, edit: &PolicyEdit) -> bool {
    match edit {
        PolicyEdit::ProjectSpend { .. } => true,
        PolicyEdit::Role(role) => {
            if doc.cut || doc.policy.cut {
                return false;
            }
            for before in &doc.policy.roles {
                if before.number == role.number {
                    return !before.authority.cut;
                }
            }
            false
        }
        PolicyEdit::Requirements { .. } | PolicyEdit::Projections { .. } => !doc.cut && !doc.policy.cut,
        PolicyEdit::Permissions { .. } => false,
    }
}
/// The unchanged source fields shown before a supported replacement.
#[must_use]
pub fn baseline(doc: &OwnerProject, ask: &Ask) -> Option<Ask> {
    if !supported(doc, ask) {
        return None;
    }
    match ask {
        Ask::ChangePolicy { edit, .. } => {
            let edit = match edit {
                PolicyEdit::ProjectSpend { .. } => PolicyEdit::ProjectSpend { period_spend: doc.policy.period_spend },
                PolicyEdit::Role(role) => {
                    let mut source = None;
                    for row in &doc.policy.roles {
                        if row.number == role.number {
                            source = Some(row);
                            break;
                        }
                    }
                    let row = source?;
                    PolicyEdit::Role(crate::contract::PolicyEditRole {
                        number: row.number,
                        authority: row.authority.whole()?,
                        period_spend: row.period_spend,
                        requests: row.requests,
                        decides: row.decides,
                    })
                }
                PolicyEdit::Requirements { .. } => {
                    PolicyEdit::Requirements { requirements: doc.policy.requirements.clone() }
                }
                PolicyEdit::Projections { .. } => PolicyEdit::Projections { grants: doc.policy.projections.clone() },
                PolicyEdit::Permissions { .. } => return None,
            };
            Some(Ask::ChangePolicy { project: doc.number, edit })
        }
        Ask::SetRoles { .. } => {
            let mut holdings = skein_lib::List::with_capacity(doc.members.capacity());
            for row in &doc.members {
                holdings.push(crate::contract::Holding { party: row.party, role: row.role }).ok()?;
            }
            Some(Ask::SetRoles { project: doc.number, holdings })
        }
        Ask::SetPool { party, .. } => {
            for pool in &doc.pools {
                if let crate::contract::Funder::Pool { project, person, .. } = pool.source {
                    if project == doc.number && person == *party {
                        return Some(Ask::SetPool { project, party: person, budget: pool.numbers.budget });
                    }
                }
            }
            None
        }
        Ask::MakeService { .. } => None,
        Ask::StartChat { .. }
        | Ask::Write { .. }
        | Ask::AnswerQuestion { .. }
        | Ask::AnswerPerson { .. }
        | Ask::Take { .. }
        | Ask::HandBack { .. }
        | Ask::DecideProposal { .. }
        | Ask::DecideEscalation { .. }
        | Ask::Release { .. }
        | Ask::Amend { .. }
        | Ask::Stop { .. }
        | Ask::Cancel { .. }
        | Ask::Close { .. }
        | Ask::TakeOver { .. }
        | Ask::SetGoal { .. }
        | Ask::Prioritise { .. }
        | Ask::MarkSeen { .. }
        | Ask::EditNote { .. }
        | Ask::DeleteNote { .. } => None,
    }
}
pub(crate) fn review(domain: &mut Domain, ask: Box<Ask>) {
    if domain.blocked || domain.project_page.settings.pending.is_some() {
        return;
    }
    let Some(doc) = &domain.project_page.owner else {
        return;
    };
    if !supported(doc, &ask) {
        domain.project_page.settings.review = None;
        domain.project_page.settings.refused = Some(Refusal::NotOffered);
    } else {
        domain.project_page.settings.before = baseline(doc, &ask).map(Box::new);
        domain.project_page.settings.review = Some(ask);
        domain.project_page.settings.refused = None;
    }
    domain.changed();
}
pub(crate) fn send(domain: &mut Domain, out: &mut Queue<Request>) {
    if domain.blocked || domain.project_page.settings.pending.is_some() {
        return;
    }
    let Some(scope) = domain.confirmed else {
        return;
    };
    let Some(doc) = &domain.project_page.owner else {
        return;
    };
    let Some(ask) = domain.project_page.settings.review.take() else {
        return;
    };
    if !supported(doc, &ask) || baseline(doc, &ask).as_ref() != domain.project_page.settings.before.as_deref() {
        domain.project_page.settings.refused = Some(Refusal::NotOffered);
        domain.changed();
        return;
    }
    let origin = Origin::Page { generation: domain.generation, purpose: IntentPurpose::ProjectSettings };
    domain.project_page.settings.pending = match domain.intents.ask(scope, origin, ask, out) {
        Some(id) => Some(id.token()),
        None => None,
    };
    domain.changed();
}
pub(crate) fn answered(domain: &mut Domain, request: Token, answer: crate::contract::Answer) {
    if domain.project_page.settings.pending == Some(request) {
        domain.project_page.settings.pending = None;
        crate::notices::show(domain, crate::notices::Kind::PageAnswer { answer });
        domain.changed();
    }
}
/// Typed settings values are supplied by the view, never parsed by the domain.
#[must_use]
pub fn action(ask: Ask) -> Act {
    Act::ProjectSettings { ask: Box::new(ask) }
}

impl Form {
    pub(crate) const fn new() -> Self {
        Self { review: None, before: None, pending: None, refused: None }
    }
}
