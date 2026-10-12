//! Supplied period boundaries (`authority.md`, 12; client plan QUESTIONS 32).
//! The period ledger owns its receipt wall and optional supplied next wall.
//! There is no calendar, reset policy or monotonic timer here.
use crate::{Domain, Funder, Request};
use skein_lib::{Queue, Wall};

/// A period's actual opening receipt wall and its source's optional next boundary.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PeriodTimes {
    pub began: Wall,
    pub next: Option<Wall>,
}

pub(crate) fn observe(domain: &mut Domain, project: u32, period: u64, next: Option<Wall>, out: &mut Queue<Request>) {
    if !domain.ready() {
        return;
    }
    for (funder, _) in &domain.funding {
        match *funder {
            Funder::Period { project: owner, period: newer } if owner == project && newer > period => return,
            Funder::Task(_) | Funder::Pool { .. } | Funder::Recurring { .. } | Funder::Period { .. } => {}
        }
    }
    let funder = Funder::Period { project, period };
    let Some(row) = domain.funding.get_mut(&funder) else {
        return;
    };
    let Some(times) = &mut row.period else {
        return;
    };
    if times.next == next {
        return;
    }
    times.next = next;
    crate::funders::save_funding(domain, funder, out);
}
