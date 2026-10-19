# Status

Updated 2026-10-19. How far jig's code is from its design, and which
plans take it there. The designer keeps this file (skein's
development.md, section 3.3).

## 1. Plans

| Plan | State | Takes the design to | Where |
|---|---|---|---|
| tools | done 2026-10-19 | `client/domain/client.md` 4, 6, 10, 11, 19, `contract.md` 3, 5.3, 6, 8, `protocol/client.md` 3, 6, `protocol/store.md` 8.3, `domain/authority.md` 7, `tasks.md` 4, `engine.md` 4, `protocol/configuration.md` 3.1: jig on skein's and smith's main (transcript version 5, jobs read as calls' outcomes); the send under a named key, held while it is (jig 02); the read of one turn, on the wire's version 9 (jig 03); a deadline given as a span, made a wall by the engine, and shapes that read no clock (jig 05); the focused suite back within 15 s after a test-cost pass (jig 04) | `docs/development/plans/tools/` of the repository downstream of jig, which it also changes |
| forge-kit | active on carbon-forge since 2026-10-19 | `README.md` 15 (how `jig-boundary` finds client crates: a dependency's client name read only under jig's and the checked project's prefixes, the project by its whole name, and skein's UI kit by its crates' names) | `docs/development/plans/forge-kit/` of the repository downstream of jig, which reads jig but for that repair |
| client-api | done 2026-10-18 | jig's client API: jig's client presentation-agnostic, its API jig's primitives as plumbing with shapes of work as sugar, and the server and the URL space the application's (section 2's items marked "(jig's client API)"). Its six steps, in order, each under jig's gate: (1) the client crates renamed by the client naming rule, with no change of behaviour, and a check of the rule (`README.md`, section 15); (2) the engine's plumbing (`domain/`): parties' `Create`, `Propose` and `Withdraw`, person tasks among their batches, cancel with a grace, `Move`, the compound request and operations per request; a task's label, flags, last activity and waiting on; `AsAllowed`; goals per project; the root's escalation holder; agents' `amend` and `release` naming revisions, `withdraw`, connectors' topics; procedures' amend, cancel and message; `me`, task queries, the two ended indexes and the ended read positions; templates in the configuration and policies' overrides; the engine with no shape-specific code; (3) the wire (`protocol/client.md`): its shape-specific records collapsed into the operations, the compound request, `me` and the task line; the typed version refusal; the application's party scope and windows; (4) the client (`client/domain/`): `jig-client`, with its helpers, over `jig-client-domain`; the link's HTTP half, `jig-client-protocol-http`; the reference UI, `jig-client-view`, on the public client alone; the client's small store with no place, address or draft; (5) the server (`protocol/service.md`): `jig-protocol-engine-clients` owning no listener and taking `Mint`, its routes `jig-protocol-engine-clients-http`, with the example guards, and `jig-test-service` mounting it at `/jig` and serving its own bundle; (6) the browser, last (`protocol/browser.md`): `jig-client-shell-web`, `jig-client-test-shell-web`, jig's stories for a browser shell and its attack stories in `jig-client-fake-person-browser`, jig's browser tier and the client's wasm32 builds; in skein, the browser harness's other origins (skein's `ui-web.md`, 14.2) | `docs/development/plans/client-api/` of the repository downstream of jig, which it also changes |
| skein-ui | done 2026-10-15 | `client/domain/view.md` 2, 3, 3.1, 7, 9, 10 and 15.3 (the view on `skein-ui`, jig's owner mapped onto its opaque owner, the provisional state gone, jig's outline rules gone, the chip's hole the reference hole, jig's constructors of skein-ui's limits, the text shell and the fake person skein's, with jig's settings); `client/domain/client.md` 12 (a sign-out's answer kept to its sign-in, in the client's joint world); in skein, `ui.md`, `ui-text.md`, `ui-web.md`, `examples.md` 9, `testing.md` 6, `browser.md` 5, 6 and 9, `lib.md` 2 (the wasm32 build) | `docs/development/plans/skein-ui/` |
| client-composition | done 2026-10-14 | `client/domain/` README 2 and 5, `client.md` 3, 4.1, 5.1, 9, 14 and 17, `contract.md` 1, 3.3, 5.3, 6, 7.1 and 7.2, `pages.md` 2, 9 and 12, `root.md` 2, 6, 7, 9 and 15, `testing.md` 2, 4, 6, 8 and 9, `view.md` 3, 3.1, 4, 4.1, 5, 10 to 13 and 15; `README.md` 10 and 15; `examples.md` 8 | `docs/development/plans/client-composition/` |
| test-budgets | done 2026-10-14 | no section of jig's design: the focused suite back within its 15 s budget (`docs/development/workflow.md`, 2; testing-strategy.md, 8), and skein's read of what has arrived in jig's readers of unframed bytes (skein's `lib.md`, 7) | `docs/development/plans/test-budgets/` |
| jig-protocol | done 2026-10-13 | `README.md` 15 (the crates' new names); `protocol/` but `llm.md`'s calls in a host's own process and `client.md` 12's browser; the `domain/` and `client/domain/` sections owed to the protocol layer, but retention (`protocol/store.md`, 11.1) | `docs/development/plans/jig-protocol/` |
| jig-client-implementation | done 2026-10-11 | `client/domain/` above the protocol line; `client/domain/contract.md` 9 (the engine's changes); `examples.md` 8 (`ops`'s client) | not versioned |
| jig-protocol-design | done 2026-10-10 | a design pass, with no code: `protocol/` | not versioned |
| jig-move | done 2026-10-09 | no section: jig in a repository of its own, with its workspace, lints and gate | not versioned |
| jig-extraction | done 2026-10-09 | `domain/`: the core and its children, the hub, the conformance world; `README.md` 6.2 (the journal, in skein-lib); `examples.md` 2–6, 9, 10 and two stories of 7 (`ops`) | not versioned |

## 2. The design

| Document | Built | Left |
|---|---|---|
| `README.md` | partly | 6.7, 9: `smith-inline-agent` in the engine, `ops`'s agents among them: not planned (agents in the engine); 1, 2, 3, 10, 11, 12, 15 (jig's client API): the layers, one `jig-client` with no view, plumbing and sugar, the application's server, transport and sign-in, jig's credentials, the optional reference UI, every client crate under the `<project>-client` prefix (section 15): client-api; 15: `jig-boundary` reading a dependency's client name only under jig's and the checked project's prefixes, the project by its whole name, and skein's UI kit by its crates' names: forge-kit; 10: a service's client: not planned (service and one-shot clients); 10: a one-shot client, its first host the command line of the application downstream of jig: not planned (the next part of the downstream repository's tools plan, its `later.md`); 10: the client's generic half in skein: not planned (the client's generic half in skein); 14: open questions only (versioning); 5, 8, 12: the accounts child's LLM accounts, with sign-ins, keys and notices, their records and keys in the secrets' store, and the domain's accounts and providers documents: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `domain/README.md` | partly | 5: hosting runs inside the engine with `smith-inline-agent`: not planned (agents in the engine); 5: a tool begun with its call from a host whose agent runs in its process: not planned (agents in the engine); 5: a run's text from agent processes, at the assignment's ask: not planned (live text); 4, 5 (jig's client API): the names and decisions for the operations, templates, flags, credentials and the escalation holder: client-api; 4, 5: a deadline as a span: the downstream repository's tools plan |
| `domain/core.md` | partly | 6.1: a project's homes, named at adoption: not planned (jig-protocol's `later.md`); 6.1: a project's resources, each adopted with its role: the core keeps only the roles live tasks' resources narrowed, and drops an owned one (`jig-domain-core/src/resources.rs`, `role`), so the project's document (`client/domain/pages.md`, 6.5) lists no owned resource and an adopted one only once a live task names it: not planned (a project's homes and adopted resources); 12: open questions only; 6.2, 7.3, 9, 11 (jig's client API): a goal as a tracked task, its defaults the goal template; shapes of work as templates, helpers, facts and models; the goal of a task the nearest tracked task at or above it: client-api |
| `domain/tasks.md` | partly | 1, 3, 4, 7, 9, 11, 12 (jig's client API): a task's label and flags (conversation, listed, retention class), last activity, waiting on, `AsAllowed`; a party's `Create` as an agent's batch, its reach, its root's escalation holder, its tasks' resources named by their specs and the grants named after them (4); tracked and priority amended, cancel with a grace, `Move`, the compound request; parties' `Propose` and `Withdraw`, and person tasks in their batches; a rejection reaching a party who proposed; goals per project, a grace's span; a spec's words and text parameters UTF-8, refused at every admission of a spec (3, 4): client-api; 7: a widening within the amender's own authority of a task the amender does not fund, funded by the amender: not planned (client-api's `later.md`); 4: a deadline as a span, made a wall when the engine makes the task: the downstream repository's tools plan; 3: a conversation's run asked for its text: not planned (live text); 3, 4, 5.5, 7: a task's LLM choice (its main model and effort, its sub-agents' models, an account), given at `Create` and changed by `Amend`: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `domain/authority.md` | partly | 1, 6, 7, 8.1, 8.4, 8.5, 9 (jig's client API): one check for parties and agents, the escalation role, templates' overrides in the policy, `AsAllowed`, explicit proposals, a connector topic's subscription by a read grant, an accepted amendment or widening funded by its accepter, which becomes its task's funder: client-api; 7: an amendment within its amender's authority that widens a task its amender does not fund, funded by the amender (the code resizes the task's present funder): not planned (client-api's `later.md`); 7: a deadline given as a span, made a wall by the decision that makes the task, a proposal keeping the span, each check judging it as the wall its own decision makes: the downstream repository's tools plan; 1, 3, 5 to 7, 8.1, 8.3 to 8.5: authority's models, a task's LLM choice checked against them, an account refused for its provider's rule, a program backend's reserve of one more turn and its built-in tools asked of smith's policy: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `domain/connectors.md` | partly | 12: an adoption naming the project's home: not planned (jig-protocol's `later.md`); 13: a system's request budget (fresh reads first, reserved shares, a stop until the reset): not planned (a connector's request budget); 1, 2, 5.3 to 5.5, 6.2, 6.3, 8 (jig's client API): procedures' amend, cancel with a grace and message, topics offered to agents, the party's own scope and windows, untracking: client-api; 6.3: a procedure's step's effects within its one decision: not planned (client-api's `later.md`) |
| `domain/engine.md` | partly | 2, 8: the engine's own hub's agent, `smith-inline-agent`: not planned (agents in the engine); 8: the fleet placing nothing on the engine's slots before its hub's hello is admitted: not planned (jig-protocol's `later.md`); 8: two claims prepared in one commit both placed on the engine's slots while a worker is free: not planned (client-api's `later.md`); 1, 4.1, 4.4, 5.3, 5.4, 6, 7.2 to 7.4, 9, 11 to 11.5, 13 (jig's client API): `me`, task queries and task lines, waiting on and last activity, a conversation known by its flag, ended reads through the indexes, offers and effects as data, the compound request's worst case, the agents' tools naming revisions, a mark on an ended conversation's loads held and one such mark loading at a time (4.2, 5.3): client-api; 7.1, 9: the brief's Lineage, Calls and Waiting sections; 4.2, 5.3: the other point loads (a proposal's, an escalation's, a person task's, a party's inputs) held, and what may change them while they load: not planned (client-api's `later.md`); 7.2: what a turn's calls to the engine made, written beside the turn by the core: not planned (client-api's `later.md`); 4: a deadline given as a span made a wall: the downstream repository's tools plan; 7.1, 11: the ask set for a conversation's run, and kind `text` kept alone, a piece of another kind dropped counted: not planned (live text); 3, 4.1, 5.4, 7.1, 7.3, 8, 9, 12 to 14, 16: LLM accounts with holders, sources, states and uses; grants of a credential with its scheme or a program source's name; the limited, refused and unauthorized notices; models declared at the hello, runs placed by their models and program logins, and a host keeping a person's logins kept to that person's runs; the catalogue's models: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `domain/hosts.md` | partly | 3, 5.2, 6.5, 11: `smith-inline-agent`, the engine's agent and a worker's by its root's choice, with its text and named calls and its runs in the hub's world: not planned (agents in the engine); 6.1: the engine's hub's one hello admitted before the fleet places on its slots: not planned (jig-protocol's `later.md`); 6.5, 2, 6.2, 12: the assignment's ask for live content, passed into smith's start, and a run's text from agent processes, by turn, retry, kind and offset: not planned (live text); 10: contained process trees for workers' agents: not planned (between hosts); 1, 8, 9 (jig's client API): the host tools in the operations' vocabulary, `amend` and `release` naming revisions, `withdraw`, cancel with a grace, connectors' topics: client-api; 2, 6.1: the accounts' notices, program sources, their checks, logins and sign-outs in the hub's link vocabulary, and the models a host declares: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `domain/people.md` | partly | 1 to 11 (jig's client API): credentials minted on the application's word (`Mint`), the operations with the compound request, its answer numbered by place, and operations per request, words that release nothing (a stopped task's words and a `Release` naming its hold's revision, one compound request), explicit proposing, the root's escalation holder, read positions kept for conversations only and ended ones in the store, the retention class, a chat as jig's template: client-api; 5.1: a party's `Amend` that widens a task they do not fund, funded from their pool: not planned (client-api's `later.md`); 6: a task forgotten once read that its party's inbox no longer held when it was read, or that a live task named then, by retention's pass: not planned (retention); 3: a service's credential reissued, a service removed: not planned (jig-protocol's `later.md`); 3, 5.1: an LLM provider apart from identity providers, and the LLM accounts' requests and who may make them: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `domain/root.md` | partly | 2, 5, 6: `smith-inline-agent` in the engine's root: its LLM completions routed below it, its facts and content drained, smith's payloads handed out between it and the hub: not planned (agents in the engine); 10: each event's room summed along its route, where every core event takes one shared ceiling: not planned (jig-protocol's `later.md`); 11, 12: `ops`'s reference root decoding the batch its watch's step gives as bytes, and adding the alerts to its words: not planned (`ops`'s remaining stories, drawing and root) |
| `domain/testing.md` | partly | 7: the real store in `ops`'s worlds, `jig-fake-store` retired: not planned (`ops` standalone; jig-protocol's `later.md`); 3 to 7, 9 (jig's client API): the stories and scenarios for the operations, fields and indexes: client-api |
| `domain/accounts.md` | no | 1 to 7: LLM accounts: each application its own client; accounts, holders, several with one LLM provider, sources (sign-in, key, program, none), states and bounds; the payer, uses and choosing the account; what people do; moving an account: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `domain/providers.md` | no | 1 to 5: the LLM providers' terms an application on jig keeps: OpenAI's approval, registration and sign-in, tokens, inference, errors, what the person sees, and keys; Anthropic's subscription credentials and keys, Claude Code in a product, its sign-in per machine, usage and billing, and scripted runs: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `client/domain/README.md` | partly | 1 to 5 (jig's client API): the client, `jig-client`, with no view, and the reference UI, `jig-client-view`, on it; the client's crates by the naming rule; the names and decisions: client-api; 6: open questions only (who may see what, with several people on one deployment; several applications in one client) |
| `client/domain/client.md` | partly | 1 to 4, 6 to 8, 10 to 12, 14 to 17, 19, 20 (jig's client API): `jig-client`, composing `jig-client-domain` and `jig-client-protocol-link`: its surface, handles and keys with their askers, the request table held by the client, watches by handle and their latest documents, a run's text by handle, the small store of the client's own records, a request whose record is not stored answered not kept, parked or not, by its key (11), `SignIn { with }`, versions per owner, the family owners and their versions the application declares, judged whole at each hello, a read of an owner not served answered at once, a kept family request of another version dropped (6, 7, 8.2, 11, 14), kept records dropped at start told by how many, under no asker, and `Link` told when a sign-out does not reach the engine (4, 11), no longer naming it once signed out, whatever ended the sign-in (12), `step` and `fire` with `wall` (4), a watch asked while signed out opened only once signed in, a host signing in after `Start { kept }` (8.1, 12), the helpers `shapes` (filling what a template does not give, and not offering a template they cannot fill), `facts`, `thread` and `words` (19), a decision's facts in `facts` and their English in `words` (19), the long-lived mode (20), a path's segments a bound of their own (14, 16), an authority's entries a bound of their own, an authority held whole and a draft made from it whole (14, 16), each of the engine's watches counted until its terminal, a closing sent once its opening is answered, a watch's opening or closing lost dropping the stream, a watch refused busy opened again after a backoff and behind, one of the engine's watches kept for `me`, and `me` ended only with the sign-in or jig's version not served (8.1, 8.2, 14), the stream's name kept beside the request table's records, past their bound by one (11, 14): client-api; 12: a bot relaying a person's decision; 17, 20: the service mode, and the one-shot client's story: not planned (service and one-shot clients); 20: the one-shot mode, its first host the command line of the application downstream of jig; 6: a `Send` carrying a name, answered held when the request table holds its key; 10: a job read as a call's outcome (`started as job N`, `job` calls, invalid calls, a call not run with its why): the downstream repository's tools plan; 10: a call to the engine carrying what the core wrote beside its turn: not planned (client-api's `later.md`); 18: open questions only; 19: shapes that read no clock, a template's deadline sent as its span, `set_goal` counting a span within both bounds: the downstream repository's tools plan; 8.3: the delivery's kind, `client.text(handle)` kind `text` only: not planned (live text); 6, 11: a request carrying a value, sent once, never kept nor resent, answered *unsent* or *lost* by the client: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `client/domain/contract.md` | partly | 1 to 10 (jig's client API): the plumbing operations, a `Create`'s tasks as the people child's batch members, a role executor its project's, a wake policy left to the executor's default when none is named, and its answer their first and last numbers (3.2, 4), the compound request, its answer numbering each operation's message by its place (3.4, 4), explicit proposing, a refusal's claim about a proposal made only where the party's role and pool decided it, with who would accept left to the offer (3.5, 4), templates, `me` with the authority a party may give a task alone (5.3), the task line, offers and a decision's effect as data, reordering offered in a project's document by the party's role alone (5.3), a cancel's counting its tree's tasks, live runs and pending proposals and never a connector's effects (5.4), task queries, the application's scopes and windows, the typed version refusal, and what the engine's domain owes for them (section 9), a path carried whole at the engine's bound on a name (5.3), an authority's lists carried whole at the engine's bounds on them, never cut, marked cut or refused for (5.3), a task's spec's words and text parameters UTF-8 wherever the tasks child admits a spec (5.3), a watch holding its place until the engine has ended it, and a party's and a stream's room at least one client's watches at its limit (7.2): client-api; 7.4: an application's streams: not planned (an application's streams; client-composition's `later.md`); 5.3: `me`'s creating bounds for a `Create` of any executor, and its executors carried whole past the documents' short lists: not planned (client-api's `later.md`); 3.1, 4: a name, the engine key a `Send` may carry in place of a drawn one, answered held by the client when its table holds it; 6, 7.1: the turn read, `turn{n, attempt, turn}`, read once and never watched, answered by the turn, unknown, busy, too large with its size, or malformed: the downstream repository's tools plan; 6: that read's gone answer, past a transcript's horizon: not planned (retention); 11: a document past its bounds, which the code answers busy, an open question (client-api's `later.md`); 3.1, 3.2, 3.6, 5.3: a deadline given as a span, made a wall when the engine makes the task, so that a name sent again makes the same ask, and within both of a draft's bounds, the engine judging it: the downstream repository's tools plan; 3.1, 3.2, 4, 5.1, 7.1, 9, 10: the party's LLM accounts' requests, a task's LLM choice, the accounts' bounds and the providers' rules refused, the party's accounts' document and the deployment's accounts, the `accounts` watch, `me`'s models: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `client/domain/objects.md` | partly | 1, 2, 4 to 7, 9 to 11 (jig's client API): jig's cards as the reference UI's, each act its operation, a refusal for authority offering a proposal, an application presenting jig's objects from their documents with `facts`: client-api; 10: a call to the engine as the cards of what it made: not planned (client-api's `later.md`); 13: open questions only |
| `client/domain/pages.md` | partly | 1 to 4, 6, 8, 9, 12 (jig's client API): the reference UI's pages on the public client: its places and their codec in `jig-client-view`, lists as task queries, chats and goals made by the helpers, a chat's status by its phase, a party it waits on and its host's last report (4), compound requests, the board's arrows on the project's offer of reordering (6.2), what each section watches and reads (12.5): client-api; 6.5: the policy section showing a project's template overrides: not planned (client-api's `later.md`); 7: a connector's request budget, and what its system last said: not planned (a connector's request budget); 7, 11: the deployment's LLM accounts with their sources, states, hosts and uses: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `client/domain/root.md` | partly | 1 to 10, 12, 15 (jig's client API): the application's client root holding `jig-client` as a component, routing by asker, families as bytes, the request table the client's, the count of kept records dropped to the child that says so (3, 8), the reference client root `jig-client-test-domain`: client-api; 11: a service's root, in the client's service mode: not planned (service and one-shot clients); 15.1: an application's own child domains: not planned (client-composition's `later.md`) |
| `client/domain/testing.md` | partly | 1 to 9 (jig's client API): the client tested through its surface, with a scripted application; jig's check in `jig-client-test-kit` (6.2), its referee saying a decision's facts in the phrasebook the application's world gives (6.2); the reference UI's worlds with skein's fake person; a sign-out told, judged against the engine's end of the sign-in, observed where the engine ends it (6.3); the browser tier, jig's stories for a browser shell and its attack stories (`jig-client-fake-person-browser`, `protocol/browser.md`, section 15), the referee's check that the engine holds no watch a client has not open (6.1), a watch and `me` refused busy among the faults (7): client-api |
| `client/domain/view.md` | partly | 1, 2, 4, 4.1, 5, 7, 9 to 11, 15, 18 (jig's client API): the reference UI, `jig-client-view`, on `jig-client` and `skein-ui` alone: its state, `jig_client_view::Ui` (18), its frame, sections, cards, confirmation, drafts and notices as primitives with a default arrangement, nothing in the tree mandated, a long path shown cut with jig's mark (7, 10), a long authority shown in part with jig's mark (7, 10), `step` and `fire` with `wall`, no sign-in asked and no report beside what it shows, a handle or read the client's limit refuses one the page left out, kept work dropped at start a notice, a read's page kept by the page that read it, no copy of a watch's document, the sign-in page once the link says signed out, whatever sign-out was lost (18, 18.1 to 18.4, 18.6, 18.7, 18.9), a decision's facts said from `words` (7): client-api; 7: executors and outcomes, with an outcome's actor and decision, in `words`, the reference UI saying them from it: not planned (client-api's `later.md`); 16: jig's own drawing kinds: not planned (skein-ui's `later.md`); 17: jig's drawing code (`draw`, drawing no kind), the view root's `draw` routed by a kind's owner, a drawing acted on, and the test family's drawing with its golden scenes and act (17.1, 17.3, 17.5): not planned (skein-ui's and client-composition's `later.md`); 9: a call to the engine as the cards of what it made: not planned (client-api's `later.md`); 14: open questions only |
| `protocol/README.md` | partly | 4, 5: smith's LLM component for `smith-inline-agent`'s runs in a host's own process: not planned (agents in the engine); 10: TLS's server side and contained process trees, between hosts: not planned (between hosts); 10: the client's generic half in skein: not planned (the client's generic half in skein); 11: the ask and a run's live content on smith's channel: not planned (live text); 11: smith's LLM component's abort and closed terminal: not planned (what jig asks of smith); 14: open questions only; 1 to 8, 10 to 13 (jig's client API): the components by the naming rule (`jig-client-protocol-wire`, `jig-client-protocol-link`, `jig-client-protocol-http`; `jig-protocol-engine-clients` with no listener, `jig-protocol-engine-clients-http`), what an application writes (its server, transport, sign-in, presentation and templates), versions with the typed refusal: client-api; 1 to 3, 5, 6, 8, 10 to 14: LLM accounts' records and keys in the secrets' records, grants with a scheme or a program source's name, and what jig needs from skein and smith for them: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `protocol/channel.md` | partly | 1, 8, 10: TLS between hosts, for the engine's listener and the worker's dial: not planned (between hosts); 5.1, 6.4: a run's text and a tool begun in the fact record, from a worker whose hub composes `smith-inline-agent`; 5.2, 6.1, 6.3, 7: the engine's own hub doing the same: not planned (agents in the engine); 6.4, 5.1, 5.2, 13: the ask in the assignment, and a piece of live content with its kind in the fact record, from agent processes; a piece of another kind dropped where it enters the engine, counted: not planned (live text); 13: smith's LLM component's abort and closed terminal: not planned (what jig asks of smith); 1, 4.3, 5.1 to 5.3, 6.3, 8, 9, 11 to 13: grants of a credential with its scheme or a program source's name; the limited, refused and unauthorized notices; program sources, checks, logins and sign-outs; a hello refused only for a model declared smaller; Claude Code's built-in tools asking smith's policy: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `protocol/client.md` | partly | 1 to 14, 16, 17 (jig's client API): jig's messages, transport-agnostic; the clients' component's messages, `Mint` and `Link` among them, and no provider route; the link with no HTTP; the wire with its shape-specific records collapsed; the typed version refusal; the client's family versions declared, at most a hello's ranges less jig's (9); the small store with the client's records only; the default HTTP mapping at both ends, a browser's client end over fetch, and wasm32 builds (12, 17); a watch's closing sent once its opening is answered admitted, a watch's opening or closing lost dropping the stream (4, 7); a stream's watches at least one client's watches at its limit (13); an authority's entries refused past the client's bound, and an authority carried with no cut mark (3), a stream per client with no refusal for its sign-in's other streams, streams bounded in all, exchanges in flight per sign-in, and the predecessor's stream sent gone by the name the small store kept (4, 4.1, 4.3, 10, 13): client-api; 3: the wire's authority `cut` field dropped at its next version: not planned (the client wire's authority cut field); 6: the turn read, answered with its bytes or its size: the downstream repository's tools plan; 6: its gone answer: not planned (retention); 15: open questions only; 3: a deadline given as a span crossing as a duration: the downstream repository's tools plan; 7.3: a delivery of a run's live content naming its kind, `reasoning` and `input` in the wire with no producer: not planned (live text); 1, 5, 10, 14, 16: a request carrying a value, its value kept in the secrets' store before the core sees it, never in the small store: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `protocol/configuration.md` | partly | 3: retention's horizons in `deployment`: not planned (retention); 3, 4: the engine's own slots with their `smith-inline-agent`'s limits, and a worker composing `smith-inline-agent`: not planned (agents in the engine); 3, 4, 7: the workers' listener's certificate and key, and a worker's engine by TLS server name and trusted roots: not planned (between hosts); 5: the first periods said by the reference service's root: `jig-test-service`'s engine loop says each itself (its `src/engine.rs`), so the decision is its protocol layer's: not planned (what the protocol layer leaves); 5: a calendar child for an application's periods, `ops`'s: not planned (`ops` standalone; jig-protocol's `later.md`); 9: open questions only; 1, 3, 3.1, 6 to 8, 10 (jig's client API): the `templates` member and policies' overrides, with their JSON, `clients` with no listener and no bundle, `providers` as the kinds identities name, operations per request checked at start: client-api; 3.1: a template's deadline span sent as a span, made a wall by the engine: the downstream repository's tools plan; 1, 3, 3.1, 4, 6 to 10: LLM providers with their sources and public registrations, no account configured, templates' LLM choice, the programs directory and its logins' checks, the accounts' bounds, no record handed in: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `protocol/browser.md` | partly | 1 to 15, 18, 19 (jig's client API): jig's browser parts on the application's server: `jig-client-shell-web` (the client's adapter over skein's fetch, the printed link's sign-in), the example guards in `jig-protocol-engine-clients-http`, the reference UI's browser shell `jig-client-test-shell-web` with its page's policy, the reference UI's codec, jig's stories for a browser shell and its attack stories in `jig-client-fake-person-browser`, jig's browser tier and the client's wasm32 builds, each load of the page a client with its own stream (10): client-api; 12.2: the Host check's certificate name between hosts: not planned (between hosts; client-api's `later.md`); 15: jig's tier calling skein's two-person stories for a browser shell: not planned (client-api's `later.md`); 16: tabs of one person sharing one stream, and HTTP/2: not planned (what the browser's layer leaves); 16: jig's own drawing kinds: not planned (skein-ui's `later.md`); 16: `ops`'s browser shell and its server: not planned (`ops` standalone); 17: open questions only |
| `protocol/llm.md` | partly | 1 to 4: LLM calls in a host's own process, for `smith-inline-agent`; 5.3, 6: a grant's value handed to smith's LLM component's table, in the engine or in a worker that composes `smith-inline-agent`; 7: smith's component's, the connection component's, the slots' contracts' and text's limits; 8: the engine's LLM path, and a worker that composes `smith-inline-agent`, in the protocol world: not planned (agents in the engine; jig-protocol's `later.md`); 9: an abort and a closed terminal for smith's LLM component: not planned (what jig asks of smith); 10: open questions only; 1 to 10: accounts' sources (sign-in, key, program, none), skein's driver made from the secrets' store's records, a sign-in through a client answered with its address, sign-out with revocation, keys from requests, the table of credentials with their scheme, program sources on their hosts, grants and notices; the handed-in record and the account's id gone: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `protocol/service.md` | partly | 2, 6, 10: agents in a host's own process, `smith-inline-agent` with smith's LLM component, in the engine (shape 2) or on a worker: not planned (agents in the engine); 6: agent processes in contained process trees: not planned (between hosts); 7, 8: `ops`'s service and its binary proving its shape: not planned (`ops` standalone); 7: the clients' listener off loopback, with its certificate and key files: not planned (between hosts; client-api's `later.md`); 10: a client in the engine's process, by an in-process transport: not planned (client-api's `later.md`); 9: open questions only; 1 to 3, 7, 8, 10 (jig's client API): the application's service owning the clients' listener and mounting the component, `jig-test-service` mounting it at `/jig` and serving its own bundle and policy, the printed link's URL printed on loopback, the browser tier (8): client-api; 2, 3, 6, 8: records and keys routed through the secrets' store, the driver made from them, program logins on workers, the live layer on a key, a sign-in or a program login: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `protocol/store.md` | partly | 1, 4.4, 7.2 to 7.4, 9, 15 (jig's client API): the ended indexes by requesting party and by project, ended read positions, task queries' reads, minted credentials' digests: client-api; 8.3: a turn read alone bounded by the read's byte bound, answering its size past it: the downstream repository's tools plan; 7.3, 11.1, 15 (item 6): retention's pass (ended tasks shrunk to summaries past a horizon, and a query's row joining the summary; transcripts dropped past a longer one; a forgotten task's remains collected) and the horizons: not planned (retention); 14: `ops`'s worlds on the real store, `jig-fake-store` retired: not planned (`ops` standalone; jig-protocol's `later.md`); 14: the reference binary killed around each phase of a turn's commit: not planned (what the protocol layer leaves); 1, 3, 4.4, 9, 10, 15: the accounts' family with the installation's identifier, records kept before lent, keys kept before the core sees them, both swept at restart: not planned: the second part of LLM accounts, after the first and after the tools and forge-kit runs |
| `examples.md` | partly | 3.1: reads charged to the connector's request budget: not planned (a connector's request budget); 4: its root composing `smith-inline-agent`: not planned (agents in the engine); 7.3: a queue for the pool, policy narrowed, a month of alerts; the conformance world crashes three stories, not all; 8: its error rate's drawing code, a scene tested by golden scenes: not planned (`ops`'s remaining stories, drawing and root); 1, 2, 8 (jig's client API): `ops` on `jig-client` with the reference UI, `jig-ops-client-domain`, `jig-ops-client-view` and `jig-ops-client-protocol-wire`, its terminal on skein's text shell over the same client, in its client world: client-api; 2, 4, 8 to 10: `ops` standalone (its systems' APIs and a real backend, its connectors' protocols, its service, server and binary, its periods' calendar child, its browser shell, its terminal's real loop, its worlds on skein-kv in memory): not planned (`ops` standalone) |

## 3. Not planned

- **Retention** (`protocol/store.md`, 7.3, 11.1 and section 15;
  `domain/people.md`, section 6; `protocol/configuration.md`, section 3;
  `client/domain/contract.md`, section 6; jig-protocol's and
  client-api's `later.md`): summaries of ended tasks, and a task query's
  row joining its summary; transcripts dropped past a longer horizon,
  and a turn read past it answered gone; decided proposals forgotten;
  and retention's pass, which erases a task forgotten once read whose
  read did not erase it (its party's inbox no longer held it, or a live
  task named it), and a forgotten task's remains. Waits for a plan of
  its own; the horizons wait for a deployment whose store nears its
  budget.
- **Agents in the engine** (`README.md`, 6.7 and section 9, the second
  shape; `domain/README.md`, section 5; `domain/engine.md`, sections 2
  and 8; `domain/hosts.md`, sections 3 and 11, 5.2 and 6.5;
  `domain/root.md`, sections 2, 5 and 6; `protocol/README.md`, sections
  4 and 5; `protocol/configuration.md`, sections 3 and 4;
  `protocol/service.md`, sections 2, 6 and 10; `protocol/channel.md`,
  5.1, 5.2, 6.1, 6.3, 6.4 and section 7; `protocol/llm.md`, sections 1
  to 4 and 6 to 8, and 5.3; `examples.md`, section 4; jig-protocol's
  `later.md`): `smith-inline-agent` composed by the engine's root,
  `ops`'s among them, or by a worker's root that chooses it, with its
  runs' text and the tools they begin; and smith's LLM component in the
  host's protocol layer, with its accounts' grants. Waits for an
  application that runs agents in its engine.
- **A run's live text from agent processes, at the assignment's ask**
  (`domain/hosts.md`, 2, 6.2, 6.5 and 12; `domain/engine.md`, 7.1 and
  11; `domain/tasks.md`, section 3; `protocol/README.md`, section 11;
  `protocol/channel.md`, 5.1, 5.2, 6.4 and 13; `protocol/client.md`,
  7.3; `client/domain/client.md`, 8.3): designed, and waits for its
  plan, `live-text`, in the downstream repository's plans, after its
  tools plan and the LLM accounts work. The assignment carries the ask,
  set for a conversation's run; smith's start takes it and its channel
  carries pieces naming turn, retry, kind and offset; the worker wire and
  the client wire carry all three kinds, and only `text` is produced and
  kept. Not designed
  here: reasoning and a tool's input as they stream, and text for the
  tasks a conversation delegates.
- **What jig asks of smith, later** (`protocol/README.md`, section 11;
  `protocol/channel.md`, section 13; `protocol/llm.md`, section 9): an
  abort and a closed terminal for smith's LLM component. Waits for a
  smith design and plan.
- **smith's parity and wall-clock changes, for jig's next repin of
  smith** (smith's parity and wall-clock plans, their `later.md`; skein's
  development.md, 5.7): the tools domain's new `Call` and `Outcome`
  variants, for jobs, in `smith-domain-tools`, which no jig crate uses;
  the transcript's version 5 and its not-run why
  (`NotRun::AfterFailedWrite`), which `jig-protocol-transcript` follows
  where it matches `Returned::NotRun`; the event vocabulary's version 3
  (`run.compaction_material`); `Purpose::Ordinary { guard }`; and the
  shipped profiles' larger `memory`, which jig's test configurations
  follow (`testing/jig-test/tests/support/mod.rs`, `tests/store/src/root.rs`).
  jig pins smith `29fa97c9`, before them. The downstream repository's
  tools plan moves jig's pin of smith to smith's `main` (transcript
  version 5, jobs); the crate it changes is `jig-protocol-transcript`.
- **`ops` standalone** (`examples.md`, sections 2, 4 and 8 to 10, and 9.1;
  `domain/testing.md`, section 7; `protocol/configuration.md`, section
  5; `protocol/service.md`, sections 7 and 8; `protocol/store.md`,
  section 14; `protocol/browser.md`, section 16; jig-protocol's
  `later.md`): its systems' APIs and a real backend on the machine's own
  services; its connectors' protocols; its service, with its server
  mounting jig's component under `/jig` and its periods' calendar child;
  its binary; its browser shell; its terminal client's loop over a real
  terminal, skein's text shell with jig's settings; and its worlds on
  skein-kv in memory in place of `jig-fake-store`. Waits for a plan of
  its own.
- **Between hosts** (`protocol/README.md`, section 10;
  `protocol/channel.md`, sections 1, 8 and 10;
  `protocol/configuration.md`, sections 3, 4 and 7;
  `protocol/service.md`, sections 6 and 7; `protocol/browser.md`, 12.2;
  `domain/hosts.md`, section 10; jig-protocol's and client-api's
  `later.md`): TLS for the engine's listeners, the workers' and the
  clients', with the workers' listener's certificate and key, a worker's
  engine named by its TLS server name and trusted roots, the clients'
  listener's certificate and key files, and the Host check's certificate
  name; and contained process trees for workers' agents. Waits for
  skein's TLS server side and skein's `draft/process.md`.
- **A connector's request budget** (`domain/connectors.md`, section 13;
  `client/domain/pages.md`, section 7; `examples.md`, 3.1): the budget
  and the reads charged to it, and the deployment's page showing it and
  what each system last said. Waits for a system that limits its rate.
- **A project's homes and adopted resources** (`domain/core.md`, 6.1;
  `domain/connectors.md`, section 12; jig-protocol's `later.md`): homes
  named at adoption, and each adopted resource kept with its role, so a
  project's document lists them. Waits for an application that projects
  goals into a system.
- **Service and one-shot clients, a bot relaying decisions, and
  services' credentials** (`README.md`, section 10;
  `client/domain/client.md`, sections 12, 17 and 20;
  `client/domain/root.md`, section 11; `domain/people.md`, section 3;
  jig-protocol's `later.md`): the client's service mode, with a
  service's root, and the one-shot client's story; a bot holding each
  person's own credential; a service's credential reissued, and a
  service removed. The one-shot mode is designed, and the next part of
  the downstream repository's tools plan builds it with its first host (section 2,
  `client/domain/client.md`). Wait for an application whose services act
  as parties, or which pages a person on call.
- **The client's generic half in skein** (`README.md`, section 10;
  `protocol/README.md`, section 10): keyed requests sent again until
  durable, level-triggered watches, the link and generic guards, moved
  to skein by a rename and a crate move. Waits for a second user.
- **Several people on one deployment** (`client/domain/README.md`,
  sections 5 and 6; jig-protocol's `later.md`): who may see what, an
  open question, and a role's entries shared and taken. The client is
  designed around one person. Waits for a deployment several people use.
- **`ops`'s remaining stories, drawing and root** (`examples.md`, 7.3
  and section 8; `domain/root.md`, sections 11 and 12): a queue for the
  pool, policy narrowed, a month of alerts, every story crashed at every
  commit; its error rate's drawing code, a scene tested by golden
  scenes; and its root reading nothing it carries between its children,
  where it decodes the batch its watch's step gives and adds the alerts
  to its words. Waits for a plan.
- **An application's own child domains** (`client/domain/root.md`,
  15.1; client-composition's `later.md`): families that belong to no
  connector, their owners numbered as connectors are, routed by the
  engine's root to a child that is not a connector. Waits for an
  application with such a domain.
- **An application's streams** (`client/domain/contract.md`, 7.4;
  client-composition's `later.md`): its data followed from an offset
  within a window, provisional, never committed state, in the pattern of
  a run's text. `jig-client` carries them as any watch. Waits for an
  application whose people follow its data as it comes.
- **jig's own drawing kinds and drawing code**
  (`client/domain/view.md`, sections 16 and 17; `protocol/browser.md`,
  section 16; skein-ui's and client-composition's `later.md`): jig's
  drawing code (`draw`, drawing no kind), the view root's `draw` routing
  each drawing by its kind's owner, a drawing acted on, and the test
  family's drawing with its golden scenes and act; and a chart every
  application shares, as jig's kind, with its data's encoding and its
  drawing code. The drawing code waits for the first drawing a person
  acts on; a kind, for a second application drawing the same.
- **What the browser's layer leaves** (`protocol/browser.md`, section
  16; `protocol/client.md`, section 15): tabs of one person sharing one
  stream, and HTTP/2. Wait for HTTP/2 in skein's HTTP, or for people
  keeping many tabs open.
- **The client wire's authority cut field** (`protocol/client.md`,
  section 3): an authority carries no cut mark, where the client wire's
  authority document still has its `cut` field. Waits for the client
  wire's next version.
- **Several applications in one client** (`client/domain/README.md`,
  section 6): one person's inbox across applications that share their
  people; an open question. Waits for such applications.
- **LLM accounts' later work** (`protocol/llm.md`, section 10;
  jig-protocol's `later.md`): an account signed in through a client; a
  handed-in account read again while the engine runs. Wait for a
  decision.
- **Versioning** (`README.md`, section 14): how jig is released and
  pinned, and how a change reaches every application; an open question.
  Waits for a decision.
- **What the client API leaves** (`domain/authority.md`, section 7;
  `domain/tasks.md`, section 7; `domain/people.md`, 5.1;
  `domain/connectors.md`, 6.3; `domain/engine.md`, 4.2, 5.3 and 7.1,
  and sections 8 and 9; `client/domain/contract.md`, 5.3;
  `client/domain/pages.md`, 6.5; `protocol/browser.md`, section 15;
  `protocol/service.md`, section 10; client-api's `later.md`): each
  waits for a jig plan after client-api, except where it says otherwise:
  - an amendment within its amender's authority that widens a task the
    amender does not fund, a party's among them, funded by the amender,
    which becomes the task's funder;
  - a procedure's step's effects within its one decision;
  - the other point loads held (a proposal's, an escalation's, a person
    task's, a party's inputs), and what may change them while they load;
  - the brief's Lineage, Calls and Waiting sections;
  - two claims prepared in one commit placed on the engine's slots
    while a worker is free: waits until placement is next touched;
  - `me`'s creating bounds for a `Create` of any executor, and its
    executors carried whole past the documents' short lists;
  - the reference UI's policy section showing a project's template
    overrides;
  - jig's tier calling skein's two-person stories for a browser shell:
    waits for skein's harness to open a link printed after a person's
    start;
  - a client in the engine's process, by an in-process transport: waits
    for the first application that runs one.
- **What the protocol layer leaves** (`domain/engine.md`, section 8;
  `domain/hosts.md`, 6.1; `domain/root.md`, section 10;
  `protocol/configuration.md`, section 5; `protocol/store.md`, section
  14):
  - every root with engine slots saying its hub's hello, and the fleet
    placing nothing on them before it is admitted (jig-protocol's
    `later.md`): waits for engine slots with a hub and an agent;
  - each event's journal room summed along its route, where every core
    event takes one shared ceiling (jig-protocol's `later.md`): waits for
    a shipped profile whose memory check refuses the real service;
  - the reference service's first periods said by its root, where its
    engine loop says them: waits for a plan;
  - the reference binary killed around each phase of a turn's commit,
    where it is killed once: waits for a plan.
- **The second part of LLM accounts** (`README.md`, 5, 8 and 12;
  `domain/accounts.md`; `domain/providers.md`; `domain/engine.md`, 3,
  4.1, 5.4, 7.1, 7.3, 8, 9, 12 to 14 and 16; `domain/authority.md`, 1,
  3, 5 to 7, 8.1 and 8.3 to 8.5; `domain/tasks.md`, 3, 4, 5.5 and 7;
  `domain/people.md`, 3 and 5.1; `domain/hosts.md`, 2 and 6.1;
  `client/domain/client.md`, 6 and 11; `client/domain/contract.md`, 3.1,
  3.2, 4, 5.1, 7.1, 9 and 10; `client/domain/pages.md`, 7 and 11;
  `protocol/README.md`, 1 to 3, 5, 6, 8 and 10 to 14;
  `protocol/channel.md`, 1, 4.3, 5.1 to 5.3, 6.3, 8, 9 and 11 to 13;
  `protocol/client.md`, 1, 5, 10, 14 and 16; `protocol/configuration.md`,
  1, 3, 3.1, 4 and 6 to 10; `protocol/llm.md`; `protocol/service.md`, 2,
  3, 6 and 8; `protocol/store.md`, 1, 3, 4.4, 9, 10 and 15): jig still
  runs on today's forms: accounts in configuration, each endpoint
  naming one, signed in or read from a handed-in file, their records in
  the driver's private files; a credential with an account's id; and
  the rejected and exhausted notices. The design's end state: accounts held
  by people or the deployment, each with its sources (a sign-in kept in
  the secrets' store, a key from a request never kept, a program's login
  on its hosts, or none), states, uses and bounds; a task's LLM choice
  within authority's models; grants of a credential with its scheme or
  a program source's name; the limited, refused and unauthorized
  notices; the party's `accounts` watch; and the providers' terms kept.
  Waits for the first part, in skein and smith, and for the tools and
  forge-kit runs.
- **The MIT licence**: a `LICENSE` file at the root and `license = "MIT"`
  in each manifest (the user's decision, with the LLM accounts design,
  2026-10-19). The second part of LLM accounts.
- **The documents' open questions** (each document's "Open questions"):
  wait for a decision, or for a deployment's measurements.

## 4. Drafts

| Draft | About | Next |
|---|---|---|
