# ADR-0018: Fuzz corpus storage

- **Status:** Accepted
- **Date:** 2026-11-22

## Context

A fuzz target's corpus is the set of inputs that reached new code. It grows with every run. A run that starts from it reaches deep code within seconds, while a run that starts from nothing spends its first minutes rediscovering the input format. So both the short per-PR runs and the nightly runs need the corpus the nightly runs grew ([testing standard](../standards/testing.md)).

A corpus changes daily and can reach tens of megabytes per target, so it doesn't belong in git history. `.gitignore` already excludes `/fuzz/corpus/` and `/fuzz/artifacts/`. It does belong somewhere that:
- CI can read on every PR, including PRs from forks, whose token is read-only;
- only trusted runs on `main` can write;
- a developer can download, to reproduce what CI saw.

The options:
- **A separate corpus repository.** It needs a second repository, and a write token or deploy key stored as a secret for the nightly job. Its history grows without bound.
- **ClusterFuzzLite.** It is a Docker-based runner whose storage is again a git repository or a cloud bucket with credentials. It replaces tooling `cargo xtask fuzz` already provides.
- **The GitHub Actions cache.** It can't be downloaded outside Actions. It is scoped by branch, and it is evicted after seven days unused or when the repository's cache is full.
- **GitHub Actions artifacts.** They are written by any job with no extra permission, read with `actions: read`, and downloadable by anyone signed in to GitHub with `gh run download`. They are kept for a set number of days.

## Decision

1. **Corpora are Actions artifacts.**
   - The nightly fuzz workflow (`fuzz-nightly.yml`) uploads each target's corpus as an artifact named `fuzz-corpus-<target>`, kept for 30 days.
   - It is the only writer, and only when it runs on `main`.
   - No secret, token or second repository is needed. There is no way to write a corpus from a developer machine.
2. **Reading takes the newest artifact for each target.**
   - `cargo xtask fuzz corpus pull [<target>]...` downloads, for each target, the newest unexpired `fuzz-corpus-<target>` artifact from a `fuzz-nightly.yml` run on `main`, whatever that run's conclusion. A crash in one target therefore doesn't hide the corpora the other targets grew in the same run.
   - It uses the `gh` CLI. In CI that runs with the job's token and `actions: read`; locally, with the developer's `gh` login.
   - Only "no such artifact" falls back to an empty corpus. Any other failure, such as the API being unreachable, fails the command, so that a PR job doesn't silently fuzz from nothing.
3. **Seeds are committed.** `fuzz/seeds/<target>/` holds small hand-picked inputs, and the input of every fixed crash. Runs read the seeds but never add to them: libFuzzer writes only to the first corpus directory it is given, and that is always `fuzz/corpus/<target>/`. Each seed is at most `-max_len` bytes, and a target's seeds stay under 1 MiB.
4. **Size budget.** Each target's corpus must be under 50 MiB after minimization.
   - The nightly workflow minimizes every corpus weekly, and when asked to, with libFuzzer's `-merge=1`.
   - It logs the coverage of the corpus before and after.
   - It fails if the merge does not complete or the result is over the budget.
5. **Retention.**
   - Corpora: 30 days. The nightly run refreshes them daily, so they expire only if fuzzing stops for a month, and then the corpus regrows from the seeds.
   - Crash reproducers from PR runs: 14 days.
   - Crash reproducers from nightly runs: 90 days, GitHub's maximum. The crash issue also records the commit, the toolchain, the limits and, for small inputs, the reproducer itself, so the evidence outlives the artifact.
6. **`.gitignore` keeps `/fuzz/corpus/` and `/fuzz/artifacts/`**, and adds `/fuzz/target/`.
7. **Generated targets keep no corpus, and their seeds are the golden vectors** (amended for PROTO-F11).
   - There is a generated target for each message, nearly 200 in all, fuzzed in shards. One corpus artifact per target would take a job per target, since a job uploads one artifact per step.
   - Instead, each run writes the target's golden vectors (PROTO-F10), Kafka's bytes for every version of the message, to `fuzz/target/golden-seeds/<target>/` and reads them as seeds. They are not copied into `fuzz/seeds/`.
   - The goldens already cover every version and field, so a run reaches the message's code from its first inputs.
   - A fixed crash's input is still committed to `fuzz/seeds/<target>/`.

## Consequences

- Fuzzing needs no credentials beyond the token every job already has. That holds for fork PRs too, whose read-only token includes `actions: read`.
- Downloading a corpus needs a `gh` login, even locally, because GitHub serves public artifacts only to signed-in users.
- A corpus can be lost, if no nightly run succeeds in uploading it for 30 days. The cost is a slower start until it regrows, not lost coverage of known bugs: those are in the seeds.
- PR fuzzing depends on the GitHub API for the download. When the API is down, the job fails rather than fuzzing from nothing.
- Generated targets don't build on earlier runs' discoveries. If their nightly coverage stops growing, storing a corpus per shard is the next step.
- TEST-0052 implements the pull command, the seeds and the minimization; TEST-0053 and TEST-0054 implement the PR and nightly jobs.

## Alternatives considered

The four options are in the context.
- **A separate repository** adds a secret with write access, and a second history that grows without bound, to store what artifacts store with none.
- **ClusterFuzzLite** duplicates `cargo xtask fuzz` and still needs storage with credentials.
- **The Actions cache** can't be downloaded to reproduce a run locally. Its seven-day eviction and per-branch scoping also make what a PR job starts from unpredictable.
