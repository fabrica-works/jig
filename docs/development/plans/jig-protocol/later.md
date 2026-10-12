# jig-protocol: later

What the design names but this plan does not do.

| Item | Design | When |
|---|---|---|
| LLM calls in a host's own process, for `smith-inline-agent` | `protocol/llm.md`, sections 1 to 4 and 7 to 9; `domain/hosts.md`, 5.2 | when an application runs agents in its engine, or in a worker's process |
| The browser's shell, wasm32 builds, the browser tier | `protocol/client.md`, section 12; `client/domain/view.md`, 11 to 13; skein's `lib.md`, section 2 | a plan of its own, once the native client serves |
| Retention: summaries of ended tasks, transcripts dropped past a horizon, answered keys expired, chats forgotten, a forgotten chat's remains (its replies' `Update` rows among them) at the next pass | `protocol/store.md`, 7.4 and 11.1; `domain/people.md`, section 6 | before a deployment's store nears its budget |
| `ops` on the real store, with its connectors' keys and codecs; `jig-fake-store` retired | `protocol/store.md`, 12 and 14; `examples.md` | when `ops` is developed again |
| Delegates' historical inputs loaded by the store | `domain/engine.md`, 5.3 | when a delegate's input names an ended result |
| TLS between hosts | `protocol/channel.md`, section 1; skein's `tls.md`, section 8 | with skein's TLS server side |
| Contained process trees for workers' agents | `domain/hosts.md`, section 10; skein's `draft/process.md` | with skein's design |
| A run's live text and named calls from workers | `protocol/README.md`, section 11 | a smith design and plan |
| The LLM component's abort and closed terminal | `protocol/README.md`, section 11 | with smith |
| A generic OAuth or OpenID sign-in provider | `protocol/client.md`, section 15 | when an application needs one |
| A handed-in account read again while the engine runs | `protocol/llm.md`, section 10 | when a deployment needs it |
| Several people on one deployment | `client/domain/README.md`, 5 and 6 | with such a deployment |
| Every root with engine slots says its hub's hello, and the fleet places nothing on them before it is admitted (ruling 146) | `domain/engine.md`, 8 ("present from construction"; "no run is placed where smith would refuse its start"); `domain/hosts.md`, 4 | when the test roots' engine slots gain a hub with an agent, or with the reference service's engine slots |
| A project's homes, named when a resource is adopted | `domain/core.md`, 6.1; `domain/connectors.md`, section 12; `domain/people.md`, 5.1 | when an application projects goals into a system |
