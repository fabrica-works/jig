//! Ended task results gathered for a preparing run (`domain/engine.md`, section 9).

use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{List, Queue, Token};

use crate::{Core, Event, Held, HistoricalResult, Now, Request, Requests};

#[derive(Debug)]
pub(crate) struct ResultRead {
    pub why: ReadFor,
    pub ids: Box<[u64]>,
    pub at: usize,
    pub results: List<HistoricalResult>,
}

#[derive(Debug)]
pub(crate) enum ReadFor {
    Preparation { task: u64, last_message: u64 },
}

pub(super) fn clear(core: &mut Core, task: u64) {
    let mut request = None;
    for (&token, read) in &core.result_reads {
        match read.why {
            ReadFor::Preparation { task: owner, .. } if owner == task => {
                request = Some(token);
                break;
            }
            ReadFor::Preparation { .. } => {}
        }
    }
    if let Some(request) = request {
        drop(core.result_reads.remove(&request));
    }
}

pub(super) fn begin(core: &mut Core, work: &mut Queue<Event>, task: u64, ids: Box<[u64]>) -> Requests {
    let mut out = Queue::with_capacity(2);
    for (_, read) in &core.result_reads {
        match read.why {
            ReadFor::Preparation { task: owner, .. } if owner == task => {
                out.push(Request::Decided);
                return Requests::Out(out);
            }
            ReadFor::Preparation { .. } => {}
        }
    }
    if core.result_reads.len() == core.result_reads.capacity() {
        work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
    } else {
        let request = Token::new(core.next_result_request);
        core.next_result_request = core.next_result_request.checked_add(1).expect("transient result request space");
        let read = ResultRead {
            why: ReadFor::Preparation {
                task,
                last_message: core.contexts.get(&task).expect("preparing context").last_message,
            },
            results: List::with_capacity(u32::try_from(ids.len()).expect("bounded dependency results")),
            ids,
            at: 0,
        };
        ask(&mut out, request, &read);
        assert!(core.result_reads.insert(request, read).is_ok(), "admitted result read");
    }
    out.push(Request::Decided);
    Requests::Out(out)
}

// Held, so that it sees the commit that ended the dependency (`domain/engine.md`, 5.3).
fn ask(out: &mut Queue<Request>, request: Token, read: &ResultRead) {
    out.push(Request::Held(Box::new(Held::Load(Box::new(Now::EndedTask {
        request,
        task: *read.ids.get(read.at).expect("pending result ID"),
    })))));
}

pub(super) fn loaded(
    core: &mut Core,
    work: &mut Queue<Event>,
    request: Token,
    row: Option<Box<tasks::TaskRecord>>,
    busy: bool,
) -> Requests {
    let mut out = Queue::with_capacity(2);
    out.push(Request::Decided);
    let Some(mut read) = core.result_reads.remove(&request) else { return Requests::Out(out) };
    match read.why {
        ReadFor::Preparation { task, last_message } => {
            let current = match core.contexts.get(&task) {
                Some(context) if context.last_message == last_message => match core.tasks.task(task) {
                    Some(row) => {
                        row.phase == tasks::Phase::Active(tasks::Active::Preparing) && row.last_message == last_message
                    }
                    None => false,
                },
                Some(_) | None => false,
            };
            if !current {
                return Requests::Out(out);
            }
            let result = if busy {
                None
            } else {
                match row {
                    Some(row) => core.dependency_result(task, *read.ids.get(read.at).expect("pending result ID"), row),
                    None => None,
                }
            };
            let Some(result) = result else {
                work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
                return Requests::Out(out);
            };
            read.results.push(result).expect("one result per ID");
            read.at = read.at.checked_add(1).expect("bounded result position");
            if read.at < read.ids.len() {
                ask(&mut out, request, &read);
                assert!(core.result_reads.insert(request, read).is_ok(), "continuing result read");
            } else {
                assert!(
                    core.dependency_results.insert(task, read.results.into_boxed()).is_ok(),
                    "live task results room"
                );
                work.push(Event::StartBrief { task });
            }
        }
    }
    Requests::Out(out)
}
