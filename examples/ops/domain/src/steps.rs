//! Procedure-step outcomes sent to their numbered connector within the decision.
use crate::domain::Work;
use jig_ops_domain_connectors::Connector;
use skein_lib::Queue;

pub(crate) fn made(number: u16, task: u64, step: u64, child: Option<u64>, work: &mut Queue<Work>) {
    match jig_ops_domain_connectors::connector(number) {
        Some(Connector::Observability) => {
            work.push(Work::Observability(jig_ops_domain_observability::Event::ProcedureDelegateOutcome {
                task,
                step,
                child,
            }));
        }
        Some(Connector::Infrastructure) => {
            work.push(Work::Infrastructure(jig_ops_domain_infrastructure::Event::ProcedureDelegateOutcome {
                task,
                step,
                child,
            }));
        }
        None => {}
    }
}
