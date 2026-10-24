//! Order across owners, using four explicit stack frames (protocol/store.md 10.3).
use super::sets::{Owner, PatchSet, PatchSets};
use skein_codec::patch::{Header, header};
use skein_lib::List;

pub(crate) const DOWNSTREAM: [Owner; 4] = [Owner::Application, Owner::Jig, Owner::Smith, Owner::Skein];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Levels {
    pub skein: Option<u32>,
    pub smith: Option<u32>,
    pub jig: Option<u32>,
    pub application: Option<u32>,
}
impl Levels {
    pub(crate) fn of(&self, owner: Owner) -> Option<u32> {
        match owner {
            Owner::Skein => self.skein,
            Owner::Smith => self.smith,
            Owner::Jig => self.jig,
            Owner::Application => self.application,
        }
    }
    fn set(&mut self, owner: Owner, level: Option<u32>) {
        match owner {
            Owner::Skein => self.skein = level,
            Owner::Smith => self.smith = level,
            Owner::Jig => self.jig = level,
            Owner::Application => self.application = level,
        }
    }
}
pub(crate) fn binary(sets: &PatchSets) -> Levels {
    let mut levels = Levels { skein: None, smith: None, jig: None, application: None };
    for set in sets.sets {
        let mut kept = false;
        for family in set.families {
            if !family.texts.is_empty() {
                kept = true;
            }
        }
        if kept {
            levels.set(set.owner, Some(u32::try_from(set.patches.len()).expect("static patch count fits u32")));
        }
    }
    levels
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Planned {
    pub owner: Owner,
    pub patch: u32,
}
#[derive(Debug)]
pub(crate) struct Plan {
    steps: List<Planned>,
}
impl Plan {
    pub(crate) fn steps(&self) -> &[Planned] {
        self.steps.as_slice()
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Unplanned {
    pub owner: Owner,
    pub patch: u32,
}
#[derive(Clone, Copy, Debug)]
struct Frame {
    owner: Owner,
    target: u32,
}

#[expect(clippy::manual_find, reason = "step code uses no closures (programming-model.md 10.3)")]
pub(crate) fn set(sets: &PatchSets, owner: Owner) -> Option<&PatchSet> {
    for set in sets.sets {
        if set.owner == owner {
            return Some(set);
        }
    }
    None
}
pub(crate) fn text(set: &PatchSet, patch: u32) -> Result<&'static str, Unplanned> {
    let ix = patch.checked_sub(1).ok_or(Unplanned { owner: set.owner, patch })?;
    match set.patches.get(usize::try_from(ix).expect("u32 fits usize")) {
        Some(text) => Ok(text),
        None => Err(Unplanned { owner: set.owner, patch }),
    }
}
fn read(set: &PatchSet, patch: u32) -> Result<Header, Unplanned> {
    let text = text(set, patch)?.as_bytes();
    let Ok(head) = header(text) else {
        return Err(Unplanned { owner: set.owner, patch });
    };
    let span = head.owner();
    let start = usize::try_from(span.start).expect("text span fits usize");
    let end = usize::try_from(span.end).expect("text span fits usize");
    if text.get(start..end) != Some(set.name.as_bytes()) || head.level() != patch {
        return Err(Unplanned { owner: set.owner, patch });
    }
    Ok(head)
}
fn validate(sets: &PatchSets, found: &Levels, wanted: &Levels) -> Result<u32, Unplanned> {
    let mut pending = 0_u32;
    for owner in DOWNSTREAM {
        let mut seen = false;
        for set in sets.sets {
            if set.owner != owner {
                continue;
            }
            if seen {
                return Err(Unplanned { owner, patch: 0 });
            }
            seen = true;
            for (index, _) in set.patches.iter().enumerate() {
                let patch = u32::try_from(index)
                    .expect("static patch index fits u32")
                    .checked_add(1)
                    .expect("static patch count fits u32");
                let head = read(set, patch)?;
                // Applied patches no longer constrain the store's upstream levels.
                if patch <= found.of(owner).unwrap_or(0) {
                    continue;
                }
                let text = text(set, patch)?.as_bytes();
                for upstream in sets.sets {
                    if upstream.owner >= owner {
                        continue;
                    }
                    if let Some(level) = head.after(text, upstream.name.as_bytes())
                        && (Some(level) < found.of(upstream.owner) || Some(level) > wanted.of(upstream.owner))
                    {
                        return Err(Unplanned { owner, patch });
                    }
                }
            }
        }
        let count = wanted
            .of(owner)
            .unwrap_or(0)
            .checked_sub(found.of(owner).unwrap_or(0))
            .expect("caller refused Ahead and Unleveled");
        pending = pending.checked_add(count).expect("static pending patch count fits u32");
    }
    Ok(pending)
}

pub(crate) fn plan(sets: &PatchSets, found: &Levels, wanted: &Levels) -> Result<Plan, Unplanned> {
    let pending = validate(sets, found, wanted)?;
    let mut steps = List::with_capacity(pending);
    let mut stack = List::<Frame>::with_capacity(4);
    let mut reached = *found;
    // An iteration emits one patch, pushes a frame, or pops one. Every pushed
    // dependency emits at least one of the pending patches. Four root frames
    // add eight pushes/pops: at most 3 * pending + 8 iterations in all.
    let turns =
        u64::from(pending).checked_mul(3).expect("u32 times three fits u64").checked_add(8).expect("bounded turns");
    for _ in 0..turns {
        if stack.is_empty() {
            for owner in DOWNSTREAM {
                if reached.of(owner) < wanted.of(owner) {
                    stack
                        .push(Frame { owner, target: wanted.of(owner).expect("pending owner has a level") })
                        .expect("one root frame");
                    break;
                }
            }
            if stack.is_empty() {
                return Ok(Plan { steps });
            }
        }
        let top = stack.len().checked_sub(1).expect("nonempty stack");
        let frame = *stack.get(top).expect("top frame");
        let current = reached.of(frame.owner).expect("pending owner is leveled");
        if current == frame.target {
            stack.truncate(top);
            continue;
        }
        let patch = current.checked_add(1).expect("pending level fits u32");
        let own = set(sets, frame.owner).ok_or(Unplanned { owner: frame.owner, patch: 0 })?;
        let head = read(own, patch)?;
        let text = text(own, patch)?.as_bytes();
        let mut dependency = None;
        for owner in DOWNSTREAM {
            if owner >= frame.owner {
                continue;
            }
            if let Some(upstream) = set(sets, owner)
                && let Some(target) = head.after(text, upstream.name.as_bytes())
            {
                let current = reached.of(owner).unwrap_or(0);
                if current > target {
                    return Err(Unplanned { owner: frame.owner, patch });
                }
                if current < target {
                    dependency = Some(Frame { owner, target });
                    break;
                }
            }
        }
        if let Some(dependency) = dependency {
            stack.push(dependency).expect("dependencies go strictly upstream: at most four frames");
        } else {
            steps.push(Planned { owner: frame.owner, patch }).expect("one entry per pending patch");
            reached.set(frame.owner, Some(patch));
        }
    }
    unreachable!("the pending patches and four owners bound the plan stack's work")
}
