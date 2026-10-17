//! Pages independent of work (client/domain/pages.md, 8).
use crate::regions::JigRegion;
use crate::{JigBindings, phrases::jig::JigPhrase, words::Words};
use skein_ui::{Builder, Role};

use crate::{Borrowed, page::Page};
/// Render starting, signing in, missing, outdated or another party's kept work.
pub fn not_the_work(b: &mut Builder, client: &Borrowed<'_>, words: &Words, _bindings: &mut JigBindings) -> bool {
    let region = match client.page() {
        Page::Starting => JigRegion::Starting,
        Page::SigningIn => JigRegion::SigningIn,
        Page::Missing => JigRegion::Missing,
        Page::OutOfDate => JigRegion::OutOfDate,
        Page::SomeoneElse => JigRegion::SomeoneElse,
        Page::Elsewhere | Page::Work { .. } => return false,
    };
    let phrase = match client.page() {
        Page::Starting => JigPhrase::Starting,
        Page::SigningIn => JigPhrase::SignInPage,
        Page::Missing => JigPhrase::MissingPage,
        Page::OutOfDate => JigPhrase::UpdateClient,
        Page::SomeoneElse => JigPhrase::SomeoneElse,
        Page::Elsewhere | Page::Work { .. } => unreachable!("only pages outside the work"),
    };
    b.open(Role::Main);
    b.open(Role::Region);
    b.region(region.region());
    b.name(&words.say(phrase, &[]));
    super::heading(b, words, phrase, &[]);
    if *client.page() == Page::SigningIn {
        b.slot(crate::signin::SLOT, &words.say(JigPhrase::SignInPage, &[]));
    }
    b.close();
    b.close();
    true
}
