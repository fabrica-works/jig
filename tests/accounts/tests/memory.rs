use jig_accounts_world::LIMITS;
use jig_domain_accounts::{
    DISPLAY, Domain, Event, FromComponent, Held, Holder, Kind, NAME, Providers, SUBJECT, Stored, max_out, step,
    worst_case,
};
use skein_lib::{Duration, Env, Queue, Time, Wall};
use skein_world::domain::heap::{self, Meter};
#[global_allocator]
static HEAP: heap::Counting = heap::Counting;
const KINDS: [Kind; 3] = [Kind::SignIn, Kind::Key, Kind::Program];
/// Bytes at their longest, distinct for each account.
fn longest(length: u32, account: u32) -> Box<[u8]> {
    let mut bytes = vec![b'a'; usize::try_from(length).expect("a small length")];
    bytes[0] = b'a' + u8::try_from(account).expect("a small number");
    bytes.into_boxed_slice()
}
/// Each account's number, its sources' following it.
fn number(account: u32, place: Option<u32>) -> u32 {
    let first = account * (LIMITS.sources + 1);
    match place {
        Some(place) => first + 1 + place,
        None => first,
    }
}
fn kind(place: u32) -> Kind {
    KINDS[usize::try_from(place).expect("a small place") % KINDS.len()]
}
/// Every account with its longest names, every source of each, a program's name at its longest,
/// a limit on each key, a grant for each signed-in source, and every credential. Each record is
/// made inside its step, as the store's decoding would make it.
#[test]
fn full_account_source_and_deadline_tables_stay_within_the_worst_case() {
    let mut out = Queue::with_capacity(max_out(&LIMITS));
    let providers = Providers::none();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: LIMITS };
    let bound = worst_case(&LIMITS).expect("world limits fit");
    let meter = Meter::new();
    let mut domain = Domain::new(&LIMITS, providers, Box::new([]));
    for account in 0..LIMITS.accounts {
        meter.start();
        let record = Stored::Account {
            number: number(account, None),
            provider: 1,
            holder: Holder::Party(u64::from(account)),
            subject: longest(SUBJECT, account),
            display: longest(DISPLAY, account),
        };
        step(&mut domain, &env, Event::Restore { record }, &mut out);
        let measured = meter.end();
        meter.check(measured, bound, &account);
        for place in 0..LIMITS.sources {
            meter.start();
            let name = match kind(place) {
                Kind::Program => longest(NAME, account),
                Kind::SignIn | Kind::Key | Kind::None => Box::new([]),
            };
            let held = match kind(place) {
                Kind::Key => Some(Held::Limited { reset: Some(Wall::from_nanos(60_000_000_000)) }),
                Kind::SignIn | Kind::Program | Kind::None => None,
            };
            let record = Stored::Source {
                account: number(account, None),
                number: number(account, Some(place)),
                place,
                kind: kind(place),
                name,
                held,
            };
            step(&mut domain, &env, Event::Restore { record }, &mut out);
            let measured = meter.end();
            meter.check(measured, bound, &(account, place));
        }
    }
    meter.start();
    step(&mut domain, &env, Event::Want, &mut out);
    let measured = meter.end();
    while out.pop().is_some() {}
    meter.check(measured, bound, &"want");
    for account in 0..LIMITS.accounts {
        for place in 0..LIMITS.sources {
            if kind(place) != Kind::SignIn {
                continue;
            }
            let source = number(account, Some(place));
            meter.start();
            let event = FromComponent::Granted { account: source, generation: 9, valid: Duration::from_secs(100) };
            step(&mut domain, &env, Event::Component(event), &mut out);
            let measured = meter.end();
            while out.pop().is_some() {}
            meter.check(measured, bound, &source);
        }
    }
    for number in 0..LIMITS.credentials {
        meter.start();
        let event = Event::Credential { number, generation: 1, valid: Duration::from_secs(100) };
        step(&mut domain, &env, event, &mut out);
        let measured = meter.end();
        while out.pop().is_some() {}
        meter.check(measured, bound, &number);
    }
    assert_eq!(domain.unrestored(), 0);
    assert_eq!(domain.accounts(), LIMITS.accounts);
    assert_eq!(domain.source_views().len(), LIMITS.accounts * LIMITS.sources);
    assert!(domain.usable(number(0, None)));
}
