//! The accounts driver, its records in the secrets' store and secret-free
//! translation (llm.md, 5; store.md, 9).
use crate::{Files, Seeds, values::Values};
use alloc::boxed::Box;
use jig_domain_accounts as domain;
use jig_domain_core as core;
use jig_protocol_accounts::{handed_in, installation, keys, party, records, table, translate};
use jig_protocol_config::{self as config, Path, Problems, Segment, What};
use jig_protocol_store as store;
use jig_test_domain as root;
use skein_io::{
    self as io,
    kernel::{Complete, Op, Submit},
};
use skein_lib::{CryptoRng, Env, List, Map, Queue, Secret, Time, Token, Wall};
use skein_oauth as oauth;
use skein_oauth_accounts as driver;

pub(super) const PREFIX: u64 = 3_u64 << 60;
/// The store's answer to the load of the sources' records; a replace is
/// answered under its source's number.
const KEY_EXCHANGE: u64 = PREFIX | (1_u64 << 33);
const LOAD: Token = Token::new(PREFIX | (1_u64 << 32_u32));
#[derive(Debug)]
pub struct AccountsRuntime {
    pub issuers: List<(u32, driver::Endpoint)>,
    /// LLM account sign-in endpoints and origins, keyed by provider number.
    pub providers: List<(u32, driver::Endpoint, List<driver::Origin>)>,
    /// Bounds of all prepared trust roots, supplied by their shell owner.
    pub trust_roots: u32,
    pub trust_root_bytes: u32,
}
/// Until the store's records are read the driver is not made; until the
/// handed-in records are kept no request reaches it (llm.md 5.1, 5.4).
#[expect(
    clippy::large_enum_variant,
    reason = "one phase per service, and the sources it loads move once into the driver"
)]
enum Phase {
    Loading { configured: Map<u32, driver::Account>, crypto: CryptoRng, after: Option<u64>, asked: bool },
    HandingIn,
    Running,
}
/// One durable erase per source; only sign-out continues to driver removal.
#[derive(Debug)]
enum Erasing {
    Revoked,
    SignOut { confirmed: bool },
    Removing,
}
pub(super) struct Accounts {
    component: Option<driver::Component>,
    phase: Phase,
    env: Env<driver::Limits>,
    ids: Map<u32, Secret>,
    providers: Map<u32, driver::Account>,
    required: Map<u32, Box<[u8]>>,
    erasing: Map<u32, Erasing>,
    records: records::Records,
    keys: keys::Keys,
    key_out: Queue<keys::Next>,
    key_events: Queue<domain::FromComponent>,
    forgotten: Queue<u32>,
    handed: Queue<(u32, oauth::SavedToken)>,
    early: Queue<domain::Request>,
    pub(super) io: Queue<io::Request>,
    above: Queue<driver::Event>,
    /// Empty: each source's keeper is the owner, so the driver files nothing.
    files: Queue<driver::FileLower>,
    /// Aborted: the store is closing, and takes no more records.
    aborting: bool,
    closed: bool,
}
fn problem(index: usize, what: What) -> Problems {
    let mut found = Problems::empty();
    found.push(
        Path::root()
            .member(b"engine")
            .member(b"llm")
            .member(b"accounts")
            .child(Segment::Index(u32::try_from(index).expect("bounded accounts")))
            .member(b"source"),
        what,
    );
    found
}
impl Files {
    pub(crate) fn contents(&self, path: &[u8]) -> Option<&[u8]> {
        for (name, value) in &self.entries {
            if name.as_ref() == path {
                return Some(value.expose());
            }
        }
        None
    }
}
fn issuers(
    runtime: Option<&mut AccountsRuntime>,
    limits: &driver::Limits,
) -> Result<Map<u32, driver::Endpoint>, Problems> {
    let mut issuers = Map::with_capacity(limits.accounts);
    let Some(runtime) = runtime else {
        return Ok(issuers);
    };
    if runtime.issuers.len() > limits.accounts {
        return Err(problem(0, What::Bound));
    }
    for (_, issuer) in &runtime.issuers {
        if let driver::Transport::Tls { .. } = issuer.transport
            && (runtime.trust_roots == 0 || runtime.trust_root_bytes == 0)
        {
            return Err(problem(0, What::Bound));
        }
    }
    for (number, endpoint) in ::core::mem::replace(&mut runtime.issuers, List::with_capacity(0)).into_boxed() {
        issuers.insert(number, endpoint).or(Err(problem(0, What::Bound)))?;
    }
    Ok(issuers)
}
/// The host resolves transports; the component owns its public registration vocabulary.
fn registration_input(
    configured: &config::llm::Registration,
    endpoint_transport: &driver::Transport,
) -> party::Registration {
    party::Registration {
        issuer: configured.issuer.clone(),
        client: configured.client.clone(),
        scopes: configured.scopes.clone(),
        name_hint: configured.name_hint.clone(),
        authorization: configured.authorization.clone(),
        token: configured.token.clone(),
        revocation: configured.revocation.clone(),
        keys: configured.keys.clone(),
        endpoint_transport: match endpoint_transport {
            driver::Transport::Tls { server_name, trust } => {
                driver::Transport::Tls { server_name: server_name.clone(), trust: trust.clone() }
            }
            driver::Transport::Plaintext => driver::Transport::Plaintext,
        },
        must_grant: configured.must_grant.clone(),
        resource: configured.resource.clone(),
        redirect_path: configured.redirect_path.clone(),
        redirect_port: configured.redirect_port,
    }
}
/// A copy of a configured source, without a record, for the check at
/// start; configured sources name no origins.
fn unkept(account: &driver::Account) -> driver::Account {
    match account {
        driver::Account::HandedIn => driver::Account::HandedIn,
        driver::Account::SignIn { registration, endpoint, keeper: _, shared, origins } => driver::Account::SignIn {
            registration: oauth::Registration {
                authorization_url: registration.authorization_url.clone(),
                token_endpoint: registration.token_endpoint.clone(),
                client_id: registration.client_id.clone(),
                redirect_uri: registration.redirect_uri.clone(),
                scope: registration.scope.clone(),
                wire: registration.wire,
                client_secret: registration.client_secret.clone(),
                pkce_for_confidential: registration.pkce_for_confidential,
                metadata_claim: registration.metadata_claim.clone(),
                terms: registration.terms.clone(),
            },
            endpoint: driver::Endpoint {
                address: endpoint.address,
                transport: match &endpoint.transport {
                    driver::Transport::Tls { server_name, trust } => {
                        driver::Transport::Tls { server_name: server_name.clone(), trust: trust.clone() }
                    }
                    driver::Transport::Plaintext => driver::Transport::Plaintext,
                },
            },
            keeper: driver::Keeper::Owner { kept: None },
            shared: *shared,
            origins: copy_origins(origins),
        },
    }
}
fn provider_templates(
    config: &config::Engine,
    runtime: Option<&mut AccountsRuntime>,
    limits: &driver::Limits,
    crypto: &CryptoRng,
    installation_id: &[u8; 16],
) -> Result<Map<u32, driver::Account>, Problems> {
    let mut providers = Map::with_capacity(u32::try_from(config.llm.providers.len()).expect("provider count"));
    if let Some(runtime) = runtime {
        for (number, endpoint, origins) in
            ::core::mem::replace(&mut runtime.providers, List::with_capacity(0)).into_boxed()
        {
            let mut registration = None;
            for provider in &config.llm.providers {
                if provider.number == number {
                    registration = provider.sign_in.as_ref();
                    break;
                }
            }
            let registration = registration.ok_or(problem(0, What::Refers))?;
            if (runtime.trust_roots == 0 || runtime.trust_root_bytes == 0) && has_tls(&endpoint, &origins) {
                return Err(problem(0, What::Bound));
            }
            let account = party::party_account(
                &registration_input(registration, &endpoint.transport),
                endpoint,
                origins,
                installation_id,
            );
            let mut check = List::with_capacity(1);
            check.push(unkept(&account)).or(Err(problem(0, What::Bound)))?;
            let _checked = driver::Component::new(check, limits, crypto.clone()).or(Err(problem(0, What::Form)))?;
            providers.insert(number, account).or(Err(problem(0, What::Bound)))?;
        }
    }
    for provider in &config.llm.providers {
        if provider.sign_in.is_some() && !providers.contains_key(&provider.number) {
            return Err(problem(0, What::Refers));
        }
    }
    Ok(providers)
}
impl Accounts {
    /// Checks the configured sources and makes them ready to start; the
    /// driver is made once the secrets' store's records are read.
    pub fn new(
        config: &config::Engine,
        files: &Files,
        seeds: &Seeds,
        mut runtime: Option<AccountsRuntime>,
        room: u32,
        secret_bytes: u32,
    ) -> Result<Self, Problems> {
        let limits = config.limits.driver;
        if limits.client.document.record_bytes > secret_bytes {
            return Err(problem(0, What::Bound));
        }
        let mut issuers = issuers(runtime.as_mut(), &limits)?;
        let mut configured = Map::with_capacity(limits.accounts);
        let mut ids = Map::with_capacity(limits.accounts);
        let mut handed = Queue::with_capacity(limits.accounts);
        for (index, account) in config.llm.accounts.iter().enumerate() {
            let mut account_id = Secret::new(Box::default());
            let source = match &account.source {
                config::Source::HandedIn { file } => {
                    let value = files.contents(file).ok_or(problem(index, What::Unreadable))?;
                    let bounds = config::DocumentLimits {
                        bytes: limits.client.document.record_bytes,
                        tokens: limits.client.document.tokens,
                        depth: limits.client.document.depth,
                    };
                    let record = config::handed_in(value, &bounds).or(Err(problem(index, What::Form)))?;
                    if let Some(id) = &record.account_id {
                        if id.len() > usize::try_from(table::ACCOUNT_ID_BYTES).expect("id bound") {
                            return Err(problem(index, What::Bound));
                        }
                        account_id = Secret::new(id.clone());
                    }
                    let record = handed_in::record(
                        account.number,
                        record.token,
                        record.lapses,
                        record.account_id,
                        &limits.client.document,
                    )
                    .or(Err(problem(index, What::Bound)))?;
                    handed.push((account.number, record));
                    driver::Account::HandedIn
                }
                config::Source::SignIn(source) => {
                    let endpoint = issuers.remove(&account.number).ok_or(problem(index, What::Refers))?;
                    let secret = match &source.secret {
                        Some(path) => Some(files.contents(path).ok_or(problem(index, What::Unreadable))?.into()),
                        None => None,
                    };
                    driver::Account::SignIn {
                        shared: false,
                        origins: List::with_capacity(0),
                        registration: oauth::Registration {
                            authorization_url: source.authorization.clone(),
                            token_endpoint: source.token.clone(),
                            client_id: source.client.clone(),
                            redirect_uri: source.redirect.clone(),
                            scope: source.scope.clone(),
                            wire: oauth::WireFormat::Form,
                            client_secret: secret,
                            pkce_for_confidential: false,
                            metadata_claim: None,
                            // The drawn identifier, for the check at start; every
                            // identifier formats to the same length, and the
                            // kept one replaces it once the store is open.
                            terms: Some(installation::terms(&seeds.installation)),
                        },
                        endpoint,
                        keeper: driver::Keeper::Owner { kept: None },
                    }
                }
            };
            configured.insert(account.number, source).or(Err(problem(index, What::Bound)))?;
            ids.insert(account.number, account_id).expect("configured account id place");
        }
        let crypto = CryptoRng::new(&seeds.crypto, 1).ok_or(problem(0, What::Bound))?;
        let mut check = List::with_capacity(limits.accounts);
        for (_, account) in &configured {
            check.push(unkept(account)).or(Err(problem(0, What::Bound)))?;
        }
        let checked = driver::Component::new(check, &limits, crypto.clone()).or(Err(problem(0, What::Form)))?;
        drop(checked);
        let providers = provider_templates(config, runtime.as_mut(), &limits, &crypto, &seeds.installation)?;
        Ok(Self {
            component: None,
            phase: Phase::Loading { configured, crypto, after: None, asked: false },
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            ids,
            providers,
            required: Map::with_capacity(limits.accounts),
            erasing: Map::with_capacity(limits.accounts),
            records: records::Records::new(limits.accounts),
            keys: keys::Keys::new(limits.accounts, limits.client.document.token_bytes),
            key_out: Queue::with_capacity(room),
            key_events: Queue::with_capacity(room),
            forgotten: Queue::with_capacity(room),
            handed,
            early: Queue::with_capacity(room),
            io: Queue::with_capacity(room),
            above: Queue::with_capacity(room),
            files: Queue::with_capacity(room),
            aborting: false,
            closed: false,
        })
    }
    fn down(&mut self, request: driver::Request) {
        if let Some(component) = &mut self.component {
            component.down(&self.env, request, &mut self.above, &mut self.io, &mut self.files);
        }
    }
    fn down_room(&self) -> bool {
        self.above.room() >= driver::MAX_OUT_DOWN.above
            && self.io.room() >= driver::MAX_OUT_DOWN.io
            && self.files.room() >= driver::MAX_OUT_DOWN.files
    }
    fn running(&self) -> bool {
        match self.phase {
            Phase::Running => self.early.is_empty(),
            Phase::Loading { .. } | Phase::HandingIn => false,
        }
    }
    pub fn up_room(&self) -> bool {
        self.above.room() >= driver::MAX_OUT_UP.above && self.io.room() >= driver::MAX_OUT_UP.io
    }
    pub fn batch_room(&self, batch: u32) -> bool {
        self.above.room() >= batch.checked_mul(driver::MAX_OUT_DOWN.above).expect("account above room")
            && self.io.room() >= batch.checked_mul(driver::MAX_OUT_DOWN.io).expect("account io room")
            && self.files.room() >= batch.checked_mul(driver::MAX_OUT_DOWN.files).expect("account files room")
            && self.key_out.room() >= batch
            && self.key_events.room() >= batch
            && self.forgotten.room() >= batch
            && (self.running() || self.early.room() >= batch)
    }
    /// The domain's request: to the driver once it runs, held until then.
    pub fn request(&mut self, request: domain::Request) {
        match request {
            domain::Request::Component(_) | domain::Request::Signing(_) => {
                if self.running() {
                    self.dispatch(request);
                } else {
                    self.early.push(request);
                }
            }
            domain::Request::Granted { .. }
            | domain::Request::Save { .. }
            | domain::Request::Erase { .. }
            | domain::Request::Answer { .. }
            | domain::Request::Closed => (),
        }
    }
    fn dispatch(&mut self, request: domain::Request) {
        match request {
            domain::Request::Component(request) => match request {
                domain::ToComponent::Want { account } if self.keys.contains(account) => {
                    if let Some(event) = self.keys.want(account, self.env.now) {
                        self.key_events.push(event);
                    }
                }
                domain::ToComponent::Rejected { account, .. } | domain::ToComponent::Release { account }
                    if self.keys.contains(account) => {}
                other @ (domain::ToComponent::Want { .. }
                | domain::ToComponent::Rejected { .. }
                | domain::ToComponent::Release { .. }
                | domain::ToComponent::Close) => self.down(translate::down(other)),
            },
            domain::Request::Signing(signing) => {
                match signing {
                    domain::Signing::Restore { .. } | domain::Signing::Restored => return,
                    domain::Signing::Keep { .. }
                    | domain::Signing::Forget { .. }
                    | domain::Signing::SignIn { .. }
                    | domain::Signing::SignOut { .. }
                    | domain::Signing::Replaced { .. } => {}
                }
                match translate::down_signing(signing) {
                    translate::Down::Keep { source, carried, replaces } => {
                        let next = self.keys.keep(source, carried, replaces);
                        self.key_out.push(next);
                    }
                    translate::Down::Forget { source } => {
                        self.keys.forget(source);
                        self.forgotten.push(source);
                    }
                    translate::Down::SignOut { source } => self.down(driver::Request::SignOut { account: source }),
                    translate::Down::SignIn { source, provider } => {
                        let Some(template) = self.providers.get(&provider) else {
                            self.above.push(driver::Event::Refused {
                                account: source,
                                asked: driver::Asked::Add,
                                why: driver::Refusal::Source,
                            });
                            return;
                        };
                        let account = unkept(template);
                        let required = match &account {
                            driver::Account::SignIn { registration, .. } => match &registration.terms {
                                Some(terms) => terms.required_scopes.clone(),
                                None => None,
                            },
                            driver::Account::HandedIn => None,
                        }
                        .unwrap_or_default();
                        let before = self.above.len();
                        self.component.as_mut().expect("running driver").add(
                            &self.env,
                            source,
                            account,
                            &mut self.above,
                        );
                        if self.above.len() != before {
                            return;
                        }
                        self.ids.insert(source, Secret::new(Box::default())).expect("driver admitted account");
                        self.required.insert(source, required).expect("driver admitted account");
                        self.records.admit(source).expect("driver admitted account");
                        self.down(driver::Request::SignIn { account: source, subject: None });
                    }
                }
            }
            domain::Request::Granted { .. }
            | domain::Request::Save { .. }
            | domain::Request::Erase { .. }
            | domain::Request::Answer { .. }
            | domain::Request::Closed => (),
        }
    }
    pub fn clock(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
    }
    pub fn up(&mut self, event: io::Event) {
        if let Some(component) = &mut self.component {
            component.up(&self.env, event, &mut self.above, &mut self.io, &mut self.files);
        }
    }
    pub fn is_empty(&self) -> bool {
        self.key_out.is_empty()
            && self.key_events.is_empty()
            && self.forgotten.is_empty()
            && self.closed
            && self.io.is_empty()
            && self.above.is_empty()
            && self.files.is_empty()
            && self.early.is_empty()
    }
    pub fn next_deadline(&self) -> Option<Time> {
        match &self.component {
            Some(component) => component.next_deadline(),
            None => None,
        }
    }
    pub fn work_pending(&self, now: Time) -> bool {
        let starting = match &self.phase {
            Phase::Loading { asked, .. } => !*asked,
            Phase::HandingIn => !self.handed.is_empty() || !self.records.keeping(),
            Phase::Running => !self.early.is_empty(),
        };
        starting
            || !self.key_out.is_empty()
            || !self.key_events.is_empty()
            || !self.forgotten.is_empty()
            || !self.above.is_empty()
            || !self.io.is_empty()
            || !self.files.is_empty()
            || match &self.component {
                Some(component) => component.has_work(),
                None => false,
            }
            || match self.next_deadline() {
                Some(deadline) => deadline <= now,
                None => false,
            }
    }
    pub fn abort(&mut self) {
        self.aborting = true;
        if self.component.is_some() {
            self.down(driver::Request::Abort);
        } else {
            self.closed = true;
        }
        self.phase = Phase::Running;
        for _ in 0..self.handed.len() {
            let _unkept = self.handed.pop();
        }
        for _ in 0..self.early.len() {
            let _unasked = self.early.pop();
        }
    }
    pub fn iterate(
        &mut self,
        now: Time,
        wall: Wall,
        values: &mut Values,
        events: &mut Queue<root::Event>,
        store: &mut crate::store::Store,
    ) {
        self.clock(now, wall);
        self.answers(store, events);
        self.start(store);
        self.key_steps(values, events, store);
        if let Some(component) = &mut self.component
            && self.above.room() >= 3
            && self.io.room() >= 3
            && self.files.room() >= 1
        {
            component.fire(&self.env, &mut self.above, &mut self.io, &mut self.files);
        }
        for _ in 0..self.above.len() {
            if events.room() == 0 || !store.room() || !self.down_room() {
                break;
            }
            let event = self.above.pop().expect("account events");
            let number = event_account(&event);
            let required = match number {
                Some(number) => self.required.get(&number),
                None => None,
            };
            let required = match required {
                Some(value) => &**value,
                None => b"",
            };
            let event = match self.records.up(event, now, required) {
                translate::Up::Granted { mut event, account, generation, value, expires } => {
                    let granted_form = self.granted(account, generation, value, expires, values);
                    match &mut event {
                        domain::FromComponent::Granted { form, .. } => *form = granted_form,
                        domain::FromComponent::Expiring { .. }
                        | domain::FromComponent::KeepFailed { .. }
                        | domain::FromComponent::Failed { .. }
                        | domain::FromComponent::Closed => unreachable!("a lent value accompanies its grant"),
                    }
                    event
                }
                translate::Up::Child(event) => event,
                translate::Up::Revoked { source } => {
                    values.accounts.remove(source);
                    self.erase(source, Erasing::Revoked, events, store);
                    domain::FromComponent::Failed { account: source, failure: domain::Failure::SignInAgain }
                }
                translate::Up::Visited { source, address } => {
                    events.push(root::Event::Core(core::Event::Account(domain::Event::Visited { source, address })));
                    continue;
                }
                translate::Up::SignedIn { source, subject, display } => {
                    events.push(root::Event::Core(core::Event::Account(domain::Event::SignedIn {
                        source,
                        subject,
                        display,
                        declined: false,
                    })));
                    continue;
                }
                translate::Up::SignInFailed { source, failure } => {
                    events
                        .push(root::Event::Core(core::Event::Account(domain::Event::SignInFailed { source, failure })));
                    // Every failed account sign-in gives its driver place back.
                    if self.ids.contains_key(&source) {
                        self.down(driver::Request::SignOut { account: source });
                    }
                    continue;
                }
                translate::Up::SignedOut { source, confirmed } => {
                    self.erase(source, Erasing::SignOut { confirmed }, events, store);
                    continue;
                }
                translate::Up::Removed { source } => {
                    let _erase = self.erasing.remove(&source);
                    let _id = self.ids.remove(&source);
                    let _required = self.required.remove(&source);
                    self.records.remove(source);
                    values.accounts.remove(source);
                    continue;
                }
                translate::Up::Keep { account, record } => {
                    if self.aborting {
                        continue;
                    }
                    match self.records.keep(account, record.0, &self.env.limits.client.document) {
                        records::Next::Driver(request) => self.down(request),
                        records::Next::Store(keep) => replace(store, keep),
                    }
                    continue;
                }
            };
            match &event {
                domain::FromComponent::Granted { .. }
                | domain::FromComponent::Expiring { .. }
                | domain::FromComponent::Failed { .. }
                | domain::FromComponent::KeepFailed { .. } => {}
                domain::FromComponent::Closed => self.closed = true,
            }
            events.push(root::Event::Core(core::Event::Account(domain::Event::Component(event))));
        }
        assert!(self.files.is_empty(), "owner keepers file nothing");
        values.accounts.expire(now);
        if let Some(component) = &mut self.component {
            component.reclaim();
        }
    }
    fn key_steps(&mut self, values: &mut Values, events: &mut Queue<root::Event>, store: &mut crate::store::Store) {
        for _ in 0..self.forgotten.len() {
            if !store.room() {
                break;
            }
            let source = self.forgotten.pop().expect("forgotten LLM key");
            values.accounts.remove(source);
            let _later = store.secrets(
                Token::new(PREFIX | u64::from(source)),
                store::Secrets::Erase { owner: store::SecretOwner::Source, number: u64::from(source) },
            );
        }
        for _ in 0..self.key_out.len() {
            if !store.room() || events.room() == 0 {
                break;
            }
            match self.key_out.pop().expect("LLM key steps") {
                keys::Next::Store { exchange, request } => {
                    let owner = Token::new(KEY_EXCHANGE | exchange.raw());
                    let request = match request {
                        keys::Store::Get { carried } => {
                            store::Secrets::Get { owner: store::SecretOwner::Application, number: carried }
                        }
                        keys::Store::Adopt { carried, source, replaces } => store::Secrets::Adopt {
                            carried,
                            owner: store::SecretOwner::Source,
                            number: u64::from(source),
                            replaces,
                        },
                    };
                    if let Some(store::Event::Secret { value, .. }) = store.secrets(owner, request)
                        && let Some(next) = self.keys.read(exchange, value)
                    {
                        self.key_out.push(next);
                    }
                }
                keys::Next::Child(event) => {
                    events.push(root::Event::Core(core::Event::Account(domain::Event::Component(event))))
                }
                keys::Next::Granted { event, source, generation, value } => {
                    values
                        .accounts
                        .granted(source, generation, value, Time::from_nanos(u64::MAX))
                        .expect("admitted LLM key fits table");
                    events.push(root::Event::Core(core::Event::Account(domain::Event::Component(event))));
                }
            }
        }
        for _ in 0..self.key_events.len() {
            if events.room() == 0 {
                break;
            }
            let event = self.key_events.pop().expect("LLM key grant events");
            events.push(root::Event::Core(core::Event::Account(domain::Event::Component(event))));
        }
    }
    fn granted(
        &self,
        account: u32,
        generation: u64,
        value: table::Value,
        expires: Time,
        values: &mut Values,
    ) -> domain::Form {
        // An account with a provider account id is jig's live test's
        // (D2): its bearer goes out beside the id until JP13.
        let id = self.ids.get(&account).expect("configured account id");
        let value = if id.expose().is_empty() {
            value
        } else {
            table::Value { token: value.token, scheme: table::Scheme::Withdrawn { id: id.clone() } }
        };
        let form = translate::form(&value.scheme);
        values
            .accounts
            .granted(account, generation, value, expires)
            .expect("startup checks token and account id bounds");
        form
    }
    fn erase(
        &mut self,
        source: u32,
        reason: Erasing,
        events: &mut Queue<root::Event>,
        store: &mut crate::store::Store,
    ) {
        if let Some(pending) = self.erasing.get_mut(&source) {
            match pending {
                Erasing::Revoked => match reason {
                    Erasing::SignOut { confirmed } => *pending = Erasing::SignOut { confirmed },
                    Erasing::Revoked | Erasing::Removing => (),
                },
                Erasing::SignOut { .. } | Erasing::Removing => (),
            }
            return;
        }
        self.erasing.insert(source, reason).expect("one erase per driver account");
        if let Some(store::Event::Erased { owner }) = store.secrets(
            Token::new(PREFIX | u64::from(source)),
            store::Secrets::Erase { owner: store::SecretOwner::Source, number: u64::from(source) },
        ) {
            self.erased(source, owner, events);
        }
    }
    fn erased(&mut self, number: u32, _owner: Token, events: &mut Queue<root::Event>) {
        let Some(pending) = self.erasing.remove(&number) else { return };
        match pending {
            Erasing::Revoked => self.records.erased(number),
            Erasing::SignOut { confirmed } => {
                self.records.erased(number);
                events.push(root::Event::Core(core::Event::Account(domain::Event::SignedOut {
                    source: number,
                    confirmed,
                })));
                if self.ids.contains_key(&number) {
                    self.erasing.insert(number, Erasing::Removing).expect("the removed erase's place");
                    self.down(driver::Request::Remove { account: number });
                }
            }
            Erasing::Removing => {
                self.erasing.insert(number, Erasing::Removing).expect("the removed erase's place");
            }
        }
    }
    /// The store's answers: the records it holds, and whether each write was
    /// kept (store.md 9).
    fn answers(&mut self, store: &mut crate::store::Store, events: &mut Queue<root::Event>) {
        for _ in 0..store.sources.len() {
            if !self.down_room() || events.room() == 0 || self.key_out.room() == 0 {
                break;
            }
            match store.sources.pop().expect("source answers") {
                store::Event::SecretsLoaded { owner: _, rows, next } => self.loaded(rows, next, &store.installation),
                store::Event::Kept { owner } if owner.raw() & KEY_EXCHANGE == KEY_EXCHANGE => {
                    if let Some(next) = self.keys.kept(Token::new(owner.raw() & !KEY_EXCHANGE), true, self.env.now) {
                        self.key_out.push(next);
                    }
                }
                store::Event::Stale { owner } if owner.raw() & KEY_EXCHANGE == KEY_EXCHANGE => {
                    if let Some(next) = self.keys.kept(Token::new(owner.raw() & !KEY_EXCHANGE), false, self.env.now) {
                        self.key_out.push(next);
                    }
                }
                store::Event::Kept { owner } => {
                    if let Some(request) = self.records.kept(source(owner)) {
                        self.down(request);
                    }
                }
                store::Event::Stale { owner } => {
                    if let Some(request) = self.records.stale(source(owner)) {
                        self.down(request);
                    }
                }
                store::Event::Erased { owner } => self.erased(source(owner), owner, events),
                store::Event::Secret { .. } | store::Event::Carried { .. } => {
                    unreachable!("the accounts only load and replace their records")
                }
                store::Event::Opened { .. }
                | store::Event::OpenRefused { .. }
                | store::Event::Committed { .. }
                | store::Event::Failed { .. }
                | store::Event::Restored { .. }
                | store::Event::Loaded { .. }
                | store::Event::Read { .. }
                | store::Event::Got { .. }
                | store::Event::Turns { .. }
                | store::Event::Unread { .. }
                | store::Event::Corrupt { .. } => {
                    unreachable!("the store hands the accounts only their secrets' answers")
                }
            }
        }
    }
    /// One page of the store's records; after the last, the driver starts
    /// from them (llm.md 5.1), each sign-in sending the installation's
    /// identifier the store keeps (llm.md 5.4).
    fn loaded(&mut self, rows: Box<[store::SecretRow]>, next: Option<u64>, kept: &[u8; 16]) {
        let Phase::Loading { configured, after, asked, .. } = &mut self.phase else {
            return;
        };
        for row in rows {
            if row.owner != store::SecretOwner::Source {
                continue;
            }
            let Ok(account) = u32::try_from(row.number) else {
                continue;
            };
            if !configured.contains_key(&account) {
                continue;
            }
            let Some(record) = self.records.load(account, row.value.expose(), &self.env.limits.client.document) else {
                continue;
            };
            if let Some(driver::Account::SignIn { keeper, .. }) = configured.get_mut(&account) {
                let id = match &record.metadata {
                    Some(id) => Secret::new(id.clone()),
                    None => Secret::new(Box::default()),
                };
                *self.ids.get_mut(&account).expect("configured account id slot") = id;
                *keeper = driver::Keeper::Owner { kept: Some(record) };
            }
        }
        if let Some(next) = next {
            *after = Some(next);
            *asked = false;
            return;
        }
        let Phase::Loading { mut configured, crypto, .. } = ::core::mem::replace(&mut self.phase, Phase::HandingIn)
        else {
            unreachable!("loading");
        };
        let mut component =
            driver::Component::new(List::with_capacity(0), &self.env.limits, crypto).expect("startup checked driver");
        for _ in 0..configured.len() {
            let (&number, _) = configured.first().expect("configured sources");
            let mut account = configured.remove(&number).expect("configured source");
            if let driver::Account::SignIn { registration, .. } = &mut account {
                registration.terms = Some(installation::terms(kept));
            }
            component.add(&self.env, number, account, &mut self.above);
            assert!(self.above.is_empty(), "startup checked configured source");
            self.records.admit(number).expect("configured record place");
        }
        // The stored installation id, rather than this start's draw, goes on account sign-ins.
        let mut after = 0;
        for _ in 0..self.providers.len() {
            let Some((&number, _)) = self.providers.range(after..).next() else { break };
            if let Some(driver::Account::SignIn { registration, .. }) = self.providers.get_mut(&number)
                && let Some(terms) = &mut registration.terms
            {
                terms.parameters.first_mut().expect("installation parameter").value = installation::urn(kept);
            }
            let Some(next) = number.checked_add(1) else { break };
            after = next;
        }
        self.component = Some(component);
    }
    /// Asks for the next page of records, hands in the handed-in records
    /// once kept, then lets the domain's requests through.
    fn start(&mut self, store: &mut crate::store::Store) {
        match &mut self.phase {
            Phase::Loading { after, asked, .. } => {
                if !*asked && store.opened && store.room() {
                    *asked = true;
                    let load = store::Secrets::Load { owner: store::SecretOwner::Source, after: *after };
                    if let Some(store::Event::SecretsLoaded { owner: _, rows, next }) = store.secrets(LOAD, load) {
                        self.loaded(rows, next, &store.installation);
                    }
                }
            }
            Phase::HandingIn => {
                for _ in 0..self.handed.len() {
                    if !store.room() || !self.down_room() {
                        break;
                    }
                    let (account, record) = self.handed.pop().expect("handed-in records");
                    match self.records.hand_in(account, record, &self.env.limits.client.document) {
                        Ok(records::Next::Driver(request)) => self.down(request),
                        Ok(records::Next::Store(keep)) => replace(store, keep),
                        Err(handed_in::Unfit) => unreachable!("startup encoded each handed-in record"),
                    }
                }
                if self.handed.is_empty() && !self.records.keeping() {
                    self.phase = Phase::Running;
                }
            }
            Phase::Running => {
                for _ in 0..self.early.len() {
                    if !self.down_room() {
                        break;
                    }
                    let request = self.early.pop().expect("early requests");
                    self.dispatch(request);
                }
            }
        }
    }
}
/// Asks the store to keep a source's record in place of the one it holds.
fn replace(store: &mut crate::store::Store, keep: records::Keep) {
    let records::Keep { account, value, replaces } = keep;
    let _later = store.secrets(
        Token::new(PREFIX | u64::from(account)),
        store::Secrets::Replace { owner: store::SecretOwner::Source, number: u64::from(account), value, replaces },
    );
}
fn source(owner: Token) -> u32 {
    u32::try_from(owner.raw() & !(15_u64 << 60)).expect("a source's number")
}
pub(super) fn earliest(left: Option<Time>, right: Option<Time>) -> Option<Time> {
    match right {
        Some(right) => Some(match left {
            Some(left) => left.min(right),
            None => right,
        }),
        None => left,
    }
}

#[derive(Debug)]
pub(super) struct Kernel {
    routes: Map<Token, (Token, Option<Token>)>,
    retired: Map<Token, Token>,
    next: u64,
}
impl Kernel {
    pub fn new(room: u32) -> Self {
        Self { routes: Map::with_capacity(room), retired: Map::with_capacity(room), next: 1 }
    }
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty() && self.retired.is_empty()
    }
    pub fn publish(&mut self, source: &mut Queue<Submit>, target: &mut Queue<Submit>) {
        for _ in 0..source.len() {
            if target.room() == 0 || self.routes.len() == self.routes.capacity() {
                break;
            }
            let mut submit = source.pop().expect("kernel submissions");
            let mut cancel = None;
            if let Op::Cancel { target } = &mut submit.kind {
                cancel = Some(*target);
                let mut found = None;
                for (external, (internal, _)) in &self.routes {
                    if *internal == *target {
                        found = Some(*external);
                        break;
                    }
                }
                *target = match found {
                    Some(external) => external,
                    None => self.retired.remove(target).expect("cancel names submitted operation"),
                };
            }
            let external = Token::new(self.next);
            self.next = self.next.checked_add(1).expect("kernel operation ids");
            self.routes.insert(external, (submit.op, cancel)).expect("kernel route room");
            submit.op = external;
            target.push(submit);
        }
    }
    pub fn complete(&mut self, mut complete: Complete, sockets: &Queue<Submit>) -> Complete {
        let (internal, cancel) = self.routes.remove(&complete.op).expect("submitted operation route");
        for queued in sockets {
            if let Op::Cancel { target } = queued.kind
                && target == internal
            {
                self.retired.insert(internal, complete.op).expect("pending cancellation route");
                break;
            }
        }
        complete.op = internal;
        if let Some(internal) = cancel
            && let Op::Cancel { target } = &mut complete.kind
        {
            *target = internal;
        }
        complete
    }
}

pub(super) fn owns(token: Token) -> bool {
    token.raw() & (15_u64 << 60) == PREFIX
}
pub(super) fn tag(mut request: io::Request) -> io::Request {
    match &mut request {
        io::Request::Listen { owner, .. }
        | io::Request::Connect { owner, .. }
        | io::Request::Bind { owner, .. }
        | io::Request::Spawn { owner, .. }
        | io::Request::Usage { owner }
        | io::Request::Window { owner, .. } => {
            assert!(owner.raw() < (1_u64 << 60_u32), "bounded accounts exchange slots");
            *owner = Token::new(owner.raw() | PREFIX);
        }
        io::Request::Reject { .. }
        | io::Request::Stream { .. }
        | io::Request::Output { .. }
        | io::Request::Signal { .. }
        | io::Request::Close { .. }
        | io::Request::Abort { .. } => {}
    }
    request
}
pub(super) fn untag(mut event: io::Event) -> io::Event {
    match &mut event {
        io::Event::Listening { owner, .. }
        | io::Event::Accepted { owner, .. }
        | io::Event::Connecting { owner, .. }
        | io::Event::Connected { owner }
        | io::Event::Stream { owner, .. }
        | io::Event::Output { owner, .. }
        | io::Event::Spawned { owner, .. }
        | io::Event::Exited { owner, .. }
        | io::Event::Usage { owner, .. }
        | io::Event::Window { owner, .. }
        | io::Event::Failed { owner, .. }
        | io::Event::Closed { owner } => {
            *owner = Token::new(owner.raw() & !(15_u64 << 60));
        }
        io::Event::Resized | io::Event::Shutdown { .. } => {}
    }
    event
}
/// The accounts' bytes beside the driver's own: their queues, the sources
/// as configured until the store's records are read, the handed-in records
/// until kept, and each source's record on its way to the store.
pub(super) fn worst_case(limits: &driver::Limits, io: &io::Limits, room: u32) -> Option<u64> {
    let record = u64::from(limits.client.document.record_bytes);
    Queue::<io::Request>::worst_case(room)?
        .checked_add(Queue::<driver::Event>::worst_case(room)?)?
        .checked_add(Queue::<driver::FileLower>::worst_case(room)?)?
        .checked_add(Queue::<domain::Request>::worst_case(room)?)?
        .checked_add(Map::<u32, driver::Account>::worst_case(limits.accounts)?)?
        .checked_add(Queue::<(u32, oauth::SavedToken)>::worst_case(limits.accounts)?)?
        .checked_add(u64::from(limits.accounts).checked_mul(record)?)?
        .checked_add(records::worst_case(limits.accounts, &limits.client.document)?)?
        .checked_add(keys::worst_case(limits.accounts, limits.client.document.token_bytes)?)?
        .checked_add(Queue::<keys::Next>::worst_case(room)?)?
        .checked_add(Queue::<domain::FromComponent>::worst_case(room)?)?
        .checked_add(Queue::<u32>::worst_case(room)?)?
        .checked_add(Map::<u32, Secret>::worst_case(limits.accounts)?)?
        .checked_add(Map::<u32, Box<[u8]>>::worst_case(limits.accounts)?)?
        .checked_add(Map::<u32, Erasing>::worst_case(limits.accounts)?)?
        .checked_add(u64::from(limits.accounts).checked_mul(u64::from(limits.client.scope_bytes))?)?
        .checked_add(u64::from(limits.accounts).checked_mul(u64::from(table::ACCOUNT_ID_BYTES))?)?
        .checked_add(u64::from(room).checked_mul(record.checked_add(4096)?)?)?
        .checked_add(Map::<Token, (Token, Option<Token>)>::worst_case(io::operations(io)?.checked_add(2)?)?)?
        .checked_add(Map::<Token, Token>::worst_case(io::operations(io)?.checked_add(2)?)?)
}

fn copy_endpoint(endpoint: &driver::Endpoint) -> driver::Endpoint {
    driver::Endpoint {
        address: endpoint.address,
        transport: match &endpoint.transport {
            driver::Transport::Plaintext => driver::Transport::Plaintext,
            driver::Transport::Tls { server_name, trust } => {
                driver::Transport::Tls { server_name: server_name.clone(), trust: trust.clone() }
            }
        },
    }
}
fn copy_origins(origins: &List<driver::Origin>) -> List<driver::Origin> {
    let mut copy = List::with_capacity(origins.len());
    for origin in origins {
        copy.push(driver::Origin { uri: origin.uri.clone(), endpoint: copy_endpoint(&origin.endpoint) })
            .expect("configured origins");
    }
    copy
}
fn has_tls(endpoint: &driver::Endpoint, origins: &List<driver::Origin>) -> bool {
    match endpoint.transport {
        driver::Transport::Tls { .. } => return true,
        driver::Transport::Plaintext => (),
    }
    for origin in origins {
        match origin.endpoint.transport {
            driver::Transport::Tls { .. } => return true,
            driver::Transport::Plaintext => (),
        }
    }
    false
}
fn event_account(event: &driver::Event) -> Option<u32> {
    match event {
        driver::Event::Granted { account, .. }
        | driver::Event::Expiring { account, .. }
        | driver::Event::Failed { account, .. }
        | driver::Event::Refused { account, .. }
        | driver::Event::Visit { account, .. }
        | driver::Event::SignedIn { account, .. }
        | driver::Event::Keep { account, .. }
        | driver::Event::Removed { account }
        | driver::Event::SignedOut { account, .. }
        | driver::Event::Lock { account }
        | driver::Event::Unlock { account } => Some(*account),
        driver::Event::Closed => None,
    }
}

/// The public account sign-in templates retained beside the driver. Two
/// one-account driver bounds include their origins and the temporary startup copy.
pub(super) fn configured_bytes(config: &config::Engine) -> Option<u64> {
    let count = u32::try_from(config.llm.providers.len()).ok()?;
    let limits = driver::Limits { accounts: 1, exchanges: 1, listeners: 1, ..config.limits.driver };
    Map::<u32, driver::Account>::worst_case(count)?
        .checked_add(driver::worst_case(&limits)?.checked_mul(u64::from(count))?.checked_mul(2)?)
}

#[cfg(test)]
#[path = "tests/accounts.rs"]
mod tests;
