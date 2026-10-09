//! Task marks, watch admission and live delivery events routed through people
//! and views (`domain/engine.md`, section 11).

use crate::{Core, Held, Limits, Now, Request, Requests};
use alloc::boxed::Box;
use jig_core_people as people;
use jig_core_tasks as tasks;
use jig_core_views as views;
use skein_lib::{Env, List, Queue, ReplyTo, Token};

pub(super) fn route_view(core: &mut Core, env: &Env<Limits>, event: views::Event) -> Requests {
    match &event {
        views::Event::Delivered { watcher } => {
            core.view_documents.remove(watcher);
        }
        views::Event::Watch { .. }
        | views::Event::Changed { .. }
        | views::Event::Made { .. }
        | views::Event::Unwatch { .. }
        | views::Event::Gone { .. } => {}
    }
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
    if !changed || core.watching.is_empty() {
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

// Resolve the child’s construction requests from the parent’s current state.
fn made_outputs(core: &mut Core, env: &Env<Limits>, mut child: Queue<views::Request>) -> Requests {
    let room = views::max_out(&env.limits.views);
    let mut ready = Queue::with_capacity(room);
    for _ in 0..child.len() {
        match child.pop().expect("view output count") {
            views::Request::Make { watcher, .. } => {
                let outcome = match core.view_subjects.get(&watcher) {
                    Some(subject) => match view_document(core, &env.limits, watcher, *subject) {
                        Some(document) => {
                            core.view_documents.insert(watcher, document).expect("bounded parent documents");
                            views::Made::Ready
                        }
                        None => views::Made::Gone { why: views::End::Gone },
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
            | views::Request::Deliver { .. }) => ready.push(request),
            request @ views::Request::Ended { watcher, .. } => {
                core.view_subjects.remove(&watcher);
                core.view_documents.remove(&watcher);
                core.watching.remove(&watcher);
                ready.push(request);
            }
        }
    }
    let mut terminal_checked = Queue::with_capacity(room);
    for _ in 0..ready.len() {
        let request = ready.pop().expect("made output count");
        match &request {
            views::Request::Ended { watcher, .. } => {
                core.view_subjects.remove(watcher);
                core.view_documents.remove(watcher);
                core.watching.remove(watcher);
            }
            views::Request::Watching { .. }
            | views::Request::Make { .. }
            | views::Request::Deliver { .. }
            | views::Request::Refused { .. } => {}
        }
        terminal_checked.push(request);
    }
    tag_view(terminal_checked, room)
}
pub(super) fn tag_view(mut child: Queue<views::Request>, room: u32) -> Requests {
    let mut out = Queue::with_capacity(room.checked_add(1).expect("view output room"));
    for _ in 0..child.len() {
        let request = match child.pop().expect("view output count") {
            views::Request::Make { .. } => unreachable!("construction is resolved by the parent route"),
            request @ (views::Request::Watching { .. }
            | views::Request::Refused { .. }
            | views::Request::Deliver { .. }
            | views::Request::Ended { .. }) => Request::Now(Box::new(Now::View(request))),
        };
        out.push(request);
    }
    out.push(Request::Decided);
    Requests::Out(out)
}

/// Resolve the project of a concrete view subject from the task hub.
#[must_use]
pub fn watch_project(core: &Core, subject: crate::ViewSubject) -> u32 {
    match subject {
        crate::ViewSubject::Run { task, .. } | crate::ViewSubject::Tree { task } => {
            match core.tasks.delegation(task.raw()) {
                Some(context) => context.project,
                None => 0,
            }
        }
        crate::ViewSubject::Goals { project } => project,
        crate::ViewSubject::Inbox { .. } => 0,
    }
}

fn watch_views(subject: people::WatchSubject, project: u32) -> crate::ViewSubject {
    match subject {
        people::WatchSubject::Run { task, attempt } => {
            crate::ViewSubject::Run { task: Token::new(task), attempt: Token::new(attempt) }
        }
        people::WatchSubject::Tree { task } => crate::ViewSubject::Tree { task: Token::new(task) },
        people::WatchSubject::Goals => crate::ViewSubject::Goals { project },
        people::WatchSubject::Inbox { party } => crate::ViewSubject::Inbox { party },
    }
}

fn view_document(
    core: &Core,
    limits: &Limits,
    watcher: Token,
    subject: crate::ViewSubject,
) -> Option<crate::documents::Document> {
    let party = *core.watching.get(&watcher)?;
    let subject = match subject {
        crate::ViewSubject::Run { task, .. } => crate::documents::Subject::Run { task: task.raw() },
        crate::ViewSubject::Tree { task } => crate::documents::Subject::Tree { task: task.raw() },
        crate::ViewSubject::Goals { project } => crate::documents::Subject::Goals { project },
        crate::ViewSubject::Inbox { party } => crate::documents::Subject::Inbox { party },
    };
    crate::documents::make(
        core,
        limits,
        party,
        &subject,
        &crate::documents::Window { rows: limits.documents.rows, after: None },
    )
    .ok()
}

fn watch_refusal(out: &mut Queue<Request>, watcher: Token, refusal: people::Refusal) {
    out.push(Request::Now(Box::new(Now::WatchRefused { watcher, reply: people::Reply::Refused(refusal) })));
}

fn people_reply(out: &mut Queue<Request>, to: ReplyTo, reply: people::Reply) {
    match reply {
        people::Reply::Outcome(people::Outcome::Watching { watcher }) => {
            out.push(Request::Now(Box::new(Now::View(views::Request::Watching { watcher }))));
        }
        people::Reply::Outcome(people::Outcome::Refused(refusal)) | people::Reply::Refused(refusal) => {
            watch_refusal(out, to.into_token(), refusal);
        }
        reply @ (people::Reply::Again(_) | people::Reply::SignedOut) => {
            out.push(Request::Now(Box::new(Now::WatchRefused { watcher: to.into_token(), reply })));
        }
        people::Reply::Outcome(_) | people::Reply::SignedIn { .. } => {
            unreachable!("watch replies only with open or refusal")
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "the keyed watch carries its caller, project and subject")]
#[expect(clippy::too_many_lines, reason = "one volatile route admits the party then opens the view")]
pub(super) fn open(
    core: &mut Core,
    env: &Env<Limits>,
    watcher: Token,
    sign_in: u64,
    key: [u8; 16],
    project: u32,
    subject: people::WatchSubject,
    ready: bool,
) -> Requests {
    let room = people::max_out(&env.limits.people).checked_mul(2).expect("bounded people outputs");
    let room = room.checked_add(views::max_out(&env.limits.views)).expect("bounded view outputs");
    let room = room.checked_add(1).expect("bounded watch outputs");
    let mut out = Queue::with_capacity(room);
    if !ready {
        out.push(Request::Now(Box::new(Now::WatchRefused {
            watcher,
            reply: people::Reply::Again(people::Again::Busy),
        })));
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    let ask = people::Ask::Watch { project, subject };
    let mut admitted = Queue::with_capacity(people::max_out(&env.limits.people));
    people::step(
        &mut core.people,
        &Env { now: env.now, wall: env.wall, limits: env.limits.people },
        people::Event::Ask { reply_to: ReplyTo::new(watcher), sign_in, key, ask },
        &mut admitted,
    );
    for _ in 0..admitted.len() {
        match admitted.pop().expect("watch admission output count") {
            people::Request::Route { request, person, project, role, ask } => {
                let people::Ask::Watch { subject, .. } = *ask else {
                    unreachable!("watch admission routes only its watch ask")
                };
                let subject = watch_views(subject, project);
                let actual = watch_project(core, subject);
                let inbox = match subject {
                    crate::ViewSubject::Inbox { .. } => true,
                    crate::ViewSubject::Run { .. }
                    | crate::ViewSubject::Tree { .. }
                    | crate::ViewSubject::Goals { .. } => false,
                };
                let mut opened = false;
                let outcome = if actual != project || (project == 0 && !inbox) {
                    people::Outcome::Refused(people::Refusal::Unknown)
                } else if project != 0 && !core.watch_authorized(project, role) {
                    people::Outcome::Refused(people::Refusal::Authority { lacking: people::Lack::Shape })
                } else if core.watching.contains_key(&watcher) || core.watching.len() >= env.limits.views.watchers {
                    people::Outcome::Again(people::Again::Busy)
                } else {
                    core.watching.insert(watcher, person).expect("bounded parent watch party");
                    match view_document(core, &env.limits, watcher, subject) {
                        Some(document) => {
                            core.view_documents.insert(watcher, document).expect("bounded parent watch documents");
                            core.view_subjects.insert(watcher, subject).expect("bounded parent watch subjects");
                            let mut views_out = Queue::with_capacity(views::max_out(&env.limits.views));
                            views::step(
                                &mut core.views,
                                &Env { now: env.now, wall: env.wall, limits: env.limits.views },
                                views::Event::Watch {
                                    watcher,
                                    party: Token::new(person),
                                    subject: watcher,
                                    window: views::Window { rows: env.limits.documents.rows, after: None },
                                },
                                &mut views_out,
                            );
                            let mut result = people::Outcome::Refused(people::Refusal::Unknown);
                            for _ in 0..views_out.len() {
                                match views_out.pop().expect("watch output count") {
                                    views::Request::Watching { watcher: opened_watcher } => {
                                        assert!(opened_watcher == watcher, "watch name is echoed");
                                        out.push(Request::Now(Box::new(Now::View(views::Request::Watching {
                                            watcher,
                                        }))));
                                        result = people::Outcome::Watching { watcher };
                                        opened = true;
                                    }
                                    views::Request::Make { watcher, .. } => {
                                        let mut made = Queue::with_capacity(views::max_out(&env.limits.views));
                                        views::step(
                                            &mut core.views,
                                            &Env { now: env.now, wall: env.wall, limits: env.limits.views },
                                            views::Event::Made { watcher, outcome: views::Made::Ready },
                                            &mut made,
                                        );
                                        for _ in 0..made.len() {
                                            out.push(Request::Now(Box::new(Now::View(
                                                made.pop().expect("made document output"),
                                            ))));
                                        }
                                    }
                                    request @ views::Request::Deliver { .. } => {
                                        out.push(Request::Now(Box::new(Now::View(request))));
                                    }
                                    views::Request::Refused { refusal, .. } => {
                                        result = match refusal {
                                            views::Refusal::Busy => people::Outcome::Again(people::Again::Busy),
                                        };
                                    }
                                    views::Request::Ended { .. } => unreachable!("new watch cannot end before opening"),
                                }
                            }
                            result
                        }
                        None => people::Outcome::Refused(people::Refusal::Limit { which: people::LimitKind::Words }),
                    }
                };
                if !opened {
                    core.view_subjects.remove(&watcher);
                    core.view_documents.remove(&watcher);
                    core.watching.remove(&watcher);
                }
                if opened {
                    assert!(core.watching.get(&watcher) == Some(&person), "admitted watch party is retained");
                }
                let mut decided = Queue::with_capacity(people::max_out(&env.limits.people));
                people::step(
                    &mut core.people,
                    &Env { now: env.now, wall: env.wall, limits: env.limits.people },
                    people::Event::Decided { request, outcome },
                    &mut decided,
                );
                while let Some(reply) = decided.pop() {
                    match reply {
                        people::Request::Reply { to, reply } => {
                            if opened {
                                match reply {
                                    people::Reply::Outcome(people::Outcome::Watching { .. }) => {}
                                    other @ (people::Reply::SignedIn { .. }
                                    | people::Reply::SignedOut
                                    | people::Reply::Again(_)
                                    | people::Reply::Outcome(_)
                                    | people::Reply::Refused(_)) => people_reply(&mut out, to, other),
                                }
                            } else {
                                people_reply(&mut out, to, reply);
                            }
                        }
                        people::Request::Save { .. }
                        | people::Request::Erase { .. }
                        | people::Request::Route { .. }
                        | people::Request::RolesApplied { .. }
                        | people::Request::RolesRefused { .. }
                        | people::Request::ServiceMade { .. }
                        | people::Request::RestoreRefused { .. } => unreachable!("watch writes nothing"),
                    }
                }
            }
            people::Request::Reply { to, reply } => people_reply(&mut out, to, reply),
            people::Request::Save { .. }
            | people::Request::Erase { .. }
            | people::Request::RolesApplied { .. }
            | people::Request::RolesRefused { .. }
            | people::Request::ServiceMade { .. }
            | people::Request::RestoreRefused { .. } => unreachable!("watch admission writes nothing"),
        }
    }
    out.push(Request::Decided);
    Requests::Out(out)
}
