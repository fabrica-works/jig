//! Bounded deployment facts read independently of the diagnostic drain
//! (client/domain/pages.md, section 7; domain/engine.md, section 8).

use skein_lib::{Duration, Map, Queue, Time, Token, Wall};

/// The owner's reason for refusing a worker's hello.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HelloRefusal {
    /// Channel zero belongs to the engine's configured host.
    ReservedChannel,
    /// The agent cannot hold a configured charter for this host kind.
    Agent { why: crate::AgentShortfall },
    /// The connected worker set has no admission room.
    Workers { limit: u32 },
    /// A worker must stop strictly before the engine's grace expires.
    StopBound { grace: Duration },
}

/// One refused hello, retaining no workstreams, listed payloads or reply rights.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HelloRefusalRecord {
    pub channel: Token,
    pub at: Wall,
    pub slots: u32,
    pub stop_bound: Duration,
    pub why: HelloRefusal,
}

/// A lost channel's identity and admitted slots remain available for display.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct LostHost {
    pub(crate) channel: Token,
    pub(crate) slots: u32,
    pub(crate) draining: bool,
    pub(crate) until: Time,
}

pub(crate) fn lost(domain: &mut crate::Domain, host: LostHost) {
    if domain.lost_hosts.capacity() == 0 {
        domain.host_history_lost = domain.host_history_lost.saturating_add(1);
        return;
    }
    if domain.lost_hosts.len() == domain.lost_hosts.capacity() && !domain.lost_hosts.contains_key(&host.channel) {
        let mut oldest: Option<(Time, Token)> = None;
        for (channel, record) in &domain.lost_hosts {
            let earlier = match oldest {
                Some(value) => (record.until, *channel) < value,
                None => true,
            };
            if earlier {
                oldest = Some((record.until, *channel));
            }
        }
        let channel = oldest.expect("a full nonempty lost history").1;
        domain.lost_hosts.remove(&channel);
        domain.host_history_lost = domain.host_history_lost.saturating_add(1);
    }
    domain.lost_hosts.insert(host.channel, host).expect("bounded lost host history");
}

pub(crate) fn refused(domain: &mut crate::Domain, record: HelloRefusalRecord) {
    if domain.hello_refusals.capacity() == 0 {
        domain.hello_history_lost = domain.hello_history_lost.saturating_add(1);
        return;
    }
    if domain.hello_refusals.room() == 0 {
        domain.hello_refusals.pop();
        domain.hello_history_lost = domain.hello_history_lost.saturating_add(1);
    }
    domain.hello_refusals.push(record);
}

pub(crate) fn worst_case(capacity: u32) -> Option<u64> {
    Map::<Token, LostHost>::worst_case(capacity)?.checked_add(Queue::<HelloRefusalRecord>::worst_case(capacity)?)
}
