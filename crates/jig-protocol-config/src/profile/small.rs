//! One person: two workers, two slots each, four LLM accounts (configuration.md, 6).
//! Byte limits are bytes; durations below state their units. Counts are items.
//! **tasks**: `tasks=8, funders=8, project_tasks=8, tree_tasks=32, depth=8, delegates=4, references=8, subscriptions=4, batch=2, dependencies=8, holdings=4, hold_kinds=8, resource_reports=128, pools=8, hold_segments=4, hold_bytes=1024, hold_waiters=8, hold_wait=Duration::from_secs(120), inputs=8, spec_bytes=4096, parameters=4, result_bytes=512, run_inbox=8, inbox_messages=8, inbox_bytes=4096, message_bytes=4096, person_stall=Duration::from_secs(3600), proposal_stall=Duration::from_secs(3600), escalation_stall=Duration::from_secs(3600), saved_resources=4, contract_choices=1, charters=4, authority_grants=4, authority_segments=4, authority_bytes=2048, executor_kinds=4, retries=RETRIES, facts=32, room_refusals=8`.
//! **people**: `people=4, inbox_entries=8, inbox_tasks=16, inbox_recipients=32, sign_ins=8, projects=2, holdings=8, goals=16, initial_owners=2, requests=32, room_refusals=8, pending=8, waiters=4, identity_bytes=1024, words=2048, amendment_bytes=8192, sign_in_lifetime=Duration::from_secs(86400), request_retention=Duration::from_secs(86400), facts=32`.
//! **fleet**: `needs_bytes=65536, grants=8, workers=2, engine_slots=0, slots=2, workstreams=8, attempts=8, calls=32, call_name_bytes=64, progress_bytes=1024, turns=32, grace=Duration::from_secs(120), facts=32, history=8`.
//! **authority**: `projects=2, roles=4, requirements=4, facts=16, grants=4, executors=4, segments=4, segment_bytes=256, implications=8, batch=2, accounts=4, writes=4`.
//! **brief**: `briefs=4, sections=4, read_bytes=8192, brief_bytes=16384`.
//! **accounts**: `credentials=3, accounts=4, backoff_base=Duration::from_secs(2), backoff_max=Duration::from_secs(60), retry=Duration::from_secs(30), spent_attention=Duration::from_secs(300), facts=32`.
//! **views**: `runs=4, watchers=4, watchers_per_party=4, subjects_per_change=32, text_bytes=65536, facts=32`.
//! **notes**: `scopes=8, entries_per_scope=16, pattern_bytes=256, description_bytes=256, body_bytes=4096, references=8, load_rows=4, lines=4, recalled=4`.
//! **documents**: `rows=4, text_bytes=4096, chips=4, lines=4, versions=core::documents::Versions { minimum: 1, maximum: 1 }`.
//! **budgets**: `task=4096, dependencies=2048, attempts=1024, plan=2048, notes=2048`.
//! **core**: `fill_mark=268435456, connectors=2, resume_bytes=65536, run_bytes=16384, policy_bytes=16384, escalation_reason_bytes=512, load_slots=4, load_bytes=65536, call_records=32, call_answer_bytes=32768, tasks=TASKS, authority=AUTHORITY, people=PEOPLE, fleet=FLEET, brief=BRIEF, brief_parts=8, brief_core_budgets=BUDGETS, accounts=ACCOUNTS, notes=NOTES, documents=DOCUMENTS, views=VIEWS`.
//! **hub**: `slots=2, accounts=8, charter_bytes=65536, transcript_bytes=262144, delivery_evidence_bytes=4096, turn_bytes=65536, outcome_bytes=4096, detail_bytes=4096, held=8, inbox=8, event_bytes=65536, run_calls=8, facts=64, told=64, turns=8, turn_queue_bytes=524288`.
//! **link**: `chunk=16384, name=32, secret=256, names=8, opening=4, handshake=Duration::from_secs(10), hello=Duration::from_secs(10), ping=Duration::from_secs(5), silence=Duration::from_secs(15), stall=Duration::from_secs(30), backoff=Duration::from_millis(250), ceiling=Duration::from_secs(30), proved=Duration::from_secs(60), busy=Duration::from_secs(2), facts=64`.
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
    people: 4,
    inbox_entries: 8,
    inbox_tasks: 16,
    inbox_recipients: 32,
    sign_ins: 8,
    projects: 2,
    holdings: 8,
    goals: 16,
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
    rows: 4,
    text_bytes: 4_096,
    chips: 4,
    lines: 4,
    versions: core::documents::Versions { minimum: 1, maximum: 1 },
};
pub(super) const BUDGETS: core::CoreBriefBudgets =
    core::CoreBriefBudgets { task: 4_096, dependencies: 2_048, attempts: 1_024, plan: 2_048, notes: 2_048 };
pub(super) const CORE: core::Limits = core::Limits {
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
    skein_lib::JournalLimits { commits: 3, writes: 12_226, held: 36_678, now: 256, release: 256 };
pub(super) const LIMITS: Limits =
    Limits { core: CORE, hub: HUB, link: LINK, token: 16384, io: IO, journal: JOURNAL, routes: 512 };
