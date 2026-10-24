//! One decode step for every transcript read, including asynchronous sites (store.md 8.6).
use super::{Unread, body};
use crate::patching::walk::held_root;
use crate::transcripts::patches::{Machine, Machines};
use crate::{Owner, codec};
use alloc::boxed::Box;
use jig_domain_core::TurnRecord;
use skein_codec::{
    patch::{Halt, PlanIx, Site, Step},
    schema::DeclIx,
    value::Node,
    version_of,
};
use skein_lib::bytes;

#[derive(Debug)]
pub(crate) enum Decode {
    Done { turn: TurnRecord, size: u32 },
    Patching(Patch),
}
#[derive(Clone, Copy, Debug)]
struct Pass {
    plan: PlanIx,
    root: DeclIx,
    first: bool,
}
#[derive(Debug)]
enum Stage {
    Body(Pass),
    Turn { pass: Pass, holder: Option<Pass> },
}
#[derive(Debug)]
pub(crate) struct Patch {
    stage: Stage,
    turn: Option<TurnRecord>,
    size: u32,
    remaining: u64,
}
#[derive(Debug)]
pub(crate) enum Progress {
    Written { of: Owner, site: Site, at: Node },
    Done { turn: TurnRecord, size: u32 },
}
fn behind(machine: &Machine, record: &[u8]) -> Result<bool, Halt> {
    let version = version_of(record).or(Err(Halt::Order))?;
    let patcher = machine.patcher.as_ref().ok_or(Halt::Order)?;
    if version > machine.last || patcher.schema(version).is_none() {
        return Err(Halt::Order);
    }
    Ok(version < machine.last)
}
fn start(machine: &mut Machine, record: Box<[u8]>) -> Result<Pass, Halt> {
    let version = version_of(&record).or(Err(Halt::Order))?;
    let patcher = machine.patcher.as_mut().ok_or(Halt::Order)?;
    let plan = patcher.plan_from(version).ok_or(Halt::Order)?;
    let root = held_root(patcher, version, machine.last, machine.root.as_bytes())?;
    patcher.start(plan, root, record)?;
    Ok(Pass { plan, root, first: true })
}
fn turns(machine: Option<&Machine>) -> Result<u64, Halt> {
    match machine {
        Some(machine) => u64::from(machine.capacity.nodes)
            .checked_add(1)
            .ok_or(Halt::Room)?
            .checked_mul(u64::from(machine.capacity.versions))
            .ok_or(Halt::Room),
        None => Ok(0),
    }
}

impl Decode {
    pub(crate) fn begin(record: &[u8], machines: &mut Machines, limits: &codec::Limits) -> Result<Self, Unread> {
        let body = body(record)?;
        let size = u32::try_from(record.len()).or(Err(Unread::Lengths))?;
        let remaining = turns(machines.body.as_ref())
            .or(Err(Unread::Record))?
            .checked_add(turns(machines.turn.as_ref()).or(Err(Unread::Record))?)
            .ok_or(Unread::Record)?
            .checked_add(4)
            .ok_or(Unread::Record)?;
        if let Some(machine) = &mut machines.body
            && behind(machine, body).or(Err(Unread::Record))?
        {
            let pass = start(machine, bytes::copy_of(body)).or(Err(Unread::Record))?;
            return Ok(Self::Patching(Patch { stage: Stage::Body(pass), turn: None, size, remaining }));
        }
        let mut turn = codec::core::read_turn(body, &codec::core_limits(limits)).or(Err(Unread::Record))?;
        if let Some(machine) = &mut machines.turn
            && behind(machine, &turn.transcript).or(Err(Unread::Record))?
        {
            let record = core::mem::replace(&mut turn.transcript, Box::new([]));
            let size =
                size.checked_sub(u32::try_from(record.len()).or(Err(Unread::Lengths))?).ok_or(Unread::Lengths)?;
            let pass = start(machine, record).or(Err(Unread::Record))?;
            return Ok(Self::Patching(Patch {
                stage: Stage::Turn { pass, holder: None },
                turn: Some(turn),
                size,
                remaining,
            }));
        }
        Ok(Self::Done { turn, size })
    }
}
impl Patch {
    pub(crate) fn next(&mut self, machines: &mut Machines, limits: &codec::Limits) -> Result<Progress, Halt> {
        // Each pass stops at most once per node, followed by Done. The sum
        // of both capacities bounds the whole walk, including held turns.
        for _ in 0..self.remaining {
            self.remaining = self.remaining.checked_sub(1).ok_or(Halt::Order)?;
            match self.stage {
                Stage::Body(pass) => {
                    let machine = machines.body.as_mut().ok_or(Halt::Order)?;
                    let patcher = machine.patcher.as_mut().ok_or(Halt::Order)?;
                    match patcher.next()? {
                        Step::Written { site, at } => return Ok(Progress::Written { of: Owner::Jig, site, at }),
                        Step::Held { at, .. } => {
                            if pass.first
                                && let Some(turn) = &mut machines.turn
                            {
                                let (old, _) = patcher.parts();
                                let record = old.content(at)?;
                                if behind(turn, record)? {
                                    let pass_turn = start(turn, bytes::copy_of(record))?;
                                    self.stage = Stage::Turn { pass: pass_turn, holder: Some(pass) };
                                    continue;
                                }
                            }
                            let (old, new) = patcher.parts();
                            new.carry(old, at)?;
                        }
                        Step::Done => {
                            if let Some(next) = next_pass(machine, pass)? {
                                self.stage = Stage::Body(next);
                                continue;
                            }
                            let body = machine.patcher.as_mut().ok_or(Halt::Order)?.finish()?;
                            let size =
                                u32::try_from(body.len()).or(Err(Halt::Room))?.checked_add(12).ok_or(Halt::Room)?;
                            let turn =
                                codec::core::read_turn(body, &codec::core_limits(limits)).or(Err(Halt::Order))?;
                            return Ok(Progress::Done { turn, size });
                        }
                    }
                }
                Stage::Turn { pass, holder } => {
                    let machine = machines.turn.as_mut().ok_or(Halt::Order)?;
                    let patcher = machine.patcher.as_mut().ok_or(Halt::Order)?;
                    match patcher.next()? {
                        Step::Written { site, at } => return Ok(Progress::Written { of: Owner::Smith, site, at }),
                        Step::Held { at, .. } => {
                            let (old, new) = patcher.parts();
                            new.carry(old, at)?;
                        }
                        Step::Done => {
                            if let Some(next) = next_pass(machine, pass)? {
                                self.stage = Stage::Turn { pass: next, holder };
                                continue;
                            }
                            let record = machine.patcher.as_mut().ok_or(Halt::Order)?.finish()?;
                            match holder {
                                Some(pass) => {
                                    let holder = machines.body.as_mut().ok_or(Halt::Order)?;
                                    let (_, new) = holder.patcher.as_mut().ok_or(Halt::Order)?.parts();
                                    new.put_content(record)?;
                                    self.stage = Stage::Body(pass);
                                }
                                None => {
                                    let mut turn = self.turn.take().ok_or(Halt::Order)?;
                                    turn.transcript = bytes::copy_of(record);
                                    let size = self
                                        .size
                                        .checked_add(u32::try_from(record.len()).or(Err(Halt::Room))?)
                                        .ok_or(Halt::Room)?;
                                    return Ok(Progress::Done { turn, size });
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(Halt::Order)
    }
}
fn next_pass(machine: &mut Machine, pass: Pass) -> Result<Option<Pass>, Halt> {
    let patcher = machine.patcher.as_mut().ok_or(Halt::Order)?;
    let version = patcher.versions(pass.plan).to;
    patcher.finish()?;
    if version == machine.last {
        return Ok(None);
    }
    let plan = patcher.plan_from(version).ok_or(Halt::Order)?;
    let root = patcher.counterpart(pass.plan, pass.root).ok_or(Halt::Order)?;
    patcher.next_pass(plan, root)?;
    Ok(Some(Pass { plan, root, first: false }))
}
