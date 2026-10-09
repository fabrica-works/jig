//! Watch admission and construction in the core's vocabulary.
//! `client/domain/contract.md`, sections 3.2 and 7. Documents are made when
//! views asks and moved into held outputs; no document remains in the core.
//! The child owns pacing and provisional text, and knows no concrete subject.
//!
//! | Event | State change | Emits |
//! | --- | --- | --- |
//! | Watch | retain authenticated construction inputs | first held delivery or refusal |
//! | Delivered | child clears its in-flight mark | next current held delivery if changed |
//! | Unwatch | child closes after outstanding terminals | held watch ending |

use crate::documents::{self, Missing, Subject, Window};
use crate::{Core, Held, Limits, Now, Request, Requests, WatchBinding, WatchRefusal};
use alloc::boxed::Box;
use jig_core_tasks as tasks;
use jig_core_views as views;
use skein_lib::{Env, List, Map, Queue, Token};

pub(super) fn route_view(core: &mut Core, env: &Env<Limits>, event: views::Event) -> Requests {
    let mut out = Queue::with_capacity(views::max_out(&env.limits.views));
    views::step(&mut core.views, &Env { now: env.now, wall: env.wall, limits: env.limits.views }, event, &mut out);
    made_outputs(core, env, out)
}
pub(super) fn task_saved_view(core: &mut Core, env: &Env<Limits>, task: &tasks::TaskRecord, out: &mut Queue<Request>) {
    let phase = tasks::view_phase(&task.phase);
    let current = (phase, task.tracked);
    let changed = match core.view_phases.get(&task.number) {
        Some(previous) => *previous != current,
        None => true,
    };
    if phase == 4 {
        core.view_phases.remove(&task.number);
    } else {
        core.view_phases.insert(task.number, current).expect("one live phase per task");
    }
    if !changed || core.view_subjects.is_empty() {
        return;
    }
    let mut trees = List::with_capacity(env.limits.tasks.depth.saturating_add(1));
    trees.push(Token::new(task.number)).expect("self is in its own bounded tree");
    let mut requester = task.requester;
    for _ in 0..env.limits.tasks.depth {
        requester = match requester {
            tasks::Party::Task(parent) => {
                trees.push(Token::new(parent)).expect("bounded ancestor depth");
                match core.tasks.delegation(parent) {
                    Some(context) => context.requester,
                    None => break,
                }
            }
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => break,
        };
    }
    out.push(Request::Held(Box::new(Held::ViewTaskPhase {
        task: Token::new(task.number),
        trees: trees.into_boxed(),
        project: task.project,
        phase,
        priority: task.tracked,
    })));
}

// Construct and move each document into the decision that asked for it.
pub(super) fn made_outputs(core: &mut Core, env: &Env<Limits>, mut child: Queue<views::Request>) -> Requests {
    let room = views::max_out(&env.limits.views);
    let mut ready = Queue::with_capacity(room);
    let mut documents = Map::with_capacity(env.limits.views.watchers);
    for _ in 0..child.len() {
        match child.pop().expect("view output count") {
            views::Request::Make { watcher, .. } => {
                let outcome = match core.view_subjects.get(&watcher) {
                    Some(binding) => match documents::make_with_clock(
                        core,
                        &env.limits,
                        binding.party,
                        &binding.subject,
                        &binding.window,
                        documents::DocumentClock { now: env.now, wall: env.wall },
                    ) {
                        Ok(document) => {
                            documents.insert(watcher, document).expect("bounded construction scratch");
                            views::Made::Ready
                        }
                        Err(Missing::NotAllowed) => views::Made::Gone { why: views::End::NotAllowed },
                        Err(Missing::Unknown | Missing::Ended | Missing::TooLarge) => {
                            views::Made::Gone { why: views::End::Gone }
                        }
                    },
                    None => views::Made::Gone { why: views::End::Gone },
                };
                views::step(
                    &mut core.views,
                    &Env { now: env.now, wall: env.wall, limits: env.limits.views },
                    views::Event::Made { watcher, outcome },
                    &mut ready,
                );
            }
            request @ (views::Request::Watching { .. }
            | views::Request::Refused { .. }
            | views::Request::Stream { .. }
            | views::Request::Deliver { .. }
            | views::Request::Ended { .. }) => ready.push(request),
        }
    }
    let mut out = Queue::with_capacity(room.checked_add(1).expect("view output room"));
    for _ in 0..ready.len() {
        match ready.pop().expect("made output count") {
            views::Request::Watching { .. } => {}
            views::Request::Make { .. } => unreachable!("construction answered within the step"),
            views::Request::Deliver { watcher, .. } => {
                let document = documents.remove(&watcher).expect("delivery follows construction");
                out.push(Request::Held(Box::new(Held::Delivery { watcher, document: Box::new(document) })));
            }
            views::Request::Ended { watcher, end: why } => {
                core.view_subjects.remove(&watcher);
                out.push(Request::Held(Box::new(Held::WatchEnded { watcher, why })));
            }
            views::Request::Refused { watcher, refusal } => {
                core.view_subjects.remove(&watcher);
                let why = match refusal {
                    views::Refusal::Busy => WatchRefusal::Busy,
                };
                out.push(Request::Now(Box::new(Now::WatchRefused { watcher, why })));
            }
            request @ views::Request::Stream { .. } => out.push(Request::Now(Box::new(Now::View(request)))),
        }
    }
    out.push(Request::Decided);
    Requests::Out(out)
}

fn refused(watcher: Token, why: WatchRefusal) -> Requests {
    let mut out = Queue::with_capacity(2);
    out.push(Request::Now(Box::new(Now::WatchRefused { watcher, why })));
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn open(
    core: &mut Core,
    env: &Env<Limits>,
    watcher: Token,
    sign_in: u64,
    subject: Subject,
    window: Window,
) -> Requests {
    let Some(party) = core.people.person(sign_in, env.now, env.wall) else {
        return refused(watcher, WatchRefusal::SignedOut);
    };
    if core.view_subjects.contains_key(&watcher) || core.view_subjects.len() >= env.limits.views.watchers {
        return refused(watcher, WatchRefusal::Busy);
    }
    if !watch_subject(&subject) {
        return refused(watcher, WatchRefusal::Unknown);
    }
    if window.rows == 0 {
        return refused(watcher, WatchRefusal::Busy);
    }
    // Construction checks all named chips, owner-only subjects and deployment
    // roles. Admission retains no made document and consumes no keyed slot.
    match documents::make(core, &env.limits, party, &subject, &window) {
        Ok(_) => {}
        Err(Missing::NotAllowed) => return refused(watcher, WatchRefusal::NotAllowed),
        Err(Missing::Unknown | Missing::Ended) => return refused(watcher, WatchRefusal::Unknown),
        Err(Missing::TooLarge) => return refused(watcher, WatchRefusal::Busy),
    }
    let subject = match subject {
        Subject::Chips { tasks: named } => {
            let mut tasks = List::with_capacity(named.len());
            for number in named.into_boxed() {
                tasks.push(number).expect("admitted chip count");
            }
            Subject::Chips { tasks }
        }
        subject @ (Subject::Frame { .. }
        | Subject::Task { .. }
        | Subject::Tree { .. }
        | Subject::Inbox { .. }
        | Subject::Goals { .. }
        | Subject::Conversations { .. }
        | Subject::Run { .. }
        | Subject::Deployment
        | Subject::Proposal { .. }
        | Subject::Escalation { .. }
        | Subject::Question { .. }
        | Subject::PersonTask { .. }
        | Subject::Result { .. }
        | Subject::Project { .. }
        | Subject::Note { .. }) => subject,
    };
    let follow = match &subject {
        Subject::Run { task } => Some(views::Follow { run: Token::new(*task), turn: None, offset: 0 }),
        Subject::Frame { .. }
        | Subject::Task { .. }
        | Subject::Tree { .. }
        | Subject::Chips { .. }
        | Subject::Inbox { .. }
        | Subject::Goals { .. }
        | Subject::Conversations { .. }
        | Subject::Deployment
        | Subject::Proposal { .. }
        | Subject::Escalation { .. }
        | Subject::Question { .. }
        | Subject::PersonTask { .. }
        | Subject::Result { .. }
        | Subject::Project { .. }
        | Subject::Note { .. } => None,
    };
    let view_window = views::Window { rows: window.rows, after: None };
    core.view_subjects
        .insert(watcher, WatchBinding { party, sign_in, subject, window })
        .expect("bounded watch binding");
    route_view(
        core,
        env,
        views::Event::Watch { watcher, party: Token::new(party), subject: watcher, window: view_window, follow },
    )
}

fn watch_subject(subject: &Subject) -> bool {
    match subject {
        Subject::Frame { .. }
        | Subject::Task { .. }
        | Subject::Tree { .. }
        | Subject::Chips { .. }
        | Subject::Inbox { .. }
        | Subject::Goals { .. }
        | Subject::Conversations { .. }
        | Subject::Run { .. }
        | Subject::Deployment => true,
        Subject::Proposal { .. }
        | Subject::Escalation { .. }
        | Subject::Question { .. }
        | Subject::PersonTask { .. }
        | Subject::Result { .. }
        | Subject::Project { .. }
        | Subject::Note { .. } => false,
    }
}
