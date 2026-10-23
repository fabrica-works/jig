//! Whole owner documents are marked by this decision's actual mutations.
use crate::{Domain, Limits, Request};
use skein_lib::{Env, List, Queue};

pub(crate) fn collect(domain: &Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if domain.client_subjects.is_empty() {
        return;
    }
    let mut changed = false;
    for _ in 0..out.len() {
        let request = out.pop().expect("fixed output count");
        changed |= match &request {
            Request::Save { .. } | Request::Erase { .. } | Request::Changed { .. } | Request::Slots { .. } => true,
            Request::Document { .. }
            | Request::ReadAnswered { .. }
            | Request::Marked { .. }
            | Request::ApplicationDone { .. }
            | Request::ApplicationBusy { .. }
            | Request::Closed { .. }
            | Request::Released { .. }
            | Request::Restarted { .. }
            | Request::Named { .. }
            | Request::Wake { .. }
            | Request::Drift { .. }
            | Request::Described { .. }
            | Request::Refused { .. }
            | Request::Make { .. }
            | Request::Outcome { .. }
            | Request::Step { .. }
            | Request::Close { .. }
            | Request::System(_) => false,
        };
        out.push(request);
    }
    if changed {
        // A conservative superset keeps removal marks authentic even when the
        // removed record no longer carries its owner. The views coalesce them.
        let mut subjects = List::with_capacity(env.limits.proposals);
        for (token, _) in &domain.client_subjects {
            subjects.push(*token).expect("bounded prepared subjects");
        }
        out.push(Request::Marked { subjects });
    }
}
