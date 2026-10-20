use jig_accounts_world::{LIMITS, provider, providers};
use jig_domain_accounts::{
    DISPLAY, Domain, Event, Failure, FromComponent, Held, Holder, Kind, Limits, NAME, Request, SUBJECT, Signing,
    Stored, max_out, step, worst_case,
};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
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
/// made inside its step, as the store's decoding would make it. Then a sign-out pending for as
/// many sources as are kept, each account's signed-in source renewed and signed out in turn, every
/// sign-in's end kept, and a sign-in under way.
#[test]
fn full_account_source_and_deadline_tables_stay_within_the_worst_case() {
    let mut out = Queue::with_capacity(max_out(&LIMITS));
    let providers = providers(vec![provider(1, true, true)]);
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
    let mut run = Run { meter, env, bound, out, request: 0 };
    for account in 0..LIMITS.accounts {
        run.sign_out(&mut domain, account, number(account, Some(0)));
    }
    let mut ends = 0;
    for _ in 1..LIMITS.sources {
        for account in 0..LIMITS.accounts {
            let source = run.sign_in(&mut domain, account);
            let event = Event::SignedIn {
                source,
                subject: longest(SUBJECT, account),
                display: longest(DISPLAY, account),
                declined: false,
            };
            run.step(&mut domain, event, "renewed");
            ends += 1;
            run.sign_out(&mut domain, account, source);
        }
    }
    while ends < LIMITS.facts {
        let source = run.sign_in(&mut domain, 0);
        run.step(&mut domain, Event::SignInFailed { source, failure: Failure::Cancelled }, "failed");
        ends += 1;
    }
    run.sign_in(&mut domain, 0);
    assert_eq!(domain.facts_lost(), 0, "every end kept");
}

/// Each step metered, its asks taken.
struct Run {
    meter: Meter,
    env: Env<Limits>,
    bound: u64,
    out: Queue<Request>,
    request: u64,
}
impl Run {
    /// The step's last ask of the component, its asks dropped before the check, as the core
    /// takes them.
    fn step(&mut self, domain: &mut Domain, event: Event, label: &str) -> Option<Signing> {
        self.meter.start();
        step(domain, &self.env, event, &mut self.out);
        let measured = self.meter.end();
        let mut signing = None;
        while let Some(request) = self.out.pop() {
            if let Request::Signing(asked) = request {
                signing = Some(asked);
            }
        }
        self.meter.check(measured, self.bound, &label);
        signing
    }
    fn token(&mut self) -> Token {
        self.request += 1;
        Token::new(self.request)
    }
    /// The account's holder signs in at provider 1: the started source's number.
    fn sign_in(&mut self, domain: &mut Domain, account: u32) -> u32 {
        let event = Event::SignIn { request: self.token(), holder: Holder::Party(u64::from(account)), provider: 1 };
        let Some(Signing::SignIn { source, .. }) = self.step(domain, event, "sign in") else {
            panic!("a sign-in started");
        };
        source
    }
    fn sign_out(&mut self, domain: &mut Domain, account: u32, source: u32) {
        let event = Event::SignOut { request: self.token(), holder: Holder::Party(u64::from(account)), source };
        assert_eq!(self.step(domain, event, "sign out"), Some(Signing::SignOut { source }));
    }
}
