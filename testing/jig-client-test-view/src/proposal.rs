//! An application-owned proposal section and dialog (client/domain/testing.md, 4).
use crate::bindings::{Bindings, Meaning};
use jig_client::contract::{ActKind, Effect, ObjectKey, Proposal, ProposalKey, ReleaseEffect};
use jig_client_test_domain::{
    Domain,
    first::{
        self,
        proposal::{Act, Choice},
    },
};
use jig_client_view::{objects::Body, phrases::jig::JigPhrase as P, words::Words};
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Region, Role, Run};
const REGION: Region = Region { owner: skein_ui::Owner { number: 1 }, number: 30 };
fn document(root: &Domain, key: ProposalKey) -> Option<Proposal> {
    let client = root.jig().borrow(root.client());
    let objects = client.objects();
    let id = objects.find(ObjectKey::Proposal(key))?;
    let object = objects.get(id)?;
    let Some(Body::Proposal(proposal)) = object.body else {
        return None;
    };
    Some(*proposal)
}
pub(crate) fn section(
    b: &mut Builder,
    root: &Domain,
    key: ProposalKey,
    words: &Words,
    family: &crate::words::Words,
    bindings: &mut Bindings,
    hide: bool,
) {
    let Some(proposal) = document(root, key) else {
        return;
    };
    b.open(Role::Region);
    b.region(REGION);
    b.name(&family.say(crate::words::Word::Proposal, &[]));
    b.text(Run::Plain, proposal.why.bytes());
    for (kind, accept, phrase) in [(ActKind::Accept, true, P::ActAccept), (ActKind::Reject, false, P::ActReject)] {
        if jig_client::facts::offer(proposal.acts.as_slice(), kind).is_some() && proposal.ending.is_none() {
            facts(b, &proposal, kind, words, hide);
            button(
                b,
                words,
                phrase,
                Act::Review(Choice { project: proposal.project, proposal: key, accept }),
                bindings,
            );
        }
    }
    b.close();
}
pub(crate) fn confirmation(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    family: &crate::words::Words,
    bindings: &mut Bindings,
    hide: bool,
) {
    let Some(choice) = root.first().proposal_confirmation() else {
        return;
    };
    let Some(proposal) = document(root, choice.proposal) else {
        return;
    };
    b.open(Role::Dialog);
    b.region(REGION);
    b.name(&family.say(crate::words::Word::ReviewProposal, &[]));
    b.text(Run::Plain, proposal.why.bytes());
    facts(b, &proposal, if choice.accept { ActKind::Accept } else { ActKind::Reject }, words, hide);
    button(b, words, P::Confirm, Act::Confirm, bindings);
    button(b, words, P::Dismiss, Act::Dismiss, bindings);
    b.close();
}
fn button(b: &mut Builder, words: &Words, phrase: P, act: Act, bindings: &mut Bindings) {
    b.open(Role::Button);
    b.name(&words.say(phrase, &[]));
    b.bind(bindings.insert(Meaning::Action(first::Act::Proposal(act))));
    b.close();
}
fn facts(b: &mut Builder, proposal: &Proposal, kind: ActKind, words: &Words, hide: bool) {
    if hide {
        return;
    }
    if jig_client::facts::dangerous(kind) {
        words.status(b, P::Dangerous, &[]);
    }
    if let Some(effect) = jig_client::facts::effect(proposal.acts.as_slice(), kind) {
        match effect {
            Effect::Move { reservation } => words.text(b, P::TakeoverFunds, &[Hole::Amount(*reservation)]),
            Effect::Release(effect) => words.text(
                b,
                match effect {
                    ReleaseEffect::Judged => P::ReleaseFresh,
                    ReleaseEffect::RunAgain => P::ResumeRelease,
                    ReleaseEffect::Uncertain => P::EngineFollowsChoice,
                },
                &[],
            ),
            Effect::Cancel(impact) => {
                for (phrase, number) in [
                    (P::TasksWaiting, impact.tasks.waiting),
                    (P::TasksRunning, impact.tasks.running),
                    (P::TasksHeld, impact.tasks.held),
                    (P::TasksClosing, impact.tasks.closing),
                    (P::TasksDone, impact.tasks.done),
                    (P::TasksFailed, impact.tasks.failed),
                    (P::TasksCancelled, impact.tasks.cancelled),
                    (P::LiveRuns, impact.live_runs),
                    (P::PendingProposals, impact.proposals),
                ] {
                    words.text(b, phrase, &[Hole::Number(u64::from(number))]);
                }
                if impact.tasks.ended > 0 {
                    words.text(b, P::TasksEnded, &[Hole::Number(u64::from(impact.tasks.ended))]);
                }
                words.text(b, P::EffectsSettle, &[]);
            }
        }
    }
    if kind == ActKind::Accept
        && let Some(after) = proposal.funding.left.checked_sub(proposal.needs.budget)
    {
        words.text(
            b,
            P::Funds,
            &[Hole::Amount(proposal.needs.budget), Hole::Amount(proposal.funding.left), Hole::Amount(after)],
        );
    }
}
