use crate::*;
use alloc::boxed::Box;
use skein_lib::{Duration, Env, List, Queue, Time, Token, Wall};
pub(crate) const ME: Handle = Handle { asker: Asker::JIG, number: 1 };
pub(crate) const TASK: Handle = Handle { asker: Asker::JIG, number: 2 };
pub(crate) const OTHER: Handle = Handle { asker: Asker::JIG, number: 5 };
pub(crate) const SUBJECT_TASK: u64 = 9;
pub(crate) const OTHER_SUBJECT: u64 = 10;
pub(crate) const INITIAL_TASK_WATCH: Token = Token::new(2);
pub(crate) fn limits() -> Limits {
    Limits {
        handles: 8,
        watches: 8,            // tests/client/src/test.rs:23
        document_bytes: 65536, // tests/client/src/test.rs:38
        backoff: Backoff {
            // tests/client/src/test.rs:51-54
            first: Duration::from_millis(10),
            most: Duration::from_millis(80),
        },
    }
}
pub(crate) fn task(task: u64) -> Watchable {
    Watchable::Jig(contract::Subject::Task { task })
}
pub(crate) fn delivery(task: u64) -> Up {
    Up::Watched {
        watch: INITIAL_TASK_WATCH,
        event: contract::WatchEvent::Delivered(contract::Delivery::Task(Box::new(document(task)))),
    }
}
#[derive(Debug)]
pub(crate) struct Client {
    pub domain: Domain,
    pub env: Env<Limits>,
    pub out: Queue<Request>,
    pub downs: List<Down>,
    pub told: List<Told>,
}
impl Client {
    pub(crate) fn new(limits: Limits) -> Self {
        Self {
            domain: Domain::new(&limits, 7),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            out: Queue::with_capacity(max_out(&limits)),
            downs: List::with_capacity(max_out(&limits)),
            told: List::with_capacity(max_out(&limits)),
        }
    }
    pub(crate) fn started() -> Self {
        let mut client = Self::new(limits());
        client.step(Event::Start);
        client.downs();
        client
    }
    pub(crate) fn step(&mut self, event: Event) {
        step(&mut self.domain, &self.env, event, &mut self.out);
        self.drain();
    }
    pub(crate) fn at(&mut self, time: Time) {
        self.env.now = time;
        fire(&mut self.domain, &self.env, &mut self.out);
        self.drain();
    }
    fn drain(&mut self) {
        while let Some(request) = self.out.pop() {
            match request {
                Request::Down { down } => self.downs.push(down).unwrap(),
                Request::Told { told } => self.told.push(told).unwrap(),
            }
        }
    }
    pub(crate) fn downs(&mut self) -> Box<[Down]> {
        core::mem::replace(&mut self.downs, List::with_capacity(max_out(&self.env.limits))).into_boxed()
    }
    pub(crate) fn told(&mut self) -> Box<[Told]> {
        core::mem::replace(&mut self.told, List::with_capacity(max_out(&self.env.limits))).into_boxed()
    }
    pub(crate) fn name(&self, subject: Watchable) -> Token {
        for (_, w) in &self.domain.watches {
            if w.subject == subject && !w.closing {
                return w.name.unwrap();
            }
        }
        unreachable!("fixture watch exists")
    }
    pub(crate) fn deliver(&mut self, task: u64) {
        let watch = self.name(self::task(task));
        self.step(Event::Up {
            up: Up::Watched {
                watch,
                event: contract::WatchEvent::Delivered(contract::Delivery::Task(Box::new(document(task)))),
            },
        });
    }
    pub(crate) fn end(&mut self, watch: Token, end: contract::WatchEnd) {
        self.step(Event::Up { up: Up::WatchEnded { watch, end } });
    }
}

fn text() -> contract::Text {
    contract::Text::new(Box::from([]), 0).unwrap()
}
fn document(number: u64) -> contract::Task {
    contract::Task {
        chip: contract::Chip {
            task: number,
            title: contract::DocumentText::complete(text()),
            executor: contract::Executor::Agent { charter: 1 },
            phase: contract::Phase::Waiting,
            details: Box::new(contract::ChipPhase::Starting),
        },
        goal: None,
        priority: None,
        lineage_chips: List::with_capacity(0),
        inputs: List::with_capacity(0),
        project: 7,
        requester: contract::OwnerRequester::Task(number),
        lineage: List::with_capacity(0),
        spec: contract::SpecificationDocument {
            specification: contract::DocumentSpecification {
                words: contract::DocumentText { bytes: text(), cut: false },
                parameters: List::with_capacity(0),
                inputs: List::with_capacity(0),
            },
            cut: false,
        },
        spec_revision: contract::Revision { value: 7 },
        person_revision: None,
        taken_by: None,
        instructions: None,
        wake: contract::OwnerWakePolicy {
            words: contract::WakeRule::Never,
            notices: contract::WakeRule::Never,
            news: contract::WakeRule::Never,
            results: contract::ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        },
        subscriptions: contract::Subscriptions { entries: List::with_capacity(0) },
        result_contract: contract::TaskContractDocument::Approval {
            state: Box::from([]),
            part: contract::OwnerPart {
                connector: 7,
                kind: contract::OwnerPartKind::Resource,
                key: contract::OwnerPartKey::Path(List::with_capacity(0)),
            },
        },
        dependencies: List::with_capacity(0),
        references: List::with_capacity(0),
        cut: false,
        resources: List::with_capacity(0),
        budget: contract::OwnerBudget {
            numbers: contract::OwnerNumbers { budget: 7, spent: 7, spent_below: 7, reserved: 7 },
            left: 7,
            funder: contract::Funder::Task(number),
            deadline: None,
        },
        authority: contract::OwnerAuthorityDocument {
            authority: contract::OwnerAuthority {
                tools: 7,
                grants: List::with_capacity(0),
                delegation: contract::OwnerDelegation { kinds: List::with_capacity(0), tasks: 7, depth: 7 },
                spend: 7,
                deadline: None,
                notes: 7,
                note_resources: List::with_capacity(0),
            },
            cut: false,
        },
        hold: None,
        proposals: List::with_capacity(0),
        activity: 7,
        changed: Wall::from_nanos(7),
        acts: List::with_capacity(0),
        result: None,
        delegates: List::with_capacity(0),
        context: None,
    }
}
