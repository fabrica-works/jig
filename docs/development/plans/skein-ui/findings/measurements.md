# skein-ui: the suites at the base

2026-10-15, on carbon-forge (8 threads, 15 GB), through `heavy --gate`,
three runs each in a worktree at `origin/main`: skein `2f282f7`, jig
`5efac1af8`. Command: `NEXTEST_DOUBLE_SPAWN=0 heavy --gate -- sh -c 'for
i in 1 2 3; do cargo nextest run --workspace 2>&1 | grep Summary; cargo
nextest run --workspace --profile fuzzy 2>&1 | grep Summary; done'`
(skein's adds `--profile browser`).

| Repository | Suite | Runs (s) | Median (s) | Budget (s) |
|---|---|---|---|---|
| skein | focused, 2106 tests | 6.793, 6.749, 6.414 | 6.749 | 15 |
| skein | fuzzy, 99 tests | 50.250, 45.879, 45.012 | 45.879 | 60 |
| skein | browser, 6 tests | 3.267, 3.695, 3.198 | 3.267 | 120 |
| jig | focused, 3636 tests | 10.671, 10.220, 10.089 | 10.220 | 15 |
| jig | fuzzy, 109 tests | 15.511, 15.750, 16.028 | 15.750 | 60 |

skein's gate started at load 4.71 with the code graph indexing (100 %
of a core); jig's at load 10.80 from the queue's builds. jig's recent
gates on `main` read 8.6 to 9.6 s focused and 13.2 to 15.5 s fuzzy
(`git log --format='%(trailers:key=Gate-Result)'`); skein's last on
another machine, 2.8 s focused and 11.1 s fuzzy. One gate swings by
about 1.3 s on carbon-forge (development.md, 5.6).
