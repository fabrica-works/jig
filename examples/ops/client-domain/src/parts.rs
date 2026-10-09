//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides, family addresses and application vocabulary.
use crate::{Domain, Fact, Limits, Request, infrastructure, observability, route};
use jig_client::contract::Part;
use skein_lib::{Env, Queue};
pub(crate) fn show(
    root: &mut Domain,
    env: &Env<Limits>,
    part: Part,
    generation: u32,
    scope: jig_client::contract::PartScope,
    out: &mut Queue<Request>,
) {
    let number = connector(&part);
    match jig_ops_connectors::connector(number) {
        Some(jig_ops_connectors::Connector::Infrastructure) => route::infrastructure(
            root,
            env,
            infrastructure::Event::Show {
                part,
                generation,
                scope: match scope {
                    jig_client::contract::PartScope::Project(project) => infrastructure::Scope::Project(project),
                    jig_client::contract::PartScope::Deployment => infrastructure::Scope::Deployment,
                },
            },
            out,
        ),
        Some(jig_ops_connectors::Connector::Observability) => {
            route::observability(
                root,
                env,
                observability::Event::Show {
                    part,
                    generation,
                    scope: match scope {
                        jig_client::contract::PartScope::Project(project) => observability::Scope::Project(project),
                        jig_client::contract::PartScope::Deployment => observability::Scope::Deployment,
                    },
                },
                out,
            );
        }
        None => out.push(Request::Fact { fact: Fact::UnknownConnector }),
    }
}
#[must_use]
pub fn connector(part: &Part) -> u16 {
    match part {
        Part::Connector { connector }
        | Part::Resource { connector, .. }
        | Part::Kind { connector, .. }
        | Part::Panel { connector, .. }
        | Part::Result { connector, .. }
        | Part::Effect { connector, .. }
        | Part::Hold { connector, .. }
        | Part::Project { connector, .. }
        | Part::Deployment { connector } => *connector,
    }
}
