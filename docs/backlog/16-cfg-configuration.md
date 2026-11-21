# CFG: Configuration

> Crates: `alluvium-config` (layer 3) · Kafka counterpart: `ConfigDef`, `KafkaConfig`, `LogConfig`, `DynamicBrokerConfig`, `*Configs.java` · Milestones: M2-M9

Typed, validated configuration using Kafka's key names, defaults and semantics, with source tracking, dynamic updates, and one task per Kafka config key.

## [To Do] CFG-F01 Config framework
P0 · M2 · Feature · deps: TYPE-F08
Scope: ConfigDef equivalent: typed keys (boolean, string, int, short, long, double, list, class, password), validators, importance, docs, defaults, sources.

### [To Do] CFG-0001 ADR: configuration model with a static key registry, Kafka key names, per-crate plain config structs and value sources
P0 · M2 · Spike · deps: none · labels: docs · ref: server/src/main/java/org/apache/kafka/server/config/AbstractKafkaConfig.java CONFIG_DEF @4.4.0
Done when: docs/adr/NNNN-configuration-model.md is accepted: registry layout (one table per Kafka *Configs class), mapping onto layer 0-2 structs (architecture §5 rule 3), validation phases (type, validator, cross-key), source precedence, dynamic update flow, and rejected alternatives (serde-derived structs, per-crate parsing) with reasons.

### [To Do] CFG-0002 ADR: Kafka config values that name Java classes, JKS keystores, JCA algorithms or JAAS modules
P0 · M2 · Spike · deps: CFG-0001 · labels: docs, security
Done when: The ADR lists every such Kafka 4.4.0 key (authorizer.class.name, principal.builder.class, the sasl.*.callback.handler.class and sasl.login.class keys, ssl.engine.factory.class, metric.reporters, kafka.metrics.reporters, client.quota.callback.class, the policy classes, replica.selector.class, remote.log.*.class.name/class.path, sasl.oauthbearer.jwt.*.class, group.streams.topology.description.plugin.class, ssl.keystore.type, ssl.truststore.type, ssl.provider, security.providers, ssl.keymanager.algorithm, ssl.trustmanager.algorithm, ssl.secure.random.implementation, sasl.jaas.config) with accepted values, the built-in each Kafka default class maps to, and the startup error for anything else; parity.md gains the deviation rows.

### [To Do] CFG-0003 Create `alluvium-config` with value types BOOLEAN, STRING, INT, SHORT, LONG, DOUBLE, LIST, CLASS and PASSWORD parsed like ConfigDef.parseType
P0 · M2 · Task · deps: CFG-0001 · ref: clients/src/main/java/org/apache/kafka/common/config/ConfigDef.java parseType @4.4.0
Done when: The crate has a `//!` doc (responsibility, layer 3, Kafka counterpart, invariants) and is registered in the xtask layer table; parsing matches Kafka for trimming, case-insensitive booleans, comma lists with whitespace, 16/32/64-bit overflow and doubles, with Kafka's error wording; tests ported from ConfigDefTest name their source.

### [To Do] CFG-0004 PASSWORD values held as `Secret<String>` and rendered `[hidden]` like Kafka's Password
P0 · M2 · Task · deps: CFG-0003 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/types/Password.java toString @4.4.0
Done when: Debug, Display and error rendering of any PASSWORD value print `[hidden]`; a test formats an effective config holding ssl.keystore.password and sasl.jaas.config and asserts the secrets are absent; the type has no Serialize impl.

### [To Do] CFG-0005 Validators: Range, ValidString, CaseInsensitiveValidString, ValidList, NonEmptyString, NonNullValidator, ThrottledReplicaListValidator and the quorum list validators
P0 · M2 · Task · deps: CFG-0003 · ref: clients/src/main/java/org/apache/kafka/common/config/ConfigDef.java Validator @4.4.0
Done when: A `Validator` enum (no trait objects) with one unit test per variant reproducing Kafka's message (`Invalid value <v> for configuration <key>: <reason>`), including ValidList's empty/null/duplicate rules; `describe()` returns Kafka's validator text, used by docs and the golden test.

### [To Do] CFG-0006 Key registry: name, type, default, validator, importance, documentation, group, dynamic mode and internal flag in one name-sorted table merged from per-area tables
P0 · M2 · Task · deps: CFG-0005, CFG-0004 · ref: clients/src/main/java/org/apache/kafka/common/config/ConfigDef.java define @4.4.0
Done when: Lookup by name; merging per-area tables rejects duplicate names like Utils.mergeConfigs (test); every default passes its own validator (ConfigDef.define does the same); internal keys are excluded from docs and unknown-key warnings.

### [To Do] CFG-0007 Typed key handles `Key<T>` so reading a key with the wrong type does not compile
P0 · M2 · Task · deps: CFG-0006
Done when: Handles are generated for every registry key; `EffectiveConfig::get(Key<T>) -> T`; a trybuild test shows a type mismatch failing to compile; an xtask check rejects string-keyed lookups outside alluvium-config.

### [To Do] CFG-0008 Dynamic update mode per key: read-only, per-broker or cluster-wide
P0 · M2 · Task · deps: CFG-0006 · ref: server/src/main/java/org/apache/kafka/server/config/DynamicBrokerConfig.java dynamicConfigUpdateModes @4.4.0
Done when: The registry carries the mode of DynamicBrokerConfig.dynamicConfigUpdateModes and flags keys accepted per listener (DynamicListenerConfig.RECONFIGURABLE_CONFIGS, LISTENER_MECHANISM_CONFIGS); a golden test asserts the 110 dynamic broker keys and their modes equal Kafka 4.4.0.

### [To Do] CFG-0009 Listener-prefixed keys: `listener.name.<listener>.<key>` and `listener.name.<listener>.<mechanism>.<key>` resolution
P0 · M2 · Task · deps: CFG-0008 · ref: server/src/main/java/org/apache/kafka/server/config/DynamicBrokerConfig.java brokerConfigSynonyms @4.4.0
Done when: A listener lookup returns the prefixed value and falls back to the unprefixed key; mechanism-scoped keys (sasl.jaas.config, sasl.server.callback.handler.class, sasl.login.class, sasl.login.callback.handler.class, connections.max.reauth.ms) resolve per mechanism; listener names are lower-cased as in ListenerName.configPrefix; tests per rule.

### [To Do] CFG-0010 Golden parity test: registry equals a dump of Kafka 4.4.0 ConfigDefs
P0 · M2 · Task · deps: CFG-0008 · labels: test · ref: server/src/main/java/org/apache/kafka/server/config/AbstractKafkaConfig.java CONFIG_DEF @4.4.0
Done when: A documented xtask script runs the Kafka 4.4.0 jars and writes a vendored JSON of AbstractKafkaConfig.CONFIG_DEF plus QuotaConfig.brokerQuotaConfigs (broker), LogConfig.configDefCopy (topic) and GroupConfig.CONFIG_DEF (group); the test compares name, type, default, validator text, importance, internal flag and dynamic mode for every key; a difference needs an allow-list entry linked to an ADR.

### [To Do] CFG-0011 `ConfigError` with key, redacted value and reason, mapped to INVALID_CONFIG or INVALID_REQUEST in one exhaustive match
P0 · M2 · Task · deps: CFG-0005, ERR-F01 · ref: clients/src/main/java/org/apache/kafka/common/config/ConfigException.java @4.4.0
Done when: Variants UnknownKey, InvalidValue, Missing, CrossKey and Provider carry the key and reason; PASSWORD values never appear; `From<&ConfigError> for ErrorCode` has no wildcard arm; unit tests per variant.

### [To Do] CFG-0012 Unknown keys warn once (`config.unknown_key` event) without failing startup, like AbstractConfig.logUnused
P1 · M2 · Task · deps: CFG-0006 · labels: observability · ref: clients/src/main/java/org/apache/kafka/common/config/AbstractConfig.java logUnused @4.4.0
Done when: Each unknown key is reported once with its source; prefixed families (listener.name.*, config.providers.*, the rsm.config./rlmm.config. prefixes) are not reported; test.

### [To Do] CFG-0013 Fuzz target: parsing and validating untrusted config values from CreateTopics, AlterConfigs and IncrementalAlterConfigs
P0 · M2 · Task · deps: CFG-0006, TEST-F07 · labels: security, test, decoder
Done when: A cargo-fuzz target feeds (key, value bytes) for every registry type and validator: no panics, no allocation beyond the value length cap; the corpus is seeded with Kafka test values; it runs in the CI fuzz smoke job.

### [To Do] CFG-0014 Cross-key validation: node ids, process roles, listeners, advertised listeners and controller quorum (KafkaConfig.validateValues)
P0 · M2 · Task · deps: CFG-0006, CFG-0009 · ref: core/src/main/scala/kafka/server/KafkaConfig.scala validateValues @4.4.0
Done when: One test per Kafka rule with its message: node.id equals broker.id when both set; process.roles values; controller.quorum.voters and bootstrap.servers exclusivity; voters contain node.id for controllers; controller.listener.names present in listeners and absent from broker advertised listeners; advertised listeners non-empty, not 0.0.0.0 and a subset of listeners; inter.broker.listener.name advertised; early.start.listeners subset of listeners; all violations are reported together.

### [To Do] CFG-0015 Cross-key validation: log, network and replication rules (KafkaConfig.validateValues)
P0 · M2 · Task · deps: CFG-0006 · ref: core/src/main/scala/kafka/server/KafkaConfig.scala validateValues @4.4.0
Done when: Tests for log.roll.ms >= 1, log.roll.jitter.ms >= 0, log.retention.ms -1 or >= 1, at least one log directory, log.cleaner.dedupe.buffer.size above 1 MiB per cleaner thread, replica.fetch.wait.max.ms <= replica.socket.timeout.ms and <= replica.lag.time.max.ms, queued.max.request.bytes <= 0 or >= socket.request.max.bytes, max.connections.per.ip = 0 only with overrides, connection.failed.authentication.delay.ms < connections.max.idle.ms, cordoned.log.dirs rules, and the classic group protocol cannot be disabled.

### [To Do] CFG-0016 Cross-key validation: inter-broker and controller SASL mechanisms enabled on their listeners; supported principal builder
P0 · M4 · Task · deps: CFG-0014, CFG-0002 · labels: security · ref: core/src/main/scala/kafka/server/KafkaConfig.scala validateValues @4.4.0
Done when: Tests: sasl.mechanism.inter.broker.protocol must be in the inter-broker listener's sasl.enabled.mechanisms; sasl.mechanism.controller.protocol likewise for controller listeners; principal.builder.class must be an accepted value under the JVM-config ADR; Kafka's messages reproduced.

### [To Do] CFG-0017 Cross-key validation: consumer group session timeout, heartbeat and assignment interval bounds (GroupCoordinatorConfig)
P1 · M5 · Task · deps: CFG-0006 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig @4.4.0
Done when: Tests for min <= default <= max of group.consumer.session.timeout.ms, group.consumer.heartbeat.interval.ms and group.consumer.assignment.interval.ms, and heartbeat interval < session timeout, with Kafka's messages.

### [To Do] CFG-0018 Cross-key validation: share and streams group bounds (GroupCoordinatorConfig, ShareGroupConfig)
P1 · M8 · Task · deps: CFG-0017 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/share/ShareGroupConfig.java ShareGroupConfig @4.4.0
Done when: Tests for share and streams min <= default <= max of session timeout, heartbeat and assignment intervals, heartbeat < session timeout, exactly one share assignor, share delivery count, record lock duration and partition record-lock bounds, group.share.max.share.sessions >= group.share.max.size, streams standby replicas <= max, and unique rack-aware assignment tags.

### [To Do] CFG-0019 Cross-key validation: broker-level local retention against retention when remote storage is enabled
P1 · M9 · Task · deps: CFG-0006, CFG-0002 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogConfig.java validateBrokerLogConfigValues @4.4.0
Done when: With remote.log.storage.system.enable=true, tests reject log.local.retention.bytes/ms of -1 or above log.retention.bytes/ms (LogConfig.validateBrokerLogConfigValues) with Kafka's messages; remote.log.storage.system.enable without an accepted storage manager fails per the JVM-config ADR.

## [To Do] CFG-F02 Config loading
P0 · M2 · Feature · deps: CFG-F01
Scope: Kafka `server.properties` format, environment overrides, config providers (file, directory, env; KIP-297) and `${provider:path:key}` substitution.

### [To Do] CFG-0020 `server.properties` parser with java.util.Properties semantics
P0 · M2 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/common/utils/Utils.java loadProps @4.4.0
Done when: Handles `#`/`!` comments, `=`, `:` and whitespace separators, backslash continuation, `\uXXXX` and escape sequences, and last-wins duplicates; files above a documented size cap are rejected with a clear error; results equal java.util.Properties on a corpus of Kafka's config/*.properties plus edge-case fixtures produced by the golden script.

### [To Do] CFG-0021 Fuzz target for the properties parser
P1 · M2 · Task · deps: CFG-0020, TEST-F07 · labels: security, test, decoder
Done when: cargo-fuzz target over arbitrary bytes: no panics, memory bounded by the input size cap; seeded with the parser corpus; runs in the CI fuzz smoke job.

### [To Do] CFG-0022 Command-line `--override key=value` overrides like kafka.Kafka
P0 · M2 · Task · deps: CFG-0020 · ref: core/src/main/scala/kafka/Kafka.scala getPropsFromArgs @4.4.0
Done when: Overrides win over the file; a malformed pair fails with the offending argument; library API consumed by the `server` command; tests.

### [To Do] CFG-0023 Environment overrides with the Kafka container convention (`KAFKA_LOG_DIRS` → log.dirs; `_`→`.`, `__`→`_`, `___`→`-`) and `ALLUVIUM_` for alluvium.* keys
P1 · M2 · Task · deps: CFG-0020 · ref: core/src/main/scala/kafka/docker/KafkaDockerWrapper.scala @4.4.0
Done when: Mapping matches KafkaDockerWrapper including its excluded variables (KAFKA_OPTS, KAFKA_HEAP_OPTS, KAFKA_VERSION…); precedence is file < environment < --override; PASSWORD values from the environment stay secret; tests per rule.

### [To Do] CFG-0024 Config provider framework (KIP-297): `config.providers`, `config.providers.<name>.class` and `.param.*`
P0 · M2 · Task · deps: CFG-0020, CFG-0002 · ref: clients/src/main/java/org/apache/kafka/common/config/AbstractConfig.java instantiateConfigProviders @4.4.0
Done when: Kafka's FileConfigProvider, DirectoryConfigProvider and EnvVarConfigProvider class names resolve to built-ins and other classes fail per the JVM-config ADR (the equivalent of AUTOMATIC_CONFIG_PROVIDERS_PROPERTY restriction); providers are created once at load and closed at shutdown; tests.

### [To Do] CFG-0025 `${provider:[path:]key}` variable substitution (ConfigTransformer)
P0 · M2 · Task · deps: CFG-0024 · ref: clients/src/main/java/org/apache/kafka/common/config/ConfigTransformer.java transform @4.4.0
Done when: Matches ConfigTransformer.DEFAULT_PATTERN, including the optional path; an unknown provider or missing key leaves the reference unchanged as Kafka does; no recursive substitution; substituted values of PASSWORD keys stay secret and record the provider as their source; tests ported from ConfigTransformerTest.

### [To Do] CFG-0026 FileConfigProvider with `allowed.paths` (KIP-993)
P0 · M2 · Task · deps: CFG-0024 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/provider/FileConfigProvider.java @4.4.0
Done when: Reads keys from a properties file; paths are normalized before the allowed.paths check so `..` and symlinks cannot escape; errors name the path, never file contents; file size bounded; tests.

### [To Do] CFG-0027 DirectoryConfigProvider: one file per key with `allowed.paths`
P1 · M2 · Task · deps: CFG-0024 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/provider/DirectoryConfigProvider.java @4.4.0
Done when: Only regular files directly inside the directory are read; values are the whole file content as in Kafka; allowed.paths enforced after normalization; tests.

### [To Do] CFG-0028 EnvVarConfigProvider with `allowlist.pattern`
P1 · M2 · Task · deps: CFG-0024 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/provider/EnvVarConfigProvider.java @4.4.0
Done when: Only environment variables matching allowlist.pattern (default `.*`) are visible; tests.

### [To Do] CFG-0029 Load pipeline and startup failure report: file, environment, overrides, substitution, type and validator checks, cross-key rules
P0 · M2 · Task · deps: CFG-0022, CFG-0023, CFG-0025, CFG-0014, CFG-0015
Done when: Startup exits non-zero and lists every error once with key, source and reason (a fixture with three independent errors reports all three); secrets are redacted; the pipeline is a pure function from inputs to EffectiveConfig or errors.

### [To Do] CFG-0030 Kafka's shipped server.properties, broker.properties and controller.properties load and validate unchanged
P1 · M2 · Task · deps: CFG-0029 · labels: test · ref: config/server.properties @4.4.0
Done when: Test loads the three 4.4.0 files from the vendored tree and validates them without errors or unknown-key warnings.

### [To Do] CFG-0031 docs/config.md 'Loading configuration': precedence, environment mapping, providers and migration notes
P1 · M2 · Task · deps: CFG-0029 · labels: docs
Done when: The section documents precedence (default < file < environment < override < dynamic), the KAFKA_/ALLUVIUM_ mapping table, provider setup with allowed.paths examples, and differences from Kafka's startup scripts.

## [To Do] CFG-F03 Effective config and sources
P1 · M2 · Feature · deps: CFG-F02, OBS-F08
Scope: Effective value with source per key; redaction; `check-config` and `/debug/config` output.

### [To Do] CFG-0032 Value source model: DescribeConfigs ConfigSource values plus file:line, environment variable, override or provider detail
P1 · M2 · Task · deps: none · ref: clients/src/main/java/org/apache/kafka/clients/admin/ConfigEntry.java ConfigSource @4.4.0
Done when: Every loaded value records its source; the Kafka ConfigSource (DEFAULT_CONFIG, STATIC_BROKER_CONFIG, DYNAMIC_DEFAULT_BROKER_CONFIG, DYNAMIC_BROKER_CONFIG, DYNAMIC_BROKER_LOGGER_CONFIG, DYNAMIC_TOPIC_CONFIG, DYNAMIC_CLIENT_METRICS_CONFIG, DYNAMIC_GROUP_CONFIG) is derivable from it; tests per source.

### [To Do] CFG-0033 EffectiveConfig snapshot: immutable, generation-numbered and swapped atomically
P1 · M2 · Task · deps: CFG-0032, OBS-F04
Done when: Readers never observe values from two generations (property test over interleaved updates and reads); each key holds value, source and synonym chain; `alluvium_config_generation` gauge exported.

### [To Do] CFG-0034 Redaction of sensitive values in logs, events, /debug/config, check-config output and errors
P0 · M2 · Task · deps: CFG-0032 · labels: security
Done when: PASSWORD keys, listener-prefixed passwords and provider-resolved sensitive values render as `[redacted]`; a test sets canary secrets in every PASSWORD key and asserts that none appear in captured logs, events, /debug/config, check-config output or error messages.

### [To Do] CFG-0035 Startup `config.effective` event listing every non-default key with its source (AbstractConfig.logAll)
P1 · M2 · Task · deps: CFG-0033, CFG-0034 · labels: observability · ref: clients/src/main/java/org/apache/kafka/common/config/AbstractConfig.java logAll @4.4.0
Done when: One structured event at startup, redacted, sorted by key; snapshot test of the event fields.

### [To Do] CFG-0036 `/debug/config` endpoint with effective values, sources, synonyms, dynamic mode and generation
P1 · M2 · Task · deps: CFG-0033, CFG-0034 · labels: observability
Done when: JSON schema documented; served on the diagnostics endpoint with its access rules; redacted; test.

### [To Do] CFG-0037 `check_config()` library API behind `alluvium check-config`
P1 · M2 · Task · deps: CFG-0033, CFG-0034, CFG-0029
Done when: Validates file, environment and overrides without starting; returns errors, warnings (unknown keys, lower-precedence synonyms that are ignored) and effective values with sources; exit status rules documented for the CLI; fixture tests.

## [To Do] CFG-F04 Dynamic configuration
P0 · M5 · Feature · deps: CFG-F01, META-F02
Scope: Dynamic configs from metadata: per-broker and cluster-default broker configs, group-resource configs, reconfigurable components, validation, listener-scoped configs.

### [To Do] CFG-0038 Dynamic broker config layers from ConfigRecord replay: per-broker and cluster default
P0 · M7 · Task · deps: none · ref: core/src/main/scala/kafka/server/DynamicBrokerConfig.scala updateCurrentConfig @4.4.0
Done when: Precedence dynamic per-broker > dynamic default > static > built-in default; a null value deletes the layer entry and the next layer applies; tests per precedence step.

### [To Do] CFG-0039 Dynamic update validation: read-only keys, per-broker-only keys at cluster level, unprefixed security keys, types after provider resolution
P0 · M7 · Task · deps: CFG-0038 · ref: server/src/main/java/org/apache/kafka/server/config/DynamicBrokerConfig.java validateConfigs @4.4.0
Done when: Each rejection returns INVALID_CONFIG/INVALID_REQUEST with Kafka's message (for example `Cannot update these configs dynamically: [...]`); provider references are resolved before type checks (DynamicBrokerConfig.resolveVariableConfigs); tests per rule.

### [To Do] CFG-0040 Dynamic listener-scoped updates and cluster-level listener keys
P0 · M7 · Task · deps: CFG-0039 · ref: server/src/main/java/org/apache/kafka/server/config/DynamicBrokerConfig.java perBrokerConfigs @4.4.0
Done when: `listener.name.<l>.*` updates affect only listener l; max.connections, max.connection.creation.rate and num.network.threads are accepted at the default level; other listener keys require a broker id; tests.

### [To Do] CFG-0041 Reconfiguration contract: declare keys, validate the new snapshot, then apply, all-or-nothing across components
P0 · M7 · Task · deps: CFG-0039 · ref: server/src/main/java/org/apache/kafka/server/config/BrokerReconfigurable.java @4.4.0
Done when: A validation failure in any component leaves every component on the previous generation; property test with randomly failing components; module docs describe the two-phase flow.

### [To Do] CFG-0042 Apply dynamic broker configs from the latest metadata snapshot before components start
P0 · M7 · Task · deps: CFG-0038 · ref: core/src/main/scala/kafka/server/DynamicBrokerConfig.scala readDynamicBrokerConfigsFromSnapshot @4.4.0
Done when: On restart the broker starts with dynamic values already in effect (no window with static values); test restarts a broker with a dynamic log.retention.ms.

### [To Do] CFG-0043 Dynamic GROUP resource configs: ConfigRecord(GROUP) replay into per-group overrides over broker group.* defaults, clamped to broker min/max
P0 · M5 · Task · deps: none · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfigManager.java @4.4.0
Done when: Per-group values override the broker defaults; deleting an override reverts to the default; values outside the broker min/max bounds are clamped as GroupCoordinatorConfig.clampDynamicConfigs does and rejected at alter time with INVALID_CONFIG as GroupConfig.validate does; a typed per-group view is published to subscribers on every change; tests.

### [To Do] CFG-0044 Reconfiguration key set and rules for thread pools: num.io.threads, num.replica.fetchers, num.recovery.threads.per.data.dir, background.threads, num.network.threads
P0 · M7 · Task · deps: CFG-0041 · ref: server/src/main/java/org/apache/kafka/server/DynamicThreadPool.java validateReconfiguration @4.4.0
Done when: Changes are validated with Kafka's rules (at least 1, at least half and at most double the current size, DynamicThreadPool.validateReconfiguration) and delivered as one typed notification to the registered thread-pool handle; tests with a recording handle; resizing the pools themselves belongs to the owning server features.

### [To Do] CFG-0045 Reconfiguration of broker log defaults: every topic-synonym log.* key and cordoned.log.dirs
P0 · M7 · Task · deps: CFG-0041, CFG-0061 · ref: server/src/main/java/org/apache/kafka/server/config/DynamicBrokerConfig.java DynamicLogConfig @4.4.0
Done when: A change recomputes the effective topic config of every topic without an override for that key and notifies the log owner with the affected topics; topics with overrides are untouched; tests with log.retention.ms and log.segment.bytes.

### [To Do] CFG-0046 Reconfiguration rules for listeners and listener.security.protocol.map
P0 · M7 · Task · deps: CFG-0040, CFG-0041 · ref: core/src/main/scala/kafka/server/DynamicBrokerConfig.scala DynamicListenerConfig @4.4.0
Done when: Added, removed and changed listeners are computed and delivered to the network handle; removing the inter-broker or controller listener and other changes DynamicListenerConfig.validateReconfiguration forbids are rejected with Kafka's messages; tests with a recording handle.

### [To Do] CFG-0047 Reconfiguration of per-listener TLS settings, including same-path keystore reload and DN/SAN change checks
P0 · M7 · Task · deps: CFG-0040, CFG-0041 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/ssl/SslFactory.java reconfigure @4.4.0
Done when: An update to ssl.* keys (or an unchanged ssl.keystore.location / ssl.truststore.location, which forces a reload) produces a new TLS settings value for that listener; certificates whose DN or SANs differ are rejected unless ssl.allow.dn.changes / ssl.allow.san.changes allow it (SslFactory CertificateEntries.ensureCompatible); tests.

### [To Do] CFG-0048 Reconfiguration of per-listener and per-mechanism SASL settings
P1 · M7 · Task · deps: CFG-0040, CFG-0041 · labels: security · ref: server/src/main/java/org/apache/kafka/server/config/DynamicBrokerConfig.java DynamicListenerConfig @4.4.0
Done when: Updates to sasl.enabled.mechanisms, per-mechanism sasl.jaas.config and OAUTHBEARER settings are validated and delivered per listener; the notification states they apply to new connections only; tests.

### [To Do] CFG-0049 Reconfiguration of connection limits: max.connections, max.connections.per.ip, max.connections.per.ip.overrides, max.connection.creation.rate
P0 · M7 · Task · deps: CFG-0041 · ref: server/src/main/java/org/apache/kafka/network/SocketServer.java RECONFIGURABLE_CONFIGS @4.4.0
Done when: Broker-level and listener-level values are validated (overrides syntax `host:limit`) and delivered as one typed limits value to the network handle; tests.

### [To Do] CFG-0050 Reconfiguration of the log cleaner key set (LogCleaner.RECONFIGURABLE_CONFIGS)
P1 · M7 · Task · deps: CFG-0041 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogCleaner.java RECONFIGURABLE_CONFIGS @4.4.0
Done when: log.cleaner.threads, dedupe buffer size and load factor, I/O buffer size, I/O rate, backoff and message.max.bytes are validated together (dedupe buffer per thread rule) and delivered as one cleaner settings value; tests.

### [To Do] CFG-0051 Reconfiguration of producer state settings: producer.id.expiration.ms and transaction.partition.verification.enable
P1 · M7 · Task · deps: CFG-0041 · ref: server/src/main/java/org/apache/kafka/server/config/DynamicBrokerConfig.java DYNAMIC_PRODUCER_STATE_MANAGER_CONFIGS @4.4.0
Done when: Validated values are delivered to the producer-state handle; tests.

### [To Do] CFG-0052 Reconfiguration of broker throttle rates: leader/follower.replication.throttled.rate and replica.alter.log.dirs.io.max.bytes.per.second
P1 · M7 · Task · deps: CFG-0041 · ref: server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java brokerQuotaConfigs @4.4.0
Done when: Dynamic-only quota keys (not in the static ConfigDef) are accepted at broker and default level and delivered to the quota handle; tests.

### [To Do] CFG-0053 Reconfiguration of group coordinator broker keys (GroupCoordinatorConfig.RECONFIGURABLE_CONFIGS)
P1 · M7 · Task · deps: CFG-0041 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java RECONFIGURABLE_CONFIGS @4.4.0
Done when: group.coordinator.cached.buffer.max.bytes, group.consumer.assignment.interval.ms and errors.deadletterqueue.* are validated against their min/max keys and delivered to the group coordinator handle; tests.

### [To Do] CFG-0054 Reconfiguration of share and streams group settings: assignment intervals and share.coordinator.cached.buffer.max.bytes
P1 · M8 · Task · deps: CFG-0041 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorConfig.java RECONFIGURABLE_CONFIGS @4.4.0
Done when: group.share.assignment.interval.ms, group.streams.assignment.interval.ms and share.coordinator.cached.buffer.max.bytes are validated and delivered to their coordinator handles; tests.

### [To Do] CFG-0055 Reconfiguration of tiered storage settings (DynamicRemoteLogConfig, DynamicReplicationConfig)
P1 · M9 · Task · deps: CFG-0041 · ref: server/src/main/java/org/apache/kafka/server/config/DynamicBrokerConfig.java DynamicRemoteLogConfig @4.4.0
Done when: remote.log.index.file.cache.total.size.bytes, remote.fetch.max.wait.ms, copy/fetch byte rates, remote.list.offsets.request.timeout.ms, the copier/expiration/follower pool sizes, remote.log.reader.threads and follower.fetch.last.tiered.offset.enable are validated and delivered to the tiered storage handle; tests.

### [To Do] CFG-0056 Dynamic config observability: `config.dynamic_update` event, flight-recorder entry and update counters
P1 · M7 · Task · deps: CFG-0041 · labels: observability
Done when: Each applied or rejected update emits an event with keys, redacted old/new values, source, generation and a `reason` enum, is recorded in the config flight recorder, and increments `alluvium_config_dynamic_updates_total{result}`; tests.

### [To Do] CFG-0057 Property test: random sequences of dynamic updates match a reference model
P1 · M7 · Task · deps: CFG-0041 · labels: test
Done when: Arbitrary valid and invalid updates and deletions at per-broker and default level never leave a partial apply, and the effective config equals the reference resolver; seed printed on failure.

### [To Do] CFG-0058 Compatibility test: kafka-configs.sh broker updates with --entity-name, --entity-default and --delete-config
P1 · M7 · Task · deps: CFG-0044, CFG-0045, TEST-F08 · labels: test
Done when: Java admin and kafka-configs.sh alter, describe and delete per-broker and default configs against Alluvium with the same results as Kafka 4.4.0.

### [To Do] CFG-0059 docs/config.md 'Dynamic configuration' section
P1 · M7 · Task · deps: CFG-0041 · labels: docs
Done when: Lists the dynamic keys by mode (generated), explains per-broker vs default vs listener scope with kafka-configs.sh examples, and states which changes affect only new connections.

## [To Do] CFG-F05 Topic config
P0 · M2 · Feature · deps: CFG-F01, META-F03
Scope: Per-topic overrides layered over broker `log.*` defaults, validation, application to logs.

### [To Do] CFG-0060 Topic-to-broker synonym table with unit conversion (ServerTopicConfigSynonyms)
P0 · M2 · Task · deps: none · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerTopicConfigSynonyms.java ALL_TOPIC_CONFIG_SYNONYMS @4.4.0
Done when: Every topic key maps to its broker synonyms in Kafka's order (for example retention.ms ← log.retention.ms, log.retention.minutes, log.retention.hours) with ms/minutes/hours conversion; golden test against Kafka's ALL_TOPIC_CONFIG_SYNONYMS.

### [To Do] CFG-0061 Per-topic overrides from ConfigRecord(TOPIC) replay layered over broker defaults
P0 · M2 · Task · deps: CFG-0060 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogConfig.java fromProps @4.4.0
Done when: Effective topic config = override, else broker dynamic/static synonym, else topic default; deleting an override reverts to the broker default; tests.

### [To Do] CFG-0062 Typed effective topic config: one typed field per topic key for building layer 0-2 config structs
P0 · M2 · Task · deps: CFG-0061
Done when: Every topic key maps onto exactly one typed field (test iterates the registry); unit conversions (ms, bytes) happen here so lower layers never parse config strings; conversion tests per key.

### [To Do] CFG-0063 Topic config validation: unknown names, per-key validators, min.compaction.lag.ms <= max.compaction.lag.ms
P0 · M2 · Task · deps: CFG-0061 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogConfig.java validate @4.4.0
Done when: Messages match LogConfig.validateNames/validateValues (`Unknown topic config name: x`, `conflict topic config setting ...`); returns ConfigError mapped to INVALID_CONFIG; tests.

### [To Do] CFG-0064 Controller-side validation entry for CreateTopics, AlterConfigs and IncrementalAlterConfigs on topics
P0 · M2 · Task · deps: CFG-0063 · ref: core/src/main/scala/kafka/server/ControllerConfigurationValidator.scala validate @4.4.0
Done when: Rejects null values and the default topic resource, validates names and values as ControllerConfigurationValidator does; used by topic creation and config records; tests per rule.

### [To Do] CFG-0065 Remote storage topic rules
P0 · M9 · Task · deps: CFG-0063 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/LogConfig.java validateTopicLogConfigValues @4.4.0
Done when: Tests: remote.storage.enable requires remote.log.storage.system.enable and delete-only cleanup; local.retention.* -1 or above retention.* rejected; remote.copy.lag.* vs local retention; disabling remote storage needs remote.log.delete.on.disable; retention rules when remote.log.copy.disable is set; Kafka's messages reproduced.

### [To Do] CFG-0066 Topic config change notifications on metadata publish
P0 · M2 · Task · deps: CFG-0062
Done when: Each metadata delta that changes a topic's overrides or a broker default produces one typed change notification per affected topic for the log owner; keys that apply lazily (segment.bytes, segment.ms at the next roll) are marked per key in the registry; tests with a recording subscriber.

### [To Do] CFG-0067 Topic config observability: `topic_config.changed` event, flight-recorder entry and counter
P1 · M2 · Task · deps: CFG-0066 · labels: observability
Done when: Event carries topic, topic id, changed keys, source and `reason`; recorded in the config flight recorder; `alluvium_topic_config_changes_total` counter; test.

### [To Do] CFG-0068 Property test: effective topic config equals a reference resolver for random overrides and broker defaults
P1 · M2 · Task · deps: CFG-0061 · labels: test
Done when: Random combinations of overrides, static and dynamic broker synonyms (including multi-unit synonyms) resolve identically; seed printed on failure.

### [To Do] CFG-0069 docs/config.md topic section: resolution order and the generated synonym table
P1 · M2 · Task · deps: CFG-0060 · labels: docs
Done when: The section explains override → broker synonym → default resolution and includes the synonym table generated from the registry.

## [To Do] CFG-F06 DescribeConfigs semantics
P0 · M7 · Feature · deps: CFG-F05
Scope: Synonym ordering, config source enums, documentation and type fields, read-only and sensitive flags.

### [To Do] CFG-0070 DescribeConfigs synonyms: Kafka's ordering and filtering per resource
P0 · M7 · Task · deps: none · ref: core/src/main/scala/kafka/server/ConfigHelper.scala createBrokerConfigEntry @4.4.0
Done when: include_synonyms returns topic override, dynamic per-broker, dynamic default, static and default entries in Kafka's order, multi-key broker synonyms (log.roll.ms/hours, log.retention.ms/minutes/hours, log.flush.interval.ms/log.flush.scheduler.interval.ms) and listener-prefixed synonyms; the broker default resource shows only DYNAMIC_DEFAULT_BROKER_CONFIG synonyms; tests.

### [To Do] CFG-0071 Config source per entry for BROKER (id and default), TOPIC, GROUP, CLIENT_METRICS and BROKER_LOGGER resources
P0 · M7 · Task · deps: CFG-0070 · ref: core/src/main/scala/kafka/server/ConfigHelper.scala describeConfigs @4.4.0
Done when: Each entry carries the correct ConfigSource; v0 is_default is derived as Kafka does; tests per resource type and version.

### [To Do] CFG-0072 Sensitive entries: null value and is_sensitive=true for PASSWORD and unknown-type keys
P0 · M7 · Task · deps: CFG-0070 · labels: security · ref: core/src/main/scala/kafka/server/KafkaConfig.scala maybeSensitive @4.4.0
Done when: PASSWORD keys, listener-prefixed passwords and keys of unknown type (KafkaConfig.maybeSensitive) never return values, even to cluster administrators, including inside synonyms; tests.

### [To Do] CFG-0073 read_only flag: true for non-dynamic broker keys, false for topic, group and client-metrics keys
P0 · M7 · Task · deps: CFG-0070 · ref: core/src/main/scala/kafka/server/ConfigHelper.scala createBrokerConfigEntry @4.4.0
Done when: Derived from the registry's dynamic mode; tests for one key of each mode and resource type.

### [To Do] CFG-0074 include_documentation and config_type fields
P0 · M7 · Task · deps: CFG-0070 · ref: core/src/main/scala/kafka/server/ConfigHelper.scala @4.4.0
Done when: Documentation comes from the registry; config_type maps BOOLEAN..PASSWORD and UNKNOWN for keys outside the registry; tests.

### [To Do] CFG-0075 configuration_keys filtering and resource errors
P0 · M7 · Task · deps: CFG-0070 · ref: core/src/main/scala/kafka/server/ConfigHelper.scala @4.4.0
Done when: A null key list returns all keys, a list returns only known requested keys; empty broker or group names, a broker id that is not this broker, and unknown resource types return Kafka's INVALID_REQUEST messages per resource; tests.

### [To Do] CFG-0076 Differential test: DescribeConfigs v1-v4 against Kafka 4.4.0 for broker, broker default, topic and group resources
P0 · M7 · Task · deps: CFG-0071, CFG-0072, CFG-0073, CFG-0074, CFG-0075, TEST-F09 · labels: test
Done when: Responses with and without synonyms and documentation match field by field after ignoring node-specific values; seeded with static, dynamic and override values.

### [To Do] CFG-0077 kafka-configs.sh --describe --all output matches Kafka for brokers, topics and groups
P1 · M7 · Task · deps: CFG-0076, TEST-F08 · labels: test
Done when: The tool output against Alluvium equals the output against Kafka 4.4.0 apart from node-specific values.

## [To Do] CFG-F07 Broker config keys
P1 · M2 · Feature · deps: CFG-F01
Scope: One task per broker/server config key: name, type, default, validation, dynamic mode, doc.

### [To Do] CFG-0078 advertised.listeners
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java SocketServerConfigs.ADVERTISED_LISTENERS_CONFIG @4.4.0
Done when: `advertised.listeners` is defined with type LIST, default null (unset), validation: no duplicate values, empty list rejected, null allowed, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0079 alter.config.policy.class.name
P1 · M7 · Task · deps: CFG-0495, CFG-0002 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.ALTER_CONFIG_POLICY_CLASS_NAME_CONFIG @4.4.0
Done when: `alter.config.policy.class.name` is defined with type CLASS, default null (unset), validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0080 alter.log.dirs.replication.quota.window.num
P1 · M7 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java QuotaConfig.NUM_ALTER_LOG_DIRS_REPLICATION_QUOTA_SAMPLES_CONFIG @4.4.0
Done when: `alter.log.dirs.replication.quota.window.num` is defined with type INT, default `11`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0081 alter.log.dirs.replication.quota.window.size.seconds
P1 · M7 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java QuotaConfig.ALTER_LOG_DIRS_REPLICATION_QUOTA_WINDOW_SIZE_SECONDS_CONFIG @4.4.0
Done when: `alter.log.dirs.replication.quota.window.size.seconds` is defined with type INT, default `1`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0082 authorizer.class.name
P1 · M4 · Task · deps: CFG-0495, CFG-0002 · labels: security · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.AUTHORIZER_CLASS_NAME_CONFIG @4.4.0
Done when: `authorizer.class.name` is defined with type STRING, default empty, validation: must not be null, importance LOW; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0083 auto.create.topics.enable
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.AUTO_CREATE_TOPICS_ENABLE_CONFIG @4.4.0
Done when: `auto.create.topics.enable` is defined with type BOOLEAN, default `true`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0084 auto.leader.rebalance.enable
P1 · M3 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.AUTO_LEADER_REBALANCE_ENABLE_CONFIG @4.4.0
Done when: `auto.leader.rebalance.enable` is defined with type BOOLEAN, default `true`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0085 background.threads
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.BACKGROUND_THREADS_CONFIG @4.4.0
Done when: `background.threads` is defined with type INT, default `10`, validation: >= 1, importance HIGH; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0086 broker.id
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.BROKER_ID_CONFIG @4.4.0
Done when: `broker.id` is defined with type INT, default `-1`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0087 broker.rack
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.BROKER_RACK_CONFIG @4.4.0
Done when: `broker.rack` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0088 client.quota.callback.class
P1 · M4 · Task · deps: CFG-0495, CFG-0002 · ref: server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java QuotaConfig.CLIENT_QUOTA_CALLBACK_CLASS_CONFIG @4.4.0
Done when: `client.quota.callback.class` is defined with type CLASS, default null (unset), validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0089 compression.gzip.level
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.COMPRESSION_GZIP_LEVEL_CONFIG @4.4.0
Done when: `compression.gzip.level` is defined with type INT, default `-1`, validation: -1 or 1 to 9 inclusive, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0090 compression.lz4.level
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.COMPRESSION_LZ4_LEVEL_CONFIG @4.4.0
Done when: `compression.lz4.level` is defined with type INT, default `9`, validation: 1 to 17 inclusive, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0091 compression.type
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.COMPRESSION_TYPE_CONFIG @4.4.0
Done when: `compression.type` is defined with type STRING, default `producer`, validation: one of uncompressed, zstd, lz4, snappy, gzip, producer, importance HIGH; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0092 compression.zstd.level
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.COMPRESSION_ZSTD_LEVEL_CONFIG @4.4.0
Done when: `compression.zstd.level` is defined with type INT, default `3`, validation: -131072 to 22 inclusive, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0093 config.providers
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/AbstractConfig.java AbstractConfig.CONFIG_PROVIDERS_CONFIG @4.4.0
Done when: `config.providers` is defined with type LIST, default empty, validation: no duplicate values, empty list allowed, null rejected, importance LOW; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0094 connection.failed.authentication.delay.ms
P1 · M4 · Task · deps: CFG-0495 · labels: security · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java SocketServerConfigs.FAILED_AUTHENTICATION_DELAY_MS_CONFIG @4.4.0
Done when: `connection.failed.authentication.delay.ms` is defined with type INT, default `100`, validation: >= 0, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0095 connections.max.idle.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java SocketServerConfigs.CONNECTIONS_MAX_IDLE_MS_CONFIG @4.4.0
Done when: `connections.max.idle.ms` is defined with type LONG, default `600000`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0096 connections.max.reauth.ms
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/internals/BrokerSecurityConfigs.java BrokerSecurityConfigs.CONNECTIONS_MAX_REAUTH_MS_CONFIG @4.4.0
Done when: `connections.max.reauth.ms` is defined with type LONG, default `0`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.<mechanism>.connections.max.reauth.ms`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0097 controlled.shutdown.enable
P1 · M3 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.CONTROLLED_SHUTDOWN_ENABLE_CONFIG @4.4.0
Done when: `controlled.shutdown.enable` is defined with type BOOLEAN, default `true`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0098 controller.quota.window.num
P1 · M4 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java QuotaConfig.NUM_CONTROLLER_QUOTA_SAMPLES_CONFIG @4.4.0
Done when: `controller.quota.window.num` is defined with type INT, default `11`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0099 controller.quota.window.size.seconds
P1 · M4 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java QuotaConfig.CONTROLLER_QUOTA_WINDOW_SIZE_SECONDS_CONFIG @4.4.0
Done when: `controller.quota.window.size.seconds` is defined with type INT, default `1`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0100 controller.socket.timeout.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.CONTROLLER_SOCKET_TIMEOUT_MS_CONFIG @4.4.0
Done when: `controller.socket.timeout.ms` is defined with type INT, default `30000`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0101 cordoned.log.dirs
P1 · M7 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.CORDONED_LOG_DIRS_CONFIG @4.4.0
Done when: `cordoned.log.dirs` is defined with type LIST, default empty, validation: no duplicate values, empty list allowed, null allowed, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0102 create.topic.policy.class.name
P1 · M2 · Task · deps: CFG-0495, CFG-0002 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.CREATE_TOPIC_POLICY_CLASS_NAME_CONFIG @4.4.0
Done when: `create.topic.policy.class.name` is defined with type CLASS, default null (unset), validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0103 default.replication.factor
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.DEFAULT_REPLICATION_FACTOR_CONFIG @4.4.0
Done when: `default.replication.factor` is defined with type INT, default `1`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0104 delegation.token.expiry.check.interval.ms
P1 · M4 · Task · deps: CFG-0495 · labels: security · ref: server-common/src/main/java/org/apache/kafka/server/config/DelegationTokenManagerConfigs.java DelegationTokenManagerConfigs.DELEGATION_TOKEN_EXPIRY_CHECK_INTERVAL_MS_CONFIG @4.4.0
Done when: `delegation.token.expiry.check.interval.ms` is defined with type LONG, default `3600000`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0105 delegation.token.expiry.time.ms
P1 · M4 · Task · deps: CFG-0495 · labels: security · ref: server-common/src/main/java/org/apache/kafka/server/config/DelegationTokenManagerConfigs.java DelegationTokenManagerConfigs.DELEGATION_TOKEN_EXPIRY_TIME_MS_CONFIG @4.4.0
Done when: `delegation.token.expiry.time.ms` is defined with type LONG, default `86400000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0106 delegation.token.max.lifetime.ms
P1 · M4 · Task · deps: CFG-0495 · labels: security · ref: server-common/src/main/java/org/apache/kafka/server/config/DelegationTokenManagerConfigs.java DelegationTokenManagerConfigs.DELEGATION_TOKEN_MAX_LIFETIME_CONFIG @4.4.0
Done when: `delegation.token.max.lifetime.ms` is defined with type LONG, default `604800000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0107 delegation.token.secret.key
P1 · M4 · Task · deps: CFG-0495, CFG-0004 · labels: security · ref: server-common/src/main/java/org/apache/kafka/server/config/DelegationTokenManagerConfigs.java DelegationTokenManagerConfigs.DELEGATION_TOKEN_SECRET_KEY_CONFIG @4.4.0
Done when: `delegation.token.secret.key` is defined with type PASSWORD, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); the value is held as a secret and reported as sensitive; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0108 delete.records.purgatory.purge.interval.requests
P1 · M7 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.DELETE_RECORDS_PURGATORY_PURGE_INTERVAL_REQUESTS_CONFIG @4.4.0
Done when: `delete.records.purgatory.purge.interval.requests` is defined with type INT, default `1`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0109 delete.topic.enable
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.DELETE_TOPIC_ENABLE_CONFIG @4.4.0
Done when: `delete.topic.enable` is defined with type BOOLEAN, default `true`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0110 early.start.listeners
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.EARLY_START_LISTENERS_CONFIG @4.4.0
Done when: `early.start.listeners` is defined with type LIST, default null (unset), validation: no duplicate values, empty list allowed, null allowed, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0111 fetch.max.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.FETCH_MAX_BYTES_CONFIG @4.4.0
Done when: `fetch.max.bytes` is defined with type INT, default `57671680`, validation: >= 1024, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0112 fetch.purgatory.purge.interval.requests
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.FETCH_PURGATORY_PURGE_INTERVAL_REQUESTS_CONFIG @4.4.0
Done when: `fetch.purgatory.purge.interval.requests` is defined with type INT, default `1000`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0113 follower.fetch.last.tiered.offset.enable
P1 · M9 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.FOLLOWER_FETCH_LAST_TIERED_OFFSET_ENABLE_CONFIG @4.4.0
Done when: `follower.fetch.last.tiered.offset.enable` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0114 follower.replication.throttled.rate
P1 · M4 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java QuotaConfig.FOLLOWER_REPLICATION_THROTTLED_RATE_CONFIG @4.4.0
Done when: `follower.replication.throttled.rate` is defined with type LONG, default `9223372036854775807` (Long.MAX_VALUE), validation: >= 0, importance MEDIUM; dynamic-only broker quota key, update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0115 inter.broker.listener.name
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.INTER_BROKER_LISTENER_NAME_CONFIG @4.4.0
Done when: `inter.broker.listener.name` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0116 kafka.metrics.polling.interval.secs
P3 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/metrics/MetricConfigs.java MetricConfigs.KAFKA_METRICS_POLLING_INTERVAL_SECONDS_CONFIG @4.4.0
Done when: `kafka.metrics.polling.interval.secs` is defined with type INT, default `10`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0117 kafka.metrics.reporters
P3 · M2 · Task · deps: CFG-0495, CFG-0002 · ref: server/src/main/java/org/apache/kafka/server/metrics/MetricConfigs.java MetricConfigs.KAFKA_METRICS_REPORTER_CLASSES_CONFIG @4.4.0
Done when: `kafka.metrics.reporters` is defined with type LIST, default empty, validation: no duplicate values, empty list allowed, null rejected, importance LOW; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0118 leader.imbalance.check.interval.seconds
P1 · M3 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.LEADER_IMBALANCE_CHECK_INTERVAL_SECONDS_CONFIG @4.4.0
Done when: `leader.imbalance.check.interval.seconds` is defined with type LONG, default `300`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0119 leader.replication.throttled.rate
P1 · M4 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java QuotaConfig.LEADER_REPLICATION_THROTTLED_RATE_CONFIG @4.4.0
Done when: `leader.replication.throttled.rate` is defined with type LONG, default `9223372036854775807` (Long.MAX_VALUE), validation: >= 0, importance MEDIUM; dynamic-only broker quota key, update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0120 listener.security.protocol.map
P1 · M2 · Task · deps: CFG-0495 · labels: security · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java SocketServerConfigs.LISTENER_SECURITY_PROTOCOL_MAP_CONFIG @4.4.0
Done when: `listener.security.protocol.map` is defined with type STRING, default `SASL_SSL:SASL_SSL,PLAINTEXT:PLAINTEXT,SSL:SSL,SASL_PLAINTEXT:SASL_PLAINTEXT`, validation: none beyond the type, importance LOW; dynamic update mode per-broker (dynamic for one broker id only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0121 listeners
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java SocketServerConfigs.LISTENERS_CONFIG @4.4.0
Done when: `listeners` is defined with type LIST, default `PLAINTEXT://:9092`, validation: no duplicate values, empty list rejected, null rejected, importance HIGH; dynamic update mode per-broker (dynamic for one broker id only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0122 log.cleaner.backoff.ms
P1 · M5 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/CleanerConfig.java CleanerConfig.LOG_CLEANER_BACKOFF_MS_PROP @4.4.0
Done when: `log.cleaner.backoff.ms` is defined with type LONG, default `15000`, validation: >= 0, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0123 log.cleaner.dedupe.buffer.size
P1 · M5 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/CleanerConfig.java CleanerConfig.LOG_CLEANER_DEDUPE_BUFFER_SIZE_PROP @4.4.0
Done when: `log.cleaner.dedupe.buffer.size` is defined with type LONG, default `134217728`, validation: none beyond the type, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0124 log.cleaner.delete.retention.ms
P1 · M5 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/CleanerConfig.java CleanerConfig.LOG_CLEANER_DELETE_RETENTION_MS_PROP @4.4.0
Done when: `log.cleaner.delete.retention.ms` is defined with type LONG, default `86400000`, validation: >= 0, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0125 log.cleaner.enable
P1 · M5 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/CleanerConfig.java CleanerConfig.LOG_CLEANER_ENABLE_PROP @4.4.0
Done when: `log.cleaner.enable` is defined with type BOOLEAN, default `true`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0126 log.cleaner.io.buffer.load.factor
P1 · M5 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/CleanerConfig.java CleanerConfig.LOG_CLEANER_DEDUPE_BUFFER_LOAD_FACTOR_PROP @4.4.0
Done when: `log.cleaner.io.buffer.load.factor` is defined with type DOUBLE, default `0.9`, validation: none beyond the type, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0127 log.cleaner.io.buffer.size
P1 · M5 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/CleanerConfig.java CleanerConfig.LOG_CLEANER_IO_BUFFER_SIZE_PROP @4.4.0
Done when: `log.cleaner.io.buffer.size` is defined with type INT, default `524288`, validation: >= 0, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0128 log.cleaner.io.max.bytes.per.second
P1 · M5 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/CleanerConfig.java CleanerConfig.LOG_CLEANER_IO_MAX_BYTES_PER_SECOND_PROP @4.4.0
Done when: `log.cleaner.io.max.bytes.per.second` is defined with type DOUBLE, default `1.7976931348623157E308` (Double.MAX_VALUE), validation: none beyond the type, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0129 log.cleaner.max.compaction.lag.ms
P1 · M5 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/CleanerConfig.java CleanerConfig.LOG_CLEANER_MAX_COMPACTION_LAG_MS_PROP @4.4.0
Done when: `log.cleaner.max.compaction.lag.ms` is defined with type LONG, default `9223372036854775807` (Long.MAX_VALUE), validation: >= 1, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0130 log.cleaner.min.cleanable.ratio
P1 · M5 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/CleanerConfig.java CleanerConfig.LOG_CLEANER_MIN_CLEAN_RATIO_PROP @4.4.0
Done when: `log.cleaner.min.cleanable.ratio` is defined with type DOUBLE, default `0.5`, validation: 0 to 1 inclusive, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0131 log.cleaner.min.compaction.lag.ms
P1 · M5 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/CleanerConfig.java CleanerConfig.LOG_CLEANER_MIN_COMPACTION_LAG_MS_PROP @4.4.0
Done when: `log.cleaner.min.compaction.lag.ms` is defined with type LONG, default `0`, validation: >= 0, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0132 log.cleaner.threads
P1 · M5 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/storage/internals/log/CleanerConfig.java CleanerConfig.LOG_CLEANER_THREADS_PROP @4.4.0
Done when: `log.cleaner.threads` is defined with type INT, default `1`, validation: >= 0, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0133 log.cleanup.policy
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_CLEANUP_POLICY_CONFIG @4.4.0
Done when: `log.cleanup.policy` is defined with type LIST, default `delete`, validation: each value one of [compact, delete], no duplicate values, empty list allowed, null rejected, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0134 log.dir
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_DIR_CONFIG @4.4.0
Done when: `log.dir` is defined with type LIST, default `/tmp/kafka-logs`, validation: no duplicate values, empty list rejected, null rejected, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0135 log.dir.failure.timeout.ms
P1 · M7 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_DIR_FAILURE_TIMEOUT_MS_CONFIG @4.4.0
Done when: `log.dir.failure.timeout.ms` is defined with type LONG, default `30000`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0136 log.dirs
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_DIRS_CONFIG @4.4.0
Done when: `log.dirs` is defined with type LIST, default null (unset), validation: no duplicate values, empty list rejected, null allowed, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0137 log.flush.interval.messages
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_FLUSH_INTERVAL_MESSAGES_CONFIG @4.4.0
Done when: `log.flush.interval.messages` is defined with type LONG, default `9223372036854775807` (Long.MAX_VALUE), validation: >= 1, importance HIGH; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0138 log.flush.interval.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_FLUSH_INTERVAL_MS_CONFIG @4.4.0
Done when: `log.flush.interval.ms` is defined with type LONG, default null (unset), validation: none beyond the type, importance HIGH; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); synonym group `log.flush.interval.ms` > `log.flush.scheduler.interval.ms` (first set key wins); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0139 log.flush.offset.checkpoint.interval.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_FLUSH_OFFSET_CHECKPOINT_INTERVAL_MS_CONFIG @4.4.0
Done when: `log.flush.offset.checkpoint.interval.ms` is defined with type INT, default `60000`, validation: >= 0, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0140 log.flush.scheduler.interval.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_FLUSH_SCHEDULER_INTERVAL_MS_CONFIG @4.4.0
Done when: `log.flush.scheduler.interval.ms` is defined with type LONG, default `9223372036854775807` (Long.MAX_VALUE), validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); synonym group `log.flush.interval.ms` > `log.flush.scheduler.interval.ms` (first set key wins); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0141 log.flush.start.offset.checkpoint.interval.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_FLUSH_START_OFFSET_CHECKPOINT_INTERVAL_MS_CONFIG @4.4.0
Done when: `log.flush.start.offset.checkpoint.interval.ms` is defined with type INT, default `60000`, validation: >= 0, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0142 log.index.interval.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_INDEX_INTERVAL_BYTES_CONFIG @4.4.0
Done when: `log.index.interval.bytes` is defined with type INT, default `4096`, validation: >= 0, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0143 log.index.size.max.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_INDEX_SIZE_MAX_BYTES_CONFIG @4.4.0
Done when: `log.index.size.max.bytes` is defined with type INT, default `10485760`, validation: >= 4, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0144 log.local.retention.bytes
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.LOG_LOCAL_RETENTION_BYTES_PROP @4.4.0
Done when: `log.local.retention.bytes` is defined with type LONG, default `-2`, validation: >= -2, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0145 log.local.retention.ms
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.LOG_LOCAL_RETENTION_MS_PROP @4.4.0
Done when: `log.local.retention.ms` is defined with type LONG, default `-2`, validation: >= -2, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0146 log.message.timestamp.after.max.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_MESSAGE_TIMESTAMP_AFTER_MAX_MS_CONFIG @4.4.0
Done when: `log.message.timestamp.after.max.ms` is defined with type LONG, default `3600000`, validation: >= 0, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0147 log.message.timestamp.before.max.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_MESSAGE_TIMESTAMP_BEFORE_MAX_MS_CONFIG @4.4.0
Done when: `log.message.timestamp.before.max.ms` is defined with type LONG, default `9223372036854775807` (Long.MAX_VALUE), validation: >= 0, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0148 log.message.timestamp.type
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_MESSAGE_TIMESTAMP_TYPE_CONFIG @4.4.0
Done when: `log.message.timestamp.type` is defined with type STRING, default `CreateTime`, validation: one of CreateTime, LogAppendTime, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0149 log.preallocate
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_PRE_ALLOCATE_CONFIG @4.4.0
Done when: `log.preallocate` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0150 log.remote.copy.lag.bytes
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.LOG_REMOTE_COPY_LAG_BYTES_PROP @4.4.0
Done when: `log.remote.copy.lag.bytes` is defined with type LONG, default `-1`, validation: >= -1, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0151 log.remote.copy.lag.ms
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.LOG_REMOTE_COPY_LAG_MS_PROP @4.4.0
Done when: `log.remote.copy.lag.ms` is defined with type LONG, default `0`, validation: >= -1, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0152 log.retention.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_RETENTION_BYTES_CONFIG @4.4.0
Done when: `log.retention.bytes` is defined with type LONG, default `-1`, validation: none beyond the type, importance HIGH; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0153 log.retention.check.interval.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_CLEANUP_INTERVAL_MS_CONFIG @4.4.0
Done when: `log.retention.check.interval.ms` is defined with type LONG, default `300000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0154 log.retention.hours
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_RETENTION_TIME_HOURS_CONFIG @4.4.0
Done when: `log.retention.hours` is defined with type INT, default `168`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); synonym group `log.retention.ms` > `log.retention.minutes` > `log.retention.hours` (first set key wins); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0155 log.retention.minutes
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_RETENTION_TIME_MINUTES_CONFIG @4.4.0
Done when: `log.retention.minutes` is defined with type INT, default null (unset), validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); synonym group `log.retention.ms` > `log.retention.minutes` > `log.retention.hours` (first set key wins); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0156 log.retention.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_RETENTION_TIME_MILLIS_CONFIG @4.4.0
Done when: `log.retention.ms` is defined with type LONG, default null (unset), validation: none beyond the type, importance HIGH; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); synonym group `log.retention.ms` > `log.retention.minutes` > `log.retention.hours` (first set key wins); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0157 log.roll.hours
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_ROLL_TIME_HOURS_CONFIG @4.4.0
Done when: `log.roll.hours` is defined with type INT, default `168`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); synonym group `log.roll.ms` > `log.roll.hours` (first set key wins); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0158 log.roll.jitter.hours
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_ROLL_TIME_JITTER_HOURS_CONFIG @4.4.0
Done when: `log.roll.jitter.hours` is defined with type INT, default `0`, validation: >= 0, importance HIGH; dynamic update mode read-only (static only); synonym group `log.roll.jitter.ms` > `log.roll.jitter.hours` (first set key wins); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0159 log.roll.jitter.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_ROLL_TIME_JITTER_MILLIS_CONFIG @4.4.0
Done when: `log.roll.jitter.ms` is defined with type LONG, default null (unset), validation: none beyond the type, importance HIGH; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); synonym group `log.roll.jitter.ms` > `log.roll.jitter.hours` (first set key wins); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0160 log.roll.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_ROLL_TIME_MILLIS_CONFIG @4.4.0
Done when: `log.roll.ms` is defined with type LONG, default null (unset), validation: none beyond the type, importance HIGH; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); synonym group `log.roll.ms` > `log.roll.hours` (first set key wins); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0161 log.segment.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_SEGMENT_BYTES_CONFIG @4.4.0
Done when: `log.segment.bytes` is defined with type INT, default `1073741824`, validation: >= 1048576, importance HIGH; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0162 log.segment.delete.delay.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.LOG_DELETE_DELAY_MS_CONFIG @4.4.0
Done when: `log.segment.delete.delay.ms` is defined with type LONG, default `60000`, validation: >= 0, importance HIGH; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0163 max.connection.creation.rate
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java SocketServerConfigs.MAX_CONNECTION_CREATION_RATE_CONFIG @4.4.0
Done when: `max.connection.creation.rate` is defined with type INT, default `2147483647` (Integer.MAX_VALUE), validation: >= 0, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); also accepted per listener as `listener.name.<listener>.max.connection.creation.rate`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0164 max.connections
P1 · M2 · Task · deps: CFG-0495, CFG-0009 · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java SocketServerConfigs.MAX_CONNECTIONS_CONFIG @4.4.0
Done when: `max.connections` is defined with type INT, default `2147483647` (Integer.MAX_VALUE), validation: >= 0, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); also accepted per listener as `listener.name.<listener>.max.connections`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0165 max.connections.per.ip
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java SocketServerConfigs.MAX_CONNECTIONS_PER_IP_CONFIG @4.4.0
Done when: `max.connections.per.ip` is defined with type INT, default `2147483647` (Integer.MAX_VALUE), validation: >= 0, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0166 max.connections.per.ip.overrides
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java SocketServerConfigs.MAX_CONNECTIONS_PER_IP_OVERRIDES_CONFIG @4.4.0
Done when: `max.connections.per.ip.overrides` is defined with type STRING, default empty, validation: none beyond the type, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0167 max.decompressed.message.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.MAX_DECOMPRESSED_MESSAGE_BYTES_CONFIG @4.4.0
Done when: `max.decompressed.message.bytes` is defined with type INT, default `2147483639`, validation: 1 to 2147483639 inclusive, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0168 max.incremental.fetch.session.cache.slots
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.MAX_INCREMENTAL_FETCH_SESSION_CACHE_SLOTS_CONFIG @4.4.0
Done when: `max.incremental.fetch.session.cache.slots` is defined with type INT, default `1000`, validation: >= 0, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0169 max.request.partition.size.limit
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.MAX_REQUEST_PARTITION_SIZE_LIMIT_CONFIG @4.4.0
Done when: `max.request.partition.size.limit` is defined with type INT, default `2000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0170 message.max.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.MESSAGE_MAX_BYTES_CONFIG @4.4.0
Done when: `message.max.bytes` is defined with type INT, default `1048588`, validation: >= 0, importance HIGH; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0171 metric.reporters
P1 · M2 · Task · deps: CFG-0495, CFG-0002 · ref: server/src/main/java/org/apache/kafka/server/metrics/MetricConfigs.java MetricConfigs.METRIC_REPORTER_CLASSES_CONFIG @4.4.0
Done when: `metric.reporters` is defined with type LIST, default `org.apache.kafka.common.metrics.JmxReporter`, validation: no duplicate values, empty list allowed, null rejected, importance LOW; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); accepted values follow the JVM-config ADR, other values fail startup naming the key; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0172 metrics.num.samples
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/metrics/MetricConfigs.java MetricConfigs.METRIC_NUM_SAMPLES_CONFIG @4.4.0
Done when: `metrics.num.samples` is defined with type INT, default `2`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0173 metrics.recording.level
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/metrics/MetricConfigs.java MetricConfigs.METRIC_RECORDING_LEVEL_CONFIG @4.4.0
Done when: `metrics.recording.level` is defined with type STRING, default `INFO`, validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0174 metrics.sample.window.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/metrics/MetricConfigs.java MetricConfigs.METRIC_SAMPLE_WINDOW_MS_CONFIG @4.4.0
Done when: `metrics.sample.window.ms` is defined with type LONG, default `30000`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0175 min.insync.replicas
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.MIN_IN_SYNC_REPLICAS_CONFIG @4.4.0
Done when: `min.insync.replicas` is defined with type INT, default `1`, validation: >= 1, importance HIGH; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0176 num.io.threads
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.NUM_IO_THREADS_CONFIG @4.4.0
Done when: `num.io.threads` is defined with type INT, default `8`, validation: >= 1, importance HIGH; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0177 num.network.threads
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java SocketServerConfigs.NUM_NETWORK_THREADS_CONFIG @4.4.0
Done when: `num.network.threads` is defined with type INT, default `3`, validation: >= 1, importance HIGH; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0178 num.partitions
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.NUM_PARTITIONS_CONFIG @4.4.0
Done when: `num.partitions` is defined with type INT, default `1`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0179 num.recovery.threads.per.data.dir
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerLogConfigs.java ServerLogConfigs.NUM_RECOVERY_THREADS_PER_DATA_DIR_CONFIG @4.4.0
Done when: `num.recovery.threads.per.data.dir` is defined with type INT, default `2`, validation: >= 1, importance HIGH; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0180 num.replica.alter.log.dirs.threads
P1 · M7 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.NUM_REPLICA_ALTER_LOG_DIRS_THREADS_CONFIG @4.4.0
Done when: `num.replica.alter.log.dirs.threads` is defined with type INT, default null (unset), validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0181 num.replica.fetchers
P1 · M3 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.NUM_REPLICA_FETCHERS_CONFIG @4.4.0
Done when: `num.replica.fetchers` is defined with type INT, default `1`, validation: >= 1, importance HIGH; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0182 principal.builder.class
P1 · M4 · Task · deps: CFG-0495, CFG-0002, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/internals/BrokerSecurityConfigs.java BrokerSecurityConfigs.PRINCIPAL_BUILDER_CLASS_CONFIG @4.4.0
Done when: `principal.builder.class` is defined with type CLASS, default `org.apache.kafka.common.security.authenticator.DefaultKafkaPrincipalBuilder`, validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); accepted values follow the JVM-config ADR, other values fail startup naming the key; also accepted per listener as `listener.name.<listener>.principal.builder.class`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0183 producer.purgatory.purge.interval.requests
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.PRODUCER_PURGATORY_PURGE_INTERVAL_REQUESTS_CONFIG @4.4.0
Done when: `producer.purgatory.purge.interval.requests` is defined with type INT, default `1000`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0184 queued.max.request.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java SocketServerConfigs.QUEUED_MAX_BYTES_CONFIG @4.4.0
Done when: `queued.max.request.bytes` is defined with type LONG, default `-1`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0185 queued.max.requests
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java SocketServerConfigs.QUEUED_MAX_REQUESTS_CONFIG @4.4.0
Done when: `queued.max.requests` is defined with type INT, default `500`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0186 quota.window.num
P1 · M4 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java QuotaConfig.NUM_QUOTA_SAMPLES_CONFIG @4.4.0
Done when: `quota.window.num` is defined with type INT, default `11`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0187 quota.window.size.seconds
P1 · M4 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java QuotaConfig.QUOTA_WINDOW_SIZE_SECONDS_CONFIG @4.4.0
Done when: `quota.window.size.seconds` is defined with type INT, default `1`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0188 remote.fetch.max.wait.ms
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_FETCH_MAX_WAIT_MS_PROP @4.4.0
Done when: `remote.fetch.max.wait.ms` is defined with type INT, default `500`, validation: >= 1, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0189 remote.list.offsets.request.timeout.ms
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LIST_OFFSETS_REQUEST_TIMEOUT_MS_PROP @4.4.0
Done when: `remote.list.offsets.request.timeout.ms` is defined with type LONG, default `30000`, validation: >= 1, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0190 remote.log.index.file.cache.total.size.bytes
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_INDEX_FILE_CACHE_TOTAL_SIZE_BYTES_PROP @4.4.0
Done when: `remote.log.index.file.cache.total.size.bytes` is defined with type LONG, default `1073741824`, validation: >= 1, importance LOW; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0191 remote.log.manager.copier.thread.pool.size
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_MANAGER_COPIER_THREAD_POOL_SIZE_PROP @4.4.0
Done when: `remote.log.manager.copier.thread.pool.size` is defined with type INT, default `10`, validation: >= 1, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0192 remote.log.manager.copy.max.bytes.per.second
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_MANAGER_COPY_MAX_BYTES_PER_SECOND_PROP @4.4.0
Done when: `remote.log.manager.copy.max.bytes.per.second` is defined with type LONG, default `9223372036854775807` (Long.MAX_VALUE), validation: >= 1, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0193 remote.log.manager.copy.quota.window.num
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_MANAGER_COPY_QUOTA_WINDOW_NUM_PROP @4.4.0
Done when: `remote.log.manager.copy.quota.window.num` is defined with type INT, default `11`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0194 remote.log.manager.copy.quota.window.size.seconds
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_MANAGER_COPY_QUOTA_WINDOW_SIZE_SECONDS_PROP @4.4.0
Done when: `remote.log.manager.copy.quota.window.size.seconds` is defined with type INT, default `1`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0195 remote.log.manager.expiration.thread.pool.size
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_MANAGER_EXPIRATION_THREAD_POOL_SIZE_PROP @4.4.0
Done when: `remote.log.manager.expiration.thread.pool.size` is defined with type INT, default `10`, validation: >= 1, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0196 remote.log.manager.fetch.max.bytes.per.second
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_MANAGER_FETCH_MAX_BYTES_PER_SECOND_PROP @4.4.0
Done when: `remote.log.manager.fetch.max.bytes.per.second` is defined with type LONG, default `9223372036854775807` (Long.MAX_VALUE), validation: >= 1, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0197 remote.log.manager.fetch.quota.window.num
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_MANAGER_FETCH_QUOTA_WINDOW_NUM_PROP @4.4.0
Done when: `remote.log.manager.fetch.quota.window.num` is defined with type INT, default `11`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0198 remote.log.manager.fetch.quota.window.size.seconds
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_MANAGER_FETCH_QUOTA_WINDOW_SIZE_SECONDS_PROP @4.4.0
Done when: `remote.log.manager.fetch.quota.window.size.seconds` is defined with type INT, default `1`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0199 remote.log.manager.follower.thread.pool.size
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_MANAGER_FOLLOWER_THREAD_POOL_SIZE_PROP @4.4.0
Done when: `remote.log.manager.follower.thread.pool.size` is defined with type INT, default `2`, validation: >= 1, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0200 remote.log.manager.task.interval.ms
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_MANAGER_TASK_INTERVAL_MS_PROP @4.4.0
Done when: `remote.log.manager.task.interval.ms` is defined with type LONG, default `30000`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0201 remote.log.manager.thread.pool.size
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_MANAGER_THREAD_POOL_SIZE_PROP @4.4.0
Done when: `remote.log.manager.thread.pool.size` is defined with type INT, default `2`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0202 remote.log.metadata.custom.metadata.max.bytes
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_METADATA_CUSTOM_METADATA_MAX_BYTES_PROP @4.4.0
Done when: `remote.log.metadata.custom.metadata.max.bytes` is defined with type INT, default `128`, validation: >= 0, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0203 remote.log.metadata.manager.class.name
P1 · M9 · Task · deps: CFG-0495, CFG-0002 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_METADATA_MANAGER_CLASS_NAME_PROP @4.4.0
Done when: `remote.log.metadata.manager.class.name` is defined with type STRING, default `org.apache.kafka.server.log.remote.metadata.storage.TopicBasedRemoteLogMetadataManager`, validation: non-empty when set, importance MEDIUM; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0204 remote.log.metadata.manager.class.path
P2 · M9 · Task · deps: CFG-0495, CFG-0002 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_METADATA_MANAGER_CLASS_PATH_PROP @4.4.0
Done when: `remote.log.metadata.manager.class.path` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0205 remote.log.metadata.manager.impl.prefix
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_METADATA_MANAGER_CONFIG_PREFIX_PROP @4.4.0
Done when: `remote.log.metadata.manager.impl.prefix` is defined with type STRING, default `rlmm.config.`, validation: non-empty when set, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0206 remote.log.metadata.manager.listener.name
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_METADATA_MANAGER_LISTENER_NAME_PROP @4.4.0
Done when: `remote.log.metadata.manager.listener.name` is defined with type STRING, default null (unset), validation: non-empty when set, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0207 remote.log.reader.max.pending.tasks
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_READER_MAX_PENDING_TASKS_PROP @4.4.0
Done when: `remote.log.reader.max.pending.tasks` is defined with type INT, default `100`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0208 remote.log.reader.threads
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_READER_THREADS_PROP @4.4.0
Done when: `remote.log.reader.threads` is defined with type INT, default `10`, validation: >= 1, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0209 remote.log.storage.manager.class.name
P1 · M9 · Task · deps: CFG-0495, CFG-0002 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_STORAGE_MANAGER_CLASS_NAME_PROP @4.4.0
Done when: `remote.log.storage.manager.class.name` is defined with type STRING, default null (unset), validation: non-empty when set, importance MEDIUM; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0210 remote.log.storage.manager.class.path
P2 · M9 · Task · deps: CFG-0495, CFG-0002 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_STORAGE_MANAGER_CLASS_PATH_PROP @4.4.0
Done when: `remote.log.storage.manager.class.path` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0211 remote.log.storage.manager.impl.prefix
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_STORAGE_MANAGER_CONFIG_PREFIX_PROP @4.4.0
Done when: `remote.log.storage.manager.impl.prefix` is defined with type STRING, default `rsm.config.`, validation: non-empty when set, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0212 remote.log.storage.system.enable
P1 · M9 · Task · deps: CFG-0495 · ref: storage/src/main/java/org/apache/kafka/server/log/remote/storage/RemoteLogManagerConfig.java RemoteLogManagerConfig.REMOTE_LOG_STORAGE_SYSTEM_ENABLE_PROP @4.4.0
Done when: `remote.log.storage.system.enable` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0213 replica.alter.log.dirs.io.max.bytes.per.second
P1 · M7 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java QuotaConfig.REPLICA_ALTER_LOG_DIRS_IO_MAX_BYTES_PER_SECOND_CONFIG @4.4.0
Done when: `replica.alter.log.dirs.io.max.bytes.per.second` is defined with type LONG, default `9223372036854775807` (Long.MAX_VALUE), validation: >= 0, importance MEDIUM; dynamic-only broker quota key, update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0214 replica.fetch.backoff.ms
P1 · M3 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.REPLICA_FETCH_BACKOFF_MS_CONFIG @4.4.0
Done when: `replica.fetch.backoff.ms` is defined with type INT, default `1000`, validation: >= 0, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0215 replica.fetch.max.bytes
P1 · M3 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.REPLICA_FETCH_MAX_BYTES_CONFIG @4.4.0
Done when: `replica.fetch.max.bytes` is defined with type INT, default `1048576`, validation: >= 0, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0216 replica.fetch.min.bytes
P1 · M3 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.REPLICA_FETCH_MIN_BYTES_CONFIG @4.4.0
Done when: `replica.fetch.min.bytes` is defined with type INT, default `1`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0217 replica.fetch.response.max.bytes
P1 · M3 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.REPLICA_FETCH_RESPONSE_MAX_BYTES_CONFIG @4.4.0
Done when: `replica.fetch.response.max.bytes` is defined with type INT, default `10485760`, validation: >= 0, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0218 replica.fetch.wait.max.ms
P1 · M3 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.REPLICA_FETCH_WAIT_MAX_MS_CONFIG @4.4.0
Done when: `replica.fetch.wait.max.ms` is defined with type INT, default `500`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0219 replica.high.watermark.checkpoint.interval.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.REPLICA_HIGH_WATERMARK_CHECKPOINT_INTERVAL_MS_CONFIG @4.4.0
Done when: `replica.high.watermark.checkpoint.interval.ms` is defined with type LONG, default `5000`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0220 replica.lag.time.max.ms
P1 · M3 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.REPLICA_LAG_TIME_MAX_MS_CONFIG @4.4.0
Done when: `replica.lag.time.max.ms` is defined with type LONG, default `30000`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0221 replica.selector.class
P1 · M3 · Task · deps: CFG-0495, CFG-0002 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.REPLICA_SELECTOR_CLASS_CONFIG @4.4.0
Done when: `replica.selector.class` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0222 replica.socket.receive.buffer.bytes
P1 · M3 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.REPLICA_SOCKET_RECEIVE_BUFFER_BYTES_CONFIG @4.4.0
Done when: `replica.socket.receive.buffer.bytes` is defined with type INT, default `65536`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0223 replica.socket.timeout.ms
P1 · M3 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.REPLICA_SOCKET_TIMEOUT_MS_CONFIG @4.4.0
Done when: `replica.socket.timeout.ms` is defined with type INT, default `30000`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0224 replication.quota.window.num
P1 · M4 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java QuotaConfig.NUM_REPLICATION_QUOTA_SAMPLES_CONFIG @4.4.0
Done when: `replication.quota.window.num` is defined with type INT, default `11`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0225 replication.quota.window.size.seconds
P1 · M4 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java QuotaConfig.REPLICATION_QUOTA_WINDOW_SIZE_SECONDS_CONFIG @4.4.0
Done when: `replication.quota.window.size.seconds` is defined with type INT, default `1`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0226 request.timeout.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.REQUEST_TIMEOUT_MS_CONFIG @4.4.0
Done when: `request.timeout.ms` is defined with type INT, default `30000`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0227 sasl.client.callback.handler.class
P1 · M4 · Task · deps: CFG-0495, CFG-0002 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_CLIENT_CALLBACK_HANDLER_CLASS @4.4.0
Done when: `sasl.client.callback.handler.class` is defined with type CLASS, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0228 sasl.enabled.mechanisms
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/internals/BrokerSecurityConfigs.java BrokerSecurityConfigs.SASL_ENABLED_MECHANISMS_CONFIG @4.4.0
Done when: `sasl.enabled.mechanisms` is defined with type LIST, default `GSSAPI`, validation: no duplicate values, empty list allowed, null rejected, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); also accepted per listener as `listener.name.<listener>.sasl.enabled.mechanisms`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0229 sasl.jaas.config
P1 · M4 · Task · deps: CFG-0495, CFG-0004, CFG-0002, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_JAAS_CONFIG @4.4.0
Done when: `sasl.jaas.config` is defined with type PASSWORD, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); the value is held as a secret and reported as sensitive; accepted values follow the JVM-config ADR, other values fail startup naming the key; also accepted per listener as `listener.name.<listener>.<mechanism>.sasl.jaas.config`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0230 sasl.kerberos.kinit.cmd
P3 · M4 · Task · deps: CFG-0495, CFG-0002, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_KERBEROS_KINIT_CMD @4.4.0
Done when: `sasl.kerberos.kinit.cmd` is defined with type STRING, default `/usr/bin/kinit`, validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); accepted values follow the JVM-config ADR, other values fail startup naming the key; also accepted per listener as `listener.name.<listener>.sasl.kerberos.kinit.cmd`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0231 sasl.kerberos.min.time.before.relogin
P3 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_KERBEROS_MIN_TIME_BEFORE_RELOGIN @4.4.0
Done when: `sasl.kerberos.min.time.before.relogin` is defined with type LONG, default `60000`, validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); also accepted per listener as `listener.name.<listener>.sasl.kerberos.min.time.before.relogin`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0232 sasl.kerberos.principal.to.local.rules
P3 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/internals/BrokerSecurityConfigs.java BrokerSecurityConfigs.SASL_KERBEROS_PRINCIPAL_TO_LOCAL_RULES_CONFIG @4.4.0
Done when: `sasl.kerberos.principal.to.local.rules` is defined with type LIST, default `DEFAULT`, validation: no duplicate values, empty list allowed, null rejected, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); also accepted per listener as `listener.name.<listener>.sasl.kerberos.principal.to.local.rules`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0233 sasl.kerberos.service.name
P3 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_KERBEROS_SERVICE_NAME @4.4.0
Done when: `sasl.kerberos.service.name` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); also accepted per listener as `listener.name.<listener>.sasl.kerberos.service.name`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0234 sasl.kerberos.ticket.renew.jitter
P3 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_KERBEROS_TICKET_RENEW_JITTER @4.4.0
Done when: `sasl.kerberos.ticket.renew.jitter` is defined with type DOUBLE, default `0.05`, validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); also accepted per listener as `listener.name.<listener>.sasl.kerberos.ticket.renew.jitter`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0235 sasl.kerberos.ticket.renew.window.factor
P3 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_KERBEROS_TICKET_RENEW_WINDOW_FACTOR @4.4.0
Done when: `sasl.kerberos.ticket.renew.window.factor` is defined with type DOUBLE, default `0.8`, validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); also accepted per listener as `listener.name.<listener>.sasl.kerberos.ticket.renew.window.factor`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0236 sasl.login.callback.handler.class
P1 · M4 · Task · deps: CFG-0495, CFG-0002, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_LOGIN_CALLBACK_HANDLER_CLASS @4.4.0
Done when: `sasl.login.callback.handler.class` is defined with type CLASS, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; also accepted per listener as `listener.name.<listener>.<mechanism>.sasl.login.callback.handler.class`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0237 sasl.login.class
P2 · M4 · Task · deps: CFG-0495, CFG-0002, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_LOGIN_CLASS @4.4.0
Done when: `sasl.login.class` is defined with type CLASS, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; also accepted per listener as `listener.name.<listener>.<mechanism>.sasl.login.class`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0238 sasl.login.connect.timeout.ms
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_LOGIN_CONNECT_TIMEOUT_MS @4.4.0
Done when: `sasl.login.connect.timeout.ms` is defined with type INT, default null (unset), validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.login.connect.timeout.ms`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0239 sasl.login.read.timeout.ms
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_LOGIN_READ_TIMEOUT_MS @4.4.0
Done when: `sasl.login.read.timeout.ms` is defined with type INT, default null (unset), validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.login.read.timeout.ms`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0240 sasl.login.refresh.buffer.seconds
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_LOGIN_REFRESH_BUFFER_SECONDS @4.4.0
Done when: `sasl.login.refresh.buffer.seconds` is defined with type SHORT, default `300`, validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); also accepted per listener as `listener.name.<listener>.sasl.login.refresh.buffer.seconds`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0241 sasl.login.refresh.min.period.seconds
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_LOGIN_REFRESH_MIN_PERIOD_SECONDS @4.4.0
Done when: `sasl.login.refresh.min.period.seconds` is defined with type SHORT, default `60`, validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); also accepted per listener as `listener.name.<listener>.sasl.login.refresh.min.period.seconds`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0242 sasl.login.refresh.window.factor
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_LOGIN_REFRESH_WINDOW_FACTOR @4.4.0
Done when: `sasl.login.refresh.window.factor` is defined with type DOUBLE, default `0.8`, validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); also accepted per listener as `listener.name.<listener>.sasl.login.refresh.window.factor`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0243 sasl.login.refresh.window.jitter
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_LOGIN_REFRESH_WINDOW_JITTER @4.4.0
Done when: `sasl.login.refresh.window.jitter` is defined with type DOUBLE, default `0.05`, validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); also accepted per listener as `listener.name.<listener>.sasl.login.refresh.window.jitter`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0244 sasl.login.retry.backoff.max.ms
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_LOGIN_RETRY_BACKOFF_MAX_MS @4.4.0
Done when: `sasl.login.retry.backoff.max.ms` is defined with type LONG, default `10000`, validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.login.retry.backoff.max.ms`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0245 sasl.login.retry.backoff.ms
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_LOGIN_RETRY_BACKOFF_MS @4.4.0
Done when: `sasl.login.retry.backoff.ms` is defined with type LONG, default `100`, validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.login.retry.backoff.ms`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0246 sasl.mechanism.inter.broker.protocol
P1 · M4 · Task · deps: CFG-0495 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/internals/BrokerSecurityConfigs.java BrokerSecurityConfigs.SASL_MECHANISM_INTER_BROKER_PROTOCOL_CONFIG @4.4.0
Done when: `sasl.mechanism.inter.broker.protocol` is defined with type STRING, default `GSSAPI`, validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0247 sasl.oauthbearer.allow.unverified.audience
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_ALLOW_UNVERIFIED_AUDIENCE @4.4.0
Done when: `sasl.oauthbearer.allow.unverified.audience` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.allow.unverified.audience`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0248 sasl.oauthbearer.allow.unverified.issuer
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_ALLOW_UNVERIFIED_ISSUER @4.4.0
Done when: `sasl.oauthbearer.allow.unverified.issuer` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.allow.unverified.issuer`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0249 sasl.oauthbearer.assertion.algorithm
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_ASSERTION_ALGORITHM @4.4.0
Done when: `sasl.oauthbearer.assertion.algorithm` is defined with type STRING, default `RS256`, validation: one of (case-insensitive) ES256, RS256, importance MEDIUM; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.assertion.algorithm`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0250 sasl.oauthbearer.assertion.claim.aud
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_ASSERTION_CLAIM_AUD @4.4.0
Done when: `sasl.oauthbearer.assertion.claim.aud` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.assertion.claim.aud`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0251 sasl.oauthbearer.assertion.claim.exp.seconds
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_ASSERTION_CLAIM_EXP_SECONDS @4.4.0
Done when: `sasl.oauthbearer.assertion.claim.exp.seconds` is defined with type INT, default `300`, validation: 0 to 86400 inclusive, importance LOW; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.assertion.claim.exp.seconds`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0252 sasl.oauthbearer.assertion.claim.iss
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_ASSERTION_CLAIM_ISS @4.4.0
Done when: `sasl.oauthbearer.assertion.claim.iss` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.assertion.claim.iss`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0253 sasl.oauthbearer.assertion.claim.jti.include
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_ASSERTION_CLAIM_JTI_INCLUDE @4.4.0
Done when: `sasl.oauthbearer.assertion.claim.jti.include` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.assertion.claim.jti.include`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0254 sasl.oauthbearer.assertion.claim.nbf.seconds
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_ASSERTION_CLAIM_NBF_SECONDS @4.4.0
Done when: `sasl.oauthbearer.assertion.claim.nbf.seconds` is defined with type INT, default `60`, validation: 0 to 3600 inclusive, importance LOW; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.assertion.claim.nbf.seconds`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0255 sasl.oauthbearer.assertion.claim.sub
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_ASSERTION_CLAIM_SUB @4.4.0
Done when: `sasl.oauthbearer.assertion.claim.sub` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.assertion.claim.sub`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0256 sasl.oauthbearer.assertion.file
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_ASSERTION_FILE @4.4.0
Done when: `sasl.oauthbearer.assertion.file` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.assertion.file`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0257 sasl.oauthbearer.assertion.private.key.file
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_ASSERTION_PRIVATE_KEY_FILE @4.4.0
Done when: `sasl.oauthbearer.assertion.private.key.file` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.assertion.private.key.file`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0258 sasl.oauthbearer.assertion.private.key.passphrase
P1 · M4 · Task · deps: CFG-0495, CFG-0004, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_ASSERTION_PRIVATE_KEY_PASSPHRASE @4.4.0
Done when: `sasl.oauthbearer.assertion.private.key.passphrase` is defined with type PASSWORD, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); the value is held as a secret and reported as sensitive; also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.assertion.private.key.passphrase`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0259 sasl.oauthbearer.assertion.template.file
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_ASSERTION_TEMPLATE_FILE @4.4.0
Done when: `sasl.oauthbearer.assertion.template.file` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.assertion.template.file`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0260 sasl.oauthbearer.client.credentials.client.id
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_CLIENT_CREDENTIALS_CLIENT_ID @4.4.0
Done when: `sasl.oauthbearer.client.credentials.client.id` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.client.credentials.client.id`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0261 sasl.oauthbearer.client.credentials.client.secret
P1 · M4 · Task · deps: CFG-0495, CFG-0004, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_CLIENT_CREDENTIALS_CLIENT_SECRET @4.4.0
Done when: `sasl.oauthbearer.client.credentials.client.secret` is defined with type PASSWORD, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); the value is held as a secret and reported as sensitive; also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.client.credentials.client.secret`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0262 sasl.oauthbearer.clock.skew.seconds
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_CLOCK_SKEW_SECONDS @4.4.0
Done when: `sasl.oauthbearer.clock.skew.seconds` is defined with type INT, default `30`, validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.clock.skew.seconds`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0263 sasl.oauthbearer.expected.audience
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_EXPECTED_AUDIENCE @4.4.0
Done when: `sasl.oauthbearer.expected.audience` is defined with type LIST, default empty, validation: no duplicate values, empty list allowed, null rejected, importance HIGH; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.expected.audience`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0264 sasl.oauthbearer.expected.issuer
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_EXPECTED_ISSUER @4.4.0
Done when: `sasl.oauthbearer.expected.issuer` is defined with type STRING, default null (unset), validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.expected.issuer`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0265 sasl.oauthbearer.jwks.endpoint.refresh.ms
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_JWKS_ENDPOINT_REFRESH_MS @4.4.0
Done when: `sasl.oauthbearer.jwks.endpoint.refresh.ms` is defined with type LONG, default `3600000`, validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.jwks.endpoint.refresh.ms`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0266 sasl.oauthbearer.jwks.endpoint.retry.backoff.max.ms
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_JWKS_ENDPOINT_RETRY_BACKOFF_MAX_MS @4.4.0
Done when: `sasl.oauthbearer.jwks.endpoint.retry.backoff.max.ms` is defined with type LONG, default `10000`, validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.jwks.endpoint.retry.backoff.max.ms`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0267 sasl.oauthbearer.jwks.endpoint.retry.backoff.ms
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_JWKS_ENDPOINT_RETRY_BACKOFF_MS @4.4.0
Done when: `sasl.oauthbearer.jwks.endpoint.retry.backoff.ms` is defined with type LONG, default `100`, validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.jwks.endpoint.retry.backoff.ms`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0268 sasl.oauthbearer.jwks.endpoint.url
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_JWKS_ENDPOINT_URL @4.4.0
Done when: `sasl.oauthbearer.jwks.endpoint.url` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.jwks.endpoint.url`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0269 sasl.oauthbearer.jwt.retriever.class
P1 · M4 · Task · deps: CFG-0495, CFG-0002, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_JWT_RETRIEVER_CLASS @4.4.0
Done when: `sasl.oauthbearer.jwt.retriever.class` is defined with type CLASS, default `org.apache.kafka.common.security.oauthbearer.DefaultJwtRetriever`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.jwt.retriever.class`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0270 sasl.oauthbearer.jwt.validator.class
P1 · M4 · Task · deps: CFG-0495, CFG-0002, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_JWT_VALIDATOR_CLASS @4.4.0
Done when: `sasl.oauthbearer.jwt.validator.class` is defined with type CLASS, default `org.apache.kafka.common.security.oauthbearer.DefaultJwtValidator`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.jwt.validator.class`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0271 sasl.oauthbearer.scope
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_SCOPE @4.4.0
Done when: `sasl.oauthbearer.scope` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.scope`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0272 sasl.oauthbearer.scope.claim.name
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_SCOPE_CLAIM_NAME @4.4.0
Done when: `sasl.oauthbearer.scope.claim.name` is defined with type STRING, default `scope`, validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.scope.claim.name`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0273 sasl.oauthbearer.sub.claim.name
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_SUB_CLAIM_NAME @4.4.0
Done when: `sasl.oauthbearer.sub.claim.name` is defined with type STRING, default `sub`, validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.sub.claim.name`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0274 sasl.oauthbearer.token.endpoint.url
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java SaslConfigs.SASL_OAUTHBEARER_TOKEN_ENDPOINT_URL @4.4.0
Done when: `sasl.oauthbearer.token.endpoint.url` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.oauthbearer.token.endpoint.url`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0275 sasl.server.callback.handler.class
P1 · M4 · Task · deps: CFG-0495, CFG-0002, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/internals/BrokerSecurityConfigs.java BrokerSecurityConfigs.SASL_SERVER_CALLBACK_HANDLER_CLASS_CONFIG @4.4.0
Done when: `sasl.server.callback.handler.class` is defined with type CLASS, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; also accepted per listener as `listener.name.<listener>.<mechanism>.sasl.server.callback.handler.class`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0276 sasl.server.max.receive.size
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/internals/BrokerSecurityConfigs.java BrokerSecurityConfigs.SASL_SERVER_MAX_RECEIVE_SIZE_CONFIG @4.4.0
Done when: `sasl.server.max.receive.size` is defined with type INT, default `524288`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.sasl.server.max.receive.size`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0277 security.inter.broker.protocol
P1 · M2 · Task · deps: CFG-0495 · labels: security · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.INTER_BROKER_SECURITY_PROTOCOL_CONFIG @4.4.0
Done when: `security.inter.broker.protocol` is defined with type STRING, default `PLAINTEXT`, validation: one of PLAINTEXT, SSL, SASL_PLAINTEXT, SASL_SSL, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0278 security.providers
P2 · M2 · Task · deps: CFG-0495, CFG-0002 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SecurityConfig.java SecurityConfig.SECURITY_PROVIDERS_CONFIG @4.4.0
Done when: `security.providers` is defined with type STRING, default null (unset), validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0279 socket.connection.setup.timeout.max.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.SOCKET_CONNECTION_SETUP_TIMEOUT_MAX_MS_CONFIG @4.4.0
Done when: `socket.connection.setup.timeout.max.ms` is defined with type LONG, default `30000`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0280 socket.connection.setup.timeout.ms
P1 · M2 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/ServerConfigs.java ServerConfigs.SOCKET_CONNECTION_SETUP_TIMEOUT_MS_CONFIG @4.4.0
Done when: `socket.connection.setup.timeout.ms` is defined with type LONG, default `10000`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0281 socket.listen.backlog.size
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java SocketServerConfigs.SOCKET_LISTEN_BACKLOG_SIZE_CONFIG @4.4.0
Done when: `socket.listen.backlog.size` is defined with type INT, default `50`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0282 socket.receive.buffer.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java SocketServerConfigs.SOCKET_RECEIVE_BUFFER_BYTES_CONFIG @4.4.0
Done when: `socket.receive.buffer.bytes` is defined with type INT, default `102400`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0283 socket.request.max.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java SocketServerConfigs.SOCKET_REQUEST_MAX_BYTES_CONFIG @4.4.0
Done when: `socket.request.max.bytes` is defined with type INT, default `104857600`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0284 socket.send.buffer.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/network/SocketServerConfigs.java SocketServerConfigs.SOCKET_SEND_BUFFER_BYTES_CONFIG @4.4.0
Done when: `socket.send.buffer.bytes` is defined with type INT, default `102400`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0285 ssl.allow.dn.changes
P1 · M7 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/internals/BrokerSecurityConfigs.java BrokerSecurityConfigs.SSL_ALLOW_DN_CHANGES_CONFIG @4.4.0
Done when: `ssl.allow.dn.changes` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.ssl.allow.dn.changes`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0286 ssl.allow.san.changes
P1 · M7 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/internals/BrokerSecurityConfigs.java BrokerSecurityConfigs.SSL_ALLOW_SAN_CHANGES_CONFIG @4.4.0
Done when: `ssl.allow.san.changes` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.ssl.allow.san.changes`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0287 ssl.cipher.suites
P1 · M2 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_CIPHER_SUITES_CONFIG @4.4.0
Done when: `ssl.cipher.suites` is defined with type LIST, default empty, validation: no duplicate values, empty list allowed, null rejected, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); also accepted per listener as `listener.name.<listener>.ssl.cipher.suites`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0288 ssl.client.auth
P1 · M2 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/internals/BrokerSecurityConfigs.java BrokerSecurityConfigs.SSL_CLIENT_AUTH_CONFIG @4.4.0
Done when: `ssl.client.auth` is defined with type STRING, default `none`, validation: one of required, requested, none, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); also accepted per listener as `listener.name.<listener>.ssl.client.auth`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0289 ssl.enabled.protocols
P1 · M2 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_ENABLED_PROTOCOLS_CONFIG @4.4.0
Done when: `ssl.enabled.protocols` is defined with type LIST, default `TLSv1.2,TLSv1.3`, validation: no duplicate values, empty list allowed, null rejected, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); also accepted per listener as `listener.name.<listener>.ssl.enabled.protocols`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0290 ssl.endpoint.identification.algorithm
P1 · M2 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_ENDPOINT_IDENTIFICATION_ALGORITHM_CONFIG @4.4.0
Done when: `ssl.endpoint.identification.algorithm` is defined with type STRING, default `https`, validation: none beyond the type, importance LOW; dynamic update mode per-broker (dynamic for one broker id only); also accepted per listener as `listener.name.<listener>.ssl.endpoint.identification.algorithm`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0291 ssl.engine.factory.class
P2 · M2 · Task · deps: CFG-0495, CFG-0002, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_ENGINE_FACTORY_CLASS_CONFIG @4.4.0
Done when: `ssl.engine.factory.class` is defined with type CLASS, default null (unset), validation: none beyond the type, importance LOW; dynamic update mode per-broker (dynamic for one broker id only); accepted values follow the JVM-config ADR, other values fail startup naming the key; also accepted per listener as `listener.name.<listener>.ssl.engine.factory.class`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0292 ssl.key.password
P1 · M2 · Task · deps: CFG-0495, CFG-0004, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_KEY_PASSWORD_CONFIG @4.4.0
Done when: `ssl.key.password` is defined with type PASSWORD, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); the value is held as a secret and reported as sensitive; also accepted per listener as `listener.name.<listener>.ssl.key.password`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0293 ssl.keymanager.algorithm
P2 · M2 · Task · deps: CFG-0495, CFG-0002, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_KEYMANAGER_ALGORITHM_CONFIG @4.4.0
Done when: `ssl.keymanager.algorithm` is defined with type STRING, default `SunX509`, validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); accepted values follow the JVM-config ADR, other values fail startup naming the key; also accepted per listener as `listener.name.<listener>.ssl.keymanager.algorithm`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0294 ssl.keystore.certificate.chain
P1 · M2 · Task · deps: CFG-0495, CFG-0004, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_KEYSTORE_CERTIFICATE_CHAIN_CONFIG @4.4.0
Done when: `ssl.keystore.certificate.chain` is defined with type PASSWORD, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); the value is held as a secret and reported as sensitive; also accepted per listener as `listener.name.<listener>.ssl.keystore.certificate.chain`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0295 ssl.keystore.key
P1 · M2 · Task · deps: CFG-0495, CFG-0004, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_KEYSTORE_KEY_CONFIG @4.4.0
Done when: `ssl.keystore.key` is defined with type PASSWORD, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); the value is held as a secret and reported as sensitive; also accepted per listener as `listener.name.<listener>.ssl.keystore.key`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0296 ssl.keystore.location
P1 · M2 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_KEYSTORE_LOCATION_CONFIG @4.4.0
Done when: `ssl.keystore.location` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); also accepted per listener as `listener.name.<listener>.ssl.keystore.location`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0297 ssl.keystore.password
P1 · M2 · Task · deps: CFG-0495, CFG-0004, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_KEYSTORE_PASSWORD_CONFIG @4.4.0
Done when: `ssl.keystore.password` is defined with type PASSWORD, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); the value is held as a secret and reported as sensitive; also accepted per listener as `listener.name.<listener>.ssl.keystore.password`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0298 ssl.keystore.type
P1 · M2 · Task · deps: CFG-0495, CFG-0002, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_KEYSTORE_TYPE_CONFIG @4.4.0
Done when: `ssl.keystore.type` is defined with type STRING, default `JKS`, validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); accepted values follow the JVM-config ADR, other values fail startup naming the key; also accepted per listener as `listener.name.<listener>.ssl.keystore.type`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0299 ssl.principal.mapping.rules
P1 · M4 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/internals/BrokerSecurityConfigs.java BrokerSecurityConfigs.SSL_PRINCIPAL_MAPPING_RULES_CONFIG @4.4.0
Done when: `ssl.principal.mapping.rules` is defined with type STRING, default `DEFAULT`, validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); also accepted per listener as `listener.name.<listener>.ssl.principal.mapping.rules`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0300 ssl.protocol
P1 · M2 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_PROTOCOL_CONFIG @4.4.0
Done when: `ssl.protocol` is defined with type STRING, default `TLSv1.3`, validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); also accepted per listener as `listener.name.<listener>.ssl.protocol`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0301 ssl.provider
P2 · M2 · Task · deps: CFG-0495, CFG-0002, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_PROVIDER_CONFIG @4.4.0
Done when: `ssl.provider` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); accepted values follow the JVM-config ADR, other values fail startup naming the key; also accepted per listener as `listener.name.<listener>.ssl.provider`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0302 ssl.secure.random.implementation
P2 · M2 · Task · deps: CFG-0495, CFG-0002, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_SECURE_RANDOM_IMPLEMENTATION_CONFIG @4.4.0
Done when: `ssl.secure.random.implementation` is defined with type STRING, default null (unset), validation: none beyond the type, importance LOW; dynamic update mode per-broker (dynamic for one broker id only); accepted values follow the JVM-config ADR, other values fail startup naming the key; also accepted per listener as `listener.name.<listener>.ssl.secure.random.implementation`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0303 ssl.trustmanager.algorithm
P2 · M2 · Task · deps: CFG-0495, CFG-0002, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_TRUSTMANAGER_ALGORITHM_CONFIG @4.4.0
Done when: `ssl.trustmanager.algorithm` is defined with type STRING, default `PKIX`, validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); accepted values follow the JVM-config ADR, other values fail startup naming the key; also accepted per listener as `listener.name.<listener>.ssl.trustmanager.algorithm`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0304 ssl.truststore.certificates
P1 · M2 · Task · deps: CFG-0495, CFG-0004, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_TRUSTSTORE_CERTIFICATES_CONFIG @4.4.0
Done when: `ssl.truststore.certificates` is defined with type PASSWORD, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); the value is held as a secret and reported as sensitive; also accepted per listener as `listener.name.<listener>.ssl.truststore.certificates`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0305 ssl.truststore.location
P1 · M2 · Task · deps: CFG-0495, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_TRUSTSTORE_LOCATION_CONFIG @4.4.0
Done when: `ssl.truststore.location` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); also accepted per listener as `listener.name.<listener>.ssl.truststore.location`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0306 ssl.truststore.password
P1 · M2 · Task · deps: CFG-0495, CFG-0004, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_TRUSTSTORE_PASSWORD_CONFIG @4.4.0
Done when: `ssl.truststore.password` is defined with type PASSWORD, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); the value is held as a secret and reported as sensitive; also accepted per listener as `listener.name.<listener>.ssl.truststore.password`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0307 ssl.truststore.type
P1 · M2 · Task · deps: CFG-0495, CFG-0002, CFG-0009 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java SslConfigs.SSL_TRUSTSTORE_TYPE_CONFIG @4.4.0
Done when: `ssl.truststore.type` is defined with type STRING, default `JKS`, validation: none beyond the type, importance MEDIUM; dynamic update mode per-broker (dynamic for one broker id only); accepted values follow the JVM-config ADR, other values fail startup naming the key; also accepted per listener as `listener.name.<listener>.ssl.truststore.type`; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0308 telemetry.max.bytes
P1 · M8 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/metrics/MetricConfigs.java MetricConfigs.CLIENT_TELEMETRY_MAX_BYTES_CONFIG @4.4.0
Done when: `telemetry.max.bytes` is defined with type INT, default `1048576`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0309 unclean.leader.election.enable
P1 · M3 · Task · deps: CFG-0495 · ref: server/src/main/java/org/apache/kafka/server/config/ReplicationConfigs.java ReplicationConfigs.UNCLEAN_LEADER_ELECTION_ENABLE_CONFIG @4.4.0
Done when: `unclean.leader.election.enable` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance HIGH; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

## [To Do] CFG-F08 Topic config keys
P1 · M2 · Feature · deps: CFG-F05
Scope: One task per topic config key.

### [To Do] CFG-0310 cleanup.policy
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.CLEANUP_POLICY_CONFIG @4.4.0
Done when: `cleanup.policy` is defined with type LIST, default `delete`, validation: each value one of [compact, delete], no duplicate values, empty list allowed, null rejected, importance MEDIUM; broker default synonym `log.cleanup.policy`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0311 compression.gzip.level
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.COMPRESSION_GZIP_LEVEL_CONFIG @4.4.0
Done when: `compression.gzip.level` is defined with type INT, default `-1`, validation: -1 or 1 to 9 inclusive, importance MEDIUM; broker default synonym `compression.gzip.level`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0312 compression.lz4.level
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.COMPRESSION_LZ4_LEVEL_CONFIG @4.4.0
Done when: `compression.lz4.level` is defined with type INT, default `9`, validation: 1 to 17 inclusive, importance MEDIUM; broker default synonym `compression.lz4.level`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0313 compression.type
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.COMPRESSION_TYPE_CONFIG @4.4.0
Done when: `compression.type` is defined with type STRING, default `producer`, validation: one of uncompressed, zstd, lz4, snappy, gzip, producer, importance MEDIUM; broker default synonym `compression.type`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0314 compression.zstd.level
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.COMPRESSION_ZSTD_LEVEL_CONFIG @4.4.0
Done when: `compression.zstd.level` is defined with type INT, default `3`, validation: -131072 to 22 inclusive, importance MEDIUM; broker default synonym `compression.zstd.level`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0315 delete.retention.ms
P1 · M5 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.DELETE_RETENTION_MS_CONFIG @4.4.0
Done when: `delete.retention.ms` is defined with type LONG, default `86400000`, validation: >= 0, importance MEDIUM; broker default synonym `log.cleaner.delete.retention.ms`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0316 errors.deadletterqueue.group.enable
P1 · M8 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.ERRORS_DEADLETTERQUEUE_GROUP_ENABLE_CONFIG @4.4.0
Done when: `errors.deadletterqueue.group.enable` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance MEDIUM; no broker default synonym; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0317 file.delete.delay.ms
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.FILE_DELETE_DELAY_MS_CONFIG @4.4.0
Done when: `file.delete.delay.ms` is defined with type LONG, default `60000`, validation: >= 0, importance MEDIUM; broker default synonym `log.segment.delete.delay.ms`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0318 flush.messages
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.FLUSH_MESSAGES_INTERVAL_CONFIG @4.4.0
Done when: `flush.messages` is defined with type LONG, default `9223372036854775807` (Long.MAX_VALUE), validation: >= 1, importance MEDIUM; broker default synonym `log.flush.interval.messages`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0319 flush.ms
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.FLUSH_MS_CONFIG @4.4.0
Done when: `flush.ms` is defined with type LONG, default `9223372036854775807` (Long.MAX_VALUE), validation: >= 0, importance MEDIUM; broker default synonym `log.flush.interval.ms`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0320 follower.replication.throttled.replicas
P1 · M4 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java QuotaConfig.FOLLOWER_REPLICATION_THROTTLED_REPLICAS_CONFIG @4.4.0
Done when: `follower.replication.throttled.replicas` is defined with type LIST, default empty, validation: `*` or a comma list of `partitionId:brokerId` pairs, importance MEDIUM; no broker default synonym; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0321 index.interval.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.INDEX_INTERVAL_BYTES_CONFIG @4.4.0
Done when: `index.interval.bytes` is defined with type INT, default `4096`, validation: >= 0, importance MEDIUM; broker default synonym `log.index.interval.bytes`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0322 leader.replication.throttled.replicas
P1 · M4 · Task · deps: CFG-0495 · ref: server-common/src/main/java/org/apache/kafka/server/config/QuotaConfig.java QuotaConfig.LEADER_REPLICATION_THROTTLED_REPLICAS_CONFIG @4.4.0
Done when: `leader.replication.throttled.replicas` is defined with type LIST, default empty, validation: `*` or a comma list of `partitionId:brokerId` pairs, importance MEDIUM; no broker default synonym; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0323 local.retention.bytes
P1 · M9 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.LOCAL_LOG_RETENTION_BYTES_CONFIG @4.4.0
Done when: `local.retention.bytes` is defined with type LONG, default `-2`, validation: >= -2, importance MEDIUM; broker default synonym `log.local.retention.bytes`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0324 local.retention.ms
P1 · M9 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.LOCAL_LOG_RETENTION_MS_CONFIG @4.4.0
Done when: `local.retention.ms` is defined with type LONG, default `-2`, validation: >= -2, importance MEDIUM; broker default synonym `log.local.retention.ms`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0325 max.compaction.lag.ms
P1 · M5 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.MAX_COMPACTION_LAG_MS_CONFIG @4.4.0
Done when: `max.compaction.lag.ms` is defined with type LONG, default `9223372036854775807` (Long.MAX_VALUE), validation: >= 1, importance MEDIUM; broker default synonym `log.cleaner.max.compaction.lag.ms`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0326 max.decompressed.message.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.MAX_DECOMPRESSED_MESSAGE_BYTES_CONFIG @4.4.0
Done when: `max.decompressed.message.bytes` is defined with type INT, default `2147483639`, validation: 1 to 2147483639 inclusive, importance MEDIUM; broker default synonym `max.decompressed.message.bytes`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0327 max.message.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.MAX_MESSAGE_BYTES_CONFIG @4.4.0
Done when: `max.message.bytes` is defined with type INT, default `1048588`, validation: >= 0, importance MEDIUM; broker default synonym `message.max.bytes`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0328 message.timestamp.after.max.ms
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.MESSAGE_TIMESTAMP_AFTER_MAX_MS_CONFIG @4.4.0
Done when: `message.timestamp.after.max.ms` is defined with type LONG, default `3600000`, validation: >= 0, importance MEDIUM; broker default synonym `log.message.timestamp.after.max.ms`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0329 message.timestamp.before.max.ms
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.MESSAGE_TIMESTAMP_BEFORE_MAX_MS_CONFIG @4.4.0
Done when: `message.timestamp.before.max.ms` is defined with type LONG, default `9223372036854775807` (Long.MAX_VALUE), validation: >= 0, importance MEDIUM; broker default synonym `log.message.timestamp.before.max.ms`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0330 message.timestamp.type
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.MESSAGE_TIMESTAMP_TYPE_CONFIG @4.4.0
Done when: `message.timestamp.type` is defined with type STRING, default `CreateTime`, validation: one of CreateTime, LogAppendTime, importance MEDIUM; broker default synonym `log.message.timestamp.type`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0331 min.cleanable.dirty.ratio
P1 · M5 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.MIN_CLEANABLE_DIRTY_RATIO_CONFIG @4.4.0
Done when: `min.cleanable.dirty.ratio` is defined with type DOUBLE, default `0.5`, validation: 0 to 1 inclusive, importance MEDIUM; broker default synonym `log.cleaner.min.cleanable.ratio`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0332 min.compaction.lag.ms
P1 · M5 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.MIN_COMPACTION_LAG_MS_CONFIG @4.4.0
Done when: `min.compaction.lag.ms` is defined with type LONG, default `0`, validation: >= 0, importance MEDIUM; broker default synonym `log.cleaner.min.compaction.lag.ms`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0333 min.insync.replicas
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.MIN_IN_SYNC_REPLICAS_CONFIG @4.4.0
Done when: `min.insync.replicas` is defined with type INT, default `1`, validation: >= 1, importance MEDIUM; broker default synonym `min.insync.replicas`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0334 preallocate
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.PREALLOCATE_CONFIG @4.4.0
Done when: `preallocate` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance MEDIUM; broker default synonym `log.preallocate`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0335 remote.copy.lag.bytes
P1 · M9 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.REMOTE_COPY_LAG_BYTES_CONFIG @4.4.0
Done when: `remote.copy.lag.bytes` is defined with type LONG, default `-1`, validation: >= -1, importance MEDIUM; broker default synonym `log.remote.copy.lag.bytes`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0336 remote.copy.lag.ms
P1 · M9 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.REMOTE_COPY_LAG_MS_CONFIG @4.4.0
Done when: `remote.copy.lag.ms` is defined with type LONG, default `0`, validation: >= -1, importance MEDIUM; broker default synonym `log.remote.copy.lag.ms`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0337 remote.log.copy.disable
P1 · M9 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.REMOTE_LOG_COPY_DISABLE_CONFIG @4.4.0
Done when: `remote.log.copy.disable` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance MEDIUM; no broker default synonym; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0338 remote.log.delete.on.disable
P1 · M9 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.REMOTE_LOG_DELETE_ON_DISABLE_CONFIG @4.4.0
Done when: `remote.log.delete.on.disable` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance MEDIUM; no broker default synonym; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0339 remote.storage.enable
P1 · M9 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.REMOTE_LOG_STORAGE_ENABLE_CONFIG @4.4.0
Done when: `remote.storage.enable` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance MEDIUM; no broker default synonym; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0340 retention.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.RETENTION_BYTES_CONFIG @4.4.0
Done when: `retention.bytes` is defined with type LONG, default `-1`, validation: none beyond the type, importance MEDIUM; broker default synonym `log.retention.bytes`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0341 retention.ms
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.RETENTION_MS_CONFIG @4.4.0
Done when: `retention.ms` is defined with type LONG, default `604800000`, validation: >= -1, importance MEDIUM; broker default synonym `log.retention.ms`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0342 segment.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.SEGMENT_BYTES_CONFIG @4.4.0
Done when: `segment.bytes` is defined with type INT, default `1073741824`, validation: >= 1048576, importance MEDIUM; broker default synonym `log.segment.bytes`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0343 segment.index.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.SEGMENT_INDEX_BYTES_CONFIG @4.4.0
Done when: `segment.index.bytes` is defined with type INT, default `10485760`, validation: >= 4, importance MEDIUM; broker default synonym `log.index.size.max.bytes`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0344 segment.jitter.ms
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.SEGMENT_JITTER_MS_CONFIG @4.4.0
Done when: `segment.jitter.ms` is defined with type LONG, default `0`, validation: >= 0, importance MEDIUM; broker default synonym `log.roll.jitter.ms`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0345 segment.ms
P1 · M2 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.SEGMENT_MS_CONFIG @4.4.0
Done when: `segment.ms` is defined with type LONG, default `604800000`, validation: >= 1, importance MEDIUM; broker default synonym `log.roll.ms`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0346 unclean.leader.election.enable
P1 · M3 · Task · deps: CFG-0495 · ref: clients/src/main/java/org/apache/kafka/common/config/TopicConfig.java TopicConfig.UNCLEAN_LEADER_ELECTION_ENABLE_CONFIG @4.4.0
Done when: `unclean.leader.election.enable` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance MEDIUM; broker default synonym `unclean.leader.election.enable`; updatable per topic with AlterConfigs/IncrementalAlterConfigs (source DYNAMIC_TOPIC_CONFIG); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

## [To Do] CFG-F09 Coordinator config keys
P1 · M5 · Feature · deps: CFG-F01
Scope: One task per group, offsets, transaction and share coordinator key.

### [To Do] CFG-0347 add.partitions.to.txn.retry.backoff.max.ms
P1 · M6 · Task · deps: CFG-0495 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/AddPartitionsToTxnConfig.java AddPartitionsToTxnConfig.ADD_PARTITIONS_TO_TXN_RETRY_BACKOFF_MAX_MS_CONFIG @4.4.0
Done when: `add.partitions.to.txn.retry.backoff.max.ms` is defined with type INT, default `100`, validation: >= 0, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0348 add.partitions.to.txn.retry.backoff.ms
P1 · M6 · Task · deps: CFG-0495 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/AddPartitionsToTxnConfig.java AddPartitionsToTxnConfig.ADD_PARTITIONS_TO_TXN_RETRY_BACKOFF_MS_CONFIG @4.4.0
Done when: `add.partitions.to.txn.retry.backoff.ms` is defined with type INT, default `20`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0349 errors.deadletterqueue.auto.create.topics.enable
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.ERRORS_DEADLETTERQUEUE_AUTO_CREATE_TOPICS_ENABLE_CONFIG @4.4.0
Done when: `errors.deadletterqueue.auto.create.topics.enable` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0350 errors.deadletterqueue.topic.name.prefix
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.ERRORS_DEADLETTERQUEUE_TOPIC_NAME_PREFIX_CONFIG @4.4.0
Done when: `errors.deadletterqueue.topic.name.prefix` is defined with type STRING, default `dlq.`, validation: none beyond the type, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0351 group.consumer.assignment.interval.ms
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.CONSUMER_GROUP_ASSIGNMENT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.consumer.assignment.interval.ms` is defined with type INT, default `1000`, validation: >= 0, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0352 group.consumer.assignors
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.CONSUMER_GROUP_ASSIGNORS_CONFIG @4.4.0
Done when: `group.consumer.assignors` is defined with type LIST, default `uniform,range`, validation: no duplicate values, empty list rejected, null rejected, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0353 group.consumer.heartbeat.interval.ms
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.CONSUMER_GROUP_HEARTBEAT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.consumer.heartbeat.interval.ms` is defined with type INT, default `5000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0354 group.consumer.max.assignment.interval.ms
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.CONSUMER_GROUP_MAX_ASSIGNMENT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.consumer.max.assignment.interval.ms` is defined with type INT, default `15000`, validation: >= 0, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0355 group.consumer.max.heartbeat.interval.ms
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.CONSUMER_GROUP_MAX_HEARTBEAT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.consumer.max.heartbeat.interval.ms` is defined with type INT, default `15000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0356 group.consumer.max.session.timeout.ms
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.CONSUMER_GROUP_MAX_SESSION_TIMEOUT_MS_CONFIG @4.4.0
Done when: `group.consumer.max.session.timeout.ms` is defined with type INT, default `60000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0357 group.consumer.max.size
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.CONSUMER_GROUP_MAX_SIZE_CONFIG @4.4.0
Done when: `group.consumer.max.size` is defined with type INT, default `2147483647` (Integer.MAX_VALUE), validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0358 group.consumer.migration.policy
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.CONSUMER_GROUP_MIGRATION_POLICY_CONFIG @4.4.0
Done when: `group.consumer.migration.policy` is defined with type STRING, default `bidirectional`, validation: one of (case-insensitive) DISABLED, DOWNGRADE, UPGRADE, BIDIRECTIONAL, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0359 group.consumer.min.assignment.interval.ms
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.CONSUMER_GROUP_MIN_ASSIGNMENT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.consumer.min.assignment.interval.ms` is defined with type INT, default `0`, validation: >= 0, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0360 group.consumer.min.heartbeat.interval.ms
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.CONSUMER_GROUP_MIN_HEARTBEAT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.consumer.min.heartbeat.interval.ms` is defined with type INT, default `5000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0361 group.consumer.min.session.timeout.ms
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.CONSUMER_GROUP_MIN_SESSION_TIMEOUT_MS_CONFIG @4.4.0
Done when: `group.consumer.min.session.timeout.ms` is defined with type INT, default `45000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0362 group.consumer.session.timeout.ms
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.CONSUMER_GROUP_SESSION_TIMEOUT_MS_CONFIG @4.4.0
Done when: `group.consumer.session.timeout.ms` is defined with type INT, default `45000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0363 group.coordinator.append.linger.ms
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.GROUP_COORDINATOR_APPEND_LINGER_MS_CONFIG @4.4.0
Done when: `group.coordinator.append.linger.ms` is defined with type INT, default `-1`, validation: >= -1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0364 group.coordinator.background.threads
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.GROUP_COORDINATOR_NUM_BACKGROUND_THREADS_CONFIG @4.4.0
Done when: `group.coordinator.background.threads` is defined with type INT, default `2`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0365 group.coordinator.cached.buffer.max.bytes
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.CACHED_BUFFER_MAX_BYTES_CONFIG @4.4.0
Done when: `group.coordinator.cached.buffer.max.bytes` is defined with type INT, default `1048588`, validation: >= 524288, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0366 group.coordinator.rebalance.protocols
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.GROUP_COORDINATOR_REBALANCE_PROTOCOLS_CONFIG @4.4.0
Done when: `group.coordinator.rebalance.protocols` is defined with type LIST, default `classic,consumer,streams`, validation: each value one of [consumer, classic, share, streams], no duplicate values, empty list rejected, null rejected, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0367 group.coordinator.threads
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.GROUP_COORDINATOR_NUM_THREADS_CONFIG @4.4.0
Done when: `group.coordinator.threads` is defined with type INT, default `4`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0368 group.initial.rebalance.delay.ms
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.GROUP_INITIAL_REBALANCE_DELAY_MS_CONFIG @4.4.0
Done when: `group.initial.rebalance.delay.ms` is defined with type INT, default `3000`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0369 group.max.session.timeout.ms
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.GROUP_MAX_SESSION_TIMEOUT_MS_CONFIG @4.4.0
Done when: `group.max.session.timeout.ms` is defined with type INT, default `1800000`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0370 group.max.size
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.GROUP_MAX_SIZE_CONFIG @4.4.0
Done when: `group.max.size` is defined with type INT, default `2147483647` (Integer.MAX_VALUE), validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0371 group.min.session.timeout.ms
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.GROUP_MIN_SESSION_TIMEOUT_MS_CONFIG @4.4.0
Done when: `group.min.session.timeout.ms` is defined with type INT, default `6000`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0372 group.share.assignment.interval.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.SHARE_GROUP_ASSIGNMENT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.share.assignment.interval.ms` is defined with type INT, default `1000`, validation: >= 0, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0373 group.share.assignors
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.SHARE_GROUP_ASSIGNORS_CONFIG @4.4.0
Done when: `group.share.assignors` is defined with type LIST, default `simple`, validation: no duplicate values, empty list rejected, null rejected, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0374 group.share.delivery.count.limit
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/share/ShareGroupConfig.java ShareGroupConfig.SHARE_GROUP_DELIVERY_COUNT_LIMIT_CONFIG @4.4.0
Done when: `group.share.delivery.count.limit` is defined with type INT, default `5`, validation: 2 to 10 inclusive, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0375 group.share.heartbeat.interval.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.SHARE_GROUP_HEARTBEAT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.share.heartbeat.interval.ms` is defined with type INT, default `5000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0376 group.share.max.assignment.interval.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.SHARE_GROUP_MAX_ASSIGNMENT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.share.max.assignment.interval.ms` is defined with type INT, default `15000`, validation: >= 0, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0377 group.share.max.delivery.count.limit
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/share/ShareGroupConfig.java ShareGroupConfig.SHARE_GROUP_MAX_DELIVERY_COUNT_LIMIT_CONFIG @4.4.0
Done when: `group.share.max.delivery.count.limit` is defined with type INT, default `10`, validation: 5 to 25 inclusive, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0378 group.share.max.heartbeat.interval.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.SHARE_GROUP_MAX_HEARTBEAT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.share.max.heartbeat.interval.ms` is defined with type INT, default `15000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0379 group.share.max.partition.max.record.locks
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/share/ShareGroupConfig.java ShareGroupConfig.SHARE_GROUP_MAX_PARTITION_MAX_RECORD_LOCKS_CONFIG @4.4.0
Done when: `group.share.max.partition.max.record.locks` is defined with type INT, default `4000`, validation: 2000 to 10000 inclusive, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0380 group.share.max.record.lock.duration.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/share/ShareGroupConfig.java ShareGroupConfig.SHARE_GROUP_MAX_RECORD_LOCK_DURATION_MS_CONFIG @4.4.0
Done when: `group.share.max.record.lock.duration.ms` is defined with type INT, default `60000`, validation: 30000 to 3600000 inclusive, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0381 group.share.max.session.timeout.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.SHARE_GROUP_MAX_SESSION_TIMEOUT_MS_CONFIG @4.4.0
Done when: `group.share.max.session.timeout.ms` is defined with type INT, default `60000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0382 group.share.max.share.sessions
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/share/ShareGroupConfig.java ShareGroupConfig.SHARE_GROUP_MAX_SHARE_SESSIONS_CONFIG @4.4.0
Done when: `group.share.max.share.sessions` is defined with type INT, default `2000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0383 group.share.max.size
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.SHARE_GROUP_MAX_SIZE_CONFIG @4.4.0
Done when: `group.share.max.size` is defined with type INT, default `200`, validation: 1 to 1000 inclusive, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0384 group.share.min.assignment.interval.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.SHARE_GROUP_MIN_ASSIGNMENT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.share.min.assignment.interval.ms` is defined with type INT, default `0`, validation: >= 0, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0385 group.share.min.delivery.count.limit
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/share/ShareGroupConfig.java ShareGroupConfig.SHARE_GROUP_MIN_DELIVERY_COUNT_LIMIT_CONFIG @4.4.0
Done when: `group.share.min.delivery.count.limit` is defined with type INT, default `2`, validation: 2 to 5 inclusive, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0386 group.share.min.heartbeat.interval.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.SHARE_GROUP_MIN_HEARTBEAT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.share.min.heartbeat.interval.ms` is defined with type INT, default `5000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0387 group.share.min.partition.max.record.locks
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/share/ShareGroupConfig.java ShareGroupConfig.SHARE_GROUP_MIN_PARTITION_MAX_RECORD_LOCKS_CONFIG @4.4.0
Done when: `group.share.min.partition.max.record.locks` is defined with type INT, default `100`, validation: 100 to 2000 inclusive, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0388 group.share.min.record.lock.duration.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/share/ShareGroupConfig.java ShareGroupConfig.SHARE_GROUP_MIN_RECORD_LOCK_DURATION_MS_CONFIG @4.4.0
Done when: `group.share.min.record.lock.duration.ms` is defined with type INT, default `15000`, validation: 1000 to 30000 inclusive, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0389 group.share.min.session.timeout.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.SHARE_GROUP_MIN_SESSION_TIMEOUT_MS_CONFIG @4.4.0
Done when: `group.share.min.session.timeout.ms` is defined with type INT, default `45000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0390 group.share.partition.max.record.locks
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/share/ShareGroupConfig.java ShareGroupConfig.SHARE_GROUP_PARTITION_MAX_RECORD_LOCKS_CONFIG @4.4.0
Done when: `group.share.partition.max.record.locks` is defined with type INT, default `2000`, validation: 100 to 10000 inclusive, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0391 group.share.record.lock.duration.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/share/ShareGroupConfig.java ShareGroupConfig.SHARE_GROUP_RECORD_LOCK_DURATION_MS_CONFIG @4.4.0
Done when: `group.share.record.lock.duration.ms` is defined with type INT, default `30000`, validation: 1000 to 3600000 inclusive, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0392 group.share.session.timeout.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.SHARE_GROUP_SESSION_TIMEOUT_MS_CONFIG @4.4.0
Done when: `group.share.session.timeout.ms` is defined with type INT, default `45000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0393 group.streams.assignment.interval.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.STREAMS_GROUP_ASSIGNMENT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.streams.assignment.interval.ms` is defined with type INT, default `1000`, validation: >= 0, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0394 group.streams.assignors
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.STREAMS_GROUP_ASSIGNORS_CONFIG @4.4.0
Done when: `group.streams.assignors` is defined with type LIST, default `sticky`, validation: no duplicate values, empty list rejected, null rejected, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0395 group.streams.heartbeat.interval.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.STREAMS_GROUP_HEARTBEAT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.streams.heartbeat.interval.ms` is defined with type INT, default `5000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0396 group.streams.initial.rebalance.delay.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.STREAMS_GROUP_INITIAL_REBALANCE_DELAY_MS_CONFIG @4.4.0
Done when: `group.streams.initial.rebalance.delay.ms` is defined with type INT, default `3000`, validation: >= 0, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0397 group.streams.max.assignment.interval.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.STREAMS_GROUP_MAX_ASSIGNMENT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.streams.max.assignment.interval.ms` is defined with type INT, default `15000`, validation: >= 0, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0398 group.streams.max.heartbeat.interval.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.STREAMS_GROUP_MAX_HEARTBEAT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.streams.max.heartbeat.interval.ms` is defined with type INT, default `15000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0399 group.streams.max.session.timeout.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.STREAMS_GROUP_MAX_SESSION_TIMEOUT_MS_CONFIG @4.4.0
Done when: `group.streams.max.session.timeout.ms` is defined with type INT, default `60000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0400 group.streams.max.size
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.STREAMS_GROUP_MAX_SIZE_CONFIG @4.4.0
Done when: `group.streams.max.size` is defined with type INT, default `2147483647` (Integer.MAX_VALUE), validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0401 group.streams.max.standby.replicas
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.STREAMS_GROUP_MAX_STANDBY_REPLICAS_CONFIG @4.4.0
Done when: `group.streams.max.standby.replicas` is defined with type INT, default `2`, validation: >= 0, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0402 group.streams.min.assignment.interval.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.STREAMS_GROUP_MIN_ASSIGNMENT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.streams.min.assignment.interval.ms` is defined with type INT, default `0`, validation: >= 0, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0403 group.streams.min.heartbeat.interval.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.STREAMS_GROUP_MIN_HEARTBEAT_INTERVAL_MS_CONFIG @4.4.0
Done when: `group.streams.min.heartbeat.interval.ms` is defined with type INT, default `5000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0404 group.streams.min.session.timeout.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.STREAMS_GROUP_MIN_SESSION_TIMEOUT_MS_CONFIG @4.4.0
Done when: `group.streams.min.session.timeout.ms` is defined with type INT, default `45000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0405 group.streams.num.standby.replicas
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.STREAMS_GROUP_NUM_STANDBY_REPLICAS_CONFIG @4.4.0
Done when: `group.streams.num.standby.replicas` is defined with type INT, default `0`, validation: >= 0, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0406 group.streams.rack.aware.assignment.tags
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.STREAMS_GROUP_RACK_AWARE_ASSIGNMENT_TAGS_CONFIG @4.4.0
Done when: `group.streams.rack.aware.assignment.tags` is defined with type LIST, default empty, validation: no duplicate values, empty list allowed, null rejected, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0407 group.streams.session.timeout.ms
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.STREAMS_GROUP_SESSION_TIMEOUT_MS_CONFIG @4.4.0
Done when: `group.streams.session.timeout.ms` is defined with type INT, default `45000`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0408 group.streams.topology.description.plugin.class
P1 · M8 · Task · deps: CFG-0495, CFG-0002 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.STREAMS_GROUP_TOPOLOGY_DESCRIPTION_PLUGIN_CLASS_CONFIG @4.4.0
Done when: `group.streams.topology.description.plugin.class` is defined with type CLASS, default null (unset), validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); accepted values follow the JVM-config ADR, other values fail startup naming the key; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0409 offset.metadata.max.bytes
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.OFFSET_METADATA_MAX_SIZE_CONFIG @4.4.0
Done when: `offset.metadata.max.bytes` is defined with type INT, default `4096`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0410 offsets.commit.timeout.ms
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.OFFSET_COMMIT_TIMEOUT_MS_CONFIG @4.4.0
Done when: `offsets.commit.timeout.ms` is defined with type INT, default `5000`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0411 offsets.load.buffer.size
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.OFFSETS_LOAD_BUFFER_SIZE_CONFIG @4.4.0
Done when: `offsets.load.buffer.size` is defined with type INT, default `5242880`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0412 offsets.retention.check.interval.ms
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.OFFSETS_RETENTION_CHECK_INTERVAL_MS_CONFIG @4.4.0
Done when: `offsets.retention.check.interval.ms` is defined with type LONG, default `600000`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0413 offsets.retention.minutes
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.OFFSETS_RETENTION_MINUTES_CONFIG @4.4.0
Done when: `offsets.retention.minutes` is defined with type INT, default `10080`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0414 offsets.topic.compression.codec
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.OFFSETS_TOPIC_COMPRESSION_CODEC_CONFIG @4.4.0
Done when: `offsets.topic.compression.codec` is defined with type INT, default `0`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0415 offsets.topic.num.partitions
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.OFFSETS_TOPIC_PARTITIONS_CONFIG @4.4.0
Done when: `offsets.topic.num.partitions` is defined with type INT, default `50`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0416 offsets.topic.replication.factor
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.OFFSETS_TOPIC_REPLICATION_FACTOR_CONFIG @4.4.0
Done when: `offsets.topic.replication.factor` is defined with type SHORT, default `3`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0417 offsets.topic.segment.bytes
P1 · M5 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupCoordinatorConfig.java GroupCoordinatorConfig.OFFSETS_TOPIC_SEGMENT_BYTES_CONFIG @4.4.0
Done when: `offsets.topic.segment.bytes` is defined with type INT, default `104857600`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0418 producer.id.expiration.ms
P1 · M6 · Task · deps: CFG-0495 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionLogConfig.java TransactionLogConfig.PRODUCER_ID_EXPIRATION_MS_CONFIG @4.4.0
Done when: `producer.id.expiration.ms` is defined with type INT, default `86400000`, validation: >= 1, importance LOW; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0419 share.coordinator.append.linger.ms
P1 · M8 · Task · deps: CFG-0495 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorConfig.java ShareCoordinatorConfig.APPEND_LINGER_MS_CONFIG @4.4.0
Done when: `share.coordinator.append.linger.ms` is defined with type INT, default `-1`, validation: >= -1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0420 share.coordinator.cached.buffer.max.bytes
P1 · M8 · Task · deps: CFG-0495 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorConfig.java ShareCoordinatorConfig.CACHED_BUFFER_MAX_BYTES_CONFIG @4.4.0
Done when: `share.coordinator.cached.buffer.max.bytes` is defined with type INT, default `1048588`, validation: >= 524288, importance MEDIUM; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0421 share.coordinator.load.buffer.size
P1 · M8 · Task · deps: CFG-0495 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorConfig.java ShareCoordinatorConfig.LOAD_BUFFER_SIZE_CONFIG @4.4.0
Done when: `share.coordinator.load.buffer.size` is defined with type INT, default `5242880`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0422 share.coordinator.snapshot.update.records.per.snapshot
P1 · M8 · Task · deps: CFG-0495 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorConfig.java ShareCoordinatorConfig.SNAPSHOT_UPDATE_RECORDS_PER_SNAPSHOT_CONFIG @4.4.0
Done when: `share.coordinator.snapshot.update.records.per.snapshot` is defined with type INT, default `500`, validation: 0 to 500 inclusive, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0423 share.coordinator.state.topic.compression.codec
P1 · M8 · Task · deps: CFG-0495 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorConfig.java ShareCoordinatorConfig.STATE_TOPIC_COMPRESSION_CODEC_CONFIG @4.4.0
Done when: `share.coordinator.state.topic.compression.codec` is defined with type INT, default `0`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0424 share.coordinator.state.topic.min.isr
P1 · M8 · Task · deps: CFG-0495 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorConfig.java ShareCoordinatorConfig.STATE_TOPIC_MIN_ISR_CONFIG @4.4.0
Done when: `share.coordinator.state.topic.min.isr` is defined with type SHORT, default `2`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0425 share.coordinator.state.topic.num.partitions
P1 · M8 · Task · deps: CFG-0495 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorConfig.java ShareCoordinatorConfig.STATE_TOPIC_NUM_PARTITIONS_CONFIG @4.4.0
Done when: `share.coordinator.state.topic.num.partitions` is defined with type INT, default `50`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0426 share.coordinator.state.topic.replication.factor
P1 · M8 · Task · deps: CFG-0495 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorConfig.java ShareCoordinatorConfig.STATE_TOPIC_REPLICATION_FACTOR_CONFIG @4.4.0
Done when: `share.coordinator.state.topic.replication.factor` is defined with type SHORT, default `3`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0427 share.coordinator.state.topic.segment.bytes
P1 · M8 · Task · deps: CFG-0495 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorConfig.java ShareCoordinatorConfig.STATE_TOPIC_SEGMENT_BYTES_CONFIG @4.4.0
Done when: `share.coordinator.state.topic.segment.bytes` is defined with type INT, default `104857600`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0428 share.coordinator.threads
P1 · M8 · Task · deps: CFG-0495 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorConfig.java ShareCoordinatorConfig.NUM_THREADS_CONFIG @4.4.0
Done when: `share.coordinator.threads` is defined with type INT, default `1`, validation: >= 1, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0429 share.coordinator.write.timeout.ms
P1 · M8 · Task · deps: CFG-0495 · ref: share-coordinator/src/main/java/org/apache/kafka/coordinator/share/ShareCoordinatorConfig.java ShareCoordinatorConfig.WRITE_TIMEOUT_MS_CONFIG @4.4.0
Done when: `share.coordinator.write.timeout.ms` is defined with type INT, default `5000`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0430 share.fetch.purgatory.purge.interval.requests
P1 · M8 · Task · deps: CFG-0495 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/modern/share/ShareGroupConfig.java ShareGroupConfig.SHARE_FETCH_PURGATORY_PURGE_INTERVAL_REQUESTS_CONFIG @4.4.0
Done when: `share.fetch.purgatory.purge.interval.requests` is defined with type INT, default `1000`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0431 transaction.abort.timed.out.transaction.cleanup.interval.ms
P1 · M6 · Task · deps: CFG-0495 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionStateManagerConfig.java TransactionStateManagerConfig.TRANSACTIONS_ABORT_TIMED_OUT_TRANSACTION_CLEANUP_INTERVAL_MS_CONFIG @4.4.0
Done when: `transaction.abort.timed.out.transaction.cleanup.interval.ms` is defined with type INT, default `10000`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0432 transaction.max.timeout.ms
P1 · M6 · Task · deps: CFG-0495 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionStateManagerConfig.java TransactionStateManagerConfig.TRANSACTIONS_MAX_TIMEOUT_MS_CONFIG @4.4.0
Done when: `transaction.max.timeout.ms` is defined with type INT, default `900000`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0433 transaction.partition.verification.enable
P1 · M6 · Task · deps: CFG-0495 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionLogConfig.java TransactionLogConfig.TRANSACTION_PARTITION_VERIFICATION_ENABLE_CONFIG @4.4.0
Done when: `transaction.partition.verification.enable` is defined with type BOOLEAN, default `true`, validation: none beyond the type, importance LOW; dynamic update mode cluster-wide (dynamic per broker or as the cluster default); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0434 transaction.remove.expired.transaction.cleanup.interval.ms
P1 · M6 · Task · deps: CFG-0495 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionStateManagerConfig.java TransactionStateManagerConfig.TRANSACTIONS_REMOVE_EXPIRED_TRANSACTIONAL_ID_CLEANUP_INTERVAL_MS_CONFIG @4.4.0
Done when: `transaction.remove.expired.transaction.cleanup.interval.ms` is defined with type INT, default `3600000`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0435 transaction.state.log.load.buffer.size
P1 · M6 · Task · deps: CFG-0495 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionLogConfig.java TransactionLogConfig.TRANSACTIONS_LOAD_BUFFER_SIZE_CONFIG @4.4.0
Done when: `transaction.state.log.load.buffer.size` is defined with type INT, default `5242880`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0436 transaction.state.log.min.isr
P1 · M6 · Task · deps: CFG-0495 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionLogConfig.java TransactionLogConfig.TRANSACTIONS_TOPIC_MIN_ISR_CONFIG @4.4.0
Done when: `transaction.state.log.min.isr` is defined with type INT, default `2`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0437 transaction.state.log.num.partitions
P1 · M6 · Task · deps: CFG-0495 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionLogConfig.java TransactionLogConfig.TRANSACTIONS_TOPIC_PARTITIONS_CONFIG @4.4.0
Done when: `transaction.state.log.num.partitions` is defined with type INT, default `50`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0438 transaction.state.log.replication.factor
P1 · M6 · Task · deps: CFG-0495 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionLogConfig.java TransactionLogConfig.TRANSACTIONS_TOPIC_REPLICATION_FACTOR_CONFIG @4.4.0
Done when: `transaction.state.log.replication.factor` is defined with type SHORT, default `3`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0439 transaction.state.log.segment.bytes
P1 · M6 · Task · deps: CFG-0495 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionLogConfig.java TransactionLogConfig.TRANSACTIONS_TOPIC_SEGMENT_BYTES_CONFIG @4.4.0
Done when: `transaction.state.log.segment.bytes` is defined with type INT, default `104857600`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0440 transaction.two.phase.commit.enable
P3 · M6 · Task · deps: CFG-0495 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionStateManagerConfig.java TransactionStateManagerConfig.TRANSACTIONS_2PC_ENABLED_CONFIG @4.4.0
Done when: `transaction.two.phase.commit.enable` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0441 transactional.id.expiration.ms
P1 · M6 · Task · deps: CFG-0495 · ref: transaction-coordinator/src/main/java/org/apache/kafka/coordinator/transaction/TransactionStateManagerConfig.java TransactionStateManagerConfig.TRANSACTIONAL_ID_EXPIRATION_MS_CONFIG @4.4.0
Done when: `transactional.id.expiration.ms` is defined with type INT, default `604800000`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0442 consumer.assignment.interval.ms
P1 · M5 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.CONSUMER_ASSIGNMENT_INTERVAL_MS_CONFIG @4.4.0
Done when: `consumer.assignment.interval.ms` is defined with type INT, default `1000`, validation: >= 0, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0443 consumer.heartbeat.interval.ms
P1 · M5 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.CONSUMER_HEARTBEAT_INTERVAL_MS_CONFIG @4.4.0
Done when: `consumer.heartbeat.interval.ms` is defined with type INT, default `5000`, validation: >= 1, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0444 consumer.session.timeout.ms
P1 · M5 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.CONSUMER_SESSION_TIMEOUT_MS_CONFIG @4.4.0
Done when: `consumer.session.timeout.ms` is defined with type INT, default `45000`, validation: >= 1, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0445 errors.deadletterqueue.copy.record.enable
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.ERRORS_DEADLETTERQUEUE_COPY_RECORD_ENABLE_CONFIG @4.4.0
Done when: `errors.deadletterqueue.copy.record.enable` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0446 errors.deadletterqueue.topic.name
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.ERRORS_DEADLETTERQUEUE_TOPIC_NAME_CONFIG @4.4.0
Done when: `errors.deadletterqueue.topic.name` is defined with type STRING, default empty, validation: none beyond the type, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0447 share.assignment.interval.ms
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.SHARE_ASSIGNMENT_INTERVAL_MS_CONFIG @4.4.0
Done when: `share.assignment.interval.ms` is defined with type INT, default `1000`, validation: >= 0, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0448 share.auto.offset.reset
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.SHARE_AUTO_OFFSET_RESET_CONFIG @4.4.0
Done when: `share.auto.offset.reset` is defined with type STRING, default `latest`, validation: `latest`, `earliest` or `by_duration:<ISO-8601 duration>`, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0449 share.delivery.count.limit
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.SHARE_DELIVERY_COUNT_LIMIT_CONFIG @4.4.0
Done when: `share.delivery.count.limit` is defined with type INT, default `5`, validation: >= 2, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0450 share.heartbeat.interval.ms
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.SHARE_HEARTBEAT_INTERVAL_MS_CONFIG @4.4.0
Done when: `share.heartbeat.interval.ms` is defined with type INT, default `5000`, validation: >= 1, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0451 share.isolation.level
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.SHARE_ISOLATION_LEVEL_CONFIG @4.4.0
Done when: `share.isolation.level` is defined with type STRING, default `read_uncommitted`, validation: one of read_committed, read_uncommitted, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0452 share.partition.max.record.locks
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.SHARE_PARTITION_MAX_RECORD_LOCKS_CONFIG @4.4.0
Done when: `share.partition.max.record.locks` is defined with type INT, default `2000`, validation: >= 100, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0453 share.record.lock.duration.ms
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.SHARE_RECORD_LOCK_DURATION_MS_CONFIG @4.4.0
Done when: `share.record.lock.duration.ms` is defined with type INT, default `30000`, validation: >= 1000, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0454 share.renew.acknowledge.enable
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.SHARE_RENEW_ACKNOWLEDGE_ENABLE_CONFIG @4.4.0
Done when: `share.renew.acknowledge.enable` is defined with type BOOLEAN, default `true`, validation: none beyond the type, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0455 share.session.timeout.ms
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.SHARE_SESSION_TIMEOUT_MS_CONFIG @4.4.0
Done when: `share.session.timeout.ms` is defined with type INT, default `45000`, validation: >= 1, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0456 streams.assignment.interval.ms
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.STREAMS_ASSIGNMENT_INTERVAL_MS_CONFIG @4.4.0
Done when: `streams.assignment.interval.ms` is defined with type INT, default `1000`, validation: >= 0, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0457 streams.assignor.name
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.STREAMS_ASSIGNOR_NAME_CONFIG @4.4.0
Done when: `streams.assignor.name` is defined with type STRING, default null (unset), validation: none beyond the type, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0458 streams.heartbeat.interval.ms
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.STREAMS_HEARTBEAT_INTERVAL_MS_CONFIG @4.4.0
Done when: `streams.heartbeat.interval.ms` is defined with type INT, default `5000`, validation: >= 1, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0459 streams.initial.rebalance.delay.ms
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.STREAMS_INITIAL_REBALANCE_DELAY_MS_CONFIG @4.4.0
Done when: `streams.initial.rebalance.delay.ms` is defined with type INT, default `3000`, validation: >= 0, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0460 streams.num.standby.replicas
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.STREAMS_NUM_STANDBY_REPLICAS_CONFIG @4.4.0
Done when: `streams.num.standby.replicas` is defined with type INT, default `0`, validation: >= 0, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0461 streams.rack.aware.assignment.tags
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.STREAMS_RACK_AWARE_ASSIGNMENT_TAGS_CONFIG @4.4.0
Done when: `streams.rack.aware.assignment.tags` is defined with type LIST, default empty, validation: no duplicate values, empty list allowed, null rejected, importance LOW; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0462 streams.session.timeout.ms
P1 · M8 · Task · deps: CFG-0495, CFG-0043 · ref: group-coordinator/src/main/java/org/apache/kafka/coordinator/group/GroupConfig.java GroupConfig.STREAMS_SESSION_TIMEOUT_MS_CONFIG @4.4.0
Done when: `streams.session.timeout.ms` is defined with type INT, default `45000`, validation: >= 1, importance MEDIUM; updatable per group with IncrementalAlterConfigs on a GROUP resource (source DYNAMIC_GROUP_CONFIG), bounded by its broker-level min/max keys; unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

## [To Do] CFG-F10 KRaft and controller config keys
P1 · M2 · Feature · deps: CFG-F01
Scope: One task per controller, quorum and metadata key.

### [To Do] CFG-0463 broker.heartbeat.interval.ms
P1 · M2 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/KRaftConfigs.java KRaftConfigs.BROKER_HEARTBEAT_INTERVAL_MS_CONFIG @4.4.0
Done when: `broker.heartbeat.interval.ms` is defined with type INT, default `2000`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0464 broker.session.timeout.ms
P1 · M2 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/KRaftConfigs.java KRaftConfigs.BROKER_SESSION_TIMEOUT_MS_CONFIG @4.4.0
Done when: `broker.session.timeout.ms` is defined with type INT, default `9000`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0465 controller.listener.names
P1 · M2 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/KRaftConfigs.java KRaftConfigs.CONTROLLER_LISTENER_NAMES_CONFIG @4.4.0
Done when: `controller.listener.names` is defined with type LIST, default none (required), validation: no duplicate values, empty list rejected, null rejected, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0466 controller.quorum.append.linger.ms
P1 · M2 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumConfig.java QuorumConfig.QUORUM_LINGER_MS_CONFIG @4.4.0
Done when: `controller.quorum.append.linger.ms` is defined with type INT, default `25`, validation: >= 0, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0467 controller.quorum.auto.join.enable
P1 · M3 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumConfig.java QuorumConfig.QUORUM_AUTO_JOIN_ENABLE_CONFIG @4.4.0
Done when: `controller.quorum.auto.join.enable` is defined with type BOOLEAN, default `false`, validation: none beyond the type, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0468 controller.quorum.bootstrap.servers
P1 · M2 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumConfig.java QuorumConfig.QUORUM_BOOTSTRAP_SERVERS_CONFIG @4.4.0
Done when: `controller.quorum.bootstrap.servers` is defined with type LIST, default empty, validation: comma list of `host:port` entries, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0469 controller.quorum.election.backoff.max.ms
P1 · M3 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumConfig.java QuorumConfig.QUORUM_ELECTION_BACKOFF_MAX_MS_CONFIG @4.4.0
Done when: `controller.quorum.election.backoff.max.ms` is defined with type INT, default `1000`, validation: >= 0, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0470 controller.quorum.election.timeout.ms
P1 · M3 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumConfig.java QuorumConfig.QUORUM_ELECTION_TIMEOUT_MS_CONFIG @4.4.0
Done when: `controller.quorum.election.timeout.ms` is defined with type INT, default `1000`, validation: >= 0, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0471 controller.quorum.fetch.max.bytes
P1 · M3 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumConfig.java QuorumConfig.QUORUM_FETCH_MAX_BYTES_CONFIG @4.4.0
Done when: `controller.quorum.fetch.max.bytes` is defined with type INT, default `1048576`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0472 controller.quorum.fetch.snapshot.max.bytes
P1 · M3 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumConfig.java QuorumConfig.QUORUM_FETCH_SNAPSHOT_MAX_BYTES_CONFIG @4.4.0
Done when: `controller.quorum.fetch.snapshot.max.bytes` is defined with type INT, default `1048576`, validation: >= 1, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0473 controller.quorum.fetch.timeout.ms
P1 · M3 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumConfig.java QuorumConfig.QUORUM_FETCH_TIMEOUT_MS_CONFIG @4.4.0
Done when: `controller.quorum.fetch.timeout.ms` is defined with type INT, default `2000`, validation: >= 0, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0474 controller.quorum.request.timeout.ms
P1 · M3 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumConfig.java QuorumConfig.QUORUM_REQUEST_TIMEOUT_MS_CONFIG @4.4.0
Done when: `controller.quorum.request.timeout.ms` is defined with type INT, default `2000`, validation: >= 0, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0475 controller.quorum.retry.backoff.ms
P1 · M3 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumConfig.java QuorumConfig.QUORUM_RETRY_BACKOFF_MS_CONFIG @4.4.0
Done when: `controller.quorum.retry.backoff.ms` is defined with type INT, default `20`, validation: >= 0, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0476 controller.quorum.voters
P1 · M2 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumConfig.java QuorumConfig.QUORUM_VOTERS_CONFIG @4.4.0
Done when: `controller.quorum.voters` is defined with type LIST, default empty, validation: comma list of `id@host:port` entries, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0477 initial.broker.registration.timeout.ms
P1 · M2 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/KRaftConfigs.java KRaftConfigs.INITIAL_BROKER_REGISTRATION_TIMEOUT_MS_CONFIG @4.4.0
Done when: `initial.broker.registration.timeout.ms` is defined with type INT, default `60000`, validation: none beyond the type, importance MEDIUM; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0478 metadata.log.dir
P1 · M2 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/MetadataLogConfig.java MetadataLogConfig.METADATA_LOG_DIR_CONFIG @4.4.0
Done when: `metadata.log.dir` is defined with type STRING, default null (unset), validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0479 metadata.log.max.record.bytes.between.snapshots
P1 · M2 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/MetadataLogConfig.java MetadataLogConfig.METADATA_SNAPSHOT_MAX_NEW_RECORD_BYTES_CONFIG @4.4.0
Done when: `metadata.log.max.record.bytes.between.snapshots` is defined with type LONG, default `20971520`, validation: >= 1, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0480 metadata.log.max.snapshot.interval.ms
P1 · M2 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/MetadataLogConfig.java MetadataLogConfig.METADATA_SNAPSHOT_MAX_INTERVAL_MS_CONFIG @4.4.0
Done when: `metadata.log.max.snapshot.interval.ms` is defined with type LONG, default `3600000`, validation: >= 0, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0481 metadata.log.segment.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/MetadataLogConfig.java MetadataLogConfig.METADATA_LOG_SEGMENT_BYTES_CONFIG @4.4.0
Done when: `metadata.log.segment.bytes` is defined with type INT, default `1073741824`, validation: >= 8388608, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0482 metadata.log.segment.ms
P1 · M2 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/MetadataLogConfig.java MetadataLogConfig.METADATA_LOG_SEGMENT_MILLIS_CONFIG @4.4.0
Done when: `metadata.log.segment.ms` is defined with type LONG, default `604800000`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0483 metadata.max.idle.interval.ms
P1 · M2 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/MetadataLogConfig.java MetadataLogConfig.METADATA_MAX_IDLE_INTERVAL_MS_CONFIG @4.4.0
Done when: `metadata.max.idle.interval.ms` is defined with type INT, default `500`, validation: >= 0, importance LOW; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0484 metadata.max.retention.bytes
P1 · M2 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/MetadataLogConfig.java MetadataLogConfig.METADATA_MAX_RETENTION_BYTES_CONFIG @4.4.0
Done when: `metadata.max.retention.bytes` is defined with type LONG, default `104857600`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0485 metadata.max.retention.ms
P1 · M2 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/MetadataLogConfig.java MetadataLogConfig.METADATA_MAX_RETENTION_MILLIS_CONFIG @4.4.0
Done when: `metadata.max.retention.ms` is defined with type LONG, default `604800000`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0486 node.id
P1 · M2 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/KRaftConfigs.java KRaftConfigs.NODE_ID_CONFIG @4.4.0
Done when: `node.id` is defined with type INT, default none (required), validation: >= 0, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0487 process.roles
P1 · M2 · Task · deps: CFG-0495 · ref: raft/src/main/java/org/apache/kafka/raft/KRaftConfigs.java KRaftConfigs.PROCESS_ROLES_CONFIG @4.4.0
Done when: `process.roles` is defined with type LIST, default none (required), validation: each value one of [broker, controller], no duplicate values, empty list rejected, null rejected, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

### [To Do] CFG-0488 sasl.mechanism.controller.protocol
P1 · M4 · Task · deps: CFG-0495 · labels: security · ref: raft/src/main/java/org/apache/kafka/raft/KRaftConfigs.java KRaftConfigs.SASL_MECHANISM_CONTROLLER_PROTOCOL_CONFIG @4.4.0
Done when: `sasl.mechanism.controller.protocol` is defined with type STRING, default `GSSAPI`, validation: none beyond the type, importance HIGH; dynamic update mode read-only (static only); unit tests cover the default, a valid non-default value and each rejected class of value; docs/config.md lists it.

## [To Do] CFG-F11 Alluvium-specific keys
P1 · M2 · Feature · deps: CFG-F01
Scope: `alluvium.*` namespace: registry, naming rules, docs; each deviation key linked to its ADR.

### [To Do] CFG-0489 Naming rules for `alluvium.*` keys enforced by a registry test
P1 · M2 · Task · deps: CFG-0006
Done when: Rules: lowercase dot-separated, unit suffixes (`.ms`, `.bytes`), booleans end in `.enable` or read as permissions (`allow`), never shadow a Kafka key; the test fails on a violating fixture; rules documented in docs/config.md.

### [To Do] CFG-0490 Each `alluvium.*` key records its ADR and whether it changes client-visible behavior; deviation keys default to Kafka behavior
P1 · M2 · Task · deps: CFG-0489
Done when: The registry field `adr` is mandatory for alluvium.* keys; a test checks that every key marked client-visible defaults to the Kafka-compatible value and appears in the parity.md deviations table.

### [To Do] CFG-0491 Unknown `alluvium.*` keys fail startup
P1 · M2 · Task · deps: CFG-0489
Done when: Unlike unknown Kafka keys (warning only), an unknown alluvium.* key is a startup error naming the key and the closest known alluvium.* key; test.

### [To Do] CFG-0492 alluvium.allow.plaintext.external
P0 · M2 · Task · deps: CFG-0490 · labels: security · ref: ADR-0006
Done when: `alluvium.allow.plaintext.external` is defined as BOOLEAN, default false, read-only, importance HIGH, linked to ADR-0006; true restores Kafka's acceptance of plaintext listeners on non-loopback addresses; docs/config.md lists it.

### [To Do] CFG-0493 docs/config.md 'Alluvium keys' section generated and grouped by ADR
P1 · M2 · Task · deps: CFG-0490 · labels: docs
Done when: Generated from the registry with the ADR link and a client-visible marker per key.

### [To Do] CFG-0494 Dynamic `alluvium.*` keys follow the same validation and reconfiguration flow as Kafka keys
P2 · M7 · Task · deps: CFG-0490, CFG-F04
Done when: A dynamic alluvium.* key can be set with kafka-configs.sh and IncrementalAlterConfigs at broker and default level and is reported by DescribeConfigs with its source; test.

### [To Do] CFG-0498 alluvium.log.filter
P1 · M2 · Task · deps: none · ref: docs/standards/debuggability.md
Done when: `alluvium.log.filter` is registered as STRING, default `info`, `RUST_LOG` filter syntax validated at startup, dynamically updatable per broker; `ALLUVIUM_LOG` overrides it; docs/config.md lists it under Alluvium-specific keys.

### [To Do] CFG-0499 alluvium.log.format
P1 · M2 · Task · deps: none · ref: docs/standards/debuggability.md
Done when: `alluvium.log.format` is registered as STRING, one of `json` or `pretty`, default `json`, read-only; docs/config.md lists it under Alluvium-specific keys.

### [To Do] CFG-0500 alluvium.metrics.per.partition.enabled
P1 · M2 · Task · deps: none · ref: docs/standards/debuggability.md
Done when: `alluvium.metrics.per.partition.enabled` is registered as BOOLEAN, default false, dynamically updatable; enables partition labels on per-partition metric families; docs/config.md lists it under Alluvium-specific keys.

### [To Do] CFG-0501 alluvium.diagnostics.listener
P1 · M2 · Task · deps: none · ref: docs/standards/debuggability.md
Done when: `alluvium.diagnostics.listener` is registered as STRING host:port, default host `127.0.0.1` (default port fixed by the OBS-F08 design), read-only; a non-loopback address requires `alluvium.diagnostics.auth.token.file` or mTLS settings, or startup fails; docs/config.md lists it under Alluvium-specific keys.

### [To Do] CFG-0502 alluvium.diagnostics.auth.token.file
P1 · M2 · Task · deps: none · ref: docs/standards/debuggability.md
Done when: `alluvium.diagnostics.auth.token.file` is registered as STRING path to a bearer-token file, default empty, read-only, sensitive (never logged); reloaded when the file changes; docs/config.md lists it under Alluvium-specific keys.

### [To Do] CFG-0503 alluvium.flight.recorder.capacity
P1 · M2 · Task · deps: none · ref: docs/standards/debuggability.md
Done when: `alluvium.flight.recorder.capacity` is registered as INT, default 256, range 16-65536, read-only; entries per component flight recorder; docs/config.md lists it under Alluvium-specific keys.

### [To Do] CFG-0504 alluvium.shutdown.drain.timeout.ms
P1 · M2 · Task · deps: none · ref: docs/standards/debuggability.md
Done when: `alluvium.shutdown.drain.timeout.ms` is registered as LONG, default 30000, at least 0, read-only; bound on draining in-flight requests at shutdown; docs/config.md lists it under Alluvium-specific keys.

### [To Do] CFG-0505 alluvium.journal.enable
P1 · M2 · Task · deps: none · ref: docs/standards/debuggability.md
Done when: `alluvium.journal.enable` is registered as BOOLEAN, default false, read-only; turns on state-machine input journaling (SRV-F15); docs/config.md lists it under Alluvium-specific keys.

### [To Do] CFG-0506 alluvium.connections.write.stall.timeout.ms
P1 · M2 · Task · deps: none · ref: docs/standards/debuggability.md
Done when: `alluvium.connections.write.stall.timeout.ms` is registered as LONG, default 60000, at least 1000, dynamically updatable; closes connections whose writes make no progress; docs/config.md lists it under Alluvium-specific keys.

### [To Do] CFG-0507 alluvium.connections.authentication.timeout.ms
P1 · M2 · Task · deps: none · ref: docs/standards/debuggability.md
Done when: `alluvium.connections.authentication.timeout.ms` is registered as LONG, default 10000, at least 100, dynamically updatable; closes connections that don't finish authentication in time; docs/config.md lists it under Alluvium-specific keys.

## [To Do] CFG-F12 Config documentation generation
P2 · M2 · Feature · deps: CFG-F01, DOCS-F01
Scope: Generate docs/config.md from the config definitions; CI drift check.

### [To Do] CFG-0495 Generate docs/config.md from the registry
P1 · M2 · Task · deps: CFG-0006 · labels: docs · ref: clients/src/main/java/org/apache/kafka/common/config/ConfigDef.java toHtml @4.4.0
Done when: `cargo xtask docs-config` writes one section per area (broker, topic, group, KRaft, Alluvium) with type, default, valid values, importance, dynamic update mode and the Kafka documentation text (with Apache-2.0 attribution) per key; output is deterministic with a stable anchor per key.

### [To Do] CFG-0496 CI drift check for docs/config.md
P1 · M2 · Task · deps: CFG-0495, FND-F06 · labels: docs
Done when: `cargo xtask ci` fails when docs/config.md differs from the generator output and prints the regeneration command.

### [To Do] CFG-0497 Per-key Alluvium status column in docs/config.md
P2 · M2 · Task · deps: CFG-0495 · labels: docs
Done when: Each key shows supported, supported with an ADR deviation (linked), or not yet consumed (with its milestone), generated from registry metadata.
