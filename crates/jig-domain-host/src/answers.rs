//! Answers retained on their run slots until acknowledged (hosts.md, 6.2).
use crate::domain::Domain;
use crate::{Answer, Bounds, Hello, Hosting, LinkUp, Phase, Request};
use alloc::boxed::Box;
use skein_lib::{Duration, List, Queue, ReplyTo, Token};

#[derive(Debug)]
pub(crate) struct Kept {
    // Retransmissions reuse the assignment's correlation token.
    pub(crate) reply_to: Token,
    pub(crate) answer: Answer,
}

pub(crate) fn kept(domain: &Domain) -> u32 {
    domain.answers.len()
}

pub(crate) fn at(domain: &Domain, index: u32) -> Option<LinkUp> {
    for (at, (_, id)) in domain.answers.iter().enumerate() {
        if u32::try_from(at).expect("bounded answer slots") == index {
            let run = domain.hosted.get(*id).expect("an answer holds its slot");
            let answer = run.answer.as_ref().expect("indexed answer");
            return Some(LinkUp::Answer {
                reply_to: ReplyTo::new(answer.reply_to),
                run: run.run,
                attempt: run.attempt,
                answer: answer.answer.clone(),
            });
        }
    }
    None
}

pub(crate) fn acknowledge(domain: &mut Domain, run: Token, attempt: Token) {
    if let Some(id) = domain.answers.remove(&(run, attempt)) {
        domain.hosted.get_mut(id).expect("retained answer").answer = None;
        domain.hosted.retire(id);
    }
}

pub(crate) fn opened(
    domain: &Domain,
    slots: u32,
    stop_bound: Duration,
    workstreams: Box<[u64]>,
    bounds: Bounds,
    out: &mut Queue<Request>,
) {
    let mut hosting = List::with_capacity(domain.hosted.len());
    for (_, id) in &domain.names {
        let entry = crate::hosted::hosting(domain, id.token()).expect("named run");
        hosting.push(entry).expect("run slots bound the hello");
    }
    for (_, id) in &domain.answers {
        let run = domain.hosted.get(*id).expect("retained answer slot");
        if run.answer.is_some() {
            hosting
                .push(Hosting { run: run.run, attempt: run.attempt, phase: Phase::Answered })
                .expect("answer retains its slot");
        }
    }
    out.push(Request::Hello { hello: Hello { slots, stop_bound, workstreams, hosting: hosting.into_boxed(), bounds } });
}
