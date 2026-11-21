# OPS: Packaging and operations

> Crates: release tooling, `deploy/` · Kafka counterpart: Kafka release process, Docker images · Milestones: M2-M9

Releases, artifacts, containers, deployment, upgrades, backup, migration from Kafka, ecosystem compatibility, dashboards and runbooks.

## [To Do] OPS-F01 Versioning and release process
P1 · M2 · Feature · deps: FND-F06
Scope: SemVer policy, changelog, release notes, release branches and checklist.

### [To Do] OPS-0001 Decide the versioning and release policy: SemVer scope, public surface, support window
P1 · M2 · Spike · deps: none · labels: docs
Done when: ADR "Versioning and release policy" is accepted and defines what MAJOR, MINOR and PATCH may change for advertised API version ranges, on-disk formats, config keys and defaults, `alluvium.*` keys, metric names, event names and CLI flags; how many release lines are supported and for how long; how Alluvium versions relate to the Kafka 4.4.0 compatibility surface; docs/releasing/versioning.md summarises it and links the ADR.

### [To Do] OPS-0002 Embed build info in the binary: version, git commit, dirty flag, target triple, rustc version, profile
P1 · M2 · Task · deps: CLI-F01 · ref: clients/src/main/java/org/apache/kafka/common/utils/internals/AppInfoParser.java getVersion, getCommitId @4.4.0
Done when: `alluvium --version` prints `alluvium <semver> (<commit>[-dirty] <target>)` and `alluvium version --json` prints every field; values are produced by the binary crate's `build.rs` from Cargo metadata, git and `SOURCE_DATE_EPOCH`, never the wall clock, so two builds of one commit embed identical bytes; building from a source tarball without `.git` yields commit `unknown`; unit test parses the JSON output.

### [To Do] OPS-0003 Send ClientSoftwareName `alluvium` and a KIP-511-valid ClientSoftwareVersion from the inter-node client
P1 · M2 · Task · deps: OPS-0002, NET-F10, API-F18 · ref: clients/src/main/java/org/apache/kafka/common/requests/ApiVersionsRequest.java isValid @4.4.0 (KIP-511)
Done when: ApiVersions v3+ requests sent by the inter-node client carry `alluvium` and the build version with SemVer build metadata (`+…`) rewritten to `-…`; a property test proves every version string the build can produce matches `[a-zA-Z0-9](?:[a-zA-Z0-9\-.]*[a-zA-Z0-9])?`, so a Kafka-semantics broker never answers INVALID_REQUEST; a test decodes the request with the ApiVersions handler and sees both fields.

### [To Do] OPS-0004 Changelog fragments under `changes/` with a CI check
P1 · M2 · Task · deps: FND-F04 · labels: docs
Done when: each PR adds `changes/<pr>.<kind>.md` with kind one of `added`, `changed`, `deprecated`, `removed`, `fixed`, `security`, `compat`; `cargo xtask changelog check` (part of `cargo xtask ci`) fails a PR touching `crates/` without a fragment unless it carries the `no-changelog` label; CONTRIBUTING.md documents the rule with an example.

### [To Do] OPS-0005 `cargo xtask changelog assemble <version>`: render fragments into CHANGELOG.md
P1 · M2 · Task · deps: OPS-0004
Done when: the command writes a Keep a Changelog section for the version grouped by kind, dated from an explicit `--date` or `SOURCE_DATE_EPOCH`, removes the consumed fragments, and is idempotent when rerun; golden test over a fixture `changes/` directory.

### [To Do] OPS-0006 Generate the public-surface manifest: advertised API versions, config keys and defaults, metrics, events, CLI commands
P1 · M2 · Task · deps: OPS-0001, API-F18, CFG-F12, OBS-F05, OBS-F07
Done when: `cargo xtask surface dump` writes `surface.json` from the code registries (ApiVersions table, config registry, metric descriptors, event catalog, CLI definition) with no hand-maintained list; the JSON schema is documented in docs/releasing/surface.md; each release commits `release/surface/<version>.json`; a test fails if any registry contributes zero entries.

### [To Do] OPS-0007 SemVer gate: diff the surface manifest against the previous release and classify each change
P1 · M2 · Task · deps: OPS-0006
Done when: `cargo xtask surface diff <prev>` classifies removed or renamed keys, metrics and events, changed defaults and narrowed API version ranges as breaking, and additions as minor; CI on `release/*` branches and release PRs fails when the version bump doesn't allow a detected change; fixture-manifest tests cover each class.

### [To Do] OPS-0008 Release notes template and `cargo xtask release notes <version>` generator
P1 · M2 · Task · deps: OPS-0005, OPS-0007 · labels: docs
Done when: the generator fills docs/releasing/release-notes-template.md with highlights (manual), upgrade notes, compatibility changes from the surface diff, the CHANGELOG section, known issues and artifact digests; golden test against fixture inputs.

### [To Do] OPS-0009 Release branch and backport policy
P1 · M2 · Task · deps: OPS-0001 · labels: docs
Done when: docs/releasing/branches.md defines `release/X.Y` branches cut from main at feature freeze, the backport criteria (security fixes, data loss, correctness regressions, crash fixes), who approves backports, how long each line is supported per the ADR, and the branch protection required for `release/*` (added to the FND-F11 settings doc).

### [To Do] OPS-0010 Backport automation: a `backport/X.Y` label opens a cherry-pick PR
P2 · M2 · Task · deps: OPS-0009
Done when: a workflow with SHA-pinned actions and only `contents: write` and `pull-requests: write` opens a PR against `release/X.Y` carrying the original changelog fragment, and comments with manual instructions when the cherry-pick conflicts; a recorded run on a test repository is linked in the PR.

### [To Do] OPS-0011 `cargo xtask release prepare <version>`: bump versions, assemble the changelog, create the release commit
P1 · M2 · Task · deps: OPS-0005, OPS-0002
Done when: the command updates `[workspace.package] version` and `Cargo.lock`, rejects versions not greater than the last tag or outside the branch line (`release/X.Y` accepts only `X.Y.*`), runs `changelog assemble`, and refuses to run on a dirty tree; `--dry-run` prints the diff; tests run against a temporary git repository.

### [To Do] OPS-0012 Release checklist and release PR template
P1 · M2 · Task · deps: OPS-0009, OPS-0011, OPS-0008 · labels: docs · ref: release/release.py @4.4.0
Done when: docs/releasing/checklist.md lists every gate with the command that proves it (`cargo xtask ci`, compatibility and differential suites, fuzz corpus replay, surface diff, reproducibility check, upgrade tests and artifact signing once they exist), release candidates tagged `vX.Y.Z-rc.N` and the rule for promoting one, and is used as `.github/PULL_REQUEST_TEMPLATE/release.md`.

## [To Do] OPS-F02 Binary builds
P2 · M2 · Feature · deps: OPS-F01
Scope: Linux x86_64/aarch64 and macOS builds; reproducibility.

### [To Do] OPS-0013 Release build profile: LTO, codegen units, panic=abort and split debug info
P2 · M2 · Task · deps: TEST-F11 · labels: perf
Done when: `[profile.release]` keeps `panic = "abort"` (ADR-0004), sets `codegen-units = 1` and the LTO mode chosen from a recorded criterion benchmark of produce and fetch hot paths plus build time, and splits debug info so symbols ship separately; choices and numbers are in docs/releasing/building.md.

### [To Do] OPS-0014 Deterministic build inputs: path remapping, `SOURCE_DATE_EPOCH`, locked dependencies, pinned toolchain
P2 · M2 · Task · deps: OPS-0013 · labels: security
Done when: `cargo xtask dist build --target <triple>` builds with `--locked`, `CARGO_INCREMENTAL=0`, the pinned toolchain and `--remap-path-prefix` for the checkout and `CARGO_HOME`; a test runs `strings` on the binary and finds no host paths, usernames or build timestamps.

### [To Do] OPS-0015 `cargo xtask dist smoke <archive>`: unpack, format, start, ApiVersions round trip, graceful stop
P2 · M2 · Task · deps: CLI-F02, CLI-F03, OBS-F09
Done when: the smoke test formats a temp dir, starts the node with the archive's loopback config, waits for `/health/ready`, sends an ApiVersions v3 request built with `alluvium-protocol` and checks the response, sends SIGTERM and expects exit 0 within the shutdown budget; any failing step is named in the error.

### [To Do] OPS-0016 Third-party license notices for binary distribution
P2 · M2 · Task · deps: FND-F08, FND-F03 · labels: security
Done when: `cargo xtask dist licenses` writes `THIRD_PARTY_LICENSES` with each crate in the release dependency graph, its version, SPDX license and license text, failing on a crate with no license text; output is reproducible and checked against the `deny.toml` allow-list.

### [To Do] OPS-0017 Archive layout `alluvium-<version>-<triple>.tar.gz` with reproducible tar metadata
P2 · M2 · Task · deps: OPS-0014, OPS-0016
Done when: each archive holds `bin/alluvium`, `config/server.properties` (combined mode, loopback listeners), `config/broker.properties`, `config/controller.properties`, `LICENSE`, `NOTICE`, `THIRD_PARTY_LICENSES` and `README.md`; tar entries are sorted with mtime `SOURCE_DATE_EPOCH`, uid/gid 0 and normalized modes; a test lists the archive and checks the layout.

### [To Do] OPS-0018 Build and smoke-test the `x86_64-unknown-linux-gnu` archive in CI
P2 · M2 · Task · deps: OPS-0017, OPS-0015, NFR-F10
Done when: a CI job builds the archive on `ubuntu-24.04`, linked against the glibc floor from NFR-F10; `objdump -T` shows no symbol newer than that floor; `cargo xtask dist smoke` passes on that platform; the archive is uploaded as a workflow artifact on every main build.

### [To Do] OPS-0019 Build and smoke-test the `aarch64-unknown-linux-gnu` archive in CI
P2 · M2 · Task · deps: OPS-0017, OPS-0015, NFR-F10
Done when: a CI job builds the archive natively on an arm64 runner (no QEMU), linked against the glibc floor from NFR-F10; `objdump -T` shows no symbol newer than that floor; `cargo xtask dist smoke` passes on that platform; the archive is uploaded as a workflow artifact on every main build.

### [To Do] OPS-0020 Build and smoke-test the `x86_64-unknown-linux-musl` archive in CI
P2 · M2 · Task · deps: OPS-0017, OPS-0015, NFR-F10
Done when: a CI job builds the archive as a fully static binary (`file` reports statically linked); a produce/fetch benchmark against the gnu build is recorded in docs/releasing/building.md and a regression above 10% triggers an allocator ADR before the target ships; `cargo xtask dist smoke` passes on that platform; the archive is uploaded as a workflow artifact on every main build.

### [To Do] OPS-0021 Build and smoke-test the `aarch64-unknown-linux-musl` archive in CI
P3 · M2 · Task · deps: OPS-0017, OPS-0015, NFR-F10
Done when: a CI job builds the archive as a fully static binary on an arm64 runner; benchmark against the aarch64 gnu build recorded in docs/releasing/building.md; `cargo xtask dist smoke` passes on that platform; the archive is uploaded as a workflow artifact on every main build.

### [To Do] OPS-0022 Build and smoke-test the `aarch64-apple-darwin` archive in CI
P2 · M2 · Task · deps: OPS-0017, OPS-0015, NFR-F10
Done when: a CI job builds the archive on a `macos-14` runner as a development-only target per NFR-F10; the Gatekeeper quarantine workaround for the unsigned binary is documented; `cargo xtask dist smoke` passes on that platform; the archive is uploaded as a workflow artifact on every main build.

### [To Do] OPS-0023 Build and smoke-test the `x86_64-apple-darwin` archive in CI
P3 · M2 · Task · deps: OPS-0017, OPS-0015, NFR-F10
Done when: a CI job builds the archive by cross-compiling on an arm64 macOS runner, with the smoke test run under Rosetta; marked development-only; `cargo xtask dist smoke` passes on that platform; the archive is uploaded as a workflow artifact on every main build.

### [To Do] OPS-0024 Split debug symbols per target and crash-report symbolization
P2 · M2 · Task · deps: OPS-0013, SRV-F16
Done when: `alluvium-<version>-<triple>.debug.tar.gz` ships DWARF (`.dwp` on Linux, `.dSYM` on macOS) matched to the binary by GNU build-id or UUID; docs/releasing/symbolizing.md shows symbolizing a SRV-F16 crash report from the stripped release binary; a test forces a panic in a test build and symbolizes it.

### [To Do] OPS-0025 `SHA256SUMS` for all release artifacts and verification instructions
P2 · M2 · Task · deps: OPS-0017 · labels: security
Done when: `SHA256SUMS` lists every archive and debug archive; docs/releasing/verifying.md shows `sha256sum -c` and `shasum -a 256 -c`; signing of the file is added by SEC-F15.

### [To Do] OPS-0026 Release workflow: build every target on a tag and publish a GitHub Release
P2 · M2 · Task · deps: OPS-0018, OPS-0019, OPS-0020, OPS-0021, OPS-0022, OPS-0023, OPS-0024, OPS-0025, OPS-0008
Done when: pushing a `v*` tag builds the target matrix, runs the smoke test per target, and publishes archives, debug archives, `SHA256SUMS` and generated release notes; `-rc.N` tags are marked pre-release; the workflow has only `contents: write`; rerunning on a published tag fails instead of overwriting assets.

### [To Do] OPS-0027 Reproducibility verification: two independent builds produce identical archives
P2 · M2 · Task · deps: OPS-0014, OPS-0017 · labels: security
Done when: a nightly job builds each Linux target twice on different runners with different checkout paths and users and compares SHA-256 of binary and archive; a mismatch fails the job and uploads a `diffoscope` report; docs/releasing/reproducing.md lets a third party rebuild a tag and compare with `SHA256SUMS`.

## [To Do] OPS-F03 Container image
P1 · M2 · Feature · deps: OPS-F02
Scope: Minimal non-root multi-arch image with SBOM.

### [To Do] OPS-0028 Choose the runtime base image: distroless, static binary on scratch, or Chainguard
P1 · M2 · Spike · deps: none · labels: security
Done when: ADR "Container base image" compares image size, CVE surface, glibc versus musl performance (OPS-F02 benchmark), debuggability without a shell (`kubectl debug` ephemeral containers), and the CA bundle needed for OAUTHBEARER JWKS over HTTPS, and records the decision.

### [To Do] OPS-0029 Dockerfile that packages the released binary for each architecture
P1 · M2 · Task · deps: OPS-0028 · ref: docker/jvm/Dockerfile @4.4.0
Done when: `deploy/container/Dockerfile` copies `bin/alluvium` from the OPS-F02 archive for `TARGETARCH` instead of compiling, and a test proves the image binary's SHA-256 equals the archive's; ENTRYPOINT `["/usr/bin/alluvium"]`, CMD `["server", "--config", "/etc/alluvium/server.properties"]`; no shell or package manager in the final image.

### [To Do] OPS-0030 Non-root, read-only-root-filesystem image with only `/var/lib/alluvium` writable
P1 · M2 · Task · deps: OPS-0029 · labels: security
Done when: the image runs as `65532:65532`, declares volume `/var/lib/alluvium` and reads config from `/etc/alluvium`; container tests start it with `--read-only --cap-drop=ALL --security-opt=no-new-privileges` and with an arbitrary UID in group 0 (OpenShift restricted SCC); a layer scan finds no setuid or setgid files.

### [To Do] OPS-0031 Container configuration: mounted config file, environment overrides and the plaintext opt-in
P1 · M2 · Task · deps: OPS-0029, CFG-F02, NET-F01 · labels: security · ref: core/src/main/scala/kafka/docker/KafkaDockerWrapper.scala @4.4.0
Done when: `deploy/container/README.md` documents volumes, ports, signals, and precedence between the mounted file and CFG-F02 environment overrides; container tests prove that a listener on `0.0.0.0` with PLAINTEXT is refused at startup unless `alluvium.allow.plaintext.external=true` is set (ADR-0006), and that the opt-in is logged as a warning.

### [To Do] OPS-0032 First-start storage formatting for containers without a shell
P1 · M2 · Task · deps: OPS-0031, CLI-F03 · ref: docker/jvm/launch @4.4.0
Done when: the documented pattern runs `alluvium format` with the cluster id and `--ignore-formatted` as a separate one-shot container or init step before `server`; a test formats, starts, stops and restarts on the same volume and keeps the data; a second format with a different cluster id fails without touching the volume.

### [To Do] OPS-0033 `alluvium probe live|ready` subcommand used by the Docker HEALTHCHECK
P1 · M2 · Task · deps: OPS-0029, OBS-F09, NFR-F06
Done when: the subcommand GETs `/health/live` or `/health/ready` on the loopback diagnostics listener with a 2 s timeout and exits 0 or 1, printing the failing check; HEALTHCHECK uses `probe ready` with start-period derived from the NFR-F06 startup target; unit tests cover timeout, connection refused and non-200; a container test reaches `healthy`.

### [To Do] OPS-0034 OCI image labels and reproducible image metadata
P2 · M2 · Task · deps: OPS-0029, OPS-0002
Done when: `org.opencontainers.image.{version,revision,source,licenses,created,title,description}` are set with `created` from `SOURCE_DATE_EPOCH`; building the same tag twice produces the same per-platform image digest (BuildKit timestamp rewriting), checked in CI.

### [To Do] OPS-0035 Multi-arch image index for linux/amd64 and linux/arm64
P1 · M2 · Task · deps: OPS-0029
Done when: `docker buildx imagetools inspect` shows both platforms in one index; each platform image passes the container test suite on a native runner; per-platform digests are written to the release notes.

### [To Do] OPS-0036 Container structure and run tests
P1 · M2 · Task · deps: OPS-0030, OPS-0033, OPS-0034 · labels: test · ref: docker/test/docker_sanity_test.py @4.4.0
Done when: `deploy/container/tests/` asserts user, entrypoint, ports, volumes, labels, absence of `/bin/sh` and file ownership, then starts a single node, produces and consumes with the TEST-F08 Java client image, sends SIGTERM and expects exit 0 within the shutdown budget; runs on every PR touching `deploy/container/`.

### [To Do] OPS-0037 Image SBOM attestation covering Rust crates and base-image packages
P1 · M2 · Task · deps: OPS-0035 · labels: security
Done when: each platform image carries an SPDX SBOM attestation listing every crate in `Cargo.lock` with its version and every base-image package; a CI check fails if a locked crate is missing from the SBOM.

### [To Do] OPS-0038 Image vulnerability scan gate
P2 · M2 · Task · deps: OPS-0037 · labels: security
Done when: CI scans the SBOM with a pinned Grype and fails on fixable High or Critical findings; exceptions live in `deploy/container/vuln-ignore.yaml`, each with a reason and expiry date; a nightly rescan of published images opens an issue on new findings.

### [To Do] OPS-0039 Publish images to ghcr.io on release tags with immutable version tags
P1 · M2 · Task · deps: OPS-0035, OPS-0036, OPS-0026
Done when: release tags push `X.Y.Z`, `X.Y` and `X` (not for pre-releases) and `latest` only for the newest stable release; release notes reference images by digest; the job has only `packages: write`; pushing an existing `X.Y.Z` tag fails.

### [To Do] OPS-0040 Sign images and attach provenance with Sigstore cosign
P1 · M9 · Task · deps: OPS-0039, SEC-F15 · labels: security
Done when: each image digest has a keyless cosign signature and an SLSA provenance attestation; `cosign verify` with the documented certificate identity and issuer passes in CI; docs/releasing/verifying.md shows the command.

### [To Do] OPS-0041 docker compose example: one combined-mode node
P1 · M2 · Task · deps: OPS-0032 · ref: docker/examples/docker-compose-files/single-node @4.4.0
Done when: `deploy/compose/single-node/compose.yaml` runs the format step, then the node with the explicit plaintext opt-in and a healthcheck, exposing 9092; CI brings it up, produces and consumes with the pinned kcat image, and tears it down.

### [To Do] OPS-0042 docker compose example: three controllers and three brokers
P2 · M3 · Task · deps: OPS-0041, RAFT-F07, SRV-F13 · ref: docker/examples/docker-compose-files/cluster @4.4.0
Done when: `deploy/compose/cluster/compose.yaml` runs a dedicated quorum through `controller.quorum.bootstrap.servers` and three brokers; CI creates an RF 3 topic with `min.insync.replicas=2`, stops one broker, and verifies `acks=all` produce and consume continue.

## [To Do] OPS-F04 Kubernetes manifests and Helm chart
P2 · M3 · Feature · deps: OPS-F03
Scope: StatefulSet, services, PDBs, config, probes.

### [To Do] OPS-0043 Decide the Kubernetes deployment model
P2 · M3 · Spike · deps: none
Done when: ADR "Kubernetes deployment" decides: a plain Helm chart (no operator in the parity track; X-AUTO adds one later), dedicated controller and broker StatefulSets plus a combined mode, node ids from pod index with StatefulSet start ordinals, exec probes because the diagnostics endpoint stays on loopback, and the minimum Kubernetes version (start ordinals and the pod-index label).

### [To Do] OPS-0044 Helm chart skeleton with lint and Kubernetes schema validation in CI
P2 · M3 · Task · deps: OPS-0043
Done when: `deploy/helm/alluvium/` has `Chart.yaml` (appVersion = release version), `values.yaml` and `_helpers.tpl` with names and `app.kubernetes.io/*` labels; CI runs `helm lint` and `kubeconform` against the oldest and newest supported Kubernetes versions on default and example values.

### [To Do] OPS-0045 `values.schema.json` validating chart values
P2 · M3 · Task · deps: OPS-0044
Done when: the schema types every value, constrains enums (`topology: combined|dedicated`) and odd controller counts, and requires `clusterId`; CI renders each `tests/values/invalid-*.yaml` and expects a schema error naming the field.

### [To Do] OPS-0046 Headless services for controllers and brokers and the client bootstrap Service
P2 · M3 · Task · deps: OPS-0044
Done when: controller and broker headless services set `publishNotReadyAddresses: true` so quorum and inter-broker DNS resolve before readiness; the bootstrap ClusterIP Service selects ready brokers only; a kind test resolves `<pod>.<service>` from a sibling pod before the pod is ready.

### [To Do] OPS-0047 Controller StatefulSet for a dedicated KRaft quorum
P2 · M3 · Task · deps: OPS-0046, RAFT-F07, SRV-F12 · ref: docs/operations/kraft.md Provisioning Nodes @4.4.0 (KIP-853)
Done when: N controllers (default 3) take `node.id` from the pod index, use `controller.quorum.bootstrap.servers` built from headless DNS names, keep the metadata log on a volumeClaimTemplate, use `podManagementPolicy: Parallel`, and format on first start with the initial controller set; a kind test shows three voters in `kafka-metadata-quorum.sh describe --status`.

### [To Do] OPS-0048 Broker StatefulSet with per-pod advertised listeners and data volumes
P2 · M3 · Task · deps: OPS-0047, CFG-F02
Done when: brokers use `.spec.ordinals.start` (default 100) so node ids never collide with controllers; `advertised.listeners` uses the pod DNS name through the env config provider; one or more data PVCs map to `log.dirs`; a format init step runs with `--ignore-formatted`; a kind test registers three brokers and creates an RF 3 topic with `kafka-topics.sh`.

### [To Do] OPS-0049 Combined-mode topology (`process.roles=broker,controller`)
P2 · M3 · Task · deps: OPS-0047
Done when: `topology: combined` renders one StatefulSet with both roles and quorum bootstrap; a kind test with three combined nodes produces and consumes; values docs state it is meant for small clusters.

### [To Do] OPS-0050 Rendered `server.properties` with per-pod values from the downward API and a config validation step
P2 · M3 · Task · deps: OPS-0048, CLI-F05
Done when: the `config:` map renders to properties; `node.id`, `advertised.listeners` and `broker.rack` come from downward-API environment variables via the env provider, so no shell is needed; a config checksum annotation triggers a rolling restart on change; `alluvium check-config` runs as an init step and blocks start on invalid config.

### [To Do] OPS-0051 PodDisruptionBudgets for controllers and brokers
P2 · M3 · Task · deps: OPS-0047, OPS-0048
Done when: controllers and brokers each get `maxUnavailable: 1`; the values docs explain the interplay with quorum size, replication factor and `min.insync.replicas`; a kind `kubectl drain` test evicts at most one broker at a time.

### [To Do] OPS-0052 Startup, liveness and readiness exec probes and termination grace period
P2 · M3 · Task · deps: OPS-0048, OPS-0033, NFR-F06
Done when: probes exec `alluvium probe live|ready`; the startupProbe's failure budget covers the NFR-F06 recovery target; `terminationGracePeriodSeconds` exceeds the controlled-shutdown timeout; a kind test shows a broker recovering a large log is not killed by liveness and stays unready until its metadata is caught up.

### [To Do] OPS-0053 Pod security: restricted Pod Security Standard and optional NetworkPolicy
P2 · M3 · Task · deps: OPS-0048 · labels: security
Done when: pods run with `runAsNonRoot`, `readOnlyRootFilesystem`, all capabilities dropped, seccomp `RuntimeDefault` and an `fsGroup` for volumes; the chart installs in a namespace enforcing the `restricted` standard on kind; an optional NetworkPolicy limits the controller port to chart pods.

### [To Do] OPS-0054 TLS keystores and SASL credentials from Kubernetes Secrets
P2 · M4 · Task · deps: OPS-0050, NET-F07, SEC-F04, SEC-F16 · labels: security
Done when: PEM key material is mounted from Secrets (with a cert-manager `Certificate` example), SCRAM bootstrap credentials reach `format` from a Secret file, and secrets are referenced through `${file:...}` and never passed in env or arguments; a render test asserts no secret value appears in any ConfigMap; a kind test rotates a certificate and the node reloads it without restart.

### [To Do] OPS-0055 Prometheus scraping with an authenticated diagnostics listener and a ServiceMonitor
P2 · M3 · Task · deps: OPS-0048, OBS-F08 · labels: security
Done when: when monitoring is enabled the diagnostics listener binds the pod IP with a bearer token from a Secret (the standard forbids unauthenticated non-loopback binding) and the ServiceMonitor uses `bearerTokenSecret`; a test shows an unauthenticated scrape gets 401 and an authenticated one returns metrics.

### [To Do] OPS-0056 External access: per-broker LoadBalancer or NodePort services with an EXTERNAL listener
P3 · M3 · Task · deps: OPS-0048 · labels: security
Done when: `external.type: LoadBalancer|NodePort` renders one Service per broker and an EXTERNAL advertised listener per pod; the schema rejects a PLAINTEXT external listener unless the explicit opt-in is set (ADR-0006); a kind NodePort test produces from outside the cluster.

### [To Do] OPS-0057 Scheduling: anti-affinity, zone spread and rack awareness
P3 · M3 · Task · deps: OPS-0048
Done when: defaults add preferred pod anti-affinity per node and topologySpreadConstraints over `topology.kubernetes.io/zone`; a `racks:` value renders one broker StatefulSet per zone with `broker.rack` set; a multi-node kind test shows replicas of a new topic spread across racks in `kafka-topics.sh --describe`.

### [To Do] OPS-0058 Template unit tests with helm-unittest
P2 · M3 · Task · deps: OPS-0045, OPS-0050, OPS-0051, OPS-0052, OPS-0049 · labels: test
Done when: tests per template cover both topologies, PVC counts, start ordinals, probes, security context and config rendering; they run in CI on every chart change.

### [To Do] OPS-0059 Chart end-to-end tests on kind: install, `helm test`, upgrade, uninstall
P2 · M3 · Task · deps: OPS-0058 · labels: test
Done when: a `helm test` hook pod creates a topic, produces and consumes with the pinned kcat image; CI installs on kind for the oldest and newest supported Kubernetes, runs `helm upgrade` with a config change and verifies one pod restarts at a time with no failed `acks=all` produce to an RF 3 topic, then uninstalls and documents that PVCs are kept.

### [To Do] OPS-0060 Chart README with a values table generated from the schema
P2 · M3 · Task · deps: OPS-0045 · labels: docs
Done when: `deploy/helm/alluvium/README.md` has install, topology and security sections and a values table generated from the schema descriptions; CI fails when the table is stale.

### [To Do] OPS-0061 Plain manifests generated from the chart
P3 · M3 · Task · deps: OPS-0059
Done when: `deploy/k8s/{combined,dedicated}/` hold `helm template` output for pinned example values; CI regenerates and fails on drift.

## [To Do] OPS-F05 systemd packaging
P3 · M9 · Feature · deps: OPS-F02
Scope: Unit file, packages, file layout.

### [To Do] OPS-0062 Filesystem layout and service user for host packages
P3 · M9 · Task · deps: none · labels: security
Done when: docs/operations/packaging.md defines `/usr/bin/alluvium`, `/etc/alluvium/` (0750 root:alluvium), `/var/lib/alluvium/` (0750 alluvium:alluvium) and logs to journald; `sysusers.d/alluvium.conf` creates a system user without a login shell and `tmpfiles.d/alluvium.conf` creates the directories.

### [To Do] OPS-0063 systemd readiness notification: READY, STATUS and STOPPING
P3 · M9 · Task · deps: OBS-F09, SRV-F03
Done when: when `NOTIFY_SOCKET` is set the server sends `READY=1` the first time readiness becomes true, `STATUS=` lines with log-recovery progress, and `STOPPING=1` when shutdown starts, using a Unix datagram socket (abstract-namespace addresses supported) and no new dependency; unit tests use a fake socket; nothing is sent when the variable is absent.

### [To Do] OPS-0064 Hardened systemd unit file
P3 · M9 · Task · deps: OPS-0062, OPS-0063 · labels: security
Done when: `deploy/systemd/alluvium.service` uses `Type=notify`, `TimeoutStopSec` above the controlled-shutdown timeout, `Restart=on-failure` with `StartLimitIntervalSec`/`StartLimitBurst` so a crash loop stops, `LimitNOFILE` from the capacity guide, and hardening (`ProtectSystem=strict`, `ReadWritePaths=/var/lib/alluvium`, `NoNewPrivileges`, `PrivateTmp`, `ProtectHome`, empty `CapabilityBoundingSet`, `SystemCallFilter=@system-service`); CI runs `systemd-analyze verify` and requires a `systemd-analyze security` exposure of 2.0 or lower.

### [To Do] OPS-0065 Debian package that never restarts the service on upgrade
P3 · M9 · Task · deps: OPS-0064
Done when: cargo-deb builds amd64 and arm64 packages with `/etc/alluvium/server.properties` as a conffile; the service is enabled but neither started on install nor restarted on upgrade, so unattended upgrades can't restart every broker at once; purge keeps `/var/lib/alluvium`; install, upgrade and remove are tested in debian:12 and ubuntu:24.04 containers.

### [To Do] OPS-0066 RPM package that never restarts the service on upgrade
P3 · M9 · Task · deps: OPS-0064
Done when: cargo-generate-rpm builds x86_64 and aarch64 packages using `%systemd_post` and `%systemd_postun` (not `_with_restart`), with the same layout and conffile handling as the Debian package; install, upgrade and remove are tested in rockylinux:9 and fedora containers.

### [To Do] OPS-0067 Package integration test under a real systemd
P3 · M9 · Task · deps: OPS-0065 · labels: test
Done when: in a systemd-enabled VM or container: install, format, `systemctl start` returns only after READY, `systemctl stop` completes within `TimeoutStopSec`, `kill -9` triggers a restart, repeated crashes stop after `StartLimitBurst`, and `journalctl -o json` shows the structured log lines.

### [To Do] OPS-0068 Attach signed packages to releases
P3 · M9 · Task · deps: OPS-0065, OPS-0066, SEC-F15 · labels: security
Done when: `.deb` and `.rpm` files are attached to the GitHub Release, listed in `SHA256SUMS` and the SBOM, and signed per SEC-F15; docs/operations/packaging.md shows install and signature verification commands.

## [To Do] OPS-F06 Rolling upgrades
P0 · M9 · Feature · deps: META-F06, TEST-F14
Scope: Upgrade procedure, feature gating, on-disk format compatibility between versions.

### [To Do] OPS-0069 Decide upgrade and downgrade compatibility rules
P0 · M9 · Spike · deps: none · ref: server-common/src/main/java/org/apache/kafka/server/common/MetadataVersion.java @4.4.0 (KIP-778, KIP-1170)
Done when: ADR "Upgrade and downgrade compatibility" defines the supported upgrade paths (every patch; each minor to the next; skip-minor limits), how each release maps to supported `metadata.version` and feature-level ranges, that a new on-disk format is only written after its gating feature level is finalized, when a binary downgrade is allowed, `metadata.version` downgrade rules matching Kafka's `metadataChanged` and `--unsafe` semantics, node upgrade order (controllers, then brokers), and the KIP-1170 rule that a node must be at least as new as the tool that formatted its storage.

### [To Do] OPS-0070 Generated table of supported feature levels per release
P0 · M9 · Task · deps: OPS-0069, CTRL-F17 · ref: server-common/src/main/java/org/apache/kafka/server/common/Feature.java @4.4.0
Done when: `cargo xtask surface features` emits min and max supported levels for `metadata.version`, `kraft.version`, `transaction.version`, `group.version`, `eligible.leader.replicas.version`, `share.version` and `streams.version` from the binary's feature registry; the table in docs/operations/upgrading.md is generated and drift-checked in CI.

### [To Do] OPS-0071 CI guard: an on-disk format change needs a version bump, a fixture and a feature gate
P0 · M9 · Task · deps: OPS-0069, STOR-F19 · labels: test
Done when: `cargo xtask upgrade format-guard` fails a PR that changes a file-format definition documented in docs/formats/ (segment, index, checkpoint, producer snapshot, quorum state, `meta.properties`, metadata record schemas) without bumping that format's version constant, adding the TEST-F14 fixture for the new version and registering the gating feature level; tested with fixture diffs.

### [To Do] OPS-0072 Forward-incompatibility test: an older release refuses newer on-disk state without modifying it
P0 · M9 · Task · deps: OPS-0069, SRV-F12 · labels: test · ref: KIP-1170
Done when: CI runs the previous release binary against data directories formatted or upgraded by the current build (newer `meta.properties` version, finalized feature level beyond its range, newer file-format version); each start exits with a named error before writing, and checksums of every file are unchanged afterwards.

### [To Do] OPS-0073 Feature-gate audit: every gated behavior has an upgrade test
P0 · M9 · Task · deps: OPS-0070, META-F06 · labels: test
Done when: `cargo xtask upgrade gates` lists every gate registered with META-F06 (feature, level, behavior) and fails when a gate has no TEST-F14 scenario asserting the behavior is absent below the level and present after `UpdateFeatures`; runs in CI.

### [To Do] OPS-0074 `alluvium admin roll-check`: partition safety before restarting a node
P1 · M9 · Task · deps: NET-F10, API-F03, API-F32, API-F75 · ref: tools/src/main/java/org/apache/kafka/tools/TopicCommand.java @4.4.0
Done when: for `--node-id N` the command lists partitions where losing N leaves the ISR below `min.insync.replicas`, and partitions already under-replicated or offline, exiting 0 only when there are none; output is human-readable or `--json`; a differential test agrees with `kafka-topics.sh --describe --at-min-isr-partitions`, `--under-min-isr-partitions` and `--unavailable-partitions` against Kafka 4.4.0 for the same layout.

### [To Do] OPS-0075 `roll-check` controller quorum safety
P1 · M9 · Task · deps: OPS-0074, API-F55, RAFT-F12 · ref: tools/src/main/java/org/apache/kafka/tools/MetadataQuorumCommand.java @4.4.0
Done when: for a voter N the check fails when the remaining voters caught up within the lag threshold (from DescribeQuorum) are not a majority, and warns when N is the current leader; tests cover a lagging voter, a 3-voter and a 5-voter quorum; data matches `kafka-metadata-quorum.sh describe --replication`.

### [To Do] OPS-0076 Rolling restart tool for Kubernetes and systemd hosts
P1 · M9 · Task · deps: OPS-0075, OPS-F04, OPS-0064
Done when: `deploy/tools/rolling-restart` restarts one node at a time (pod delete or `systemctl restart` over SSH), waits for `probe ready`, runs `roll-check` before the next node, and stops with the node and reason on failure; tested on kind with live `acks=all` traffic and no non-retriable produce errors.

### [To Do] OPS-0077 Upgrade procedure: roll binaries, verify, finalize `metadata.version`
P0 · M9 · Task · deps: OPS-0076, OPS-0070 · labels: docs · ref: docs/getting-started/upgrade.md Upgrading Servers to 4.4.0 @4.4.0
Done when: docs/operations/upgrading.md gives exact steps: check the supported path and feature table, roll nodes in the order the ADR defines with `rolling-restart`, verify (no under-replicated partitions, `roll-check` clean), finalize with `kafka-features.sh upgrade --release-version`, and what changes after finalizing; the TEST-F14 rolling-upgrade and `metadata.version` scenarios execute these commands verbatim, so the doc can't drift.

### [To Do] OPS-0078 Downgrade procedure and limits
P0 · M9 · Task · deps: OPS-0069, OPS-0072, CTRL-F17 · labels: docs · ref: tools/src/main/java/org/apache/kafka/tools/FeatureCommand.java @4.4.0; server-common/src/main/java/org/apache/kafka/server/common/MetadataVersion.java @4.4.0
Done when: docs/operations/upgrading.md has a downgrade section: when a binary downgrade is safe (finalized levels within the older release's range, no newer format written), how `kafka-features.sh downgrade` behaves with and without `--unsafe` when a level in between has `metadataChanged=true`, and the error an unsafe attempt produces; the TEST-F14 downgrade scenario executes the documented commands.

### [To Do] OPS-0079 Version upgrade through the Helm chart under live traffic
P1 · M9 · Task · deps: OPS-0077, OPS-F04 · labels: test
Done when: a kind test installs the previous release's chart, runs producers (`acks=all`, idempotent) and a consumer group, upgrades with `helm upgrade --set image.tag=<current>` using the chart's update strategy gated by `roll-check`, then finalizes `metadata.version`; no acknowledged record is lost and committed offsets are preserved.

### [To Do] OPS-0080 Generated upgrade section in release notes
P1 · M9 · Task · deps: OPS-0070, OPS-0008 · labels: docs
Done when: `cargo xtask release notes` adds an "Upgrading" section listing feature-level range changes, new on-disk formats and their gates, and breaking surface changes; tested with fixture manifests.

## [To Do] OPS-F07 Backup and restore
P1 · M9 · Feature · deps: META-F05
Scope: Metadata export and import; topic data backup approaches.

### [To Do] OPS-0081 Decide backup and restore scope and guarantees
P1 · M9 · Spike · deps: none
Done when: ADR "Backup and restore" defines which metadata an export contains (topics, configs, ACLs, quotas, SCRAM credentials, delegation tokens, feature levels) and excludes (broker and controller registrations, producer id blocks), whether topic IDs are preserved on restore, the supported topic-data approaches (mirror cluster, volume snapshots, sink export, tiered storage limits) with their RPO and RTO, and what is out of scope (point-in-time restore of a live cluster).

### [To Do] OPS-0082 `alluvium metadata export`: consistent export of the metadata image
P1 · M9 · Task · deps: OPS-0081, CLI-F04 · ref: shell/src/main/java/org/apache/kafka/shell/MetadataShell.java @4.4.0; metadata/src/main/java/org/apache/kafka/metadata/util/SnapshotFileReader.java @4.4.0
Done when: the command reads the latest snapshot plus the committed log tail from a controller's metadata log directory read-only (safe on a running node), and writes a snapshot-format file plus `manifest.json` (cluster id, `metadata.version`, last offset and epoch, record counts by type, CRC32C); `alluvium dump metadata` of the export equals the image at that offset; a `metadata_export_completed` event records offset, epoch and size.

### [To Do] OPS-0083 Export credential handling and file permissions
P1 · M9 · Task · deps: OPS-0082 · labels: security · ref: metadata/src/main/java/org/apache/kafka/metadata/util/RecordRedactor.java @4.4.0
Done when: SCRAM credential and delegation-token records are excluded unless `--include-credentials` is given; the file is created with mode 0600; dumping an export redacts sensitive fields as Kafka's RecordRedactor does; tests assert no credential bytes appear without the flag.

### [To Do] OPS-0084 Restore exported metadata into a new cluster at format time
P1 · M9 · Task · deps: OPS-0082, SRV-F12
Done when: `alluvium format --restore-metadata <export>` bootstraps a new quorum with topics (names, partition counts, replication factor, configs; topic IDs preserved or regenerated per flag), ACLs, quotas, dynamic configs and, when present, SCRAM credentials; registrations and producer id blocks are excluded; an export whose `metadata.version` exceeds this build's support is refused; export, restore, export again yields equal records for every included type.

### [To Do] OPS-0085 Restore drill: compare original and restored clusters with Kafka tools
P1 · M9 · Task · deps: OPS-0084, OPS-0083 · labels: test
Done when: a nightly drill populates a cluster with topics, configs, ACLs, quotas and SCRAM users, exports, restores into a new cluster, and compares `kafka-topics.sh --describe`, `kafka-configs.sh --describe --all`, `kafka-acls.sh --list` and client/user quota output, equal apart from topic ids (when regenerated) and node ids.

### [To Do] OPS-0086 Consumer group offset backup and restore
P1 · M9 · Task · deps: GRP-F08
Done when: the documented procedure exports offsets with `kafka-consumer-groups.sh --reset-offsets --to-current --export --dry-run` and restores them with `--reset-offsets --from-file --execute`; a test restores offsets into a restored cluster and a consumer resumes at the exported positions.

### [To Do] OPS-0087 Volume snapshot backups: crash-consistency test and requirements
P1 · M9 · Task · deps: STOR-F09, TEST-F12 · labels: test
Done when: a chaos-harness test snapshots a broker's log directories under `acks=all` load, restores them to a fresh node with the same node id, and recovery succeeds with `alluvium verify log` clean and the replica catching up from leaders; the doc states that all log directories of a node must be snapshotted atomically and that restored replicas truncate to the leader.

### [To Do] OPS-0088 Topic data backup by mirroring to a standby cluster
P1 · M9 · Task · deps: OPS-0184
Done when: `deploy/backup/mm2-standby.properties` is a tested MirrorMaker 2 config for continuous backup; the recovery procedure uses checkpoint-translated offsets; RPO is read from MirrorMaker 2's `replication-latency-ms`; a test fails over a consumer group to the standby with no lost records.

### [To Do] OPS-0089 Tiered storage as a backup source: limits and recovery procedure
P2 · M9 · Task · deps: TIER-F08
Done when: the doc explains that remote segments can't be restored without their remote log metadata, what can and can't be recovered, and that deleting a topic deletes its remote data so tiering is not protection against deletion; a test demonstrates the deletion behavior.

### [To Do] OPS-0090 Backup and restore guide
P1 · M9 · Task · deps: OPS-0085, OPS-0086, OPS-0087, OPS-0088, OPS-0089 · labels: docs
Done when: docs/operations/backup-restore.md gives RPO and RTO per approach, schedules, exact commands and a restore-drill checklist; the commands are the ones executed by the nightly drill.

## [To Do] OPS-F08 Migration from Kafka
P1 · M9 · Feature · deps: TEST-F08
Scope: MirrorMaker 2 guide and tests, offset translation, cutover procedure.

### [To Do] OPS-0091 Decide the Kafka-to-Alluvium migration approach
P1 · M9 · Spike · deps: none · ref: KIP-382
Done when: ADR "Migration from Kafka" chooses MirrorMaker 2 with `IdentityReplicationPolicy` (no mixed clusters), lists what moves automatically (records, topic configs, consumer offsets via checkpoints) and what needs tooling (topics pre-created, ACLs, quotas; SCRAM credentials re-provisioned because their hashes can't be read), how transactional data is mirrored (`read_committed`), cutover ordering, and rollback by reverse mirroring.

### [To Do] OPS-0092 `alluvium migrate inventory`: read-only collection of a Kafka cluster's definitions
P1 · M9 · Task · deps: OPS-0091, NET-F10 · labels: security
Done when: the command connects with TLS and SASL and writes `inventory.json` from Metadata, DescribeConfigs, DescribeAcls, DescribeClientQuotas, ListGroups, DescribeGroups, ApiVersions finalized features and DescribeLogDirs sizes; a test with a principal granted only Describe ACLs proves it sends no mutating request; tested against Kafka 4.4.0.

### [To Do] OPS-0093 `alluvium migrate assess`: compatibility report from the inventory
P1 · M9 · Task · deps: OPS-0092, CFG-F11
Done when: rules flag config keys Alluvium doesn't support (from the config registry), custom plugin classes (`authorizer.class.name`, `principal.builder.class`, `client.quota.callback.class`, remote storage plugins), GSSAPI listeners, and finalized features above Alluvium's support, each with severity and a docs link; output is Markdown or JSON; one unit test per rule.

### [To Do] OPS-0094 MirrorMaker 2 configuration templates for migration
P1 · M9 · Task · deps: OPS-0091 · ref: connect/mirror-client/src/main/java/org/apache/kafka/connect/mirror/IdentityReplicationPolicy.java @4.4.0
Done when: `deploy/migration/mm2/` has a dedicated-mode `connect-mirror-maker.properties` and a Connect-distributed variant with `IdentityReplicationPolicy`, `sync.group.offsets.enabled=true`, `emit.checkpoints.interval.seconds`, `offset-syncs.topic.location=target`, `sync.topic.configs.enabled=true` and consumer `isolation.level=read_committed`, every key commented with its reason; used unchanged by the migration test.

### [To Do] OPS-0095 `alluvium migrate copy-topics`: pre-create topics with identical partitions, replication factor and configs
P1 · M9 · Task · deps: OPS-0092, API-F19, API-F44
Done when: the command prints a dry-run diff, creates missing topics with the source partition count, replication factor and non-default configs, skips internal topics, and never alters an existing topic without `--apply-config-changes`; tested from Kafka 4.4.0 to Alluvium, including rerun idempotence.

### [To Do] OPS-0096 `alluvium migrate copy-acls` and `copy-quotas`
P1 · M9 · Task · deps: OPS-0092, API-F30, API-F49 · labels: security
Done when: ACL bindings and client quotas are copied with a dry-run mode, idempotent reruns and an optional principal-mapping file; a test copies from Kafka 4.4.0 and `kafka-acls.sh --list` and `kafka-configs.sh --describe --entity-type users,clients` match on both sides.

### [To Do] OPS-0097 Credential re-provisioning procedure
P1 · M9 · Task · deps: SEC-F04, API-F51 · labels: security, docs
Done when: docs/migration/credentials.md explains that DescribeUserScramCredentials returns only mechanism and iterations, so users are re-created with `kafka-configs.sh --alter --add-config 'SCRAM-SHA-512=[...]'` or AlterUserScramCredentials with client-computed salted passwords; a script template is tested against Alluvium.

### [To Do] OPS-0098 MirrorMaker 2 replication test from Kafka 4.4.0 to Alluvium
P1 · M9 · Task · deps: OPS-0094, OPS-0095, OPS-0184 · labels: test
Done when: with live producers on Kafka (including transactional producers that abort), MirrorMaker 2 replicates to Alluvium; per partition the record count, keys, value hashes, headers and timestamps are equal and aborted records are absent; topic configs are synced; runs nightly.

### [To Do] OPS-0099 Offset translation verification and report
P1 · M9 · Task · deps: OPS-0098 · ref: connect/mirror/src/main/java/org/apache/kafka/connect/mirror/OffsetSyncStore.java translateDownstream @4.4.0; connect/mirror/src/main/java/org/apache/kafka/connect/mirror/MirrorCheckpointTask.java syncGroupOffset @4.4.0
Done when: for groups consuming on Kafka, offsets synced to Alluvium are never ahead of the exact translated position and consumers switched to Alluvium reprocess at most the window bounded by `offset.lag.max`, never skipping a record; `alluvium migrate offsets-report` prints per partition the source offset, translated offset and expected duplicates.

### [To Do] OPS-0100 Dual-run validation: `alluvium migrate validate` compares source and target record streams
P1 · M9 · Task · deps: OPS-0098
Done when: the validator consumes both clusters per partition (assigned, `read_committed`) with bounded memory, compares key, value hash, headers and timestamp record by record up to MirrorMaker 2's position, and reports the first divergence with offsets on both sides; it runs inside the migration test.

### [To Do] OPS-0101 Cutover runbook
P1 · M9 · Task · deps: OPS-0099, OPS-0100, OPS-0093, OPS-0096, OPS-0097 · labels: docs
Done when: docs/migration/cutover.md walks through assess, provision, pre-create, mirror, validate, cut over consumers, cut over producers, decommission, with exact commands and go/no-go checks (MirrorMaker 2 lag zero, `validate` clean, no ongoing transactions), plus Kafka Streams and Connect application steps; the migration drill executes the commands verbatim.

### [To Do] OPS-0102 Rollback by reverse mirroring from Alluvium to Kafka
P1 · M9 · Task · deps: OPS-0101
Done when: a tested config and procedure mirror writes made after cutover back to Kafka; a test rolls back mid-migration and loses no record.

### [To Do] OPS-0103 Scripted migration drill in CI
P1 · M9 · Task · deps: OPS-0101, OPS-0102, OPS-0182, OPS-0180 · labels: test
Done when: a nightly run takes a Kafka 4.4.0 cluster with Java producers, consumer groups, a Kafka Streams app and a Connect sink through the runbook to Alluvium; no record is lost, duplicates stay within the reported window, and group lag recovers after cutover.

## [To Do] OPS-F09 Capacity planning
P2 · M9 · Feature · deps: NFR-F01
Scope: Sizing guide and calculator.

### [To Do] OPS-0104 Measure capacity coefficients from the standard workloads
P2 · M9 · Task · deps: NFR-F05, TEST-F11 · labels: perf
Done when: a benchmark job writes `docs/operations/capacity/coefficients.json`: CPU seconds per MB produced and fetched by batch size, codec, TLS on or off and acks; memory per partition and per connection; on-disk overhead per partition and segment (indexes, `segment.index.bytes` preallocation); with the hardware description and the command that reproduces it.

### [To Do] OPS-0105 Sizing model for disk and network
P2 · M9 · Task · deps: OPS-0104 · labels: docs
Done when: docs/operations/capacity.md gives formulas for disk (ingress × retention × replication factor ÷ compression ratio + segment and index overhead + headroom kept free by STOR-F25 disk-full protection) and network (produce ingress × replication factor + consumer fan-out + reassignment headroom), with worked examples that match the calculator.

### [To Do] OPS-0106 Sizing model for partitions, memory, file descriptors and controllers
P2 · M9 · Task · deps: OPS-0104, NFR-F02, NFR-F06 · labels: docs
Done when: the guide covers per-partition memory, file descriptors (segments × files per segment + connections) and the resulting `LimitNOFILE`, memory-map limits if indexes are mapped, recovery time versus partitions per broker (NFR-F06), controller metadata size per partition, and the NFR-F02 scalability limits.

### [To Do] OPS-0107 Capacity calculator core: pure function from workload to resource plan
P2 · M9 · Task · deps: OPS-0105, OPS-0106
Done when: a module in the binary crate (no I/O) maps topics, rates, record sizes, replication factor, retention, consumer fan-out and TLS to broker count, disk, NIC bandwidth, memory, CPU and file descriptors, with warnings when NFR-F02 limits are exceeded; property tests show outputs are monotonic in ingress, retention and replication factor; unit tests reproduce the guide's examples.

### [To Do] OPS-0108 `alluvium capacity plan --workload <file>`
P2 · M9 · Task · deps: OPS-0107
Done when: the subcommand reads a TOML workload, prints a human table or `--json`, and names the field on invalid input; the guide's examples are generated from the CLI and drift-checked in CI.

### [To Do] OPS-0109 Validate calculator predictions against a measured cluster
P2 · M9 · Task · deps: OPS-0108, TEST-F13 · labels: test
Done when: for three reference workloads the predicted CPU, disk and network are within ±20% of soak-test measurements; results are committed and each deviation is explained or fixed by updating coefficients.

## [To Do] OPS-F10 Runbooks
P1 · M9 · Feature · deps: OBS-F07
Scope: One runbook per alert and per common failure.

### [To Do] OPS-0110 Runbook template and index
P1 · M9 · Task · deps: none · labels: docs
Done when: docs/runbooks/README.md indexes all runbooks; the template has Alert, Symptoms, Impact, Diagnosis (PromQL, `/debug` routes, event names, Kafka tool and `alluvium` commands), Mitigation, Verification and Escalation (attach `alluvium debug-bundle`) sections, and anchors that alert `runbook_url` annotations point to.

### [To Do] OPS-0111 Runbook: under-replicated partitions
P1 · M9 · Task · deps: OPS-0110, REPL-F17, REPL-F05 · labels: docs
Done when: docs/runbooks/under-replicated-partitions.md follows the template and covers causes (follower down, slow disk or network, replica throttling); diagnosis with the metric mapped from `kafka.server:type=ReplicaManager,name=UnderReplicatedPartitions`, `kafka-topics.sh --describe --under-replicated-partitions`, `isr_shrink` events and their reasons, and follower fetch lag; mitigation and how to confirm recovery; every command in it was run against a cluster in that state.

### [To Do] OPS-0112 Runbook: offline partitions
P1 · M9 · Task · deps: OPS-0110, CTRL-F19, CTRL-F08 · labels: docs
Done when: docs/runbooks/offline-partitions.md follows the template and covers diagnosis with the metric mapped from `kafka.controller:type=KafkaController,name=OfflinePartitionsCount`, `kafka-topics.sh --describe --unavailable-partitions` and leader election events; the trade-offs of eligible leader replicas and `unclean.leader.election.enable` with `kafka-leader-election.sh --election-type unclean`; every command in it was run against a cluster in that state.

### [To Do] OPS-0113 Runbook: partitions under min ISR
P1 · M9 · Task · deps: OPS-0110, REPL-F17 · labels: docs
Done when: docs/runbooks/under-min-isr.md follows the template and covers the producer-visible symptoms (`NOT_ENOUGH_REPLICAS`, `NOT_ENOUGH_REPLICAS_AFTER_APPEND` for `acks=all`), diagnosis with the metric mapped from `UnderMinIsrPartitionCount` and `kafka-topics.sh --describe --under-min-isr-partitions`, and why lowering `min.insync.replicas` trades durability; every command in it was run against a cluster in that state.

### [To Do] OPS-0114 Runbook: controller unavailable or quorum lost
P1 · M9 · Task · deps: OPS-0110, RAFT-F14, CTRL-F19 · labels: docs
Done when: docs/runbooks/controller-unavailable.md follows the template and covers diagnosis with the active-controller metric (sum must be 1), `kafka-metadata-quorum.sh describe --status` and `describe --replication`, Raft leader-change events and `/debug/raft`; recovery when a minority or a majority of voters is lost, including KIP-853 voter removal; every command in it was run against a cluster in that state.

### [To Do] OPS-0115 Runbook: broker fenced or not registering
P2 · M9 · Task · deps: OPS-0110, SRV-F13, CTRL-F02 · labels: docs
Done when: docs/runbooks/broker-fenced.md follows the template and covers diagnosis with broker lifecycle events, heartbeat errors (`STALE_BROKER_EPOCH`, `INCONSISTENT_CLUSTER_ID`), `kafka-cluster.sh` and metadata lag; common causes such as a reused node id or mismatched cluster id; every command in it was run against a cluster in that state.

### [To Do] OPS-0116 Runbook: log directory filling up or full
P1 · M9 · Task · deps: OPS-0110, STOR-F25, STOR-F13 · labels: docs
Done when: docs/runbooks/disk-full.md follows the template and covers diagnosis with log-dir usage metrics and disk-full protection events; mitigation by lowering retention, `kafka-delete-records.sh`, reassigning partitions or adding disks, and the rule never to delete segment files by hand; every command in it was run against a cluster in that state.

### [To Do] OPS-0117 Runbook: log directory failure
P1 · M9 · Task · deps: OPS-0110, STOR-F21 · labels: docs
Done when: docs/runbooks/log-dir-failure.md follows the template and covers diagnosis with the metric mapped from `OfflineLogDirectoryCount`, `kafka-log-dirs.sh --describe` and the log-dir failure event; replacing the disk, restarting the node and letting replicas re-sync; every command in it was run against a cluster in that state.

### [To Do] OPS-0118 Runbook: corrupted segment found during recovery
P2 · M9 · Task · deps: OPS-0110, STOR-F09, CLI-F04 · labels: docs
Done when: docs/runbooks/corrupted-segment.md follows the template and covers diagnosis from the recovery truncation event (truncated range and reason), `alluvium verify log` and `alluvium dump log`; when to let the replica re-sync from the leader and what to attach to a bug report; every command in it was run against a cluster in that state.

### [To Do] OPS-0119 Runbook: high request latency
P1 · M9 · Task · deps: OPS-0110, SRV-F08 · labels: docs
Done when: docs/runbooks/high-request-latency.md follows the template and covers locating the slow phase from the per-API phase histograms (request queue, local, remote, throttle, response queue, send) as Kafka's `RequestMetrics` do, with the likely cause per phase and the matching `/debug` routes and request log filters; every command in it was run against a cluster in that state.

### [To Do] OPS-0120 Runbook: request queue saturation and muted connections
P1 · M9 · Task · deps: OPS-0110, SRV-F05, NET-F05 · labels: docs
Done when: docs/runbooks/request-queue-saturation.md follows the template and covers diagnosis with request queue depth and capacity, handler idle ratio (Kafka `RequestHandlerAvgIdlePercent`), network idle ratio (`NetworkProcessorAvgIdlePercent`) and connection-mute events; mitigation by finding hot clients and tuning handler threads; every command in it was run against a cluster in that state.

### [To Do] OPS-0121 Runbook: consumer lag growing
P1 · M9 · Task · deps: OPS-0110, GRP-F11 · labels: docs
Done when: docs/runbooks/consumer-lag.md follows the template and covers diagnosis with `kafka-consumer-groups.sh --describe`, group lag metrics and the distinction between slow consumers, stuck partitions (a hanging transaction holding back the last stable offset) and rebalancing; every command in it was run against a cluster in that state.

### [To Do] OPS-0122 Runbook: rebalance storms
P1 · M9 · Task · deps: OPS-0110, GRP-F11, GRP-F04, GRP-F05 · labels: docs
Done when: docs/runbooks/rebalance-storm.md follows the template and covers diagnosis with rebalance-rate metrics and group state events with reasons; causes such as `max.poll.interval.ms` overruns, session timeouts and non-static members during rolling deploys; mitigations including static membership, cooperative assignors and the consumer group protocol; every command in it was run against a cluster in that state.

### [To Do] OPS-0123 Runbook: hanging transactions holding back the last stable offset
P1 · M9 · Task · deps: OPS-0110, TXN-F13, TXN-F16 · labels: docs
Done when: docs/runbooks/hanging-transaction.md follows the template and covers diagnosis with the gap between high watermark and last stable offset, ongoing-transaction age metrics, `kafka-transactions.sh find-hanging` and `describe`; mitigation with `kafka-transactions.sh abort` (KIP-664) and its risks; every command in it was run against a cluster in that state.

### [To Do] OPS-0124 Runbook: TLS certificate expiry
P1 · M9 · Task · deps: OPS-0110, NET-F07, NET-F11 · labels: docs
Done when: docs/runbooks/certificate-expiry.md follows the template and covers diagnosis with the certificate expiry metric and TLS handshake failure counts per listener; rotation without restart through hot reload, for client and inter-node listeners; every command in it was run against a cluster in that state.

### [To Do] OPS-0125 Runbook: clients throttled by quotas
P2 · M9 · Task · deps: OPS-0110, QUOTA-F09, QUOTA-F07 · labels: docs
Done when: docs/runbooks/quota-throttling.md follows the template and covers diagnosis with throttle-time metrics per user and client id and `kafka-configs.sh --describe --entity-type users,clients`; telling quota throttling from request-queue saturation; every command in it was run against a cluster in that state.

### [To Do] OPS-0126 Runbook: authentication failure spike
P2 · M9 · Task · deps: OPS-0110, SEC-F02, NET-F11 · labels: docs
Done when: docs/runbooks/authentication-failures.md follows the template and covers diagnosis with authentication failure counts by listener and mechanism, the SASL failure events (never including credentials) and peer addresses; expired SCRAM credentials, clock skew for OAUTHBEARER and misconfigured clients; every command in it was run against a cluster in that state.

### [To Do] OPS-0127 Runbook: leader imbalance
P2 · M9 · Task · deps: OPS-0110, CTRL-F08, API-F43 · labels: docs
Done when: docs/runbooks/leader-imbalance.md follows the template and covers diagnosis with preferred-replica imbalance metrics; `kafka-leader-election.sh --election-type preferred` and `auto.leader.rebalance.enable`; every command in it was run against a cluster in that state.

### [To Do] OPS-0128 Runbook: tiered storage copy lag or remote errors
P2 · M9 · Task · deps: OPS-0110, TIER-F09 · labels: docs
Done when: docs/runbooks/tiered-storage-lag.md follows the template and covers diagnosis with remote copy lag and remote error metrics; local disk pressure caused by copy lag; remote storage outages; every command in it was run against a cluster in that state.

### [To Do] OPS-0129 Runbook: node crash loop
P1 · M9 · Task · deps: OPS-0110, SRV-F16, OBS-F09, CLI-F05 · labels: docs
Done when: docs/runbooks/crash-loop.md follows the template and covers finding the crash report (backtrace, span stack, flight recorder) in the previous container or journald logs, telling config errors (`alluvium check-config`) from panics, startup probes killing slow recoveries, and filing a bug with `alluvium debug-bundle`; every command in it was run against a cluster in that state.

### [To Do] OPS-0130 Runbook drills: diagnosis steps verified against induced failures
P1 · M9 · Task · deps: OPS-0111, OPS-0112, OPS-0113, OPS-0114, OPS-0116, OPS-0117, OPS-0124, OPS-0129, TEST-F12 · labels: test
Done when: chaos scenarios induce under-replication, offline partitions, under-min-ISR, quorum loss, a full disk, a failed log directory, a near-expiry certificate and a crash loop; the diagnosis commands marked as testable in each runbook run and their output matches the documented examples, failing CI on drift.

## [To Do] OPS-F11 Alert rules and dashboards
P1 · M9 · Feature · deps: OBS-F05
Scope: Prometheus alert rules and Grafana dashboards.

### [To Do] OPS-0131 Alert rule layout, conventions and CI checks
P1 · M9 · Task · deps: OBS-F05 · labels: test
Done when: rules live in `deploy/monitoring/prometheus/rules/<subsystem>.yaml` with required labels `severity` (`page` or `ticket`) and `subsystem` and annotations `summary`, `description` and `runbook_url`; CI runs `promtool check rules` and `promtool test rules`, and fails when a rule uses a metric missing from docs/metrics.md or a `runbook_url` anchor that doesn't exist.

### [To Do] OPS-0132 Alert rules: replication
P1 · M9 · Task · deps: OPS-0131, REPL-F17, OPS-0111, OPS-0113
Done when: `rules/replication.yaml` alerts on under-replicated partitions, partitions under and at min ISR, and ISR shrink rate; every rule has a `for` duration and threshold justified in a comment, a runbook link, and `promtool test rules` cases that fire and that stay silent.

### [To Do] OPS-0133 Alert rules: controller
P1 · M9 · Task · deps: OPS-0131, CTRL-F19, RAFT-F14, OPS-0114, OPS-0112, OPS-0115
Done when: `rules/controller.yaml` alerts on active controller count not exactly 1, offline partitions, quorum leader churn, high metadata commit latency, fenced brokers and broker metadata lag; every rule has a `for` duration and threshold justified in a comment, a runbook link, and `promtool test rules` cases that fire and that stay silent.

### [To Do] OPS-0134 Alert rules: storage
P1 · M9 · Task · deps: OPS-0131, STOR-F25, STOR-F21, OPS-0116, OPS-0117
Done when: `rules/storage.yaml` alerts on log-dir usage above thresholds and predicted to fill within 24 h (`predict_linear`), offline log directories and slow flushes; every rule has a `for` duration and threshold justified in a comment, a runbook link, and `promtool test rules` cases that fire and that stay silent.

### [To Do] OPS-0135 Alert rules: requests
P1 · M9 · Task · deps: OPS-0131, SRV-F08, SRV-F05, OPS-0119, OPS-0120
Done when: `rules/requests.yaml` alerts on per-API p99 total time above budget, error ratio by `error_code`, request queue saturation and low handler idle ratio; every rule has a `for` duration and threshold justified in a comment, a runbook link, and `promtool test rules` cases that fire and that stay silent.

### [To Do] OPS-0136 Alert rules: network
P1 · M9 · Task · deps: OPS-0131, NET-F11, NET-F07, OPS-0124
Done when: `rules/network.yaml` alerts on connection count near `max.connections`, connection-rate spikes, TLS handshake failures and certificates expiring within 14 and 3 days; every rule has a `for` duration and threshold justified in a comment, a runbook link, and `promtool test rules` cases that fire and that stay silent.

### [To Do] OPS-0137 Alert rules: groups
P1 · M9 · Task · deps: OPS-0131, GRP-F11, OPS-0121, OPS-0122
Done when: `rules/groups.yaml` alerts on consumer lag growth per group, rebalance rate and slow coordinator loading; every rule has a `for` duration and threshold justified in a comment, a runbook link, and `promtool test rules` cases that fire and that stay silent.

### [To Do] OPS-0138 Alert rules: transactions
P1 · M9 · Task · deps: OPS-0131, TXN-F16, OPS-0123
Done when: `rules/transactions.yaml` alerts on ongoing transactions older than `transaction.max.timeout.ms` and a growing gap between high watermark and last stable offset; every rule has a `for` duration and threshold justified in a comment, a runbook link, and `promtool test rules` cases that fire and that stay silent.

### [To Do] OPS-0139 Alert rules: quotas
P2 · M9 · Task · deps: OPS-0131, QUOTA-F09, OPS-0125
Done when: `rules/quotas.yaml` alerts on sustained throttle time per user and client id; every rule has a `for` duration and threshold justified in a comment, a runbook link, and `promtool test rules` cases that fire and that stay silent.

### [To Do] OPS-0140 Alert rules: security
P2 · M9 · Task · deps: OPS-0131, SEC-F08, NET-F08, OPS-0126
Done when: `rules/security.yaml` alerts on authentication failure rate and authorization denial spikes; every rule has a `for` duration and threshold justified in a comment, a runbook link, and `promtool test rules` cases that fire and that stay silent.

### [To Do] OPS-0141 Alert rules: process
P1 · M9 · Task · deps: OPS-0131, OBS-F04, SRV-F16, OPS-0129
Done when: `rules/process.yaml` alerts on restarts and crash loops, memory near its limit, file descriptors near `LimitNOFILE` and a node not scraped; every rule has a `for` duration and threshold justified in a comment, a runbook link, and `promtool test rules` cases that fire and that stay silent.

### [To Do] OPS-0142 Alert rules: tiered
P2 · M9 · Task · deps: OPS-0131, TIER-F09, OPS-0128
Done when: `rules/tiered.yaml` alerts on remote copy lag and remote storage error rate; every rule has a `for` duration and threshold justified in a comment, a runbook link, and `promtool test rules` cases that fire and that stay silent.

### [To Do] OPS-0143 Alert rules: share
P2 · M9 · Task · deps: OPS-0131, SHARE-F10
Done when: `rules/share.yaml` alerts on share-group delivery attempts reaching the delivery count limit and acquisition-lock timeouts; every rule has a `for` duration and threshold justified in a comment, a runbook link, and `promtool test rules` cases that fire and that stay silent.

### [To Do] OPS-0144 Grafana dashboard conventions and CI checks
P1 · M9 · Task · deps: OBS-F05 · labels: test
Done when: dashboards live in `deploy/monitoring/grafana/` with a pinned schema version, stable `uid`s and `datasource`, `cluster` and `node` variables; CI lints them, fails when a query uses a metric missing from docs/metrics.md, and provisions them into a pinned Grafana container that reports no errors.

### [To Do] OPS-0145 Grafana dashboard: Cluster overview
P1 · M9 · Task · deps: OPS-0144, REPL-F17, CTRL-F19, SRV-F08
Done when: `grafana/overview.json` shows nodes up, active controller, partition and leader counts, under-replicated and offline partitions, bytes in and out, request p99; each panel has a description and links to the related runbook; a CI test runs every panel query against a cluster under the standard workload and gets a non-empty result.

### [To Do] OPS-0146 Grafana dashboard: Node
P1 · M9 · Task · deps: OPS-0144, SRV-F08, NET-F11, OBS-F04
Done when: `grafana/node.json` shows CPU, memory, file descriptors, network, request phases, queue depths and supervised task counts for one node; each panel has a description and links to the related runbook; a CI test runs every panel query against a cluster under the standard workload and gets a non-empty result.

### [To Do] OPS-0147 Grafana dashboard: Topics and partitions
P1 · M9 · Task · deps: OPS-0144, REPL-F17
Done when: `grafana/topics.json` shows bytes and records in and out per topic, log size and per-partition panels shown only when `alluvium.metrics.per.partition.enabled` is set; each panel has a description and links to the related runbook; a CI test runs every panel query against a cluster under the standard workload and gets a non-empty result.

### [To Do] OPS-0148 Grafana dashboard: Replication
P1 · M9 · Task · deps: OPS-0144, REPL-F17, REPL-F16
Done when: `grafana/replication.json` shows ISR changes, follower fetch lag, high watermark progress and replication throttling; each panel has a description and links to the related runbook; a CI test runs every panel query against a cluster under the standard workload and gets a non-empty result.

### [To Do] OPS-0149 Grafana dashboard: KRaft and controller
P1 · M9 · Task · deps: OPS-0144, RAFT-F14, CTRL-F19
Done when: `grafana/kraft.json` shows quorum leader and epoch, commit latency, voter lag, broker metadata offset lag, snapshot generation and controller event rates; each panel has a description and links to the related runbook; a CI test runs every panel query against a cluster under the standard workload and gets a non-empty result.

### [To Do] OPS-0150 Grafana dashboard: Storage
P1 · M9 · Task · deps: OPS-0144, STOR-F25, STOR-F14
Done when: `grafana/storage.json` shows log-dir usage, flush latency, segment rolls, retention deletions, recovery progress and log cleaner activity; each panel has a description and links to the related runbook; a CI test runs every panel query against a cluster under the standard workload and gets a non-empty result.

### [To Do] OPS-0151 Grafana dashboard: Requests and network
P1 · M9 · Task · deps: OPS-0144, SRV-F08, NET-F11
Done when: `grafana/requests.json` shows per-API rate, errors by code, phase latency heatmaps, connections, TLS handshakes and muted connections; each panel has a description and links to the related runbook; a CI test runs every panel query against a cluster under the standard workload and gets a non-empty result.

### [To Do] OPS-0152 Grafana dashboard: Consumer groups
P1 · M9 · Task · deps: OPS-0144, GRP-F11
Done when: `grafana/groups.json` shows lag per group and topic, rebalances, members and offset commit rate; each panel has a description and links to the related runbook; a CI test runs every panel query against a cluster under the standard workload and gets a non-empty result.

### [To Do] OPS-0153 Grafana dashboard: Producers and transactions
P1 · M9 · Task · deps: OPS-0144, TXN-F16
Done when: `grafana/transactions.json` shows ongoing transactions and their age, commits and aborts, producer id allocation and last-stable-offset lag; each panel has a description and links to the related runbook; a CI test runs every panel query against a cluster under the standard workload and gets a non-empty result.

### [To Do] OPS-0154 Grafana dashboard: Quotas
P2 · M9 · Task · deps: OPS-0144, QUOTA-F09
Done when: `grafana/quotas.json` shows throttle time and quota usage per user and client id; each panel has a description and links to the related runbook; a CI test runs every panel query against a cluster under the standard workload and gets a non-empty result.

### [To Do] OPS-0155 Grafana dashboard: Security
P2 · M9 · Task · deps: OPS-0144, SEC-F08, NET-F08
Done when: `grafana/security.json` shows authentication results by mechanism and listener, authorization denials by operation; each panel has a description and links to the related runbook; a CI test runs every panel query against a cluster under the standard workload and gets a non-empty result.

### [To Do] OPS-0156 Grafana dashboard: Tiered storage
P2 · M9 · Task · deps: OPS-0144, TIER-F09
Done when: `grafana/tiered.json` shows remote copy and fetch rates, lag, errors and local versus remote size; each panel has a description and links to the related runbook; a CI test runs every panel query against a cluster under the standard workload and gets a non-empty result.

### [To Do] OPS-0157 Grafana dashboard: Share groups
P2 · M9 · Task · deps: OPS-0144, SHARE-F10
Done when: `grafana/share.json` shows acquired and in-flight records, delivery counts, acknowledgements and lock timeouts; each panel has a description and links to the related runbook; a CI test runs every panel query against a cluster under the standard workload and gets a non-empty result.

### [To Do] OPS-0158 Alert firing tests against induced failures
P1 · M9 · Task · deps: OPS-0132, OPS-0133, OPS-0134, OPS-0136, OPS-0139, TEST-F12 · labels: test
Done when: the chaos harness with Prometheus kills a broker, fills a disk, stops a controller majority, serves a near-expiry certificate and exceeds a quota; the matching alerts fire within their `for` plus two scrape intervals and resolve after recovery.

### [To Do] OPS-0159 Ship rules and dashboards in the Helm chart
P2 · M9 · Task · deps: OPS-0158, OPS-0145, OPS-0055
Done when: the chart optionally renders a `PrometheusRule` from the rule files and Grafana dashboard ConfigMaps with the sidecar label; a render test checks they match `deploy/monitoring/` byte for byte.

## [To Do] OPS-F12 Ecosystem compatibility
P1 · M2 · Feature · deps: TEST-F08
Scope: One task per ecosystem tool or framework (Connect, Streams, Debezium, Flink, Spark, UIs, operators).

### [To Do] OPS-0160 Ecosystem suite layout and pinned-tool manifest
P1 · M2 · Task · deps: none · labels: test
Done when: each tool has `tests/compat/ecosystem/<tool>/tool.toml` (version, image digest, required Alluvium features, scenarios) and scripted scenarios run on the TEST-F08 harness by `cargo xtask compat --suite ecosystem [--tool <tool>]`; a nightly CI job runs every enabled tool; a failure reports tool, scenario, client logs and node logs.

### [To Do] OPS-0161 Ecosystem results file consumed by the compatibility docs
P1 · M2 · Task · deps: OPS-0160
Done when: each nightly run writes `tests/compat/ecosystem/results.json` (tool, version, scenarios with pass/fail, Alluvium commit, date); a tool is marked supported only when every scenario passes; the schema is documented for DOCS-F10 to render.

### [To Do] OPS-0162 Automated version-bump PRs for pinned ecosystem tools
P3 · M2 · Task · deps: OPS-0160
Done when: a scheduled dependency-update config proposes digest and version bumps for every `tool.toml`; each bump PR runs that tool's suite before review.

### [To Do] OPS-0163 kcat: produce, consume by partition, metadata listing and offset lookup
P1 · M2 · Task · deps: OPS-0160, API-F00, API-F01, API-F02, API-F03 · labels: test
Done when: `tests/compat/ecosystem/kcat/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool kcat` passes: `kcat -P` with keys and headers (`-H`), `kcat -C -p 0 -o beginning -e`, `kcat -L` listing brokers and topics as Metadata reports them, `kcat -Q` timestamp lookups, and `-z gzip|snappy|lz4|zstd` round trips; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0164 kcat balanced consumer (`-G`)
P2 · M5 · Task · deps: OPS-0160, GRP-F04 · labels: test
Done when: `tests/compat/ecosystem/kcat-groups/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool kcat-groups` passes: two `kcat -G` members split partitions, rebalance when one exits, commit offsets and resume from them; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0165 rskafka (pure-Rust partition client)
P3 · M2 · Task · deps: OPS-0160, API-F00, API-F01, API-F02, API-F19, API-F20 · labels: test
Done when: `tests/compat/ecosystem/rskafka/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool rskafka` passes: produce, fetch, list offsets, create and delete topics through its partition client; its DeleteRecords scenario is tagged as requiring API-F21 and runs once that lands; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0166 confluent-kafka-python: Producer, Consumer and AdminClient
P1 · M5 · Task · deps: OPS-0160, GRP-F04, GRP-F05, API-F19 · labels: test
Done when: `tests/compat/ecosystem/confluent-kafka-python/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool confluent-kafka-python` passes: delivery reports with compression, consumers with `group.protocol=classic` and `consumer` committing and seeking, and AdminClient create, describe and delete topics and describe cluster; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0167 confluent-kafka-go: Producer, Consumer and AdminClient
P1 · M5 · Task · deps: OPS-0160, GRP-F04, GRP-F05, API-F19 · labels: test
Done when: `tests/compat/ecosystem/confluent-kafka-go/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool confluent-kafka-go` passes: the same producer, consumer (both group protocols) and admin scenarios as confluent-kafka-python; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0168 confluent-kafka-dotnet: Producer, Consumer and AdminClient
P1 · M5 · Task · deps: OPS-0160, GRP-F04, GRP-F05, API-F19 · labels: test
Done when: `tests/compat/ecosystem/confluent-kafka-dotnet/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool confluent-kafka-dotnet` passes: the same producer, consumer (both group protocols) and admin scenarios as confluent-kafka-python; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0169 rust-rdkafka: FutureProducer, StreamConsumer and AdminClient
P1 · M5 · Task · deps: OPS-0160, GRP-F04, GRP-F05, API-F19 · labels: test
Done when: `tests/compat/ecosystem/rust-rdkafka/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool rust-rdkafka` passes: async produce with delivery futures, `StreamConsumer` in a group with manual commits under both group protocols, and admin topic creation and deletion; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0170 KafkaJS: producer, consumer group and admin
P2 · M5 · Task · deps: OPS-0160, GRP-F04, API-F19 · labels: test
Done when: `tests/compat/ecosystem/kafkajs/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool kafkajs` passes: gzip producer, consumer group with `eachBatch`, admin `createTopics` and `fetchTopicOffsets`, with the negotiated API versions recorded to cover the low end of the advertised ranges; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0171 Sarama: producers, consumer groups and ClusterAdmin
P1 · M5 · Task · deps: OPS-0160, GRP-F04, API-F19 · labels: test
Done when: `tests/compat/ecosystem/sarama/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool sarama` passes: SyncProducer, AsyncProducer, ConsumerGroup with range and sticky strategies, and ClusterAdmin, run with `Config.Version` set both to the lowest version Kafka 4.x still accepts (KIP-896) and to the newest constant; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0172 segmentio/kafka-go: Writer, group Reader and Client
P2 · M5 · Task · deps: OPS-0160, GRP-F04, API-F19 · labels: test
Done when: `tests/compat/ecosystem/kafka-go/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool kafka-go` passes: Writer with compression, Reader with `GroupID` committing offsets, and Client CreateTopics and ListOffsets; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0173 kafka-python: producer, consumer group and admin
P2 · M5 · Task · deps: OPS-0160, GRP-F04, API-F19 · labels: test
Done when: `tests/compat/ecosystem/kafka-python/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool kafka-python` passes: KafkaProducer, KafkaConsumer in a group with commits, and KafkaAdminClient topic operations; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0174 aiokafka: async producer and consumer group
P3 · M5 · Task · deps: OPS-0160, GRP-F04 · labels: test
Done when: `tests/compat/ecosystem/aiokafka/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool aiokafka` passes: AIOKafkaProducer with batches and AIOKafkaConsumer in a group with manual commits; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0175 samsa (pure-Rust client with consumer groups)
P3 · M5 · Task · deps: OPS-0160, GRP-F04 · labels: test
Done when: `tests/compat/ecosystem/samsa/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool samsa` passes: producer, partition consumer and consumer-group streams from its examples; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0176 Karafka (Ruby, librdkafka): consumers and WaterDrop producer
P2 · M5 · Task · deps: OPS-0160, GRP-F04 · labels: test
Done when: `tests/compat/ecosystem/karafka/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool karafka` passes: a Karafka app consumes in a group with offset commits and produces with WaterDrop, including restart and rebalance; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0177 ruby-kafka (archived): confirm it fails the same way as against Kafka 4.4.0
P3 · M5 · Task · deps: OPS-0160, TEST-F09 · labels: test · ref: KIP-896
Done when: a differential scenario shows ruby-kafka gets the same responses from Alluvium and Kafka 4.4.0 for the API versions Kafka 4.0 removed; the matrix marks it unsupported with that reason.

### [To Do] OPS-0178 Spring for Apache Kafka: templates, listeners, error handling and transactions
P1 · M6 · Task · deps: OPS-0160, GRP-F04, TXN-F08 · labels: test
Done when: `tests/compat/ecosystem/spring-kafka/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool spring-kafka` passes: `KafkaTemplate` with idempotence on by default, concurrent `@KafkaListener` containers, `DefaultErrorHandler` with `DeadLetterPublishingRecoverer`, `@RetryableTopic` topics created through `KafkaAdmin`, and `executeInTransaction` plus `KafkaTransactionManager` commits and rollbacks; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0179 Kafka Connect standalone with the FileStream connectors
P1 · M6 · Task · deps: OPS-0160, GRP-F04, TXN-F01 · labels: test · ref: docs/getting-started/quickstart.md Step 6 @4.4.0
Done when: `tests/compat/ecosystem/connect-standalone/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool connect-standalone` passes: the Kafka quickstart file-source to file-sink pipeline runs, source offsets persist in the offsets file, the sink's consumer group commits, and a worker restart resumes without loss; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0180 Kafka Connect distributed: internal topics, rebalancing and REST
P1 · M6 · Task · deps: OPS-0160, GRP-F04, STOR-F14, TXN-F01 · labels: test
Done when: `tests/compat/ecosystem/connect-distributed/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool connect-distributed` passes: three workers create config, offset and status topics with `cleanup.policy=compact` (verified with `kafka-configs.sh`), connectors are created through REST, killing a worker triggers an incremental cooperative rebalance that reassigns its tasks with offsets preserved, and `connect-internal-topics.sh` (KIP-1209) creates the topics when automatic creation is off; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0181 Kafka Connect exactly-once source connectors (KIP-618)
P1 · M6 · Task · deps: OPS-0160, TXN-F08, OPS-0180 · labels: test · ref: KIP-618
Done when: `tests/compat/ecosystem/connect-eos/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool connect-eos` passes: with `exactly.once.source.support=enabled`, source records and offsets commit atomically, zombie tasks are fenced through their transactional ids, and a worker crash mid-transaction produces no duplicates for `read_committed` consumers; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0182 Kafka Streams with the classic group protocol and at-least-once processing
P1 · M6 · Task · deps: OPS-0160, GRP-F04, STOR-F14, TXN-F01 · labels: test
Done when: `tests/compat/ecosystem/streams-alos/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool streams-alos` passes: WordCount and a windowed stream-table join create repartition and compacted changelog topics, standby tasks take over when an instance is killed, state restores from changelogs, and `kafka-streams-application-reset.sh` resets the app; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0183 Kafka Streams `exactly_once_v2` through a broker rolling restart
P1 · M6 · Task · deps: OPS-0160, TXN-F10, TXN-F15, OPS-0182 · labels: test
Done when: `tests/compat/ecosystem/streams-eos/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool streams-eos` passes: an EOS app with state stores and standbys keeps running while every broker restarts in turn; its output equals an offline reference computation with no duplicates under `read_committed`, and no transaction stays open past its timeout; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0184 MirrorMaker 2 between Alluvium clusters and from Alluvium to Kafka
P1 · M6 · Task · deps: OPS-0160, GRP-F04, GRP-F07, STOR-F14, TXN-F01 · labels: test · ref: connect/mirror/src/main/java/org/apache/kafka/connect/mirror/MirrorMaker.java @4.4.0
Done when: `tests/compat/ecosystem/mm2/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool mm2` passes: dedicated-mode `connect-mirror-maker.sh` replicates Alluvium to Alluvium with `DefaultReplicationPolicy`, heartbeats, checkpoints, group offset sync and topic config sync, and Alluvium to Kafka 4.4.0; record counts and checksums match and MirrorMaker metrics such as `replication-latency-ms` are reported; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0185 Debezium PostgreSQL connector on Kafka Connect
P2 · M6 · Task · deps: OPS-0160, OPS-0180 · labels: test
Done when: `tests/compat/ecosystem/debezium/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool debezium` passes: initial snapshot and streaming of inserts, updates and deletes (with tombstones) into Alluvium topics created via Connect `topic.creation.*`, signalling through the signal table, and resumption from the stored LSN after a worker restart; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0186 Apache Flink Kafka connector: KafkaSource and exactly-once KafkaSink
P1 · M6 · Task · deps: OPS-0160, GRP-F07, TXN-F08, TXN-F09 · labels: test
Done when: `tests/compat/ecosystem/flink/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool flink` passes: KafkaSource with timestamp-based start offsets commits offsets on checkpoints; KafkaSink with `DeliveryGuarantee.EXACTLY_ONCE` and a `transactionalIdPrefix` survives a TaskManager crash with no duplicates under `read_committed` and lingering transactions aborted on recovery; a `transaction.timeout.ms` above the broker's `transaction.max.timeout.ms` fails with INVALID_TRANSACTION_TIMEOUT as with Kafka; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0187 Spark Structured Streaming Kafka source and sink
P2 · M6 · Task · deps: OPS-0160, API-F02, GRP-F07, TXN-F01 · labels: test
Done when: `tests/compat/ecosystem/spark/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool spark` passes: `readStream.format("kafka")` with `startingOffsetsByTimestamp` and `maxOffsetsPerTrigger` recovers from its checkpoint, the Kafka sink writes results, a batch read with `endingOffsets` returns the exact range, and `failOnDataLoss` reacts to retention-deleted offsets as it does with Kafka; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0188 ksqlDB server
P2 · M6 · Task · deps: OPS-0160, TXN-F08, OPS-0182, OPS-0189 · labels: test
Done when: `tests/compat/ecosystem/ksqldb/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool ksqldb` passes: the transactional command topic is created and replayed on restart, `CREATE STREAM`, `CREATE TABLE`, push and pull queries work, a persistent query runs with `exactly_once_v2`, and `SHOW TOPICS` matches the cluster; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0189 Confluent Schema Registry using Alluvium as its store
P1 · M6 · Task · deps: OPS-0160, GRP-F04, STOR-F14, TXN-F01 · labels: test
Done when: `tests/compat/ecosystem/schema-registry/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool schema-registry` passes: the `_schemas` topic is created compacted with one partition, two registry instances elect a leader through the classic group protocol and fail over when the leader stops, and schemas survive a restart by replaying the topic; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0190 Confluent serializers: Avro, Protobuf and JSON Schema
P1 · M6 · Task · deps: OPS-0160, OPS-0189 · labels: test
Done when: `tests/compat/ecosystem/schema-registry-serdes/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool schema-registry-serdes` passes: KafkaAvroSerializer, KafkaProtobufSerializer and KafkaJsonSchemaSerializer round-trip records with schema ids in the wire header, and a BACKWARD-incompatible schema is rejected at produce time; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0191 Kafka UI (kafbat): browse, administer topics, groups and ACLs
P2 · M7 · Task · deps: OPS-0160, GRP-F08, SEC-F09, API-F35, API-F44 · labels: test
Done when: `tests/compat/ecosystem/kafka-ui/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool kafka-ui` passes: brokers and log-dir sizes, topic create, config edit and delete, message browsing with filters, consumer group describe and offset reset, and the ACL pages, driven through its REST API with no errors in its logs; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0192 AKHQ: browse, administer topics, groups and ACLs
P2 · M7 · Task · deps: OPS-0160, GRP-F08, SEC-F09, API-F35, API-F44 · labels: test
Done when: `tests/compat/ecosystem/akhq/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool akhq` passes: the same topic, message, consumer group, ACL and node pages as the Kafka UI scenario; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0193 Redpanda Console against Alluvium
P2 · M7 · Task · deps: OPS-0160, GRP-F08, SEC-F09, API-F35, API-F44, API-F48 · labels: test
Done when: `tests/compat/ecosystem/redpanda-console/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool redpanda-console` passes: overview, topics with configs and messages, consumer groups with offset edits, ACLs and quotas pages load and act correctly; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0194 Conduktor Console (free tier) against Alluvium
P3 · M7 · Task · deps: OPS-0160, GRP-F08, SEC-F09, API-F35, API-F44 · labels: test
Done when: `tests/compat/ecosystem/conduktor/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool conduktor` passes: cluster connection, topic and consumer group pages, message browsing and topic creation; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0195 Cruise Control with the Prometheus metric sampler
P2 · M7 · Task · deps: OPS-0160, API-F34, API-F35, API-F43, API-F45, API-F46, OBS-F05 · labels: test
Done when: `tests/compat/ecosystem/cruise-control/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool cruise-control` passes: because Cruise Control's broker-side metrics reporter can't run in Alluvium, `PrometheusMetricSampler` uses a committed query mapping in `deploy/cruise-control/`; proposals are generated and executed through AlterPartitionReassignments with throttles, ElectLeaders and AlterReplicaLogDirs, ending with the goals met; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0196 Evaluate compatibility with the Strimzi operator
P2 · M7 · Spike · deps: none
Done when: ADR "Strimzi compatibility" records that Strimzi's cluster operator generates Kafka-specific images and configuration and can't run Alluvium nodes, evaluates its standalone Topic and User Operators against an Alluvium cluster, and recommends the path (Helm chart now, X-AUTO operator later).

### [To Do] OPS-0197 Strimzi standalone Topic and User Operators against Alluvium
P3 · M7 · Task · deps: OPS-0160, SEC-F09, API-F51, OPS-0196 · labels: test
Done when: `tests/compat/ecosystem/strimzi-operators/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool strimzi-operators` passes: KafkaTopic resources reconcile creation, partition increases and config changes; KafkaUser resources create SCRAM users and ACLs; out-of-band changes are detected; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0198 Burrow lag evaluation from `__consumer_offsets`
P2 · M5 · Task · deps: OPS-0160, GRP-F02, GRP-F07 · labels: test
Done when: `tests/compat/ecosystem/burrow/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool burrow` passes: Burrow decodes Alluvium's `__consumer_offsets` offset-commit and group-metadata records, proving the internal topic's byte format, and its OK, WARN and STALL evaluations match the same scenario against Kafka 4.4.0; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0199 Kafka Exporter (danielqsj/kafka_exporter)
P1 · M5 · Task · deps: OPS-0160, GRP-F07, GRP-F08 · labels: test
Done when: `tests/compat/ecosystem/kafka-exporter/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool kafka-exporter` passes: `kafka_brokers`, `kafka_topic_partition_current_offset`, `kafka_topic_partition_oldest_offset` and `kafka_consumergroup_lag` equal the values exported for the same scenario against Kafka 4.4.0; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0200 Vector kafka source and sink
P2 · M5 · Task · deps: OPS-0160, GRP-F04 · labels: test
Done when: `tests/compat/ecosystem/vector/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool vector` passes: the sink produces with compression and `acks=all`, the source consumes in a group with end-to-end acknowledgements, and event counts match; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0201 Fluent Bit `out_kafka` and `in_kafka`
P2 · M5 · Task · deps: OPS-0160, GRP-F04 · labels: test
Done when: `tests/compat/ecosystem/fluent-bit/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool fluent-bit` passes: records written by `out_kafka` are read back by `in_kafka` in a group with matching counts and content; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0202 Logstash kafka input and output plugins
P2 · M6 · Task · deps: OPS-0160, GRP-F04, TXN-F01 · labels: test
Done when: `tests/compat/ecosystem/logstash/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool logstash` passes: the output plugin with its default producer settings and the input plugin with a group and decorated events move a fixed dataset with matching counts; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0203 Telegraf `kafka_consumer` input and `kafka` output
P3 · M5 · Task · deps: OPS-0160, GRP-F04 · labels: test
Done when: `tests/compat/ecosystem/telegraf/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool telegraf` passes: metrics written by the output are consumed by the input in a group with matching counts; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

### [To Do] OPS-0204 OpenTelemetry Collector `kafkaexporter` and `kafkareceiver`
P2 · M5 · Task · deps: OPS-0160, GRP-F04 · labels: test
Done when: `tests/compat/ecosystem/otel-collector/` pins the tool by version and image digest and `cargo xtask compat --suite ecosystem --tool otel-collector` passes: traces, metrics and logs exported with `otlp_proto` encoding are received by a second collector with identical counts, including partitioning by trace id; the result is recorded in the ecosystem matrix and every gap is filed as a Bug under the owning feature.

## [To Do] OPS-F13 Log output management
P2 · M2 · Feature · deps: OBS-F01
Scope: stdout and file output, rotation guidance, log shipping examples.

### [To Do] OPS-0205 JSON log line schema and stability rules
P2 · M2 · Task · deps: none · labels: docs
Done when: docs/operations/logging.md documents the JSON line keys (RFC 3339 UTC `timestamp`, `level`, `target`, `message`, event fields, span fields with the standard names, `spans`), the stability rule (new fields are minor, renames are breaking), and examples; a test validates lines from a started node against `docs/operations/log-line.schema.json`.

### [To Do] OPS-0206 Non-blocking bounded log writer with drop accounting
P2 · M2 · Task · deps: none · labels: nfr
Done when: log output goes through a bounded queue drained by a dedicated thread, so a blocked stdout never stalls Tokio workers; when the queue is full lines are dropped and counted in `alluvium_log_lines_dropped_total{level}`, and a `log_lines_dropped` warning summarises drops at most every 10 s; a test with a blocked stdout pipe shows request latency unaffected.

### [To Do] OPS-0207 File output with reopen on SIGHUP
P2 · M2 · Task · deps: OPS-0206, CFG-F11
Done when: when the file-output key registered by CFG-F11 is set, logs go to that file (mode 0640) instead of stdout; SIGHUP reopens the path so `logrotate` works without `copytruncate`; a test renames the file mid-stream and finds no line lost or duplicated.

### [To Do] OPS-0208 Rotation and retention guidance per environment
P2 · M2 · Task · deps: OPS-0207 · labels: docs
Done when: docs/operations/logging.md gives a `/etc/logrotate.d/alluvium` sample with a HUP `postrotate`, journald `SystemMaxUse` and rate-limit settings, kubelet `containerLogMaxSize` and `containerLogMaxFiles`, and Docker `json-file` `max-size`; CI dry-runs the logrotate sample with `logrotate -d`.

### [To Do] OPS-0209 Mapping from Kafka log4j loggers to Alluvium tracing targets
P3 · M2 · Task · deps: OBS-F02 · labels: docs · ref: config/log4j2.yaml @4.4.0
Done when: a table maps `kafka.request.logger`, `kafka.authorizer.logger`, `state.change.logger`, `org.apache.kafka.controller` and the log cleaner loggers to `ALLUVIUM_LOG` filter strings; a test checks each documented filter emits the named target's events.

### [To Do] OPS-0210 Log shipping example: Vector to Loki
P2 · M2 · Task · deps: OPS-0205
Done when: `deploy/logging/vector/vector.toml` parses JSON lines, maps levels, uses only bounded fields (`node_id`, `level`, `target`) as Loki labels and keeps the rest as structured fields; `vector test` unit tests run on sample lines in CI.

### [To Do] OPS-0211 Log shipping example: Fluent Bit DaemonSet on Kubernetes
P2 · M2 · Task · deps: OPS-0205
Done when: `deploy/logging/fluent-bit/` has a parser and `kubernetes` filter config for container stdout; CI runs Fluent Bit on sample lines and checks the parsed fields.

### [To Do] OPS-0212 Log shipping example: OpenTelemetry Collector filelog receiver
P3 · M2 · Task · deps: OPS-0205
Done when: `deploy/logging/otel/collector.yaml` uses the filelog receiver with a JSON parser and severity mapping; CI runs the collector on sample lines and checks the resulting log records.

### [To Do] OPS-0213 Log volume budget test
P2 · M2 · Task · deps: OPS-0205, NFR-F01 · labels: nfr, test
Done when: a test runs the NFR-F01 standard workload for a fixed number of requests and fails if any `info`-or-higher line is emitted per request or record, and if an idle node logs more than the documented number of `info` lines per minute, naming the offending events.
