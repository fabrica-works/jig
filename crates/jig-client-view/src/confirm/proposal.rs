//! A refused amendment's explicit proposal has its own confirmation (view.md, 18.4).
use crate::{Ui, confirm::ReasonInput, objects::Object};
use alloc::boxed::Box;
use jig_client::contract::{ActKind, Admission, Ask, Beyond, Recipient};
use skein_lib::{Env, Id};

pub(crate) fn recipient(object: &Object) -> Option<Recipient> {
    match crate::acts::offered(object, ActKind::Amend) {
        Admission::Bounded { beyond: Beyond::Proposed { to }, .. } | Admission::Proposed { to } => Some(to),
        Admission::Allowed | Admission::Bounded { .. } | Admission::Refused(_) => None,
    }
}
pub(crate) fn open(domain: &mut Ui, env: &Env<crate::Limits>, id: Id<Object>, lent: &jig_client::Client) {
    let Some(object) = domain.borrow(lent).objects().get(id) else { return };
    let crate::objects::Card::Refused {
        refusal: jig_client::contract::Refusal::Authority { proposable: Some(true), .. },
    } = &object.card
    else {
        return;
    };
    let Some(ask) = domain.refused_amendments.get(&object.key).cloned() else { return };
    let Ask::Amend { revision: Some(revision), change, reason, .. } = *ask else { return };
    if recipient(&object).is_none() {
        return;
    }
    super::open(domain, env, id, ActKind::Amend, lent);
    let Some(open) = domain.confirming.as_mut() else { return };
    domain.proposing = Some(id);
    open.revision = Some(revision);
    open.choices.change = Some(*change);
    open.reason_input = ReasonInput::Supplied;
    if domain.write_field(crate::fields::Purpose::Reason, reason).is_err() {
        super::close(domain);
        return;
    }
    super::sync(domain, lent);
}

pub(crate) fn remember(
    domain: &mut Ui,
    object: jig_client::contract::ObjectKey,
    ask: Box<Ask>,
    answer: &jig_client::contract::Answer,
) {
    let mut stale = skein_lib::List::with_capacity(domain.limits.objects);
    for (key, _) in &domain.refused_amendments {
        if domain.objects.find(*key).is_none() {
            stale.push(*key).expect("map fits objects");
        }
    }
    for key in stale.into_boxed() {
        domain.refused_amendments.remove(&key);
    }
    domain.refused_amendments.remove(&object);
    if let jig_client::contract::Answer::Refused {
        refusal: jig_client::contract::Refusal::Authority { proposable: Some(true), .. },
        ..
    } = answer
        && let Ask::Amend { revision: Some(_), .. } = ask.as_ref()
        && domain.objects.find(object).is_some()
    {
        domain.refused_amendments.insert(object, ask).expect("one refused amendment per object");
    }
}

pub(crate) fn forget(domain: &mut Ui) {
    let mut keys = skein_lib::List::with_capacity(domain.refused_amendments.len());
    for (key, _) in &domain.refused_amendments {
        keys.push(*key).expect("map keys fit");
    }
    for key in keys.into_boxed() {
        domain.refused_amendments.remove(&key);
    }
}
