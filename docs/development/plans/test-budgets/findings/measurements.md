# The suites at this plan's base

Measured by the planner on carbon-forge (i5-8265U, 4 cores, 8 threads,
15 GB), 2026-10-13, through `heavy --gate`, unoptimized as each gate
builds, at skein `2068257`, smith `0305f2fd` (`origin/main`) and jig
`d91a89c45`. Other sessions ran beside the queue during these runs (load
averages of 7 to 13 as the gate tasks started), so single numbers vary
by 10 to 30 %; the gates' own `Gate-Result` lines are given beside them.

## 1. The focused suites

| Repository | Tests | Last gate here | `measure`, 8 threads: wall / summed | `-j 1`: summed |
|---|---|---|---|---|
| jig | 3,813 | 15.041 s (`2048f39b7`) | 20.1 s / 158.7 s (load 8.7); the gate's own per-test times: 119.0 s summed | 49.9 s |
| smith | 1,996 | 24.068 s (`6449a197`, without the timeout; ruling 121) | 21.0 s / 148.5 s; 7 tests fail here (section 4) | 53.8 s |
| skein | 2,087 | 9.474 to 11.405 s (`886a7d4` to `b5d3a2c`) | 12.5 s / 98.1 s (load 13) | 33.6 s |

The logs, in `~/src/rust/tmp/logs/`: jig `20261013T205916-87.log` (8
threads) and `20261013T210209-90.log` (`-j 1`), and the last gate's
per-test times in `~/src/rust/tmp/jig/jig-protocol/logs/20261013T202400-86.log`;
smith `20261013T210114-88.log` and `20261013T210305-92.log`; skein
`20261013T210521-95.log` and `20261013T210642-97.log`.

## 2. Where jig's time goes (the last gate's per-test times)

| Binary | Tests | Gate, summed | Each | Alone (`-j 1`), summed |
|---|---|---|---|---|
| `jig-channel-world::channel` | 26 | 21.5 s | 827 ms | 9.1 s |
| `jig-view` | 301 | 11.4 s | 38 ms | 4.7 s |
| `jig-client-world` | 266 | 10.7 s | 40 ms | 4.6 s |
| `jig-protocol-store` | 417 | 5.5 s | 13 ms | 2.7 s |
| `jig-client-protocol-world::protocol` | 10 | 4.4 s | 443 ms | 1.8 s |
| `jig-conformance-world::snapshot` | 25 | 4.0 s | 159 ms | 1.9 s |
| `jig-boundary` | 67 | 3.4 s | 51 ms | 1.4 s |
| `jig-domain-core` | 217 | 3.0 s | 14 ms | 1.3 s |
| the other binaries | 2,484 | 55.1 s | 22 ms | 22.4 s |

## 3. What a test's process costs

A test binary run directly, one trivial test, alone: 1.3 to 2.5 ms. The
same test under nextest: about 5 ms alone, about 12 ms of the gate's
summed time. nextest's double spawn (it starts each test through a
process of its own first) is about half of that
(`findings/cost-jig-units.md`, 1). Paired runs in the gate lane of four
of jig's unit-test crates (`jig-domain-core`, `jig-protocol-store`,
`jig-domain-client`, `jig-protocol-client-link`; 992 tests,
`20261013T215233-146.log`):

| Run | With the double spawn | Without |
|---|---|---|
| 1 | 1.117 s | 0.493 s |
| 2 | 0.951 s | 0.501 s |

Unit-test binaries in one process (libtest, its threads), alone, for
decisions.md, 5: `jig-protocol-store` 417 tests in 0.07 s at 8 threads,
0.28 s on one; `jig-view` 301 in 0.67 s and 2.10 s; `jig-client-world`'s
library 266 in 0.52 s and 1.87 s.

## 4. smith's failures on carbon-forge

On smith `0305f2fd`, seven tests fail here, alone as in the suite, while
the reliability machine's gate passed the same commit: four of
`smith-local-process-world::authentication`, two of
`smith-local-process-world::memory`, and `smith::end_to_end`'s
`smith_on_a_terminal_signs_in_and_commits_in_place`
(`findings/cost-smith.md`, its cause).

## 5. The binaries the sessions cite, alone

Summed `-j 1` times at this plan's base (jig `20261013T210209-90.log`,
smith `20261013T210305-92.log`):

| Binary | Tests | Summed alone |
|---|---|---|
| jig `jig-view` | 301 | 4.69 s |
| jig `jig-protocol-store` | 417 | 2.72 s |
| jig `jig-client-protocol-world::protocol` | 10 | 1.76 s |
| jig `jig-boundary` | 67 | 1.42 s |
| smith `smith-local-process-world::simulation` | 27 | 3.90 s |
| smith `smith-agent-service` | 116 | 2.84 s |
| smith `smith-agent-world::messages` | 13 | 1.45 s |
| smith `smith-run-world::simulation` | 33 | 1.09 s |
| smith `smith-agent-process-world::stories` | 15 | 1.09 s |
