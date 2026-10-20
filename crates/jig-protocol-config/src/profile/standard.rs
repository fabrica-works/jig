//! A team: eight workers, four slots each, sixteen LLM accounts (configuration.md, 6).
//! Every field omitted below has the small profile's documented value. Token: 16384 bytes.
//! Byte limits are bytes; durations below state their units. Counts are items.
//! **tasks**: `tasks=32, funders=32, project_tasks=32, project_goals=16, tree_tasks=128, delegates=8, resource_reports=256, pools=32, hold_waiters=32, charters=8, facts=128, room_refusals=32, batch=4, dependencies=16, inputs=16, references=16, holdings=8, parameters=8, saved_resources=8, authority_grants=8, authority_bytes=4096, executor_kinds=8`.
//! **people**: `operations=16, people=16, inbox_tasks=64, sign_ins=32, projects=8, holdings=32, goals=64, initial_owners=4, requests=128, room_refusals=32, pending=32, facts=128`.
//! **fleet**: `needs_bytes=262144, grants=32, workers=8, slots=4, workstreams=64, attempts=64, calls=256, turns=128, facts=256, history=64`.
//! **authority**: `projects=8, accounts=16, roles=8, requirements=8, grants=8, executors=8, batch=4, writes=8`.
//! **brief**: `briefs=32`.
//! **accounts**: `credentials=15, accounts=16, sign_ins=4, facts=128`.
//! **views**: `runs=32, watchers=16, facts=256`.
//! **notes**: `scopes=16, entries_per_scope=64, body_bytes=8192, lines=8, recalled=8`.
//! **documents**: `rows=8, text_bytes=8192, chips=8, lines=8`.
//! **core**: `templates=16, fill_mark=1073741824, connectors=4, load_slots=16, call_records=128, tasks=TASKS, authority=AUTHORITY, people=PEOPLE, fleet=FLEET, brief=BRIEF, accounts=ACCOUNTS, notes=NOTES, views=VIEWS, resume_bytes=262144, run_bytes=65536, documents=DOCUMENTS`.
//! **hub**: `slots=4, accounts=32, facts=256, told=256`.
//! **link**: `opening=16, facts=256`.
//! **driver**: `exchanges=8, listeners=ACCOUNTS.sign_ins`.
use super::small;
use super::{Limits, Link};
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_brief as brief;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
pub(super) const TASKS: tasks::Limits = tasks::Limits {
    tasks: 32,
    funders: 32,
    project_tasks: 32,
    project_goals: 16,
    tree_tasks: 128,
    delegates: 8,
    resource_reports: 256,
    pools: 32,
    hold_waiters: 32,
    charters: 8,
    facts: 128,
    room_refusals: 32,
    batch: 4,
    dependencies: 16,
    inputs: 16,
    references: 16,
    holdings: 8,
    parameters: 8,
    saved_resources: 8,
    authority_grants: 8,
    authority_bytes: 4_096,
    executor_kinds: 8,
    ..small::TASKS
};
pub(super) const PEOPLE: people::Limits = people::Limits {
    operations: 16,
    batch: 4,
    people: 16,
    inbox_tasks: 64,
    sign_ins: 32,
    projects: 8,
    holdings: 32,
    initial_owners: 4,
    requests: 128,
    room_refusals: 32,
    pending: 32,
    facts: 128,
    ..small::PEOPLE
};
pub(super) const FLEET: fleet::Limits = fleet::Limits {
    workers: 8,
    needs_bytes: 262_144,
    grants: 32,
    slots: 4,
    workstreams: 64,
    attempts: 64,
    calls: 256,
    turns: 128,
    facts: 256,
    history: 64,
    ..small::FLEET
};
pub(super) const AUTHORITY: authority::Limits = authority::Limits {
    projects: 8,
    accounts: 16,
    roles: 8,
    requirements: 8,
    grants: 8,
    executors: 8,
    batch: 4,
    writes: 8,
    ..small::AUTHORITY
};
pub(super) const BRIEF: brief::Limits = brief::Limits { briefs: 32, ..small::BRIEF };
pub(super) const ACCOUNTS: accounts::Limits =
    accounts::Limits { accounts: 16, sign_ins: 4, credentials: 15, facts: 128, ..small::ACCOUNTS };
pub(super) const VIEWS: views::Limits = views::Limits { runs: 32, watchers: 16, facts: 256, ..small::VIEWS };
pub(super) const NOTES: notes::Limits =
    notes::Limits { scopes: 16, entries_per_scope: 64, body_bytes: 8_192, lines: 8, recalled: 8, ..small::NOTES };
pub(super) const DOCUMENTS: core::documents::DocumentLimits = core::documents::DocumentLimits {
    roles: AUTHORITY.roles,
    grants: super::authority_grants(&TASKS, &AUTHORITY),
    executors: super::authority_executors(&TASKS, &AUTHORITY),
    implications: AUTHORITY.implications,
    rows: 8,
    text_bytes: 8_192,
    chips: 8,
    lines: 8,
    ..small::DOCUMENTS
};
pub(super) const CORE: core::Limits = core::Limits {
    templates: 16,
    fill_mark: 1_073_741_824,
    connectors: 4,
    load_slots: 16,
    call_records: 128,
    tasks: TASKS,
    authority: AUTHORITY,
    people: PEOPLE,
    fleet: FLEET,
    brief: BRIEF,
    accounts: ACCOUNTS,
    notes: NOTES,
    views: VIEWS,
    resume_bytes: 262_144,
    run_bytes: 65_536,
    documents: DOCUMENTS,
    ..small::CORE
};
pub(super) const HUB: host::Limits = host::Limits { slots: 4, accounts: 32, facts: 256, told: 256, ..small::HUB };
pub(super) const LINK: Link = Link { names: 64, opening: 16, facts: 256, ..small::LINK };
pub(super) const LIMITS: Limits = Limits {
    core: CORE,
    store: STORE,
    hub: HUB,
    link: LINK,
    // Smith keeps the two-byte id length and up to 256 id bytes in its 8192-byte value.
    token: 7934,
    driver: skein_oauth_accounts::Limits {
        accounts: CORE.accounts.accounts.checked_add(1).expect("account zero slot"),
        exchanges: 8,
        listeners: CORE.accounts.sign_ins,
        io: skein_io::Limits { sockets: 64, names: 64, ..small::IO },
        ..small::DRIVER
    },
    clients: CLIENTS,
    clients_http: jig_protocol_engine_clients_http::Limits {
        calls: 16,
        streams: 8,
        body: CLIENTS.message_bytes,
        read: 16384,
        send: 16384,
        event: 87416,
    },
    io: skein_io::Limits { sockets: 64, names: 64, ..small::IO },
    journal: JOURNAL,
    routes: 2048,
};

pub(super) const CLIENTS: jig_protocol_engine_clients::Limits =
    jig_protocol_engine_clients::Limits { streams: 8, exchanges: 32, mints: 16, ..small::CLIENTS };

const JOURNAL: skein_lib::JournalLimits =
    skein_lib::JournalLimits { commits: 3, writes: 142_684, held: 428_052, now: 1024, release: 1024 };
// Same 19.s3 composition as small; the standard domain supplies the larger value and operation bounds.
const STORE_KV: skein_kv::Limits = skein_kv::Limits {
    value: 10_396_559,
    ops: 285_370,
    commit: 864_288_042,
    queued_bytes: 2_592_864_126,
    segment: 1_728_576_084,
    snapshot_after: 864_288_042,
    chunk: 10_398_658,
    page: skein_kv::Page { rows: 2, bytes: 10_398_638 },
    ..small::STORE_KV
};
const STORE: jig_protocol_store::Limits = jig_protocol_store::Limits {
    core: CORE,
    store: STORE_KV,
    secrets: STORE_KV,
    journal: JOURNAL,
    joins: 16,
    reads: 48,
    ..small::STORE
};
