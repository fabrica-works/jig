//! One page held until its commit, with one record stopped at at most one site.
use super::{Patching, Pending, refuse, state, take};
use crate::patching::{
    Counts,
    machines::FamilyIx,
    plan::Planned,
    walk::{self, Stop},
};
use crate::{
    Below, Cursor, Event, KeptMap, Limits, OpenRefusal, Owner, Store, codec, commits, format, keys, open, pages,
    secrets,
};
use alloc::boxed::Box;
use jig_domain_core::{CoreKey, CoreRecord, Deployment, Key, Record};
use skein_codec::patch::{Halt, Patcher};
use skein_lib::{Env, List, Queue, Token, bytes};

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
    Commit { map: KeptMap, number: u64 },
}
#[derive(Debug)]
pub(crate) struct Walking {
    pending: Box<Pending>,
    before: List<crate::patching::check::Counted>,
    header: Deployment,
    next: u64,
    secrets_next: u64,
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
    refusal: Option<OpenRefusal>,
}
impl Walking {
    fn step(&self) -> Option<Planned> {
        self.pending.plan.steps().get(self.step).copied()
    }
    pub(super) fn committed(&mut self, map: KeptMap, number: u64) -> bool {
        match self.waiting {
            Waiting::Commit { map: expected_map, number: expected } if map == expected_map && number == expected => {
                self.cursor = self.through.take();
                self.counts.commits = self.counts.commits.checked_add(1).expect("commit count bounded by number");
                self.waiting = Waiting::Ready;
                true
            }
            Waiting::Ready | Waiting::Site(_) | Waiting::Commit { .. } => false,
        }
    }
    fn refused(&self, key: &[u8], halt: Halt) -> OpenRefusal {
        let step = self.step().expect("a running record has a plan step");
        let key = bytes::copy_of(
            relative(&self.pending.machines, FamilyIx(self.family), key).expect("walk key has its prefix"),
        );
        OpenRefusal::Patch { owner: step.owner, patch: step.patch, key, halt }
    }
    pub(super) fn answer(&mut self, patcher: Box<Patcher>, result: Result<(), Halt>) -> Result<(), OpenRefusal> {
        let family = match self.waiting {
            Waiting::Site(family) => family,
            Waiting::Ready | Waiting::Commit { .. } => return Err(OpenRefusal::Failed),
        };
        walk::give(&mut self.pending.machines, family, patcher);
        let running = self.running.as_mut().expect("site has a record");
        match walk::resume(&mut self.pending.machines, &mut running.walk, result) {
            Ok(stop) => {
                running.stop = stop;
                self.waiting = Waiting::Ready;
                Ok(())
            }
            Err(halt) => Err(self.refused(&self.running.as_ref().expect("site has a record").key, halt)),
        }
    }
}
pub(super) fn opened(
    store: &mut Store,
    pending: Box<Pending>,
    before: List<crate::patching::check::Counted>,
) -> Option<Walking> {
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
        before,
        header,
        next: last.checked_add(1)?,
        secrets_next: store.stores.secrets.last().checked_add(1)?,
        step: 0,
        selected,
        family: 0,
        range: 0,
        cursor: None,
        page: Queue::with_capacity(page_capacity(&store.limits)),
        more: false,
        loaded: false,
        running: None,
        deferred: None,
        ops: List::with_capacity(ops_capacity(&store.limits)),
        bytes: 26,
        through: None,
        counts: Counts { records: 0, commits: 0 },
        waiting: Waiting::Ready,
        refusal: None,
    })
}
fn interval(store: &Store, walking: &Walking) -> Option<skein_kv::Range> {
    let machine = walking.pending.machines.get(FamilyIx(walking.family));
    let range = walking.pending.machines.keys(FamilyIx(walking.family)).get(walking.range)?;
    pages::kept_interval(store, machine.owner, walking.pending.machines.map(FamilyIx(walking.family)), range)
}
/// No request is issued here. Each iteration consumes a row, a range, a
/// family or a plan step; commits and sites leave through the step's outputs.
fn drive(store: &mut Store, walking: &mut Walking, above: &mut Queue<Event>) -> Option<bool> {
    match walking.waiting {
        Waiting::Site(_) | Waiting::Commit { .. } => return Some(false),
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
        .checked_add(usize::try_from(ops_capacity(&store.limits)).ok()?)?
        .checked_mul(3)?;
    for _ in 0..turns {
        let Some(step) = walking.step() else { return Some(true) };
        if walking.family == families {
            walking.pending.machines.advance(step);
            // Keep found as the row's levels before the walk, for the manifest.
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
                    let key =
                        bytes::copy_of(relative(&walking.pending.machines, FamilyIx(walking.family), &running.key)?);
                    let patcher = walk::take(&mut walking.pending.machines, family);
                    walking.waiting = Waiting::Site(family);
                    above.push(Event::Site { owner: store.directory.owner, of, key, site, at, patcher });
                    return Some(false);
                }
                Stop::Done => {}
            }
            let running = walking.running.take().expect("just inspected");
            finish(walking, running)?;
        }
        if !append(store, walking)? {
            return Some(false);
        }
        if let Some(row) = walking.page.pop() {
            let family = FamilyIx(walking.family);
            let root = walking.pending.machines.keys(family).get(walking.range)?.root;
            start(walking, family, root, row)?;
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
        let map = walking.pending.machines.map(FamilyIx(walking.family));
        let page = pages::walk_map(store, map, &[range], walking.cursor.as_ref(), puts(&store.limits, map))?;
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
    let map = walking.pending.machines.map(FamilyIx(walking.family));
    let limits = pages::map_limits(store, map);
    let size = u64::try_from(put.key.len()).ok()?.checked_add(u64::try_from(put.value.len()).ok()?)?.checked_add(9)?;
    // No part of this page is submitted on a put that cannot fit alone.
    if size.checked_add(26)? > u64::from(limits.commit)
        || puts(&store.limits, map) == 0
        || put.value.len() > usize::try_from(limits.value).ok()?
    {
        let key = bytes::copy_of(relative(&walking.pending.machines, FamilyIx(walking.family), &put.key)?);
        walking.refusal = Some(OpenRefusal::Check { why: crate::Check::Large { key } });
        return None;
    }
    let total = walking.bytes.checked_add(size)?;
    if walking.ops.len() == puts(&store.limits, map) || total > u64::from(limits.commit) {
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
        | Patching::Checking(_)
        | Patching::Manifest(_)
        | Patching::Leaving { .. }
        | Patching::Switching(_) => unreachable!("resume routed Walking"),
    };
    let Some(done) = drive(store, &mut walking, above) else {
        return open::refuse(
            store,
            store.directory.owner,
            walking.refusal.take().unwrap_or(OpenRefusal::Failed),
            above,
        );
    };
    if done {
        let before = walking.pending.found;
        let header = walking.header;
        let number = walking.next;
        let counts = walking.before;
        return seal(store, env, header, number, before, counts, above, below);
    }
    match walking.waiting {
        Waiting::Ready if !walking.ops.is_empty() => {
            let map = walking.pending.machines.map(FamilyIx(walking.family));
            let counter = match map {
                KeptMap::Store => &mut walking.next,
                KeptMap::Secrets => &mut walking.secrets_next,
            };
            let number = *counter;
            let Some(next) = number.checked_add(1) else { return refuse(store, above) };
            *counter = next;
            walking.waiting = Waiting::Commit { map, number };
            let ops =
                core::mem::replace(&mut walking.ops, List::with_capacity(ops_capacity(&store.limits))).into_boxed();
            walking.bytes = 26;
            state(store, Patching::Walking(walking));
            match map {
                KeptMap::Store => commits::submit(store, env, number, ops, above, below),
                KeptMap::Secrets => {
                    let kv_env = crate::directory::env(store, env, true);
                    let mut events = Queue::with_capacity(skein_kv::MAX_OUT.above);
                    skein_kv::down(
                        &mut store.stores.secrets,
                        &kv_env,
                        skein_kv::Request::Commit { owner: Token::new(number), ops },
                        &mut events,
                        &mut below.secrets,
                    );
                    crate::directory::events_up(store, true, &mut events, above, below);
                }
            }
        }
        Waiting::Ready | Waiting::Site(_) | Waiting::Commit { .. } => state(store, Patching::Walking(walking)),
    }
}
#[expect(clippy::too_many_arguments, reason = "the seal carries the levels and counts into the check")]
fn seal(
    store: &mut Store,
    env: &Env<Limits>,
    mut header: Deployment,
    number: u64,
    before: crate::patching::plan::Levels,
    counts: List<crate::patching::check::Counted>,
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
    let mut format = format::current(store.application.version, header.id);
    format.levels = format::recorded(crate::patching::plan::binary(&store.patch_sets));
    let Ok(format) = format.encode() else {
        return refuse(store, above);
    };
    let Some(header) = codec::encode(&Record::Core(CoreRecord::Deployment(header)), &store.codec, &store.names) else {
        return refuse(store, above);
    };
    let mut size = 26_u64;
    let Some(layouts_key) = keys::encode_own(&keys::Own::Layouts, store.limits.store.key) else {
        return refuse(store, above);
    };
    if store.limits.store.ops < 3 {
        return refuse(store, above);
    }
    let mut ops = List::with_capacity(3);
    for (key, value) in [(format_key, format), (header_key, header), (layouts_key, store.layouts.encode())] {
        let Some(total) = put_size(&key, &value, size) else { return refuse(store, above) };
        size = total;
        if size > u64::from(store.limits.store.commit) {
            return refuse(store, above);
        }
        ops.push(skein_kv::Op::Put { key, value }).expect("three sealing puts");
    }
    state(store, Patching::Sealing { number, before, counts });
    commits::submit(store, env, number, ops.into_boxed(), above, below);
}

fn put_size(key: &[u8], value: &[u8], before: u64) -> Option<u64> {
    before.checked_add(u64::try_from(key.len()).ok()?)?.checked_add(u64::try_from(value.len()).ok()?)?.checked_add(9)
}

pub(super) fn worst_case(limits: &Limits) -> Option<u64> {
    let puts = ops_capacity(limits);
    let rows = page_capacity(limits);
    let page = limits.store.page.bytes.max(limits.secrets.page.bytes);
    let key = limits.store.key.max(limits.secrets.key);
    let value = limits.store.value.max(limits.secrets.value);
    let commit = limits.store.commit.max(limits.secrets.commit);
    u64::try_from(size_of::<Walking>())
        .ok()?
        .checked_add(Queue::<pages::WalkRow>::worst_case(rows)?)?
        .checked_add(List::<pages::WalkRow>::worst_case(rows)?)?
        .checked_add(List::<skein_kv::Op>::worst_case(puts.max(3))?.checked_mul(2)?)?
        // Loaded rows (including their key cursors), output puts, and a
        // deferred finished record can coexist; page loads make one temporary.
        .checked_add(u64::from(page).checked_mul(2)?)?
        .checked_add(u64::from(key).checked_mul(u64::from(rows))?.checked_mul(3)?)?
        .checked_add(u64::from(commit))?
        .checked_add(u64::from(value).checked_mul(3)?)?
        .checked_add(u64::from(key).checked_mul(8)?)
}

fn start(walking: &mut Walking, family: FamilyIx, root: &str, row: pages::WalkRow) -> Option<()> {
    let step = walking.step().expect("a record has a plan step");
    let (walk, stopped) = match walk::start(&mut walking.pending.machines, step, family, root, row.row.value) {
        Ok(started) => started,
        Err(halt) => {
            walking.refusal = Some(walking.refused(&row.row.key, halt));
            return None;
        }
    };
    walking.running = Some(Running { key: row.row.key, cursor: row.cursor, walk, stop: stopped });
    Some(())
}

fn finish(walking: &mut Walking, running: Running) -> Option<()> {
    let value = match walk::finished(&mut walking.pending.machines, &running.walk) {
        Ok(value) => bytes::copy_of(value),
        Err(halt) => {
            walking.refusal = Some(walking.refused(&running.key, halt));
            return None;
        }
    };
    walking.deferred = Some(Put { key: running.key, value, cursor: running.cursor });
    Some(())
}

fn puts(limits: &Limits, map: KeptMap) -> u32 {
    match map {
        KeptMap::Store => limits.journal.writes.min(limits.store.ops),
        KeptMap::Secrets => limits.secrets.ops,
    }
}
fn ops_capacity(limits: &Limits) -> u32 {
    puts(limits, KeptMap::Store).max(puts(limits, KeptMap::Secrets))
}
fn page_capacity(limits: &Limits) -> u32 {
    puts(limits, KeptMap::Store)
        .min(limits.store.page.bytes)
        .max(puts(limits, KeptMap::Secrets).min(limits.secrets.page.bytes))
}
fn relative<'a>(machines: &crate::patching::machines::Machines, family: FamilyIx, key: &'a [u8]) -> Option<&'a [u8]> {
    if machines.get(family).owner != Owner::Application {
        return Some(key);
    }
    match machines.map(family) {
        KeptMap::Store => keys::of_application(key),
        KeptMap::Secrets => secrets::of_application(key),
    }
}
