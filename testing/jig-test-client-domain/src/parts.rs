//! A part routes by its connector, without reading the application's value.
use crate::{Domain, Fact, Limits, Request, first, owner::Connector, route};
use jig_client::contract::Part;
use skein_lib::{Env, Queue};
pub(crate) fn show(root: &mut Domain, env: &Env<Limits>, part: Part, generation: u32, out: &mut Queue<Request>) {
    let number = connector(&part);
    match Connector::of(number) {
        Some(Connector::First) => route::first(root, env, first::Event::Show { part, generation }, out),
        Some(Connector::Second) => root.second.show(part, generation),
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
