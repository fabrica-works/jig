use jig_accounts_world::{LIMITS, provider, providers};
use jig_domain_accounts::{
    ADDRESS, DISPLAY, Domain, Event, Failure, Form, FromComponent, Held, Holder, Kind, Limits, NAME, Request, SUBJECT,
    Scheme, Signing, Stored, max_out, step, worst_case,
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
fn granted(source: u32) -> FromComponent {
    FromComponent::Granted {
        form: Form::Credential(Scheme::Bearer),
        account: source,
        generation: 9,
        valid: Duration::from_secs(100),
    }
}
/// Every account with its longest names, every source of each, a program's name at its longest,
/// a limit on each key, a grant for each signed-in source, and every credential. Each record is
/// made inside its step, as the store's decoding would make it. Then a sign-out pending for as
/// many sources as are kept, each account's signed-in source renewed and signed out in turn, every
/// sign-in's end kept, and a sign-in under way.
#[test]
#[expect(clippy::too_many_lines, reason = "one metered saturation story exercises every retained account state")]
fn full_account_source_and_deadline_tables_stay_within_the_worst_case() {
    let mut out = Queue::with_capacity(max_out(&LIMITS));
    let providers = providers(vec![provider(1, true, true)]);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: LIMITS };
    let bound = worst_case(&LIMITS).expect("world limits fit");
    let meter = Meter::new();
    let mut domain = Domain::new(&LIMITS, providers);
    for account in 0..LIMITS.accounts {
        meter.start();
        let record = Stored::Account {
            place: 0,
            uses: Some((0..jig_domain_accounts::USES).map(|n| jig_domain_accounts::Use::Party(u64::from(n))).collect()),
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
            let event = granted(source);
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
    let source = run.sign_in(&mut domain, 0);
    run.visit(&mut domain, source);
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
    /// Keep the account sign-in answer's allocation in the queue for its heap check.
    fn visit(&mut self, domain: &mut Domain, source: u32) {
        self.meter.start();
        let address = longest(ADDRESS, 0);
        step(domain, &self.env, Event::Visited { source, address }, &mut self.out);
        let measured = self.meter.end();
        self.meter.check(measured, self.bound, &"account sign-in address retained");
    }

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
        let event = Event::SignIn {
            request: self.token(),
            holder: Holder::Party(u64::from(account)),
            by: u64::from(account),
            provider: 1,
        };
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

#[test]
fn pending_llm_key_keeps_and_account_removals_stay_within_the_worst_case() {
    use jig_domain_accounts::{Change, USES, Use};
    let providers = providers(vec![provider(1, true, true)]);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: LIMITS };
    let bound = worst_case(&LIMITS).expect("world limits");
    let mut run = Run { meter: Meter::new(), env, bound, out: Queue::with_capacity(max_out(&LIMITS)), request: 0 };
    let mut domain = Domain::new(&LIMITS, providers);
    for number in 0..LIMITS.accounts {
        let account = jig_accounts_world::account(number * 2, 1, Holder::Party(u64::from(number)), b"subject");
        // Each subscription has its own subject.
        let Stored::Account { number: account, provider, holder, place, mut subject, display, .. } = account else {
            unreachable!()
        };
        subject[0] = u8::try_from(number).expect("tiny number");
        run.step(
            &mut domain,
            Event::Restore {
                record: Stored::Account {
                    number: account,
                    provider,
                    holder,
                    place,
                    subject,
                    display,
                    uses: Some((0..USES).map(|n| Use::Party(u64::from(n))).collect()),
                },
            },
            "uses restored",
        );
        run.step(
            &mut domain,
            Event::Restore { record: jig_accounts_world::source(account, account + 1, 0, Kind::SignIn) },
            "source restored",
        );
    }
    run.step(&mut domain, Event::Want, "restored");
    let change = Change::AddKey { provider: 1, join: Some(0), carried: 1 << 63 };
    assert!(matches!(
        run.step(
            &mut domain,
            Event::Change { request: Token::new(50), holder: Holder::Party(0), change },
            "LLM key being kept"
        ),
        Some(Signing::Keep { .. })
    ));
    for number in 1..LIMITS.accounts {
        run.step(
            &mut domain,
            Event::Change {
                request: Token::new(u64::from(number)),
                holder: Holder::Party(u64::from(number)),
                change: Change::Remove { account: number * 2 },
            },
            "account awaiting sign-out",
        );
    }
    let pending = domain.next_number() - 1;
    run.step(
        &mut domain,
        Event::Component(FromComponent::KeepFailed {
            account: pending,
            why: jig_domain_accounts::KeyFailure::NotStored,
        }),
        "LLM key keep failed",
    );
    run.step(
        &mut domain,
        Event::Change { request: Token::new(0), holder: Holder::Party(0), change: Change::Remove { account: 0 } },
        "all removals pending",
    );
    for number in 0..LIMITS.accounts {
        run.step(&mut domain, Event::SignedOut { source: number * 2 + 1, confirmed: true }, "removal confirmed");
    }
}
