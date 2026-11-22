# ADR-0019: Benchmark regression gating

- **Status:** Accepted
- **Date:** 2026-11-22

## Context

The [Rust standard](../standards/rust.md#performance) says a regression above the agreed threshold fails the benchmark job, and NFR-0004 sets per-hot-path budgets. A gate is only useful if it fails on real regressions and never on noise. A noisy gate is either ignored or disabled.

The benchmarks so far use criterion, which measures wall time ([testing standard](../standards/testing.md#benchmarks)). CI runs on GitHub's shared `ubuntu-24.04` runners. A dedicated runner would give stable wall times, but a self-hosted runner on a public repository runs code from any pull request, and the project has no machine to dedicate.

The spike (TEST-0095) measured the murmur2 benchmark (16 B to 64 KiB keys) 20 times: 5 runs in each of 4 jobs. The jobs ran on three CPU models (AMD EPYC 7763, AMD EPYC 9V74, Intel Xeon 6975P-C). Each run measured two ways.

| Method | Spread of the 4 machines' means | Within one job: run-to-run CV | Within one job: largest change between consecutive runs |
|---|---|---|---|
| criterion wall time | 24% (64 KiB) to 62% (16 B) | ≤ 0.58% | ≤ 1.43% |
| Instruction counts (callgrind, via gungraun) | **0** | **0** | **0** |

Every one of the 20 instruction counts for each size was identical: 166, 676, 8,946 and 139,506 instructions on all three CPU models. Wall time is only comparable within one job. Even there it moves with code layout, which a change can alter without changing the work done, and its outliers come from neighbours on a shared host, which 20 runs can't bound.

## Decision

1. **The gate counts instructions.** Gated benchmarks are [gungraun](https://github.com/gungraun/gungraun) library benchmarks, which run each benchmark once under valgrind's callgrind. The gated metric is instructions executed (callgrind's `Ir`); estimated cycles, from callgrind's cache simulation, are reported beside it but don't gate.
2. **The job compares a pull request with its base in the same job.**
   - `cargo xtask bench --base <ref>` builds and runs the gated benchmarks at the base commit, in a git worktree, and then in the pull request.
   - It prints both counts and the change for each benchmark, in the job summary.
   - It fails when an instruction count grows by more than its threshold.
   - A benchmark the base doesn't have is reported as new.
   - Counts differ between architectures (x86-64 and aarch64 differ by about 6% for murmur2), so they are never compared across jobs.
3. **Thresholds live in `benches/thresholds.toml`.** The default is **3%**. The counts have no noise, so the threshold is not a noise margin but the smallest regression worth stopping a merge for. A benchmark may override it, with a reason. An override naming no existing benchmark is an error.
4. **Gated benchmarks are separate targets**, `crates/<crate>/benches/<area>_instructions.rs`, named `<area>/<operation>/<case>`. Criterion targets stay, for wall time on a developer's machine and for the numbers a performance claim needs. Both draw their inputs from the same function, so they measure the same work.
5. **Tools.**
   - `gungraun` is a dev-dependency.
   - `gungraun-runner`, at the same version, is pinned in xtask's tool table.
   - valgrind comes from the runner's Ubuntu packages.
   - Valgrind runs only on Linux: on macOS the gated benchmarks run in the devcontainer.

## Consequences

- A benchmark failure is a real change in work done, reproducible on any Linux machine of the same architecture. No retry or noise budget is needed.
- Instruction counts don't see memory latency, contention or system calls. Wall-time and macro benchmarks (TEST-F11's M2 tasks, NFR-F01) cover those, and a performance claim still needs wall-time numbers.
- The benchmark job builds the workspace's benchmarks twice and runs them under valgrind, which is 10 to 50 times slower than native but runs each benchmark once.
- gungraun brings about 45 crates into dev builds only. None of them ships.
- History across commits (TEST-0097) can store counts directly, because they don't depend on the machine.

## Alternatives considered

- **criterion wall time within one job.**
  - The measured within-job noise (≤ 1.4%) would allow a threshold of about 5%.
  - But a pull request's binary differs from its base's in code layout, which moves micro-benchmark wall times by several percent with no change in work.
  - The outliers a shared host produces are unbounded by 20 runs.
  - The gate would be either flaky or blind to small regressions.
- **A dedicated or self-hosted runner.** It runs untrusted pull-request code on a persistent machine, which is a security risk on a public repository, and needs hardware the project doesn't have.
- **Hardware counters (`perf stat`).** GitHub's virtual machines don't expose performance counters.
- **iai-callgrind.** It is the same approach under its former name; development continues as gungraun.
