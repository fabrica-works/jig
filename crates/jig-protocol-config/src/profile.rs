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
    pub hub: host::Limits,
    pub link: Link,
    pub token: u32,
}
#[must_use]
pub fn profile(profile: Profile) -> Limits {
    match profile {
        Profile::Small => small::LIMITS,
        Profile::Standard => standard::LIMITS,
    }
}
