//! Turns and their current windows commit together (domain/engine.md, 7.2).
use crate::{Core, CoreKey, CoreRecord, Held, Key, Limits, Record, Request, Requests, Write};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue};

#[expect(clippy::too_many_arguments, reason = "the route retains its event fields")]
pub(super) fn route_accepted_turn(
    core: &mut Core,
    env: &Env<Limits>,
    task: u64,
    attempt: u64,
    turn: u32,
    accepted: tasks::Accepted,
    cumulative: u64,
    read: Option<u64>,
    transcript: Box<[u8]>,
    opens_window: bool,
) -> Requests {
    let refused = &core.tasks.task(task).expect("accepted turn task live").refused;
    let proof = core.proofs.get_mut(&task).expect("turn proof reserved before child mutation");
    assert!(proof.attempt == attempt, "turn callback retains its claim");
    let mut out = Queue::with_capacity(env.limits.call_records.checked_add(6).expect("turn route room"));
    match accepted {
        tasks::Accepted::New => {
            let mut outstanding = skein_lib::List::with_capacity(env.limits.tasks.run_inbox);
            for &number in &proof.outstanding {
                if Some(number) > read && !refused.contains(&number) {
                    outstanding.push(number).expect("remaining offers fit");
                }
            }
            proof.outstanding = outstanding.into_boxed();
            proof.turn = Some(crate::TurnProof { turn, cumulative, read });
            if opens_window {
                proof.window = Some((attempt, turn));
            }
            out.push(Request::Write(Write::Save(Record::Core(CoreRecord::RunProof(proof.clone())))));
            out.push(Request::Write(Write::Save(Record::Core(CoreRecord::Turn(crate::TurnRecord {
                task,
                attempt,
                opens_window,
                turn,
                spent: cumulative,
                read,
                at: env.wall,
                transcript,
            })))));
            for key in core.retire_calls(task, attempt, turn, env.limits.call_records) {
                out.push(Request::Write(Write::Erase(Key::Core(CoreKey::Call(key)))));
            }
            super::view_routes::touch_number(core, &env.limits, task);
        }
        tasks::Accepted::Already => {}
    }
    out.push(Request::Held(Box::new(Held::TaskTurnKept { task, attempt, turn })));
    if let Some(relay) = crate::relays::next(core, task, env.limits.tasks.run_inbox) {
        out.push(Request::Held(Box::new(relay)));
    }
    out.push(Request::Decided);
    Requests::Out(out)
}
