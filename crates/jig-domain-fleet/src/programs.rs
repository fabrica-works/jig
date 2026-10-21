//! Fan-out belongs to current channels, never replayed onto replacements (accounts.md, 3.4).
use crate::sources::{Pending, Reply, Sources};
use crate::{Domain, HostKind, Request, SourceName};
use skein_lib::{List, Queue, Token};

pub(crate) fn check(domain: &mut Domain, name: SourceName, out: &mut Queue<Request>) {
    for (_, id) in &domain.tokens {
        let host = domain.channels.get_mut(*id).expect("current channel");
        if host.kind == HostKind::Engine {
            continue;
        }
        if host.sources.sent.contains(&name) {
            out.push(Request::Check { channel: host.token, name });
        } else if let Some(desired) = &host.sources.desired
            && desired.contains(&name)
        {
            host.sources.checks.insert(name).expect("one check per declared name");
        }
    }
}
pub(crate) fn sign_out(domain: &mut Domain, source: u32, name: SourceName, out: &mut Queue<Request>) {
    let count = domain.worker_names.len();
    let mut workers = List::with_capacity(count);
    for position in 0..count {
        workers
            .push(domain.worker_names.worker(position).expect("admitted configured position"))
            .expect("configured worker count");
    }
    out.push(Request::ProgramTargets { source, workers: workers.into_boxed() });
    for position in 0..count {
        let worker = domain.worker_names.worker(position).expect("admitted configured position");
        let mut target = None;
        for (_, id) in &domain.tokens {
            let host = domain.channels.get(*id).expect("current channel");
            if host.worker == Some(position) {
                target = Some(*id);
            }
        }
        let Some(id) = target else {
            out.push(Request::ProgramSignedOut { source, worker, result: crate::ProgramResult::NotInContact });
            continue;
        };
        let host = domain.channels.get_mut(id).expect("current configured worker channel");
        if let Some(pending) = host.sources.pending.get_mut(&name) {
            let reply = core::mem::replace(&mut pending.reply, Reply::Cleanup);
            pending.reply = match reply {
                Reply::Cleanup => Reply::Program { source, worker },
                Reply::Direct(to) => Reply::DirectProgram { to, source, worker },
                kept @ (Reply::Program { .. } | Reply::DirectProgram { .. }) => {
                    out.push(Request::ProgramSignedOut { source, worker, result: crate::ProgramResult::Failed });
                    kept
                }
            };
            continue;
        }
        if host.sources.pending.len() == host.sources.pending.capacity() {
            out.push(Request::ProgramSignedOut { source, worker, result: crate::ProgramResult::Failed });
            continue;
        }
        host.sources
            .pending
            .insert(name, Pending { reply: Reply::Program { source, worker }, sent: false })
            .expect("one pending operation per admitted name");
        flush(&mut host.sources, host.token, out);
    }
}
/// After reconnect, desired absence is enforced once per name on this channel.
/// A failed cleanup remains visible/restrictive; it never completes an old answer.
pub(crate) fn cleanup(domain: &mut Domain, channel: Token, out: &mut Queue<Request>) {
    let Some(id) = domain.tokens.get(&channel) else { return };
    let host = domain.channels.get_mut(*id).expect("current channel");
    if host.kind == HostKind::Engine || host.sources.awaiting() {
        return;
    }
    for name in &domain.cleanup_programs {
        if host.sources.pending.contains_key(name) || host.sources.results.contains_key(name) {
            continue;
        }
        let mut retained = false;
        for row in host.sources.logins() {
            if row.name == *name {
                retained = !crate::sources::absent(&row.state);
            }
        }
        if retained {
            host.sources
                .pending
                .insert(*name, Pending { reply: Reply::Cleanup, sent: false })
                .expect("cleanup names fit the declared source capacity");
        }
    }
    flush(&mut host.sources, channel, out);
}
pub(crate) fn flush(sources: &mut Sources, channel: Token, out: &mut Queue<Request>) {
    let mut pending = List::with_capacity(sources.pending.capacity());
    for (name, row) in &sources.pending {
        if !row.sent && sources.sent.contains(name) {
            pending.push(*name).expect("one per pending source");
        }
    }
    for name in &pending {
        let row = sources.pending.get_mut(name).expect("collected from pending");
        row.sent = true;
        out.push(Request::SignOut { channel, name: *name });
    }
    let mut checked = List::with_capacity(sources.checks.capacity());
    for name in &sources.checks {
        if sources.sent.contains(name) {
            checked.push(*name).expect("one per check");
        }
    }
    for name in &checked {
        sources.checks.remove(name);
        if !sources.pending.contains_key(name) {
            out.push(Request::Check { channel, name: *name });
        }
    }
}

fn flushable(sources: &Sources) -> bool {
    for (name, row) in &sources.pending {
        if !row.sent && sources.sent.contains(name) {
            return true;
        }
    }
    for name in &sources.checks {
        if sources.sent.contains(name) {
            return true;
        }
    }
    false
}
pub(crate) fn ready(domain: &Domain) -> bool {
    for (_, id) in &domain.tokens {
        if let Some(host) = domain.channels.get(*id)
            && flushable(&host.sources)
        {
            return true;
        }
    }
    false
}
pub(crate) fn resume(domain: &mut Domain, out: &mut Queue<Request>) {
    for (_, id) in &domain.tokens {
        if let Some(host) = domain.channels.get_mut(*id)
            && flushable(&host.sources)
        {
            flush(&mut host.sources, host.token, out);
            return;
        }
    }
}
