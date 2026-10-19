//! Resolve working copies at each decision, never the retained ask or proposal
//! (domain/authority.md, 7 and 8.1; domain/tasks.md, 4 and 9).
use jig_domain_authority as authority;
use jig_domain_tasks as tasks;
use skein_lib::Wall;

pub(crate) fn authority(given: &tasks::Authority, deadline: tasks::Deadline, decision: Wall) -> authority::Authority {
    let mut value = crate::translate::authority_value(given);
    if let tasks::Deadline::Span(span) = deadline {
        value.budget.deadline = Some(authority::deadline_after(decision, span));
    }
    value
}

pub(crate) fn resolve(batch: &mut [tasks::New], decision: Wall) {
    for member in batch {
        if let tasks::Deadline::Span(span) = member.deadline {
            member.authority.budget.deadline = Some(authority::deadline_after(decision, span));
            member.deadline = tasks::Deadline::Named;
        }
    }
}

/// Fund the same resolved working copy that the accepting decision authorized.
pub(super) fn fund(batch: &mut [tasks::New], source: tasks::Funder, decision: Wall) {
    resolve(batch, decision);
    for member in batch {
        member.funder = source;
    }
}
