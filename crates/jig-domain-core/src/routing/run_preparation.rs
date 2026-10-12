//! Run admission, briefs and workspace preparation across the task, brief,
//! notes and account children (`domain/engine.md`, sections 4.4 and 7).

use super::NoteRoute;
use super::children::remember_due;
use crate::{Ask, Core, Event, Limits, Now, Request, Requests, RunCharter, RunProof};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_brief as brief;
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, ReplyTo, Token};

pub(super) fn brief_assembled(core: &mut Core, work: &mut Queue<Event>, task: u64, ready: bool) -> Requests {
    let mut out = Queue::with_capacity(2);
    if !ready {
        work.push(Event::PreparationFailed { task });
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    let Some(context) = core.contexts.get(&task).cloned() else {
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    let Some(attempt) = crate::fresh(&mut core.counters, crate::Family::Run) else {
        work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    assert!(core.workspace_pending.insert(task, attempt) == Ok(None), "one workspace flight per task");
    out.push(Request::Now(Box::new(Now::WorkspaceRequest { task, attempt, context })));
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn workspace_prepared(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    task: u64,
    attempt: u64,
    writes: Option<Box<[authority::Write]>>,
) -> Requests {
    let mut out = Queue::with_capacity(2);
    if core.workspace_pending.remove(&task) != Some(attempt) {
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    let Some(context) = core.contexts.remove(&task) else {
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    drop(core.dependency_results.remove(&task));
    let transcript = core.transcripts.remove(&task).expect("rendered task owns loaded transcript");
    let Some(writes) = writes else {
        work.push(Event::Tasks(tasks::Event::Hold { task, why: tasks::Hold::Effects }));
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    let why = match core.run_admission(&context, env.wall, writes) {
        crate::RunAdmission::Allow => None,
        crate::RunAdmission::Account => Some(tasks::Hold::Effects),
        crate::RunAdmission::Hold(why) => Some(why),
    };
    if let Some(why) = why {
        work.push(Event::Tasks(tasks::Event::Hold { task, why }));
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    let Some(grant) = core.accounts.grant(core.charter_of(task).account(), env.now) else {
        work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    if !core.proofs.contains_key(&task) && core.proofs.len() == core.proofs.capacity() {
        work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    let resume = core.charter_of(task).policy.resume
        && context.tries.transient == 0
        && transcript.bytes <= u64::from(core.settings.resume_bytes);
    let conversation = if resume && transcript.last.is_some() { transcript.from } else { attempt };
    let messages = start_messages(context.inbox, env.limits.tasks.run_inbox);
    let proof = start_proof(core, task, attempt, conversation, &messages);
    let previous = core.proofs.insert(task, proof).expect("claim proof reserved before child mutation");
    if let Some(previous) = previous {
        assert!(core.replaced.insert(task, previous) == Ok(None), "one pending replacement per task");
    }
    let turns = if resume { core.resumed_turns(transcript) } else { Box::new([]) };
    let mut answered = List::with_capacity(env.limits.call_records);
    let mut after = None;
    for _ in 0..core.call_parts.len() {
        let mut next = None;
        for (&key, _) in &core.call_parts {
            if key.task == task
                && key.attempt < attempt
                && match after {
                    Some(previous) => key > previous,
                    None => true,
                }
                && match next {
                    Some(previous) => key < previous,
                    None => true,
                }
            {
                next = Some(key);
            }
        }
        let Some(key) = next else { break };
        answered.push(key).expect("retained call bound");
        after = Some(key);
    }
    let policy = core.run_policy(task, resume);
    let run = RunCharter {
        policy,
        contract: context.contract,
        authority: context.authority,
        budget: authority::left(crate::translate::authority_numbers(context.numbers))
            .min(core.authority.rules().maximum_run_spend),
    };
    assert!(core.claiming.insert(task, attempt) == Ok(None), "one pending claim per task");
    out.push(Request::Now(Box::new(Now::RunPrepared {
        task,
        attempt,
        conversation,
        charter: core.charter_of(task).number,
        run: Box::new(run),
        messages,
        saved: context.saved,
        transcript: turns,
        answered: answered.into_boxed(),
        grant,
    })));
    out.push(Request::Decided);
    Requests::Out(out)
}

fn start_messages(inbox: Box<[tasks::Word]>, most: u32) -> Box<[tasks::Word]> {
    let mut messages = List::with_capacity(most);
    for word in inbox {
        if messages.len() == most {
            break;
        }
        messages.push(word).expect("start inbox bound");
    }
    messages.into_boxed()
}

fn start_proof(core: &Core, task: u64, attempt: u64, conversation: u64, messages: &[tasks::Word]) -> RunProof {
    let mut offered = None;
    for word in messages {
        offered = Some(word.number);
    }
    core.claim_proof(task, attempt, conversation, offered)
}

pub(super) fn start_brief(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Event>, task: u64) -> Requests {
    let mut out = Queue::with_capacity(1);
    let Some(context) = core.contexts.get(&task) else {
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    let Some(current) = core.tasks.task(task) else {
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    if current.phase != tasks::Phase::Active(tasks::Active::Preparing) || current.last_message != context.last_message {
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    if core.notes.busy() {
        core.pending_note_briefs.insert(task, true).expect("brief count bounds waiting notes");
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    let mut scopes = List::with_capacity(env.limits.notes.scopes);
    scopes.push(notes::Scope::Goal { project: context.project, goal: current.root }).expect("goal scope room");
    scopes.push(notes::Scope::Project { project: context.project }).expect("project scope room");
    scopes.push(notes::Scope::Deployment).expect("deployment scope room");
    for resource in &context.authority.note_resources {
        let last = match &resource.pattern.last {
            tasks::Last::Exact(value) => notes::Last::Exact(value.clone()),
            tasks::Last::Open(value) => notes::Last::Open(value.clone()),
        };
        scopes
            .push(notes::Scope::Resources {
                project: context.project,
                connector: resource.connector,
                pattern: notes::Pattern { segments: resource.pattern.segments.clone(), last },
            })
            .expect("admitted note resource scope room");
    }
    let owner = core.begin_note_route(NoteRoute::Brief { task });
    work.push(Event::Notes(notes::Event::Index { owner, scopes, most: env.limits.notes.lines }));
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn plan_brief(
    core: &Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    task: u64,
    lines: List<notes::Line>,
    more: u32,
) -> Requests {
    let mut out = Queue::with_capacity(2);
    let Some(context) = core.contexts.get(&task) else {
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    let Some(current) = core.tasks.task(task) else {
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    if current.phase != tasks::Phase::Active(tasks::Active::Preparing) || current.last_message != context.last_message {
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    let text_limits = crate::BriefTextLimits {
        parts: env.limits.brief_parts,
        read_bytes: env.limits.brief.read_bytes,
        brief_bytes: env.limits.brief.brief_bytes,
    };
    let mut wanted = List::with_capacity(env.limits.brief.sections);
    let Some(task_text) = crate::read_brief_part(core, text_limits, task, crate::BriefPart::Spec) else {
        work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    wanted
        .push(brief::Planned::Core {
            kind: brief::Core::Task,
            text: task_text,
            limit: env.limits.brief_core_budgets.task,
            priority: 0,
            required: true,
        })
        .expect("task brief room");
    if !context.dependencies.is_empty() || !context.spec.inputs.is_empty() {
        let Some(text) = crate::read_brief_part(core, text_limits, task, crate::BriefPart::Dependencies) else {
            work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
            out.push(Request::Decided);
            return Requests::Out(out);
        };
        wanted
            .push(brief::Planned::Core {
                kind: brief::Core::Results,
                text,
                limit: env.limits.brief_core_budgets.dependencies,
                priority: 1,
                required: true,
            })
            .expect("dependency result section room");
    }
    if core.transcript_oversized(task) || context.tries.transient != 0 || !core.charter_of(task).policy.resume {
        let Some(text) = crate::read_brief_part(core, text_limits, task, crate::BriefPart::TranscriptTail) else {
            work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
            out.push(Request::Decided);
            return Requests::Out(out);
        };
        wanted
            .push(brief::Planned::Core {
                kind: brief::Core::TranscriptTail,
                text,
                limit: env.limits.brief_core_budgets.task,
                priority: 2,
                required: true,
            })
            .expect("tail brief room");
    }
    plan_note_index(&mut wanted, &env.limits, &lines, more);
    if !context.delegates.is_empty()
        && wanted.room() > 0
        && let Some(text) = crate::read_brief_part(core, text_limits, task, crate::BriefPart::Delegates)
    {
        wanted
            .push(brief::Planned::Core {
                kind: brief::Core::Plan,
                text,
                limit: env.limits.brief_core_budgets.plan,
                priority: 3,
                required: false,
            })
            .expect("delegate section room");
    }
    if context.tries != tasks::Tries::NONE
        && wanted.room() > 0
        && let Some(text) = crate::read_brief_part(core, text_limits, task, crate::BriefPart::Attempts)
    {
        wanted
            .push(brief::Planned::Core {
                kind: brief::Core::Attempts,
                text,
                limit: env.limits.brief_core_budgets.attempts,
                priority: 4,
                required: false,
            })
            .expect("attempt brief room");
    }
    let parent = brief_parent(context.requester);
    out.push(Request::Now(Box::new(Now::BriefCorePlanned { task, parent, sections: wanted })));
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn brief_parent(requester: tasks::Party) -> Option<u64> {
    match requester {
        tasks::Party::Task(parent) => Some(parent),
        tasks::Party::Person(_) | tasks::Party::Deployment { .. } => None,
    }
}

pub(super) fn plan_note_index(
    wanted: &mut List<brief::Planned>,
    limits: &Limits,
    lines: &List<notes::Line>,
    more: u32,
) {
    if wanted.room() > 0 {
        wanted
            .push(brief::Planned::Core {
                kind: brief::Core::NotesIndex,
                text: crate::note_index_text(lines, more, limits.brief.read_bytes.min(limits.brief_core_budgets.notes)),
                limit: limits.brief_core_budgets.notes,
                priority: 4,
                required: false,
            })
            .expect("note index section room");
    }
}

pub(super) fn route_prepared_agent(
    core: &mut Core,
    context: Box<tasks::RunContext>,
    transcript_waiter: Option<Token>,
    busy: bool,
) -> Requests {
    let mut out = Queue::with_capacity(2);
    if busy {
        remember_due(core, context);
    } else {
        let number = context.task;
        let previous_attempt = context.previous_attempt;
        assert!(core.contexts.insert(number, context).is_ok(), "bounded activation context");
        core.begin_transcript(number, previous_attempt);
        out.push(Request::Now(Box::new(Now::StartPreparation { task: number, transcript_waiter })));
    }
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn route_activate(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    context: Box<tasks::RunContext>,
    ready: bool,
) -> Requests {
    let number = context.task;
    let mut out = Queue::with_capacity(2);
    if !ready || core.restart_step().is_some() || core.restart_failure().is_some() {
        remember_due(core, context);
    } else if core.tasks.recurring_template(number).is_none() {
        match context.executor {
            tasks::Executor::Person(_) => {}
            tasks::Executor::Procedure { connector, .. } => {
                if let Some(step) = context.previous_attempt.checked_add(1) {
                    out.push(Request::Ask { connector, ask: Ask::StartProcedure { context, step } });
                } else {
                    work.push(Event::Tasks(tasks::Event::Hold { task: number, why: tasks::Hold::Effects }));
                }
            }
            tasks::Executor::Agent { .. } => match core.run_admission(&context, env.wall, Box::new([])) {
                crate::RunAdmission::Allow => out.push(Request::Now(Box::new(Now::PrepareAgent { context }))),
                crate::RunAdmission::Account => remember_due(core, context),
                crate::RunAdmission::Hold(why) => {
                    work.push(Event::Tasks(tasks::Event::Hold { task: number, why }));
                }
            },
        }
    }
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn route_preparation_failed(core: &mut Core, work: &mut Queue<Event>, task: u64) -> Requests {
    drop(core.contexts.remove(&task));
    drop(core.dependency_results.remove(&task));
    drop(core.transcripts.remove(&task));
    work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn route_claim_refused(core: &mut Core, work: &mut Queue<Event>, task: u64) -> Requests {
    let mut out = Queue::with_capacity(2);
    if core.claiming.remove(&task).is_some() {
        core.restore_replaced_proof(task);
        out.push(Request::Now(Box::new(Now::DropAssignment { task })));
        work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
    }
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn route_claim_prepared(
    core: &mut Core,
    work: &mut Queue<Event>,
    task: u64,
    attempt: u64,
    budget: u64,
    writes: Box<[tasks::Name]>,
) -> Requests {
    if core.claiming.get(&task) == Some(&attempt) {
        let earlier_transcript = match core.proofs.get(&task) {
            Some(proof) => proof.conversation != attempt,
            None => false,
        };
        let turned = match core.tasks.task(task) {
            Some(row) => row.ever_turned,
            None => false,
        };
        let resumed = earlier_transcript && turned;
        work.push(Event::Tasks(tasks::Event::Claim {
            reply_to: ReplyTo::new(Token::new(task)),
            task,
            attempt,
            resumed,
            budget,
            writes,
        }));
    }
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}
