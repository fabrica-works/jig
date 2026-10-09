//! Pages independent of work (client/domain/pages.md, 8).
use crate::{Builder, JigBindings, Role, phrases::jig::JigPhrase, words::Words};
use jig_client::{Domain, action::Act, pages::Page};
/// Render starting, signing in, missing, outdated or another party's kept work.
pub fn not_the_work(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) {
    let phrase = match client.page() {
        Page::Starting => JigPhrase::Starting,
        Page::SigningIn => JigPhrase::SignInPage,
        Page::Missing => JigPhrase::MissingPage,
        Page::OutOfDate => JigPhrase::UpdateClient,
        Page::SomeoneElse => JigPhrase::SomeoneElse,
        Page::Elsewhere | Page::Work { .. } => return,
    };
    super::heading(b, words, phrase, &[]);
    if *client.page() == Page::SigningIn {
        for provider in client.signin().providers() {
            crate::frame::control(
                b,
                words,
                bindings,
                Role::Button,
                JigPhrase::SignInWith,
                &[crate::words::name(provider.name.bytes())],
                Act::SignIn { provider: provider.number },
            );
        }
    }
}
