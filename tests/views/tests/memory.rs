use jig_core_views::{self as views, Domain, Event, Window};
use jig_views_world::LIMITS;
use skein_lib::{Env, Queue, Time, Token, Wall};
use skein_world::domain::heap::{self, Meter};
#[global_allocator]
static HEAP: heap::Counting = heap::Counting;
#[test]
fn full_watches_fit_the_declared_worst_case() {
    let meter = Meter::new();
    meter.start();
    let mut domain = Domain::new(&LIMITS);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: LIMITS };
    let mut out = Queue::with_capacity(views::max_out(&LIMITS));
    for watcher in 0..LIMITS.watchers {
        views::step(
            &mut domain,
            &env,
            Event::Watch {
                watcher: Token::new(u64::from(watcher)),
                party: Token::new(1),
                subject: Token::new(7),
                window: Window { rows: 10, after: None },
                follow: None,
            },
            &mut out,
        );
        while out.pop().is_some() {}
    }
    let measured = meter.end();
    let queue = Queue::<views::Request>::worst_case(views::max_out(&LIMITS)).expect("queue bound");
    meter.check(measured, views::worst_case(&LIMITS).expect("valid limits") + queue, &"full watches");
}
