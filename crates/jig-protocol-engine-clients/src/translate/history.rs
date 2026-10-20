#![expect(clippy::manual_map, reason = "step code carries optional owner facts without closures")]
//! Original history revisions, snapshots and lifecycle facts (protocol/client.md, 3).
use super::{objects, owner, w};
use jig_domain_core::documents as d;
use jig_domain_tasks as t;
pub fn line(
    models: &[crate::Model],
    value: &d::HistoryLine,
    limits: &w::Limits,
) -> Result<w::OwnerHistoryLine, w::Problem> {
    let change = match value.change {
        t::Change::Cancelled => w::OwnerHistoryChange::Cancelled,
        t::Change::Released => w::OwnerHistoryChange::Released,
        t::Change::Amended => w::OwnerHistoryChange::Amended,
        t::Change::Moved => w::OwnerHistoryChange::Moved,
        t::Change::Proposed => w::OwnerHistoryChange::Proposed,
        t::Change::ProposalPassed => w::OwnerHistoryChange::ProposalPassed,
        t::Change::ProposalAccepted => w::OwnerHistoryChange::ProposalAccepted,
        t::Change::ProposalRejected => w::OwnerHistoryChange::ProposalRejected,
        t::Change::ProposalWithdrawn => w::OwnerHistoryChange::ProposalWithdrawn,
    };
    w::OwnerHistoryLine::new(
        limits,
        w::OwnerHistoryLineParts {
            task: value.task,
            revision: value.revision,
            by: owner::requester(value.by, limits)?,
            change,
            reason: owner::text(&value.reason, limits)?,
            proposal: match &value.proposal {
                Some(item) => Some(objects::historical::proposal_history(models, item, limits)?),
                None => None,
            },
            amendment: match &value.amendment {
                Some((before, after)) => Some(w::OwnerHistoryAmendment::new(
                    limits,
                    w::OwnerHistoryAmendmentParts {
                        before: objects::proposals::amendment(models, before, limits)?,
                        after: objects::proposals::amendment(models, after, limits)?,
                    },
                )?),
                None => None,
            },
            at: match value.at {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
        },
    )
}
