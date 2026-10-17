//! The application fills the sign-in slot; secrets stay below the view.
//! Sign-out parks via the root and return reopens the frame; client/domain/client.md, 12.
use crate::{Request, Ui, page::Page};
use skein_lib::{Queue, Token};
/// The application owns the ways to sign in, under its root owner.
pub const SLOT: skein_ui::SlotRef =
    skein_ui::SlotRef { owner: crate::Owner::Application { number: 0 }.opaque(), token: Token::new(0) };
/// Presentation state while the client ends the old frame watch.
#[derive(Debug)]
pub(crate) struct SignIn {
    pub(crate) application: bool,
    pub(crate) service: bool,
}
impl SignIn {
    /// Starts without an outstanding sign-in return.
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self { application: false, service: false }
    }
}
/// The shell owns the exchange; retain the frame until it answers signed out.
pub(crate) fn sign_out(domain: &Ui, out: &mut Queue<Request>) {
    if domain.confirmed.is_some() && !domain.signin.service {
        out.push(Request::Client { asked: jig_client::Event::SignOut });
    }
}
/// A lost exchange only adds the frame notice; the sign-in still works.
pub(crate) fn sign_out_lost(domain: &mut Ui) {
    if domain.confirmed.is_some() && !domain.signin.service {
        crate::notices::kept::show(domain, crate::notices::kept::Kind::SignOutLost);
    }
}
/// Ends the confirmed sign-in, preserving pending intents and kept records.
pub(crate) fn signed_out(domain: &mut Ui, out: &mut Queue<Request>, _lent: &jig_client::Client) {
    if domain.blocked {
        return;
    }
    crate::notices::kept::clear_sign_out_lost(domain);
    let service = domain.signin.service;
    domain.signin.application = domain.page == Page::Elsewhere;
    crate::page::routing::leave_page(domain, out);
    domain.confirmed = None;
    domain.page = if service { Page::Elsewhere } else { Page::SigningIn };
    out.push(Request::Confirmed { party: None });
    domain.changed();
}
