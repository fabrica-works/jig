use jig_accounts_world::LIMITS;
use jig_domain_accounts::{Domain, Event, FromComponent, max_out, step, worst_case};
use skein_lib::{Duration, Env, Queue, Time, Wall};
use skein_world::domain::heap::{self, Meter};
#[global_allocator]
static HEAP: heap::Counting = heap::Counting;
#[test]
fn full_account_and_deadline_tables_stay_within_the_worst_case() {
    let mut out = Queue::with_capacity(max_out(&LIMITS));
    let meter = Meter::new();
    let mut domain = Domain::new(&LIMITS, Box::new([0, 1]));
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: LIMITS };
    let bound = worst_case(&LIMITS).expect("world limits fit");
    step(&mut domain, &env, Event::Want, &mut out);
    while out.pop().is_some() {}
    for account in 0..LIMITS.accounts {
        meter.start();
        step(
            &mut domain,
            &env,
            Event::Component(FromComponent::Granted { account, generation: 9, valid: Duration::from_secs(100) }),
            &mut out,
        );
        let measured = meter.end();
        while out.pop().is_some() {}
        meter.check(measured, bound, &account);
    }
    assert_eq!(domain.accounts(), LIMITS.accounts);
}
