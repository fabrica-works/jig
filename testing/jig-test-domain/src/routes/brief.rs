//! Keep the brief child's ordered, bounded parts until the claim (domain/root.md, 7).
use crate::{Domain, Limits, Work};
use alloc::boxed::Box;
use jig_domain_brief as brief;
use jig_domain_core as core;
use skein_lib::{Duration, Env, List, Queue, Token};

pub(super) fn plan(env: &Env<Limits>, task: u64, sections: List<brief::Planned>, work: &mut Queue<Work>) {
    work.push(Work::Core(core::Event::Brief(brief::GatherEvent::Plan {
        brief: Token::new(task),
        budget: env.limits.core.brief.brief_bytes,
        deadline: env.now.saturating_add(Duration::from_secs(30)),
        sections: sections.into_boxed(),
    })));
}
pub(super) fn complete(domain: &mut Domain, token: Token, order: Box<[brief::GatherPlaced]>, work: &mut Queue<Work>) {
    domain.briefs.insert(token.raw(), order).expect("one bounded brief per task");
    work.push(Work::Core(core::Event::BriefAssembled { task: token.raw(), ready: true }));
}
