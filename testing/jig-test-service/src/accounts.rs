//! The accounts driver, its private keeper and secret-free translation (llm.md, 5).
use crate::{Files, Seeds, values::Values};
use alloc::boxed::Box;
use jig_domain_accounts as domain;
use jig_domain_core as core;
use jig_protocol_accounts::{handed_in, table, translate};
use jig_protocol_config::{self as config, Path, Problems, Segment, What};
use jig_test_domain as root;
use skein_io::{
    self as io, file, file_layer,
    kernel::{Complete, Fd, Op, Submit},
};
use skein_lib::{CryptoRng, Decimal, Env, List, Map, Queue, Secret, Time, Token, Wall};
use skein_oauth as oauth;
use skein_oauth_accounts as driver;

pub(super) const PREFIX: u64 = 3_u64 << 60;
#[derive(Debug)]
pub struct AccountsRuntime {
    pub root: Fd,
    pub effective_user: u32,
    pub issuers: List<(u32, driver::Endpoint)>,
    /// Bounds of all prepared trust roots, supplied by their shell owner.
    pub trust_roots: u32,
    pub trust_root_bytes: u32,
}
pub(super) struct Accounts {
    component: driver::Component,
    env: Env<driver::Limits>,
    ids: List<Secret>,
    loads: Map<Token, u32>,
    pub(super) io: Queue<io::Request>,
    above: Queue<driver::Event>,
    files: Queue<driver::FileLower>,
    pending: Queue<(file::Request, Time)>,
    pub(super) submitted: Queue<Submit>,
    filed: Queue<file::Event>,
    keeper: Option<file_layer::FileIo>,
    root: Option<Token>,
    closed: bool,
    initial: Option<Box<[driver::Request]>>,
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
fn private_keeper(
    runtime: Option<&AccountsRuntime>,
    limits: &driver::Limits,
    seeds: &Seeds,
) -> Result<(Option<file_layer::FileIo>, Option<Token>), Problems> {
    let Some(runtime) = runtime else {
        return Ok((None, None));
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
    let mut keeper = file_layer::FileIo::with_whole_limit(
        limits.accounts.checked_add(2).expect("private roots"),
        limits.http.read,
        1,
        limits.client.document.record_bytes,
        limits.file_stall,
    )
    .with_effective_user(runtime.effective_user);
    keeper.seed_randomness(seeds.seed);
    let adopted = keeper.adopt_root(runtime.root).expect("root slot");
    Ok((Some(keeper), Some(adopted)))
}
impl Accounts {
    pub fn new(
        config: &config::Engine,
        files: &Files,
        seeds: &Seeds,
        runtime: Option<AccountsRuntime>,
        room: u32,
    ) -> Result<Self, Problems> {
        let mut limits = config.limits.driver;
        limits.accounts = config.limits.core.accounts.accounts.checked_add(1).expect("numbered account slots");

        let (keeper, file_root) = private_keeper(runtime.as_ref(), &limits, seeds)?;
        let mut issuers = Map::with_capacity(limits.accounts);
        if let Some(runtime) = runtime {
            for (number, endpoint) in runtime.issuers.into_boxed() {
                issuers.insert(number, endpoint).or(Err(problem(0, What::Bound)))?;
            }
        }
        let mut configured = List::with_capacity(limits.accounts);
        let mut ids = List::with_capacity(limits.accounts);
        let mut records = List::with_capacity(limits.accounts);
        for _ in 0..limits.accounts {
            configured.push(driver::Account::HandedIn).or(Err(problem(0, What::Bound)))?;
            ids.push(Secret::new(Box::default())).expect("account id slots");
        }
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
                    records
                        .push(
                            handed_in::hand_in(
                                account.number,
                                record.token,
                                record.lapses,
                                record.account_id,
                                &limits.client.document,
                            )
                            .or(Err(problem(index, What::Bound)))?,
                        )
                        .or(Err(problem(index, What::Bound)))?;
                    driver::Account::HandedIn
                }
                config::Source::SignIn(source) => {
                    let private_root = file_root.ok_or(problem(index, What::Refers))?;
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
                            terms: None,
                        },
                        endpoint,
                        keeper: driver::Keeper::Private {
                            root: private_root,
                            directory: config.llm.records.clone(),
                            file: Decimal::of(u64::from(account.number)).as_bytes().into(),
                        },
                    }
                }
            };
            *configured.get_mut(account.number).ok_or(problem(index, What::Bound))? = source;
            *ids.get_mut(account.number).expect("account id slot") = account_id;
        }
        let crypto = CryptoRng::new(&seeds.crypto, 1).ok_or(problem(0, What::Bound))?;
        let component = driver::Component::new(configured, &limits, crypto).or(Err(problem(0, What::Form)))?;
        let value = Self {
            component,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            ids,
            loads: Map::with_capacity(limits.accounts),
            io: Queue::with_capacity(room),
            above: Queue::with_capacity(room),
            files: Queue::with_capacity(room),
            pending: Queue::with_capacity(room),
            submitted: Queue::with_capacity(room),
            filed: Queue::with_capacity(room),
            keeper,
            root: file_root,
            closed: false,
            initial: Some(records.into_boxed()),
        };
        Ok(value)
    }
    fn down(&mut self, request: driver::Request) {
        self.component.down(&self.env, request, &mut self.above, &mut self.io, &mut self.files);
    }
    pub fn up_room(&self) -> bool {
        self.above.room() >= driver::MAX_OUT_UP.above && self.io.room() >= driver::MAX_OUT_UP.io
    }
    pub fn batch_room(&self, batch: u32) -> bool {
        self.above.room() >= batch.checked_mul(driver::MAX_OUT_DOWN.above).expect("account above room")
            && self.io.room() >= batch.checked_mul(driver::MAX_OUT_DOWN.io).expect("account io room")
            && self.files.room() >= batch.checked_mul(driver::MAX_OUT_DOWN.files).expect("account files room")
    }
    pub fn request(&mut self, request: domain::Request) {
        if let domain::Request::Component(request) = request {
            self.down(translate::down(request));
        }
    }
    pub fn clock(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
    }
    pub fn up(&mut self, event: io::Event) {
        self.component.up(&self.env, event, &mut self.above, &mut self.io, &mut self.files);
    }
    pub fn complete(&mut self, completion: Complete) {
        file_layer::up(
            self.keeper.as_mut().expect("file operation has keeper"),
            completion,
            &mut self.filed,
            &mut self.submitted,
        );
    }
    pub fn is_empty(&self) -> bool {
        self.closed
            && self.loads.is_empty()
            && self.root.is_none()
            && self.io.is_empty()
            && self.above.is_empty()
            && self.pending.is_empty()
            && self.files.is_empty()
            && self.submitted.is_empty()
            && self.filed.is_empty()
            && match &self.keeper {
                Some(keeper) => keeper_empty(keeper),
                None => true,
            }
    }
    pub fn next_deadline(&self) -> Option<Time> {
        let mut next = self.component.next_deadline();
        if let Some(keeper) = &self.keeper {
            next = earliest(next, keeper.next_deadline());
        }
        for (_, deadline) in &self.pending {
            next = earliest(next, Some(*deadline));
        }
        next
    }
    pub fn work_pending(&self, now: Time) -> bool {
        self.component.has_work()
            || !self.above.is_empty()
            || !self.io.is_empty()
            || !self.files.is_empty()
            || !self.filed.is_empty()
            || !self.submitted.is_empty()
            || (match &self.keeper {
                Some(keeper) => keeper.takes(),
                None => false,
            } && !self.pending.is_empty())
            || match self.next_deadline() {
                Some(deadline) => deadline <= now,
                None => false,
            }
    }
    pub fn abort(&mut self) {
        self.down(driver::Request::Abort);
    }
    pub fn iterate(&mut self, now: Time, wall: Wall, values: &mut Values, events: &mut Queue<root::Event>) {
        self.clock(now, wall);
        if let Some(records) = self.initial.take() {
            for record in records {
                self.down(record);
            }
        }
        if self.above.room() >= 3 && self.io.room() >= 3 && self.files.room() >= 1 {
            self.component.fire(&self.env, &mut self.above, &mut self.io, &mut self.files);
        }
        self.drive_files();
        for _ in 0..self.filed.len() {
            if self.above.room() < 3 || self.io.room() < 3 || self.files.room() < 1 {
                break;
            }
            let mut event = self.filed.pop().expect("file events");
            let expected = self.loads.remove(&event.owner());
            if let Some(expected) = expected
                && let file::Event::Loaded { bytes, .. } = &mut event
                && let Ok(record) = oauth::decode_record(bytes, &self.env.limits.client.document)
                && record.key == expected
            {
                let id = record.metadata.unwrap_or_default();
                if id.len() > usize::try_from(table::ACCOUNT_ID_BYTES).expect("account id bound") {
                    *bytes = Box::default();
                } else {
                    *self.ids.get_mut(expected).expect("configured account id slot") = Secret::new(id);
                }
            }
            if event.owner() != Token::new(u64::MAX) {
                self.component.filed(&self.env, event, &mut self.above, &mut self.io, &mut self.files);
            }
        }
        for _ in 0..self.above.len() {
            if events.room() == 0 {
                break;
            }
            let event = self.above.pop().expect("account events");
            let event = match translate::up(event, now) {
                translate::Up::Granted { event, account, generation, value, expires } => {
                    values
                        .accounts
                        .granted(
                            account,
                            generation,
                            table::Value {
                                token: value,
                                account_id: self.ids.get(account).expect("configured account id").clone(),
                            },
                            expires,
                        )
                        .expect("startup checks token and account id bounds");
                    event
                }
                translate::Up::Child(event) => event,
            };
            match &event {
                domain::FromComponent::Granted { .. }
                | domain::FromComponent::Expiring { .. }
                | domain::FromComponent::Failed { .. } => {}
                domain::FromComponent::Closed => self.closed = true,
            }
            events.push(root::Event::Core(core::Event::Account(domain::Event::Component(event))));
        }
        values.accounts.expire(now);
        self.component.reclaim();
    }
    fn drive_files(&mut self) {
        for _ in 0..self.files.len() {
            match self.files.pop().expect("private requests") {
                driver::FileLower::Request { request, deadline } => {
                    if let file::Request::Load { owner, path, .. } = &request {
                        let number = crate::tools::number(path).expect("private record named by configured account");
                        self.loads.insert(*owner, number).expect("one load per configured account");
                    }
                    self.pending.push((request, deadline));
                }
                driver::FileLower::Remove { .. } => unreachable!("the owner never signs out or removes an account"),
                driver::FileLower::Cancel { owner } => {
                    for _ in 0..self.pending.len() {
                        let (request, deadline) = self.pending.pop().expect("queued files");
                        if file_owner(&request) == owner {
                            self.filed.push(file::Event::Cancelled { owner });
                        } else {
                            self.pending.push((request, deadline));
                        }
                    }
                    if let Some(keeper) = &mut self.keeper {
                        file_layer::cancel(keeper, owner, &mut self.submitted);
                    }
                }
            }
        }
        if let Some(keeper) = &mut self.keeper {
            file_layer::expire(keeper, self.env.now, &mut self.submitted);
            if keeper.takes() {
                if let Some((request, deadline)) = self.pending.pop() {
                    if deadline <= self.env.now {
                        self.filed.push(file::Event::Failed {
                            owner: file_owner(&request),
                            error: io::kernel::Error::TimedOut,
                            committed: false,
                            residue: None,
                        });
                    } else {
                        file_layer::down_until(keeper, deadline, request, &mut self.filed, &mut self.submitted);
                    }
                } else if self.closed
                    && let Some(file) = self.root.take()
                {
                    file_layer::down(
                        keeper,
                        self.env.now,
                        file::Request::Close { owner: Token::new(u64::MAX), file },
                        &mut self.filed,
                        &mut self.submitted,
                    );
                }
            }
        }
    }
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
fn file_owner(request: &file::Request) -> Token {
    match request {
        file::Request::MakeDirectory { owner, .. }
        | file::Request::OpenPrivate { owner, .. }
        | file::Request::Create { owner, .. }
        | file::Request::CreateNoFollow { owner, .. }
        | file::Request::OpenRead { owner, .. }
        | file::Request::OpenReadNoFollow { owner, .. }
        | file::Request::OpenWrite { owner, .. }
        | file::Request::OpenWriteNoFollow { owner, .. }
        | file::Request::OpenDirectory { owner, .. }
        | file::Request::Load { owner, .. }
        | file::Request::Scan { owner, .. }
        | file::Request::Store { owner, .. }
        | file::Request::Stat { owner, .. }
        | file::Request::WriteAt { owner, .. }
        | file::Request::ReadAt { owner, .. }
        | file::Request::Cut { owner, .. }
        | file::Request::Sync { owner, .. }
        | file::Request::Close { owner, .. }
        | file::Request::SyncDirectory { owner, .. }
        | file::Request::Rename { owner, .. }
        | file::Request::Remove { owner, .. }
        | file::Request::List { owner, .. }
        | file::Request::ListPage { owner, .. }
        | file::Request::Lock { owner, .. } => *owner,
    }
}

#[derive(Debug)]
pub(super) struct Kernel {
    routes: Map<Token, (bool, Token, Option<Token>)>,
    retired: Map<(bool, Token), Token>,
    next: u64,
}
impl Kernel {
    pub fn new(room: u32) -> Self {
        Self { routes: Map::with_capacity(room), retired: Map::with_capacity(room), next: 1 }
    }
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty() && self.retired.is_empty()
    }
    pub fn publish(&mut self, files: bool, source: &mut Queue<Submit>, target: &mut Queue<Submit>) {
        for _ in 0..source.len() {
            if target.room() == 0 || self.routes.len() == self.routes.capacity() {
                break;
            }
            let mut submit = source.pop().expect("kernel submissions");
            let mut cancel = None;
            if let Op::Cancel { target } = &mut submit.kind {
                cancel = Some(*target);
                let mut found = None;
                for (external, (owner, internal, _)) in &self.routes {
                    if *owner == files && *internal == *target {
                        found = Some(*external);
                        break;
                    }
                }
                *target = match found {
                    Some(external) => external,
                    None => self.retired.remove(&(files, *target)).expect("cancel names submitted operation"),
                };
            }
            let external = Token::new(self.next);
            self.next = self.next.checked_add(1).expect("kernel operation ids");
            self.routes.insert(external, (files, submit.op, cancel)).expect("kernel route room");
            submit.op = external;
            target.push(submit);
        }
    }
    pub fn complete(
        &mut self,
        mut complete: Complete,
        sockets: &Queue<Submit>,
        files_queue: Option<&Queue<Submit>>,
    ) -> (bool, Complete) {
        let (files, internal, cancel) = self.routes.remove(&complete.op).expect("submitted operation route");
        let source = if files { files_queue.expect("file route has queue") } else { sockets };
        for queued in source {
            if let Op::Cancel { target } = queued.kind
                && target == internal
            {
                self.retired.insert((files, internal), complete.op).expect("pending cancellation route");
                break;
            }
        }
        complete.op = internal;
        if let Some(internal) = cancel
            && let Op::Cancel { target } = &mut complete.kind
        {
            *target = internal;
        }
        (files, complete)
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
pub(super) fn worst_case(limits: &driver::Limits, io: &io::Limits, room: u32) -> Option<u64> {
    Queue::<io::Request>::worst_case(room)?
        .checked_add(Queue::<driver::Event>::worst_case(room)?)?
        .checked_add(Queue::<driver::FileLower>::worst_case(room)?)?
        .checked_add(Queue::<(file::Request, Time)>::worst_case(room)?)?
        .checked_add(Queue::<Submit>::worst_case(room)?)?
        .checked_add(Queue::<file::Event>::worst_case(room)?)?
        .checked_add(List::<Secret>::worst_case(limits.accounts)?)?
        .checked_add(Map::<Token, u32>::worst_case(limits.accounts)?)?
        .checked_add(u64::from(limits.accounts).checked_mul(u64::from(table::ACCOUNT_ID_BYTES))?)?
        .checked_add(u64::from(room).checked_mul(u64::from(limits.client.document.record_bytes).checked_add(4096)?)?)?
        .checked_add(List::<driver::Request>::worst_case(limits.accounts)?)?
        .checked_add(Map::<Token, (bool, Token, Option<Token>)>::worst_case(io::operations(io)?.checked_add(2)?)?)?
        .checked_add(Map::<(bool, Token), Token>::worst_case(io::operations(io)?.checked_add(2)?)?)
}

fn keeper_empty(keeper: &file_layer::FileIo) -> bool {
    keeper.takes() && keeper.open_files() == 0
}
