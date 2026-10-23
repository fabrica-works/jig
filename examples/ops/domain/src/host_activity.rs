//! The engine's hub reports on its permanent fleet link (hosts.md, 6.1).
use crate::domain::Work;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use skein_lib::{Queue, Token};

pub(crate) fn drain(domain: &mut host::Domain, capacity: u32, work: &mut Queue<Work>) {
    for _ in 0..capacity {
        let Some(record) = domain.pop_activity() else { break };
        let host::LinkUp::Activity { run, attempt, activity } = record else { continue };
        work.push(Work::Core(core::Event::Fleet(fleet::Event::Observe {
            channel: Token::new(0),
            run,
            attempt,
            report: report(activity),
        })));
    }
}

fn report(activity: host::Activity) -> fleet::Report {
    match activity {
        host::Activity::Phase(phase) => fleet::Report::Activity(match phase {
            host::Phase::Preparing => fleet::Activity::Preparing,
            host::Phase::Starting => fleet::Activity::Starting,
            host::Phase::Active => fleet::Activity::Active,
            host::Phase::Waiting => fleet::Activity::Waiting,
            host::Phase::Ending | host::Phase::Answered => fleet::Activity::Ending,
        }),
        host::Activity::Long { operation, span } => fleet::Report::Activity(fleet::Activity::Long {
            operation: match operation {
                host::Operation::Checks => fleet::Operation::Checks,
                host::Operation::Completion => fleet::Operation::Completion,
                host::Operation::Command => fleet::Operation::Command,
            },
            span,
        }),
        host::Activity::LongDone => fleet::Report::Activity(fleet::Activity::Active),
        host::Activity::Count { kind, elapsed, count } => fleet::Report::Count {
            kind: match kind {
                host::CountKind::Admitted => fleet::CountKind::Admitted,
                host::CountKind::Ended => fleet::CountKind::Ended,
                host::CountKind::LlmStarted => fleet::CountKind::LlmStarted,
                host::CountKind::LlmRetried => fleet::CountKind::LlmRetried,
                host::CountKind::LlmFinished => fleet::CountKind::LlmFinished,
                host::CountKind::ToolStarted => fleet::CountKind::ToolStarted,
                host::CountKind::ToolFinished => fleet::CountKind::ToolFinished,
                host::CountKind::CheckStarted => fleet::CountKind::CheckStarted,
                host::CountKind::CheckFinished => fleet::CountKind::CheckFinished,
                host::CountKind::TextArrived => fleet::CountKind::TextArrived,
            },
            elapsed,
            count,
        },
    }
}
