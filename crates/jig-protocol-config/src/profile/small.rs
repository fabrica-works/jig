//! One person: two workers, two slots each, four LLM accounts (configuration.md, 6).
//! Byte limits are bytes; durations below state their units. Counts are items.
//! **tasks**: `tasks=8, funders=8, project_tasks=8, project_goals=8, tree_tasks=32, depth=8, delegates=4, references=8, subscriptions=4, batch=2, dependencies=8, holdings=4, hold_kinds=8, resource_reports=128, pools=8, hold_segments=4, hold_bytes=1024, hold_waiters=8, hold_wait=Duration::from_secs(120), grace=Duration::from_secs(600), inputs=8, spec_bytes=4096, parameters=4, result_bytes=512, run_inbox=8, inbox_messages=8, inbox_bytes=4096, message_bytes=4096, person_stall=Duration::from_secs(3600), proposal_stall=Duration::from_secs(3600), escalation_stall=Duration::from_secs(3600), saved_resources=4, contract_choices=1, charters=4, authority_grants=4, authority_segments=4, authority_bytes=2048, executor_kinds=4, retries=RETRIES, facts=32, room_refusals=8`.
//! **people**: `operations=8, people=4, inbox_entries=8, inbox_tasks=16, inbox_recipients=32, sign_ins=8, projects=2, holdings=8, goals=16, initial_owners=2, requests=32, room_refusals=8, pending=8, waiters=4, identity_bytes=1024, words=2048, amendment_bytes=8192, sign_in_lifetime=Duration::from_secs(86400), request_retention=Duration::from_secs(86400), facts=32`.
//! **fleet**: `needs_bytes=65536, grants=8, workers=2, engine_slots=0, slots=2, workstreams=8, attempts=8, calls=32, call_name_bytes=64, progress_bytes=1024, turns=32, grace=Duration::from_secs(120), facts=32, history=8`.
//! **authority**: `projects=2, roles=4, requirements=4, facts=16, grants=4, executors=4, segments=4, segment_bytes=256, implications=8, batch=2, accounts=4, writes=4`.
//! **brief**: `briefs=4, sections=4, read_bytes=8192, brief_bytes=16384`.
//! **accounts**: `credentials=3, accounts=4, holder_accounts=4, sources=3, sign_ins=2, backoff_base=Duration::from_secs(2), backoff_max=Duration::from_secs(60), retry=Duration::from_secs(30), spent_attention=Duration::from_secs(300), facts=32`.
//! **views**: `runs=4, watchers=4, watchers_per_party=4, subjects_per_change=32, text_bytes=65536, facts=32`.
//! **notes**: `scopes=8, entries_per_scope=16, pattern_bytes=256, description_bytes=256, body_bytes=4096, references=8, load_rows=4, lines=4, recalled=4`.
//! **documents**: `segments=4, rows=4, text_bytes=4096, chips=4, lines=4, versions=core::documents::Versions { minimum: 1, maximum: 1 }`.
//! **budgets**: `task=4096, dependencies=2048, attempts=1024, plan=2048, notes=2048`.
//! **core**: `templates=8, fill_mark=268435456, connectors=2, resume_bytes=65536, run_bytes=16384, policy_bytes=16384, escalation_reason_bytes=512, load_slots=4, load_bytes=65536, call_records=32, call_answer_bytes=32768, tasks=TASKS, authority=AUTHORITY, people=PEOPLE, fleet=FLEET, brief=BRIEF, brief_parts=8, brief_core_budgets=BUDGETS, accounts=ACCOUNTS, notes=NOTES, documents=DOCUMENTS, views=VIEWS`.
//! **hub**: `slots=2, accounts=8, charter_bytes=65536, transcript_bytes=262144, delivery_evidence_bytes=4096, turn_bytes=65536, outcome_bytes=4096, detail_bytes=4096, held=8, inbox=8, event_bytes=65536, run_calls=8, facts=64, told=64, turns=8, turn_queue_bytes=524288`.
//! **link**: `chunk=16384, name=32, secret=256, names=8, opening=4, handshake=Duration::from_secs(10), hello=Duration::from_secs(10), ping=Duration::from_secs(5), silence=Duration::from_secs(15), stall=Duration::from_secs(30), backoff=Duration::from_millis(250), ceiling=Duration::from_secs(30), proved=Duration::from_secs(60), busy=Duration::from_secs(2), facts=64`.
//! **driver**: `listeners=ACCOUNTS.sign_ins`, the sign-in deadline `client.sign_in_time=Duration::from_secs(300)`.
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
use skein_lib::Duration;
const RETRY: tasks::Retry = tasks::Retry { retries: 3, base: Duration::from_secs(1), max: Duration::from_secs(30) };
const RETRIES: tasks::Retries =
    tasks::Retries { transient: RETRY, permanent: RETRY, run: RETRY, agent: RETRY, lost: RETRY, invalid: RETRY };
pub(super) const TASKS: tasks::Limits = tasks::Limits {
    tasks: 8,
    funders: 8,
    project_tasks: 8,
    project_goals: 8,
    tree_tasks: 32,
    depth: 8,
    delegates: 4,
    references: 8,
    subscriptions: 4,
    batch: 2,
    dependencies: 8,
    holdings: 4,
    hold_kinds: 8,
    resource_reports: 128,
    pools: 8,
    hold_segments: 4,
    hold_bytes: 1_024,
    hold_waiters: 8,
    grace: Duration::from_secs(600),
    hold_wait: Duration::from_secs(120),
    inputs: 8,
    spec_bytes: 4_096,
    parameters: 4,
    result_bytes: 512,
    run_inbox: 8,
    inbox_messages: 8,
    inbox_bytes: 4_096,
    message_bytes: 4_096,
    person_stall: Duration::from_secs(3600),
    proposal_stall: Duration::from_secs(3600),
    escalation_stall: Duration::from_secs(3600),
    saved_resources: 4,
    contract_choices: 1,
    charters: 4,
    authority_grants: 4,
    authority_segments: 4,
    authority_bytes: 2_048,
    executor_kinds: 4,
    retries: RETRIES,
    facts: 32,
    room_refusals: 8,
};
pub(super) const PEOPLE: people::Limits = people::Limits {
    operations: 8,
    batch: 2,
    people: 4,
    inbox_entries: 8,
    inbox_tasks: 16,
    inbox_recipients: 32,
    sign_ins: 8,
    projects: 2,
    holdings: 8,
    initial_owners: 2,
    requests: 32,
    room_refusals: 8,
    pending: 8,
    waiters: 4,
    identity_bytes: 1_024,
    words: 2_048,
    amendment_bytes: 8_192,
    sign_in_lifetime: Duration::from_secs(86400),
    request_retention: Duration::from_secs(86400),
    facts: 32,
};
pub(super) const FLEET: fleet::Limits = fleet::Limits {
    workers: 2,
    needs_bytes: 65_536,
    grants: 8,
    engine_slots: 0,
    slots: 2,
    workstreams: 8,
    attempts: 8,
    calls: 32,
    call_name_bytes: 64,
    progress_bytes: 1_024,
    turns: 32,
    grace: Duration::from_secs(120),
    facts: 32,
    history: 8,
};
pub(super) const AUTHORITY: authority::Limits = authority::Limits {
    projects: 2,
    roles: 4,
    requirements: 4,
    facts: 16,
    grants: 4,
    executors: 4,
    segments: 4,
    segment_bytes: 256,
    implications: 8,
    batch: 2,
    accounts: 4,
    writes: 4,
};
pub(super) const BRIEF: brief::Limits =
    brief::Limits { briefs: 4, sections: 4, read_bytes: 8_192, brief_bytes: 16_384 };
pub(super) const ACCOUNTS: accounts::Limits = accounts::Limits {
    accounts: 4,
    holder_accounts: 4,
    sources: 3,
    sign_ins: 2,
    credentials: 3,
    backoff_base: Duration::from_secs(2),
    backoff_max: Duration::from_secs(60),
    retry: Duration::from_secs(30),
    spent_attention: Duration::from_secs(300),
    facts: 32,
};
pub(super) const VIEWS: views::Limits = views::Limits {
    runs: 4,
    watchers: 4,
    watchers_per_party: 4,
    subjects_per_change: 32,
    text_bytes: 65_536,
    facts: 32,
};
pub(super) const NOTES: notes::Limits = notes::Limits {
    scopes: 8,
    entries_per_scope: 16,
    pattern_bytes: 256,
    description_bytes: 256,
    body_bytes: 4_096,
    references: 8,
    load_rows: 4,
    lines: 4,
    recalled: 4,
};
pub(super) const DOCUMENTS: core::documents::DocumentLimits = core::documents::DocumentLimits {
    roles: AUTHORITY.roles,
    grants: super::authority_grants(&TASKS, &AUTHORITY),
    executors: super::authority_executors(&TASKS, &AUTHORITY),
    implications: AUTHORITY.implications,
    segments: super::name_segments(&TASKS, &AUTHORITY),
    rows: 4,
    text_bytes: 4_096,
    chips: 4,
    lines: 4,
    versions: core::documents::Versions { minimum: 1, maximum: 1 },
};
pub(super) const BUDGETS: core::CoreBriefBudgets =
    core::CoreBriefBudgets { task: 4_096, dependencies: 2_048, attempts: 1_024, plan: 2_048, notes: 2_048 };
pub(super) const CORE: core::Limits = core::Limits {
    templates: 8,
    fill_mark: 268_435_456,
    connectors: 2,
    resume_bytes: 65_536,
    run_bytes: 16_384,
    policy_bytes: 16_384,
    escalation_reason_bytes: 512,
    load_slots: 4,
    load_bytes: 65_536,
    call_records: 32,
    call_answer_bytes: 32_768,
    tasks: TASKS,
    authority: AUTHORITY,
    people: PEOPLE,
    fleet: FLEET,
    brief: BRIEF,
    brief_parts: 8,
    brief_core_budgets: BUDGETS,
    accounts: ACCOUNTS,
    notes: NOTES,
    documents: DOCUMENTS,
    views: VIEWS,
};
pub(super) const HUB: host::Limits = host::Limits {
    slots: 2,
    accounts: 8,
    charter_bytes: 65_536,
    transcript_bytes: 262_144,
    delivery_evidence_bytes: 4_096,
    turn_bytes: 65_536,
    outcome_bytes: 4_096,
    detail_bytes: 4_096,
    held: 8,
    inbox: 8,
    event_bytes: 65_536,
    run_calls: 8,
    facts: 64,
    told: 64,
    turns: 8,
    turn_queue_bytes: 524_288,
};
pub(super) const LINK: Link = Link {
    chunk: 16_384,
    name: 32,
    secret: 256,
    names: 8,
    opening: 4,
    handshake: Duration::from_secs(10),
    hello: Duration::from_secs(10),
    ping: Duration::from_secs(5),
    silence: Duration::from_secs(15),
    stall: Duration::from_secs(30),
    backoff: Duration::from_millis(250),
    ceiling: Duration::from_secs(30),
    proved: Duration::from_secs(60),
    busy: Duration::from_secs(2),
    facts: 64,
};
pub(super) const IO: skein_io::Limits = skein_io::Limits {
    sockets: 16,
    names: 8,
    refusals: 8,
    intake: 1_048_576,
    receive: 16_384,
    output: 1_048_576,
    sends: 8,
    accepts: 4,
    backlog: 16,
    close_timeout: Duration::from_secs(30),
    retry: Duration::from_millis(250),
};
pub(super) const JOURNAL: skein_lib::JournalLimits =
    skein_lib::JournalLimits { commits: 3, writes: 18_501, held: 55_503, now: 256, release: 256 };
pub(super) const LIMITS: Limits = Limits {
    core: CORE,
    store: STORE,
    hub: HUB,
    link: LINK,
    // Smith keeps the two-byte id length and up to 256 id bytes in its 8192-byte value.
    token: 7934,
    driver: DRIVER,
    clients: CLIENTS,
    clients_http: jig_protocol_engine_clients_http::Limits {
        calls: 4,
        streams: 2,
        body: CLIENTS.message_bytes,
        read: 16384,
        send: 16384,
        event: 87416,
    },
    io: IO,
    journal: JOURNAL,
    routes: 512,
};

/// Native client HTTP: 4 sockets, 8 exchanges, 5 minute link, 30 second stall.
pub(super) const CLIENTS: jig_protocol_engine_clients::Limits = jig_protocol_engine_clients::Limits {
    streams: 2,
    watches_per_stream: 4,
    queued_bytes: 65536,
    identity_bytes: 1024,
    families: 32,
    providers: 32,
    mints: 4,
    exchanges: 8,
    exchanges_per_client: 2,
    message_bytes: 65536,
    wire: jig_client_protocol_wire::v10::CEILINGS,
    link_span: Duration::from_secs(300),
};

/// Accounts use bounded OAuth exchanges and the shared socket limits.
pub(super) const DRIVER: skein_oauth_accounts::Limits = skein_oauth_accounts::Limits {
    accounts: CORE.accounts.accounts.checked_add(1).expect("account zero slot"),
    file_stall: Duration::from_secs(30),
    lock_wait: Duration::from_secs(1),
    exchanges: 4,
    listeners: CORE.accounts.sign_ins,
    server: skein_http::server::Limits { head: 4096, headers: 32, body: 1024, read: 1024, response: 4096, send: 1024 },
    refresh_lead: Duration::from_secs(60),
    client: skein_oauth::ClientLimits {
        document: skein_oauth::Limits {
            document_bytes: 65536,
            string_bytes: 16384,
            token_bytes: 7934,
            client_bytes: 1024,
            detail_bytes: 1024,
            record_bytes: 65536,
            issued_client_bytes: 0,
            granted_bytes: 4096,
            subject_bytes: 0,
            id_token_bytes: 7934,
            depth: 16,
            tokens: 4096,
        },
        uri_bytes: 4096,
        scope_bytes: 4096,
        state_bytes: 64,
        code_bytes: 4096,
        url_bytes: 16384,
        request_bytes: 65536,
        // The registration's one parameter, the installation's identifier
        // (providers.md 2.2): `ext_agent_host_id`'s 17 bytes and a
        // `urn:uuid:`'s 45.
        parameter_bytes: 62,
        resource_bytes: 0,
        nonce_bytes: 0,
        metadata_bytes: 0,
        key_set_bytes: 0,
        keys: 0,
        modulus_bits: 2048,
        skew: Duration::ZERO,
        sign_in_time: Duration::from_secs(300),
        request_time: Duration::from_secs(30),
        backoff_base: Duration::from_secs(1),
        backoff_ceiling: Duration::from_secs(30),
        max_attempts: 5,
    },
    http: skein_http::client::Limits { request: 65536, head: 16384, headers: 32, read: 16384, send: 16384 },
    tls: skein_tls::client::Limits { read: 16384, send: 65536, records: 131_072 },
    io: IO,
};

// 19.s3: application key2078/value4304, one orphan erase, 256 MiB map.
pub(super) const STORE_KV: skein_kv::Limits = skein_kv::Limits {
    key: 2079,
    value: 1_305_143,
    ops: 37_005,
    commit: 345_701_922,
    queued: 3,
    queued_bytes: 1_037_105_766,
    budget: 268_435_456,
    segment: 691_403_844,
    snapshot_after: 345_701_922,
    chunk: 1_307_242,
    page: skein_kv::Page { rows: 2, bytes: 1_307_222 },
    deadline: Duration::from_secs(30),
};
pub(super) const STORE: jig_protocol_store::Limits = jig_protocol_store::Limits {
    core: CORE,
    store: STORE_KV,
    secrets: STORE_KV,
    journal: JOURNAL,
    joins: 4,
    queue: 3,
    files: 8,
    reads: 12,
    orphan_erases: 1,
};
