use crate::tests::domain::spine;
use jig_client::{Client, Down, Event, Handle, Request, Up, contract};
use skein_lib::{Duration, Env, List, Queue, Time, Token, Wall};
pub struct Fixture {
    pub ui: crate::Domain,
    pub client: Client,
    pub env: Env<crate::Limits>,
    pub out: Queue<crate::Request>,
    pub watch: Token,
    pub frame_native: Option<Token>,
}
impl Fixture {
    pub fn new() -> Self {
        let env = spine::env(Time::ZERO);
        let ui = env.limits;
        let domain = jig_client::DomainLimits {
            owners: &[],
            requests: skein_lib::RequestLimits {
                requests: 4,
                bytes: 65536,
                out: 16,
                first: Duration::from_nanos(10),
                most: Duration::from_nanos(80),
                margin: Duration::from_secs(1),
            },
            ask_bytes: 16384,
            handles: 8,
            watches: 8,
            reads: 8,
            turn_bytes: ui.text,
            document_bytes: ui.document_bytes,
            backoff: jig_client::Backoff { first: Duration::from_nanos(10), most: Duration::from_nanos(80) },
            rows: ui.rows,
            segments: ui.segments,
            text: ui.text,
            projects: ui.projects,
            charters: ui.charters,
            fields: ui.fields,
            delegates: ui.delegates,
            records: ui.records,
            kept: ui.kept,
            parts: ui.parts,
            heartbeat: Duration::from_nanos(100),
        };
        let mut value = Self {
            ui: crate::Domain::new(&ui),
            client: Client::unlinked(&jig_client::Limits { domain, exchanges: 8 }, 9).unwrap(),
            env,
            out: Queue::with_capacity(crate::max_out(&ui)),
            watch: Token::new(0),
            frame_native: None,
        };
        value.watch = spine::start(&mut value.ui, List::with_capacity(0));
        value.event(Event::StartUnlinked { kept: List::with_capacity(0) });
        value.event(Event::Up { up: Up::SignedIn });
        value.event(Event::Up { up: Up::Served { served: true, versions: None } });
        crate::step(&mut value.ui, &value.env, crate::Event::Hello { served: true }, &mut value.out);
        while value.out.pop().is_some() {}
        value
    }
    pub fn event(&mut self, event: Event) -> Queue<Request> {
        let mut out = Queue::with_capacity(512);
        self.client.step(Time::ZERO, Wall::EPOCH, event, &mut out);
        for _ in 0..out.len() {
            let request = out.pop().unwrap();
            if let Request::Down {
                down: Down::Open { watch, subject: jig_client::Watchable::Jig(contract::Subject::Frame) },
            } = &request
            {
                self.frame_native = Some(*watch);
            }
            out.push(request);
        }
        out
    }
    pub fn deliver(&mut self, watch: Token, subject: contract::Subject, delivery: contract::Delivery) {
        let frame = subject == contract::Subject::Frame;
        let request = crate::Request::Open { watch, subject };
        let mut out = self.event(crate::bridge::asked(&request, 8, self.env.limits.watches).unwrap());
        let mut native = None;
        while let Some(request) = out.pop() {
            if let Request::Down { down: Down::Open { watch, .. } } = request {
                native = Some(watch);
            }
        }
        self.event(Event::Up {
            up: Up::Watched {
                watch: native.or(if frame { self.frame_native } else { None }).unwrap(),
                event: contract::WatchEvent::Delivered(delivery),
            },
        });
    }
    pub fn tell(&mut self, told: jig_client::Told) {
        crate::bridge::told(&mut self.ui, &self.env, &self.client, told, 8, &mut self.out);
    }
    pub fn handle(&self) -> Handle {
        Handle { asker: jig_client::Asker::JIG, number: self.watch.raw() }
    }
    pub fn frame(&mut self) {
        self.deliver(self.watch, contract::Subject::Frame, contract::Delivery::Frame(Box::new(spine::frame(7))));
        self.tell(jig_client::Told::Changed { handle: self.handle() });
    }
}
