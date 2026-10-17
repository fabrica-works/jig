//! Translate the decoded family into the engine root (client/domain/root.md, 7).
use alloc::boxed::Box;
use jig_client_test_protocol_wire::codec::{self, Undecodable};
use jig_test_connector as d;
/// Decode the owner body before handing its unchanged admission envelope to the root.
pub fn to_root(
    request: jig_protocol_engine_clients::ToApplication,
    bound: &d::Limits,
    rows: u32,
) -> Result<jig_test_domain::Event, Box<jig_protocol_engine_clients::Event>> {
    use jig_protocol_engine_clients::{Event, FromApplication, ToApplication};
    match request {
        ToApplication::Request { reply_to, sign_in, key, family, version, record, digest } => {
            let decoded = if family == 1 && version == jig_client_test_protocol_wire::VERSION {
                codec::request(&record, bound)
            } else {
                Err(Undecodable)
            };
            let Ok(decoded) = decoded else {
                return Err(Box::new(Event::Application(FromApplication::Undecodable { to: reply_to })));
            };
            Ok(jig_test_domain::Event::Application {
                reply_to,
                sign_in,
                key,
                scope: jig_domain_people::ApplicationScope::Project(decoded.project),
                number: family,
                digest,
                value: decoded.value,
            })
        }
        ToApplication::Read { reply, sign_in, family, version, record } => {
            let decoded = if family == 1 && version == jig_client_test_protocol_wire::VERSION {
                codec::read_value(&record, bound)
            } else {
                Err(Undecodable)
            };
            let Ok(subject) = decoded else {
                return Err(Box::new(Event::Application(FromApplication::Read { reply, family, record: None })));
            };
            Ok(jig_test_domain::Event::ReadApplication {
                reply,
                sign_in,
                scope: scope(subject),
                number: family,
                subject,
                window: jig_domain_core::documents::Window { rows, after: None },
            })
        }
        ToApplication::Watch { watcher, sign_in, family, version, record } => {
            let decoded = if family == 1 && version == jig_client_test_protocol_wire::VERSION {
                codec::subject_value(&record, bound)
            } else {
                Err(Undecodable)
            };
            let Ok(subject) = decoded else {
                return Err(Box::new(Event::WatchRefused { watcher, why: jig_domain_core::WatchRefusal::Unknown }));
            };
            Ok(jig_test_domain::Event::WatchApplication {
                watcher,
                sign_in,
                scope: scope(subject),
                number: family,
                subject,
                window: jig_domain_core::documents::Window { rows, after: None },
            })
        }
    }
}
fn scope(subject: d::ClientSubject) -> jig_domain_people::ApplicationScope {
    use jig_domain_people::ApplicationScope;
    match subject {
        d::ClientSubject::Object { project, .. }
        | d::ClientSubject::Panel { project, .. }
        | d::ClientSubject::Result { project, .. }
        | d::ClientSubject::Effect { project, .. }
        | d::ClientSubject::Outbox { project, .. }
        | d::ClientSubject::Hold { project, .. }
        | d::ClientSubject::Project { project } => ApplicationScope::Project(project),
        d::ClientSubject::Deployment => ApplicationScope::Deployment,
    }
}
