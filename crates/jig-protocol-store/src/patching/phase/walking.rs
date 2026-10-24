//! One page held until its commit, with one record stopped at at most one site.
use super::{Patching, Pending, refuse, state, take};
use crate::patching::{
    Counts,
    machines::FamilyIx,
    plan::Planned,
    walk::{self, Stop},
};
use crate::{Below, Cursor, Event, Limits, Owner, Store, codec, commits, format, keys, open, pages};
use alloc::boxed::Box;
use jig_domain_core::{CoreKey, CoreRecord, Deployment, Key, Record};
use skein_codec::patch::{Halt, Patcher};
use skein_lib::{Env, List, Queue, bytes};

#[derive(Debug)]
struct Running {
    key: Box<[u8]>,
    cursor: Cursor,
    walk: walk::Walk,
    stop: Stop,
}
#[derive(Debug)]
struct Put {
    key: Box<[u8]>,
    value: Box<[u8]>,
    cursor: Cursor,
}
#[derive(Debug)]
enum Waiting {
    Ready,
    Site(FamilyIx),
    Commit(u64),
}
#[derive(Debug)]
pub(crate) struct Walking {
    pending: Box<Pending>,
    header: Deployment,
    next: u64,
    step: usize,
    selected: List<bool>,
    family: u32,
    range: usize,
    cursor: Option<Cursor>,
    page: Queue<pages::WalkRow>,
    more: bool,
    loaded: bool,
    running: Option<Running>,
    deferred: Option<Put>,
    ops: List<skein_kv::Op>,
    bytes: u64,
    through: Option<Cursor>,
    counts: Counts,
    waiting: Waiting,
}
impl Walking {
    fn step(&self) -> Option<Planned> {
        self.pending.plan.steps().get(self.step).copied()
    }
    pub(super) fn committed(&mut self, number: u64) -> bool {
        match self.waiting {
            Waiting::Commit(expected) if number == expected => {
                self.cursor = self.through.take();
                self.counts.commits = self.counts.commits.checked_add(1).expect("commit count bounded by number");
                self.waiting = Waiting::Ready;
                true
            }
            Waiting::Ready | Waiting::Site(_) | Waiting::Commit(_) => false,
        }
    }
    pub(super) fn answer(&mut self, patcher: Box<Patcher>, result: Result<(), Halt>) -> bool {
        let family = match self.waiting {
            Waiting::Site(family) => family,
            Waiting::Ready | Waiting::Commit(_) => return false,
        };
        walk::give(&mut self.pending.machines, family, patcher);
        let running = self.running.as_mut().expect("site has a record");
        match walk::resume(&mut self.pending.machines, &mut running.walk, result) {
            Ok(stop) => {
                running.stop = stop;
                self.waiting = Waiting::Ready;
                true
            }
            Err(_) => false,
        }
    }
}
pub(super) fn opened(store: &mut Store, pending: Box<Pending>) -> Option<Walking> {
    let key = Key::Core(CoreKey::Deployment);
    let encoded = keys::encode(&key, store.limits.store.key)?;
    let bytes = open::get(store, store.directory.owner, encoded).ok()??;
    let record = codec::decode(&key, &bytes, &store.codec, &store.names)?;
    let header = match record {
        Record::Core(CoreRecord::Deployment(header)) => header,
        Record::Core(
            CoreRecord::Settings(_)
            | CoreRecord::Projection(_)
            | CoreRecord::Call(_)
            | CoreRecord::RunProof(_)
            | CoreRecord::Turn(_)
            | CoreRecord::Terminal(_)
            | CoreRecord::ProposalDecision(_)
            | CoreRecord::EscalationDecision(_),
        )
        | Record::Tasks(_)
        | Record::People(_)
        | Record::Notes(_)
        | Record::Accounts(_) => return None,
    };
    let last = store.stores.store.last();
    if header.commits != last {
        return None;
    }
    let selected = match pending.plan.steps().first() {
        Some(step) => pending.machines.walked(*step),
        None => List::with_capacity(0),
    };
    Some(Walking {
        pending,
        header,
        next: last.checked_add(1)?,
        step: 0,
        selected,
        family: 0,
        range: 0,
        cursor: None,
        page: Queue::with_capacity(
            store.limits.journal.writes.min(store.limits.store.ops).min(store.limits.store.page.bytes),
        ),
        more: false,
        loaded: false,
        running: None,
        deferred: None,
        ops: List::with_capacity(store.limits.journal.writes.min(store.limits.store.ops)),
        bytes: 26,
        through: None,
        counts: Counts { records: 0, commits: 0 },
        waiting: Waiting::Ready,
    })
}
fn interval(store: &Store, walking: &Walking) -> Option<skein_kv::Range> {
    let machine = walking.pending.machines.get(FamilyIx(walking.family));
    let range = walking.pending.machines.keys(FamilyIx(walking.family)).get(walking.range)?;
    if machine.owner == Owner::Application {
        Some(skein_kv::Range {
            start: keys::application(range.from, store.limits.store.key)?,
            end: Some(keys::application(range.to, store.limits.store.key)?),
        })
    } else {
        Some(skein_kv::Range { start: bytes::copy_of(range.from), end: Some(bytes::copy_of(range.to)) })
    }
}
/// No request is issued here. Each iteration consumes a row, a range, a
/// family or a plan step; commits and sites leave through the step's outputs.
fn drive(store: &mut Store, walking: &mut Walking, above: &mut Queue<Event>) -> Option<bool> {
    match walking.waiting {
        Waiting::Site(_) | Waiting::Commit(_) => return Some(false),
        Waiting::Ready => {}
    }
    let families = walking.pending.machines.len();
    let mut ranges = 0_usize;
    for ix in 0..families {
        ranges = ranges.checked_add(walking.pending.machines.keys(FamilyIx(ix)).len())?;
    }
    let steps = walking.pending.plan.steps().len();
    let turns = ranges
        .checked_add(usize::try_from(families).ok()?)?
        .checked_add(1)?
        .checked_mul(steps.checked_add(1)?)?
        .checked_add(usize::try_from(store.limits.journal.writes.min(store.limits.store.ops)).ok()?)?
        .checked_mul(3)?;
    for _ in 0..turns {
        let Some(step) = walking.step() else { return Some(true) };
        if walking.family == families {
            walking.pending.machines.advance(step);
            walking.pending.found.set(step.owner, Some(step.patch));
            above.push(Event::Patched {
                owner: store.directory.owner,
                of: step.owner,
                patch: step.patch,
                counts: walking.counts,
            });
            walking.step = walking.step.checked_add(1)?;
            walking.family = 0;
            walking.counts = Counts { records: 0, commits: 0 };
            if let Some(next) = walking.step() {
                walking.selected = walking.pending.machines.walked(next);
            }
            // At most one Patched per invocation, so MAX_OUT stays independent
            // of the number of patches in the binary. fire resumes the next.
            return Some(false);
        }
        if !*walking.selected.get(walking.family).expect("one bit per machine") {
            walking.family = walking.family.checked_add(1)?;
            continue;
        }
        if walking.range == walking.pending.machines.keys(FamilyIx(walking.family)).len() {
            walking.family = walking.family.checked_add(1)?;
            walking.range = 0;
            continue;
        }
        if let Some(running) = &walking.running {
            match running.stop {
                Stop::Site { family, site, at } => {
                    let of = walking.pending.machines.get(family).owner;
                    let key = if of == Owner::Application {
                        bytes::copy_of(keys::of_application(&running.key)?)
                    } else {
                        running.key.clone()
                    };
                    let patcher = walk::take(&mut walking.pending.machines, family);
                    walking.waiting = Waiting::Site(family);
                    above.push(Event::Site { owner: store.directory.owner, of, key, site, at, patcher });
                    return Some(false);
                }
                Stop::Done => {}
            }
            let running = walking.running.take().expect("just inspected");
            let value = bytes::copy_of(walk::finished(&mut walking.pending.machines, &running.walk).ok()?);
            walking.deferred = Some(Put { key: running.key, value, cursor: running.cursor });
        }
        if !append(store, walking)? {
            return Some(false);
        }
        if let Some(row) = walking.page.pop() {
            let family = FamilyIx(walking.family);
            let root = walking.pending.machines.keys(family).get(walking.range)?.root;
            let (walk, stopped) = walk::start(&mut walking.pending.machines, step, family, root, row.row.value).ok()?;
            walking.running = Some(Running { key: row.row.key, cursor: row.cursor, walk, stop: stopped });
            continue;
        }
        if !walking.ops.is_empty() {
            return Some(false);
        }
        if walking.loaded && !walking.more {
            walking.range = walking.range.checked_add(1)?;
            walking.cursor = None;
            walking.loaded = false;
            continue;
        }
        let range = interval(store, walking)?;
        let page = pages::walk(
            store,
            &[range],
            walking.cursor.as_ref(),
            store.limits.journal.writes.min(store.limits.store.ops),
        )?;
        if page.rows.is_empty() && page.next.is_some() {
            return None;
        }
        walking.more = page.next.is_some();
        walking.loaded = true;
        for row in page.rows {
            walking.page.push(row);
        }
    }
    None
}
/// A finished record that does not fit waits for the preceding page's commit.
fn append(store: &Store, walking: &mut Walking) -> Option<bool> {
    let Some(put) = walking.deferred.take() else { return Some(true) };
    let size = u64::try_from(put.key.len()).ok()?.checked_add(u64::try_from(put.value.len()).ok()?)?.checked_add(9)?;
    // M4: OpenRefusal::Check. No part of this page is submitted on a
    // put that cannot fit alone, or a walk halt.
    if size.checked_add(26)? > u64::from(store.limits.store.commit)
        || store.limits.journal.writes.min(store.limits.store.ops) == 0
        || put.value.len() > usize::try_from(store.limits.store.value).ok()?
    {
        return None;
    }
    let total = walking.bytes.checked_add(size)?;
    if walking.ops.len() == store.limits.journal.writes.min(store.limits.store.ops)
        || total > u64::from(store.limits.store.commit)
    {
        walking.deferred = Some(put);
        return Some(false);
    }
    walking.bytes = total;
    walking.through = Some(put.cursor);
    walking.ops.push(skein_kv::Op::Put { key: put.key, value: put.value }).ok()?;
    walking.counts.records = walking.counts.records.checked_add(1)?;
    Some(true)
}
pub(super) fn resume(store: &mut Store, env: &Env<Limits>, above: &mut Queue<Event>, below: &mut Below) {
    let phase = take(store);
    let mut walking = match *phase {
        Patching::Walking(walking) => walking,
        Patching::Closing { .. }
        | Patching::Dropping(_)
        | Patching::Copying(_)
        | Patching::Mapping { .. }
        | Patching::Recovering { .. }
        | Patching::Sealing { .. }
        | Patching::Manifest(_)
        | Patching::Leaving { .. }
        | Patching::Switching(_) => unreachable!("resume routed Walking"),
    };
    let Some(done) = drive(store, &mut walking, above) else { return refuse(store, above) };
    if done {
        return seal(store, env, walking.header, walking.next, above, below);
    }
    match walking.waiting {
        Waiting::Ready if !walking.ops.is_empty() => {
            let number = walking.next;
            let Some(next) = number.checked_add(1) else { return refuse(store, above) };
            walking.next = next;
            walking.waiting = Waiting::Commit(number);
            let ops = core::mem::replace(
                &mut walking.ops,
                List::with_capacity(store.limits.journal.writes.min(store.limits.store.ops)),
            )
            .into_boxed();
            walking.bytes = 26;
            state(store, Patching::Walking(walking));
            commits::submit(store, env, number, ops, above, below);
        }
        Waiting::Ready | Waiting::Site(_) | Waiting::Commit(_) => state(store, Patching::Walking(walking)),
    }
}
fn seal(
    store: &mut Store,
    env: &Env<Limits>,
    mut header: Deployment,
    number: u64,
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    header.commits = number;
    let Some(format_key) = keys::encode_own(&keys::Own::Format, store.limits.store.key) else {
        return refuse(store, above);
    };
    let Some(header_key) = keys::encode(&Key::Core(CoreKey::Deployment), store.limits.store.key) else {
        return refuse(store, above);
    };
    let Ok(format) = format::current(store.application.version, header.id).encode() else {
        return refuse(store, above);
    };
    let Some(header) = codec::encode(&Record::Core(CoreRecord::Deployment(header)), &store.codec, &store.names) else {
        return refuse(store, above);
    };
    let mut size = 26_u64;
    let mut ops = List::with_capacity(2);
    for (key, value) in [(format_key, format), (header_key, header)] {
        let Some(total) = put_size(&key, &value, size) else { return refuse(store, above) };
        size = total;
        if size > u64::from(store.limits.store.commit) {
            return refuse(store, above);
        }
        ops.push(skein_kv::Op::Put { key, value }).expect("two sealing puts");
    }
    state(store, Patching::Sealing { number });
    commits::submit(store, env, number, ops.into_boxed(), above, below);
}

fn put_size(key: &[u8], value: &[u8], before: u64) -> Option<u64> {
    before.checked_add(u64::try_from(key.len()).ok()?)?.checked_add(u64::try_from(value.len()).ok()?)?.checked_add(9)
}

pub(super) fn worst_case(limits: &Limits) -> Option<u64> {
    let puts = limits.journal.writes.min(limits.store.ops);
    let rows = puts.min(limits.store.page.bytes);
    u64::try_from(size_of::<Walking>())
        .ok()?
        .checked_add(Queue::<pages::WalkRow>::worst_case(rows)?)?
        .checked_add(List::<pages::WalkRow>::worst_case(rows)?)?
        .checked_add(List::<skein_kv::Op>::worst_case(puts.max(2))?.checked_mul(2)?)?
        // Loaded rows (including their key cursors), output puts, and a
        // deferred finished record can coexist; page loads make one temporary.
        .checked_add(u64::from(limits.store.page.bytes).checked_mul(2)?)?
        .checked_add(u64::from(limits.store.key).checked_mul(u64::from(rows))?.checked_mul(3)?)?
        .checked_add(u64::from(limits.store.commit))?
        .checked_add(u64::from(limits.store.value).checked_mul(3)?)?
        .checked_add(u64::from(limits.store.key).checked_mul(8)?)
}
