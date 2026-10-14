# client-composition: the suites at the plan's base

Measured once by the planner, 2026-10-14, on carbon-forge (8 threads), at
jig `2ae75cac2`, through the gate lane, alone (`heavy: load 0.88 0.72
0.69 on 8 cores as it starts alone`). The test binaries were built first,
then each suite ran three times:

```sh
NEXTEST_DOUBLE_SPAWN=0 heavy --gate -- <script>
# the script: cargo nextest run --workspace --no-run; then
#   3 × cargo nextest run --workspace
#   3 × cargo nextest run --workspace --profile fuzzy
```

Log: `~/src/rust/tmp/logs/20261014T114402-86.log` on carbon-forge.

| Suite | Run 1 | Run 2 | Run 3 | Median | Budget | Tests |
|---|---|---|---|---|---|---|
| focused | 8.487 s | 8.234 s | 9.219 s | **8.487 s** | 15 s | 3,514 |
| fuzzy | 13.963 s | 13.239 s | 14.097 s | **13.963 s** | 60 s | 109 |

The plan's allowance (README, "Test time") is taken from these medians:
they are the "before" of every time this plan reports, and nobody
measures them again.
