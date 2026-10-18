//! Named consistent limits, in each owner's types (configuration.md, 6).
mod small;
mod standard;
use jig_domain_core as core;
use jig_domain_host as host;
use skein_lib::Duration;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Profile {
    Small,
    Standard,
}
/// The channel components' own numbers (channel.md, 11).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Link {
    pub chunk: u32,
    pub name: u32,
    pub secret: u32,
    pub names: u32,
    pub opening: u32,
    pub handshake: Duration,
    pub hello: Duration,
    pub ping: Duration,
    pub silence: Duration,
    pub stall: Duration,
    pub backoff: Duration,
    pub ceiling: Duration,
    pub proved: Duration,
    pub busy: Duration,
    pub facts: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub core: core::Limits,
    pub store: jig_protocol_store::Limits,
    pub hub: host::Limits,
    pub link: Link,
    pub token: u32,
    pub driver: skein_oauth_accounts::Limits,
    pub io: skein_io::Limits,
    pub journal: skein_lib::JournalLimits,
    pub routes: u32,
    pub clients: jig_protocol_engine_clients::Limits,
    pub clients_http: jig_protocol_engine_clients_http::Limits,
}
#[must_use]
pub fn profile(profile: Profile) -> Limits {
    match profile {
        Profile::Small => small::LIMITS,
        Profile::Standard => standard::LIMITS,
    }
}

/// A document retains every name the engine can admit.
pub(crate) const fn name_segments(tasks: &jig_domain_tasks::Limits, authority: &jig_domain_authority::Limits) -> u32 {
    let task =
        if tasks.hold_segments > tasks.authority_segments { tasks.hold_segments } else { tasks.authority_segments };
    if authority.segments > task { authority.segments } else { task }
}
