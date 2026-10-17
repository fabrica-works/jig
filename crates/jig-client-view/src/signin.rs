//! Sign in providers are configured by the application and secrets stay below.
//! Sign-out parks via the root and return reopens the frame; client/domain/client.md, 12.
use jig_client::contract::Text;
/// A configured provider number and display name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Provider {
    pub number: u16,
    pub name: Text,
}

use crate::{Domain, Request, page::Page, watches::Owner};
use jig_client::contract::Subject;
use skein_lib::{Env, List, Queue};
/// Providers installed by the application before start, and a return awaiting
/// the old frame's terminal. Credentials never cross this state.
#[derive(Debug)]
pub struct SignIn {
    pub(crate) providers: List<Provider>,
    pub(crate) returning: bool,
    pub(crate) service: bool,
}
impl SignIn {
    /// Allocates the application's configured provider slots.
    #[must_use]
    pub fn new(capacity: u32) -> Self {
        Self { providers: List::with_capacity(capacity), returning: false, service: false }
    }
    /// Providers shown by the sign-in page.
    #[must_use]
    pub const fn providers(&self) -> &List<Provider> {
        &self.providers
    }
}
pub(crate) fn choose(domain: &Domain, provider: u16, out: &mut Queue<Request>) {
    match domain.page {
        Page::SigningIn => {
            for value in &domain.signin.providers {
                if value.number == provider {
                    out.push(Request::SignIn { provider, back: domain.address });
                    return;
                }
            }
        }
        Page::Starting | Page::Work { .. } | Page::Elsewhere | Page::Missing | Page::OutOfDate | Page::SomeoneElse => {}
    }
}
/// The shell owns the exchange; retain the frame until it answers signed out.
pub(crate) fn sign_out(domain: &Domain, out: &mut Queue<Request>) {
    if domain.confirmed.is_some() && !domain.signin.service {
        out.push(Request::SignOut);
    }
}
/// A lost exchange only adds the frame notice; the sign-in still works.
pub(crate) fn sign_out_lost(domain: &mut Domain) {
    if domain.confirmed.is_some() && !domain.signin.service {
        crate::notices::kept::show(domain, crate::notices::kept::Kind::SignOutLost);
    }
}
/// Ends the confirmed sign-in, preserving pending intents and kept records.
pub(crate) fn signed_out(domain: &mut Domain, out: &mut Queue<Request>) {
    if domain.blocked {
        return;
    }
    crate::notices::kept::clear_sign_out_lost(domain);
    let service = domain.signin.service;
    domain.close_all(out);
    domain.reads.abandon(domain.generation);
    out.push(Request::PartsLeft { generation: domain.generation });
    domain.objects.clear();
    domain.confirmed = None;
    domain.frame.document = None;
    domain.page = if service { Page::Elsewhere } else { Page::SigningIn };
    domain.set_link(crate::link::Link::Offline, out);
    out.push(Request::Confirmed { party: None });
    domain.changed();
}
/// A shell returned: reopen the frame once its prior attempt has ended.
pub(crate) fn returned(domain: &mut Domain, env: &Env<crate::Limits>, out: &mut Queue<Request>) {
    if domain.signin.returning && domain.frame_watch.is_none() && !domain.blocked {
        domain.frame_watch = domain.open(env, Subject::Frame, Owner::Frame, out);
        if domain.frame_watch.is_some() {
            domain.signin.returning = false;
        }
    }
}
/// Provider entries and their configured display names.
#[must_use]
pub fn worst_case(limits: &crate::Limits) -> Option<u64> {
    List::<Provider>::worst_case(limits.providers)?
        .checked_add(u64::from(limits.providers).checked_mul(u64::from(limits.text))?)
}
