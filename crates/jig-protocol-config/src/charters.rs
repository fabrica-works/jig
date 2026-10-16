//! Offered run policies and their reserve admission (configuration.md, 3 and 6).
use crate::{Limits, Llm, Path, Problems, Segment, What, check, read};
use alloc::boxed::Box;
use jig_domain_core::{Charter, Model, RunPolicy};
use jig_domain_fleet::Kinds;
use skein_json::Document;
use skein_lib::List;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Chats {
    pub charter: u32,
    pub close_grace: skein_lib::Duration,
}
pub(crate) fn chats(
    document: &Document,
    at: u32,
    path: &Path,
    charters: &[Charter],
    grace: skein_lib::Duration,
    problems: &mut Problems,
) -> Option<Chats> {
    let mut object = check::object(document, at, path, problems)?;
    let charter = check::count(&mut object, document, b"charter", path, problems);
    let close_grace = check::duration(&mut object, document, b"close_grace_ms", path, problems);
    object.finish(document, path, problems);
    let charter = charter?;
    let mut offered = false;
    for configured in charters {
        offered |= configured.number == charter;
    }
    if !offered {
        problems.push(path.member(b"charter"), What::Refers);
    }
    let close_grace = close_grace?;
    if close_grace > grace {
        problems.push(path.member(b"close_grace_ms"), What::Bound);
    }
    Some(Chats { charter, close_grace })
}

pub(crate) fn parse(
    document: &Document,
    at: u32,
    path: &Path,
    llm: &Llm,
    limits: &Limits,
    problems: &mut Problems,
) -> Option<Box<[Charter]>> {
    let items = match read::list(document, at, limits.core.tasks.charters) {
        Ok(items) => items,
        Err(what) => {
            problems.push(path.clone(), what);
            return None;
        }
    };
    let mut charters = List::<Charter>::with_capacity(limits.core.tasks.charters);
    for (index, at) in items.iter().enumerate() {
        let path = path.child(Segment::Index(u32::try_from(index).expect("bounded charter count")));
        if let Some(charter) = charter(document, *at, &path, llm, problems) {
            for previous in &charters {
                if previous.number == charter.number {
                    problems.push(path.member(b"number"), What::Unique);
                    break;
                }
            }
            if charter.number == 0 {
                problems.push(path.member(b"number"), What::Bound);
            }
            match jig_domain_core::policy_owned_bytes(&charter.policy) {
                Some(bytes) if bytes <= u64::from(limits.core.run_bytes) => {}
                Some(_) | None => problems.push(path.clone(), What::Bound),
            }
            charters.push(charter).expect("input charter count bounded");
        }
    }
    Some(charters.into_boxed())
}
pub(crate) fn files(engine: &crate::Engine, files: &mut List<crate::File>) {
    for (index, charter) in engine.charters.iter().enumerate() {
        let path = &charter.policy.instructions;
        let policy = jig_domain_core::policy_owned_bytes(&charter.policy).expect("validated policy size");
        let overhead = policy
            .checked_sub(u64::try_from(path.len()).expect("instruction path fits u64"))
            .expect("policy includes its instructions");
        let max =
            u64::from(engine.limits.core.run_bytes).checked_sub(overhead).expect("validated policy fits run bound");
        let at = check::index(&Path::root().member(b"charters"), index).member(b"instructions");
        crate::files::push(files, at, path, u32::try_from(max).expect("run bytes bound is u32"), false);
    }
}

fn charter(document: &Document, at: u32, path: &Path, llm: &Llm, problems: &mut Problems) -> Option<Charter> {
    let mut object = check::object(document, at, path, problems)?;
    let number = check::count(&mut object, document, b"number", path, problems);
    let instructions = check::text(&mut object, document, b"instructions", path, problems);
    let waiting = check::duration(&mut object, document, b"waiting_ms", path, problems);
    let resume = check::boolean(&mut object, document, b"resume", path, problems);
    let turns = check::count(&mut object, document, b"turns", path, problems);
    let time = check::duration(&mut object, document, b"time_ms", path, problems);
    let reserve_turns = check::count(&mut object, document, b"reserve_turns", path, problems);
    let reserve_time = check::duration(&mut object, document, b"reserve_time_ms", path, problems);
    let spend = check::integer(&mut object, document, b"spend", path, problems);
    let main = check::text(&mut object, document, b"model", path, problems);
    let model = match main {
        Some(name) => model(&name, llm, &path.member(b"model"), problems),
        None => None,
    };
    let alternatives = alternatives(&mut object, document, path, llm, problems);
    let hosts = hosts(&mut object, document, path, problems);
    let inspect = check::boolean(&mut object, document, b"inspect", path, problems);
    let modify = check::boolean(&mut object, document, b"modify", path, problems);
    let shell = check::boolean(&mut object, document, b"shell", path, problems);
    let agents = check::boolean(&mut object, document, b"agents", path, problems);
    let call_timeout = check::duration(&mut object, document, b"call_timeout_ms", path, problems);
    object.finish(document, path, problems);
    let charter = Charter {
        number: number?,
        hosts: hosts?,
        spend: spend?,
        policy: RunPolicy {
            instructions: instructions?,
            waiting: waiting?,
            resume: resume?,
            turns: turns?,
            time: time?,
            reserve_turns: reserve_turns?,
            reserve_time: reserve_time?,
            model: model?,
            alternatives: alternatives?,
            inspect: inspect?,
            modify: modify?,
            shell: shell?,
            agents: agents?,
            call_timeout: call_timeout?,
        },
    };
    validate(&charter, path, problems);
    Some(charter)
}

fn alternatives(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    llm: &Llm,
    problems: &mut Problems,
) -> Option<Box<[Model]>> {
    let items = check::list(object, document, b"alternatives", document.len(), path, problems)?;
    let mut models = List::with_capacity(items.len());
    for (index, at) in items.iter().enumerate() {
        let path = path.member(b"alternatives").child(Segment::Index(u32::try_from(index).expect("bounded models")));
        match read::text(document, *at, 4_096) {
            Ok(name) => {
                if let Some(model) = model(&name, llm, &path, problems) {
                    for previous in &models {
                        if previous == &model {
                            problems.push(path.clone(), What::Unique);
                            break;
                        }
                    }
                    models.push(model).expect("bounded alternative count");
                }
            }
            Err(what) => problems.push(path, what),
        }
    }
    Some(models.into_boxed())
}
fn model(name: &[u8], llm: &Llm, path: &Path, problems: &mut Problems) -> Option<Model> {
    for model in &llm.models {
        if model.name.as_ref() == name {
            return Some(model.clone());
        }
    }
    problems.push(path.clone(), What::Refers);
    None
}
fn hosts(object: &mut read::Object, document: &Document, path: &Path, problems: &mut Problems) -> Option<Kinds> {
    let name = check::text(object, document, b"hosts", path, problems)?;
    match name.as_ref() {
        b"workers" => Some(Kinds::Workers),
        b"engine" => Some(Kinds::Engine),
        b"both" => Some(Kinds::Both),
        _ => {
            problems.push(path.member(b"hosts"), What::Form);
            None
        }
    }
}
fn validate(charter: &Charter, path: &Path, problems: &mut Problems) {
    let policy = &charter.policy;
    if policy.turns <= policy.reserve_turns || policy.time <= policy.reserve_time {
        problems.push(path.clone(), What::Reserve { charter: charter.number });
    }
    for model in &policy.alternatives {
        if model.account != policy.model.account {
            problems.push(path.clone(), What::Accounts { charter: charter.number });
            break;
        }
    }
    let enough = match jig_domain_core::reserve_floor(policy) {
        Some(floor) => charter.spend >= floor,
        None => false,
    };
    if !enough {
        problems.push(path.clone(), What::Budget { charter: charter.number });
    }
}
