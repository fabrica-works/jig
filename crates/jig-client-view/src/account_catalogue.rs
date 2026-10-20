//! Reusable rows for configured offers and authorized payer readiness (pages.md, 7 and 11).
use crate::owner_deployment::{field_number, text};
use crate::owner_deployment_words::{AccountPhrase as A, DeploymentPhrase as P};
use crate::words::Words;
use jig_client::contract::{OwnerAccountUse, PayerReadiness, ProviderOffer, Readiness};
use skein_lib::List;
use skein_ui::{Builder, Role};

/// Applications can supply `me.providers` and their accounts document's readiness.
/// The enclosing builder must reserve `max_nodes` and whole manage-target bytes.
pub fn render(
    b: &mut Builder,
    words: &Words,
    providers: &List<ProviderOffer>,
    providers_cut: bool,
    key_bytes: u32,
    readiness: &List<PayerReadiness>,
    readiness_cut: bool,
) {
    text(b, words, P::AccountDetail(A::Catalogue), &[]);
    field_number(b, words, P::AccountDetail(A::KeyBytes), u64::from(key_bytes));
    b.open(Role::List);
    for value in providers {
        b.open(Role::Item);
        field_number(b, words, P::AccountDetail(A::Provider), u64::from(value.provider));
        crate::literal_segment::render(b, value.name.bytes());
        if value.name.cut {
            text(b, words, P::Cut, &[]);
        }
        if value.sign_in {
            text(b, words, P::AccountDetail(A::SignIn), &[]);
        }
        if value.key {
            text(b, words, P::AccountDetail(A::Key), &[]);
        }
        for program in &value.programs {
            text(b, words, P::AccountDetail(A::Program), &[]);
            crate::literal_segment::render(b, program.bytes());
        }
        if value.programs_cut {
            text(b, words, P::Cut, &[]);
        }
        if value.none {
            text(b, words, P::AccountDetail(A::None), &[]);
        }
        if value.signed_in_plan {
            text(b, words, P::AccountDetail(A::SignInPlan), &[]);
        }
        if let Some(address) = &value.manage_usage {
            b.open(Role::Link);
            b.external(address);
            text(b, words, P::AccountDetail(A::ManageUsage), &[]);
            b.close();
        }
        b.close();
    }
    b.close();
    if providers_cut {
        text(b, words, P::AccountDetail(A::OffersCut), &[]);
    }
    b.open(Role::List);
    for value in readiness {
        b.open(Role::Item);
        field_number(b, words, P::AccountDetail(A::Provider), u64::from(value.provider));
        match value.payer {
            OwnerAccountUse::Party(party) => field_number(b, words, P::AccountDetail(A::PaysParty), party),
            OwnerAccountUse::Project(project) => {
                field_number(b, words, P::AccountDetail(A::PaysProject), u64::from(project));
            }
        }
        let phrase = match value.readiness {
            Readiness::Usable => A::Ready,
            Readiness::Unavailable => A::Unavailable,
            Readiness::Unknown => A::UnknownReadiness,
        };
        text(b, words, P::AccountDetail(phrase), &[]);
        text(
            b,
            words,
            P::AccountDetail(match value.settled {
                Some(true) => A::Settled,
                Some(false) => A::Pending,
                None => A::UnknownSettlement,
            }),
            &[],
        );
        b.close();
    }
    b.close();
    if readiness_cut {
        text(b, words, P::AccountDetail(A::ReadinessCut), &[]);
    }
}
/// Both independent lists fill together; each offer can carry a full program list.
#[must_use]
pub fn max_nodes(l: &crate::Limits) -> Option<u32> {
    l.rows.checked_mul(l.rows.checked_mul(2)?.checked_add(12)?)?.checked_add(l.rows.checked_mul(5)?)?.checked_add(6)
}
/// Check whole identities and manage addresses before constructing a fragment.
#[must_use]
pub fn fits(providers: &List<ProviderOffer>, readiness: &List<PayerReadiness>, l: &crate::Limits) -> Option<()> {
    if providers.capacity() > l.rows || readiness.capacity() > l.rows {
        return None;
    }
    for value in providers {
        if !value.name.bytes.fits(l.text) || value.programs.capacity() > l.rows {
            return None;
        }
        for program in &value.programs {
            if !program.fits(l.text) {
                return None;
            }
        }
        if let Some(address) = &value.manage_usage
            && !jig_client::contract::valid_manage_usage(address)
        {
            return None;
        }
    }
    Some(())
}
