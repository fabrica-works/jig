//! Supplied frame identity, membership, names and independent loss facts.
//! See client/domain/contract.md, 5.1, and pages.md, 2.
use crate::document_text;
use crate::phrases::jig::JigPhrase;
use crate::words::{Words, name};
use skein_ui::Builder;
use skein_ui::phrases::Hole;

use alloc::boxed::Box;
use jig_client::contract::{Me, MeParty, MeProject, PartyKind};

pub(crate) fn project_label(words: &Words, project: &MeProject) -> Box<[u8]> {
    match &project.name {
        Some(value) => words.say(JigPhrase::Project, &[name(value.bytes())]),
        None => words.say(JigPhrase::FrameProjectNumber, &[Hole::Number(u64::from(project.project))]),
    }
}
pub(crate) fn project_loss(b: &mut Builder, words: &Words, project: &MeProject) {
    if let Some(value) = &project.name {
        document_text::loss(b, value, words);
    }
}
pub(crate) fn identity(b: &mut Builder, words: &Words, party: &MeParty) {
    words.status(b, JigPhrase::SignedIn, &[name(party.name.bytes())]);
    document_text::loss(b, &party.name, words);
    let kind = match party.kind {
        PartyKind::Person => JigPhrase::FramePerson,
        PartyKind::Service => JigPhrase::FrameService,
    };
    words.text(b, kind, &[]);
    words.text(b, JigPhrase::FrameLogin, &[name(party.login.bytes())]);
    document_text::loss(b, &party.login, words);
}
pub(crate) fn membership(b: &mut Builder, words: &Words, project: &MeProject) {
    words.held_role(b, Some(project.role));
    match &project.pool {
        Some(ledger) => crate::owner_budget::ledger(b, ledger, words),
        None => words.text(b, JigPhrase::FramePoolUnopened, &[]),
    }
}
pub(crate) fn counts(b: &mut Builder, words: &Words, frame: &Me) {
    words.text(
        b,
        JigPhrase::FrameNeedsKinds,
        &[
            Hole::Number(u64::from(frame.needs.questions)),
            Hole::Number(u64::from(frame.needs.proposals)),
            Hole::Number(u64::from(frame.needs.escalations)),
            Hole::Number(u64::from(frame.needs.person_tasks)),
        ],
    );
    words.text(b, JigPhrase::FrameResultsSeen, &[Hole::Number(frame.results_seen)]);
    if frame.cut {
        words.text(b, JigPhrase::FrameProjectsCut, &[]);
    }
}
