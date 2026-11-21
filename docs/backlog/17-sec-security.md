# SEC: Security

> Crates: `alluvium-security` (layer 1), `alluvium-network` · Kafka counterpart: `common/security`, `metadata/authorizer/StandardAuthorizer` · Milestones: M2-M9

Principals, SASL mechanisms, the ACL authorizer, delegation tokens, secret handling, re-authentication, inter-node security and the ongoing security review process.

## [To Do] SEC-F01 Principal model
P0 · M2 · Feature · deps: TYPE-F04
Scope: `KafkaPrincipal` (type:name), principal builder, SSL principal mapping rules, SASL principal mapping.

### [To Do] SEC-0001 `KafkaPrincipal`: type and name, `User:ANONYMOUS`, parsing `Type:name` and token-authenticated flag
P0 · M2 · Task · deps: none · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/auth/KafkaPrincipal.java @4.4.0
Done when: Parsing splits on the first ':' and rejects missing parts with Kafka's message; equality uses type and name only; `Display` matches Kafka's `toString`; unit tests ported from Kafka.

### [To Do] SEC-0002 Principal serde for forwarded requests (DefaultPrincipalData)
P0 · M2 · Task · deps: SEC-0001, TEST-F07 · labels: security, decoder · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/DefaultKafkaPrincipalBuilder.java serialize, deserialize @4.4.0
Done when: Principals serialize and deserialize in Kafka's versioned `DefaultPrincipalData` format (type, name, token-authenticated) for Envelope; bytes match Kafka's for the same principal; decoding untrusted bytes never panics; fuzz target added.

### [To Do] SEC-0003 Parse `ssl.principal.mapping.rules`: `RULE:pattern/replacement/[LU]` and `DEFAULT`
P0 · M2 · Task · deps: none · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/ssl/SslPrincipalMapper.java parseRules @4.4.0
Done when: Rules parse with Kafka's syntax and escaping; invalid rules fail startup naming the rule; tests ported from SslPrincipalMapperTest.

### [To Do] SEC-0004 Java-regex compatibility for mapping rules: supported subset and startup rejection of unsupported constructs
P0 · M2 · Task · deps: SEC-0003 · labels: security
Done when: The regex engine choice is justified per the dependency policy; patterns using Java-only constructs that the engine cannot express fail startup with a clear message; docs list the supported syntax; tests with common real-world rules.

### [To Do] SEC-0005 Apply mapping rules: first match wins, group substitution, lower/upper case, DEFAULT passthrough
P0 · M2 · Task · deps: SEC-0004 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/ssl/SslPrincipalMapper.java getName @4.4.0
Done when: Mapping returns the first matching rule's result with `$n` substitution and L/U case conversion; DEFAULT returns the DN unchanged; no match is an authentication failure as in Kafka; table tests.

### [To Do] SEC-0006 Principal builder: ANONYMOUS for plaintext, mapped DN for TLS, authorization id for SASL
P0 · M2 · Task · deps: SEC-0001, SEC-0005 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/DefaultKafkaPrincipalBuilder.java build @4.4.0
Done when: PLAINTEXT yields `User:ANONYMOUS`; SSL yields the mapped DN or ANONYMOUS without a client certificate; SASL yields `User:<authorization id>` with the token-authenticated flag for delegation tokens; GSSAPI is delegated to the Kerberos short-namer when supported; tests per protocol.

### [To Do] SEC-0007 Reject custom `principal.builder.class` values
P0 · M2 · Task · deps: SEC-0006, CFG-F07 · labels: security · ref: core/src/main/scala/kafka/server/KafkaConfig.scala validateValues (principal.builder.class) @4.4.0
Done when: A non-default `principal.builder.class` fails startup with a message explaining the deviation (no JVM plug-ins) and pointing to mapping rules; deviation recorded in docs/parity.md; test.

### [To Do] SEC-0008 Document principals and mapping rules
P1 · M2 · Task · deps: SEC-0005 · labels: docs
Done when: Docs describe principal types, the mapping rule syntax, the supported regex subset and examples; doc build passes.

## [To Do] SEC-F02 SASL framework
P0 · M4 · Feature · deps: SEC-F01
Scope: Sans-I/O mechanism state machines, per-listener mechanism config, server callbacks.

### [To Do] SEC-0009 Sans-I/O SASL server mechanism interface with challenge, success and failure outcomes
P0 · M4 · Task · deps: none · labels: security
Done when: A mechanism consumes client tokens and returns a challenge, success (authorization id, negotiated credential lifetime) or failure (client-safe message and internal reason); it reads credentials from an injected snapshot, does no I/O and never reads the clock directly; unit tests with a test mechanism.

### [To Do] SEC-0010 Sans-I/O SASL client mechanism interface for inter-node connections
P0 · M4 · Task · deps: SEC-0009 · labels: security
Done when: Client mechanisms produce the initial token, answer challenges and verify the server's final message; used by SEC-F16; unit tests with a test mechanism.

### [To Do] SEC-0011 Per-listener `sasl.enabled.mechanisms` and inter-broker mechanism validation
P0 · M4 · Task · deps: SEC-0009, CFG-F07 · labels: security · ref: core/src/main/scala/kafka/server/KafkaConfig.scala validateValues (saslEnabledMechanisms) @4.4.0
Done when: Each SASL listener's enabled set comes from the listener-prefixed or global key; unsupported names fail startup; `sasl.mechanism.inter.broker.protocol` must be enabled on the inter-broker listener (Kafka validation); Kafka's default (GSSAPI) is handled per SEC-F06's decision; tests.

### [To Do] SEC-0012 Parse `sasl.jaas.config` (JAAS syntax) for supported login modules
P0 · M4 · Task · deps: SEC-0011, SEC-F11 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/JaasConfig.java, clients/src/main/java/org/apache/kafka/common/security/JaasContext.java loadServerContext @4.4.0
Done when: Listener- and mechanism-prefixed `sasl.jaas.config` values parse (module, control flag, quoted options, terminating ';'); PlainLoginModule, ScramLoginModule and OAuthBearerLoginModule are recognized; other modules fail startup; option values are `Secret`; parser tests ported from JaasConfig cases.

### [To Do] SEC-0013 Reject JVM plug-in SASL settings with a documented deviation
P0 · M4 · Task · deps: SEC-0011 · labels: security
Done when: Non-default `sasl.server.callback.handler.class`, `sasl.login.class` and `sasl.login.callback.handler.class` fail startup with a message naming the built-in alternative; deviation in docs/parity.md; tests.

### [To Do] SEC-0014 Credential snapshot interface fed from the metadata image
P0 · M4 · Task · deps: SEC-0009 · labels: security · ref: server-common/src/main/java/org/apache/kafka/security/CredentialProvider.java @4.4.0
Done when: Mechanisms read SCRAM credentials and delegation tokens through an immutable snapshot swapped on publish; a lookup never blocks on I/O; tests swap snapshots during authentication.

### [To Do] SEC-0015 Authentication error model: client-safe messages, internal reasons, SASL_AUTHENTICATION_FAILED mapping
P0 · M4 · Task · deps: SEC-0009 · labels: security
Done when: Each failure carries a client message identical to Kafka's for that mechanism and an internal reason enum (UnknownUser, BadCredentials, ExpiredCredential, InvalidMessage, MechanismError); only the message reaches the client; one exhaustive mapping to error codes; tests.

### [To Do] SEC-0016 Constant-time comparison helper for credentials
P0 · M4 · Task · deps: SEC-0009 · labels: security
Done when: All credential and proof comparisons go through one helper built on `subtle`; the security review checklist has an item for it; unit tests of the helper.

### [To Do] SEC-0017 Authentication events with reasons
P0 · M4 · Task · deps: SEC-0015, OBS-F07 · labels: security, nfr
Done when: `authentication_succeeded` (debug) and `authentication_failed{listener, mechanism, reason}` (info) are in the events catalog; no credentials or tokens are logged; tests capture both.

### [To Do] SEC-0018 SASL framework module docs and configuration docs
P1 · M4 · Task · deps: SEC-0012, SEC-0013 · labels: docs
Done when: Module docs state invariants (sans-I/O, no secret logging); docs list supported mechanisms, JAAS options per module and the plug-in deviation; doc build passes.

## [To Do] SEC-F03 SASL/PLAIN
P0 · M4 · Feature · deps: SEC-F02
Scope: PLAIN mechanism, user credential configuration, constant-time checks.

### [To Do] SEC-0019 Parse PLAIN messages `[authzid] NUL authcid NUL passwd` (RFC 4616)
P0 · M4 · Task · deps: TEST-F07 · labels: security, decoder · ref: clients/src/main/java/org/apache/kafka/common/security/plain/internals/PlainSaslServer.java evaluateResponse @4.4.0
Done when: Exactly three fields are required; empty username or password and an authorization id different from the username fail with Kafka's messages; fuzz target `sasl_plain` runs clean.

### [To Do] SEC-0020 PLAIN user credentials from JAAS `user_<name>="<password>"` options
P0 · M4 · Task · deps: none · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/plain/internals/PlainServerCallbackHandler.java @4.4.0
Done when: Users and passwords load from the listener's PlainLoginModule options as secrets; reload follows dynamic JAAS updates when CFG-F04 lands; tests.

### [To Do] SEC-0021 Constant-time password verification with indistinguishable unknown-user and wrong-password outcomes
P0 · M4 · Task · deps: SEC-0019, SEC-0020 · labels: security
Done when: Unknown user and wrong password both return Kafka's 'Authentication failed: Invalid username or password' and do comparable work (comparison against a dummy secret); tests.

### [To Do] SEC-0022 Refuse PLAIN on non-loopback SASL_PLAINTEXT listeners unless explicitly allowed
P0 · M4 · Task · deps: SEC-0020, CFG-F11 · labels: security · ref: ADR-0006
Done when: Startup fails when PLAIN is enabled on a SASL_PLAINTEXT listener bound to a non-loopback address without the `alluvium.*` opt-in; deviation recorded in ADR-0006; tests.

### [To Do] SEC-0023 PLAIN client mechanism for inter-node connections
P0 · M4 · Task · deps: SEC-0021 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/plain/PlainLoginModule.java @4.4.0
Done when: The client sends `NUL username NUL password` from the inter-broker listener's JAAS options and handles failure; tests against the server mechanism.

### [To Do] SEC-0024 Compatibility tests: Java client and librdkafka with SASL_PLAINTEXT and SASL_SSL PLAIN
P0 · M4 · Task · deps: SEC-0021, TEST-F08 · labels: test, security
Done when: Both clients authenticate, produce and consume; a wrong password fails with the same error code and message as Kafka 4.4.0.

## [To Do] SEC-F04 SASL/SCRAM
P0 · M4 · Feature · deps: SEC-F02, CTRL-F14
Scope: SCRAM-SHA-256/512, nonce handling, credential lookup from metadata, iterations.

### [To Do] SEC-0025 SCRAM primitives: Hi (PBKDF2), salted password, client, stored and server keys, signatures
P0 · M4 · Task · deps: none · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/scram/internals/ScramFormatter.java @4.4.0
Done when: Functions for SHA-256 and SHA-512 match RFC 5802/7677 test vectors and Kafka's ScramFormatter outputs; dependency choice justified per policy; unit tests.

### [To Do] SEC-0026 SCRAM message codecs: client-first, server-first, client-final, server-final
P0 · M4 · Task · deps: TEST-F07 · labels: security, decoder · ref: clients/src/main/java/org/apache/kafka/common/security/scram/internals/ScramMessages.java @4.4.0
Done when: Parsing follows Kafka's patterns (gs2 header, authzid, saslname escaping `=2C`/`=3D`, nonce, extensions, channel binding `biws`, proof, error); invalid messages are typed errors; fuzz target `sasl_scram_messages` runs clean.

### [To Do] SEC-0027 SCRAM-SHA-256 and SCRAM-SHA-512 with minimum and maximum iterations
P0 · M4 · Task · deps: SEC-0025 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/scram/internals/ScramMechanism.java @4.4.0
Done when: Both mechanisms exist with Kafka's iteration bounds (4096 to 16384); stored credentials below the minimum fail authentication with Kafka's message; tests.

### [To Do] SEC-0028 SCRAM server state machine: first message, final message, proof verification
P0 · M4 · Task · deps: SEC-0026, SEC-0027 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/scram/internals/ScramSaslServer.java @4.4.0
Done when: States RECEIVE_CLIENT_FIRST_MESSAGE, RECEIVE_CLIENT_FINAL_MESSAGE, COMPLETE and FAILED; server nonce appended from the injected RNG; authorization id must equal the username; final nonce must match; proof compared in constant time; server signature returned; each failure has Kafka's message; tests.

### [To Do] SEC-0029 SCRAM credential snapshot from the metadata image
P0 · M4 · Task · deps: SEC-0028 · labels: security · ref: metadata/src/main/java/org/apache/kafka/metadata/publisher/ScramPublisher.java @4.4.0
Done when: SCRAM credential records in the image update the credential snapshot (add, change, delete) in publisher order; authentication right after a credential change uses the new credential; tests.

### [To Do] SEC-0030 SCRAM client mechanism for inter-node connections
P0 · M4 · Task · deps: SEC-0025, SEC-0026 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/scram/internals/ScramSaslClient.java @4.4.0
Done when: The client computes the proof, verifies the server signature and rejects a server nonce that does not extend its own; tests against the server mechanism.

### [To Do] SEC-0031 Compatibility tests: SCRAM-SHA-256 and SCRAM-SHA-512 with credentials created by `kafka-configs.sh`
P0 · M4 · Task · deps: SEC-0029, TEST-F08 · labels: test, security
Done when: Credentials created through AlterUserScramCredentials let the Java client and librdkafka authenticate; deleted credentials fail; errors match Kafka 4.4.0.

### [To Do] SEC-0032 Document SCRAM setup and credential management
P1 · M4 · Task · deps: SEC-0029 · labels: docs
Done when: Docs cover creating credentials at format time and at runtime, iterations and mechanisms; doc build passes.

## [To Do] SEC-F05 SASL/OAUTHBEARER
P0 · M4 · Feature · deps: SEC-F02
Scope: Unsecured JWS for development, JWKS validation, claims, scopes, expiry and re-authentication.

### [To Do] SEC-0033 Parse the OAUTHBEARER client initial response (GS2 header, `auth=Bearer`, extensions)
P0 · M4 · Task · deps: TEST-F07 · labels: security, decoder · ref: clients/src/main/java/org/apache/kafka/common/security/oauthbearer/internals/OAuthBearerClientInitialResponse.java @4.4.0
Done when: The `n,[a=authzid],` header, `\x01`-separated key/value pairs and the mandatory `auth` value parse with Kafka's validation of extension names and values; invalid messages are typed errors; fuzz target runs clean.

### [To Do] SEC-0034 OAUTHBEARER server mechanism: validation, JSON error challenge and failure
P0 · M4 · Task · deps: SEC-0033 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/oauthbearer/internals/OAuthBearerSaslServer.java @4.4.0
Done when: A valid token completes with the principal; an invalid token returns the `{"status":"invalid_token"...}` challenge, waits for the client's `\x01` and then fails; an authorization id different from the principal fails; tests.

### [To Do] SEC-0035 Bounded JWT parsing: header, payload and signature segments
P0 · M4 · Task · deps: TEST-F07 · labels: security, decoder
Done when: Compact JWS parsing bounds segment sizes, decodes base64url and JSON with depth and size limits and never panics; fuzz target runs clean.

### [To Do] SEC-0036 Unsecured JWS validator for development with Kafka's options
P0 · M4 · Task · deps: SEC-0034, SEC-0035, CFG-F11 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/oauthbearer/internals/unsecured/OAuthBearerUnsecuredValidatorCallbackHandler.java @4.4.0
Done when: Tokens with `alg: none` validate per `unsecuredValidatorPrincipalClaimName`, `unsecuredValidatorScopeClaimName`, `unsecuredValidatorRequiredScope` and `unsecuredValidatorAllowableClockSkewMs`; it is refused on non-loopback listeners unless an `alluvium.*` opt-in is set (ADR-0006 deviation, Kafka's default is unsecured); tests.

### [To Do] SEC-0037 Secure JWT validation: signature, exp, iat, issuer, audience, clock skew, subject and scope claims
P0 · M4 · Task · deps: SEC-0034, SEC-0035 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/oauthbearer/BrokerJwtValidator.java, clients/src/main/java/org/apache/kafka/common/security/oauthbearer/internals/secured/ClaimValidationUtils.java @4.4.0
Done when: Tokens are verified against cached keys with an algorithm allow-list; `exp` and `iat` are required; `sasl.oauthbearer.expected.issuer`, `sasl.oauthbearer.expected.audience`, `sasl.oauthbearer.clock.skew.seconds`, `sasl.oauthbearer.sub.claim.name` and `sasl.oauthbearer.scope.claim.name` apply as in Kafka; each failure has a test.

### [To Do] SEC-0038 JWKS from a `file:` URL with reload on change
P0 · M4 · Task · deps: SEC-0037 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/oauthbearer/internals/secured/JwksFileVerificationKeyResolver.java @4.4.0
Done when: `sasl.oauthbearer.jwks.endpoint.url` with a file URL loads keys at startup and reloads when the file changes; bad files keep the previous keys and log a warn event; tests.

### [To Do] SEC-0039 JWKS over HTTPS with background refresh, unknown-kid refresh and no network on the request path
P0 · M4 · Task · deps: SEC-0037, NET-F07 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/oauthbearer/internals/secured/RefreshingHttpsJwks.java @4.4.0
Done when: Keys are fetched at startup and refreshed every `sasl.oauthbearer.jwks.endpoint.refresh.ms` with `..retry.backoff.ms`/`..retry.backoff.max.ms`; an unknown `kid` triggers a rate-limited background refresh while the request fails; allowed URLs are restricted by config; the HTTP client dependency is justified; tests with a local JWKS server.

### [To Do] SEC-0040 SASL extensions: keep only validated extensions
P0 · M4 · Task · deps: SEC-0034 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/oauthbearer/internals/OAuthBearerSaslServer.java processExtensions @4.4.0
Done when: Extensions sent by the client are kept only when validated (none by default) and ignored otherwise, as Kafka's server does; tests.

### [To Do] SEC-0041 Token expiry becomes the negotiated credential lifetime for session expiry
P0 · M4 · Task · deps: SEC-0037, NET-F08 · labels: security
Done when: The session lifetime from NET-F08 uses the token's `exp`; a token near expiry yields a short session; tests.

### [To Do] SEC-0042 JWKS refresh events and metrics
P1 · M4 · Task · deps: SEC-0039, OBS-F04 · labels: nfr
Done when: `jwks_refreshed` (debug), `jwks_refresh_failed` (warn) and `alluvium_sasl_oauth_jwks_refresh_total{result}` exist; tests.

### [To Do] SEC-0043 OAUTHBEARER client for inter-node connections with client-credentials token retrieval
P2 · M4 · Task · deps: SEC-0034, SEC-0039 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/oauthbearer/ClientCredentialsJwtRetriever.java @4.4.0
Done when: Inter-node connections obtain tokens from `sasl.oauthbearer.token.endpoint.url` with client id and secret, refresh before expiry and send them in the initial response; tests with a local token endpoint.

### [To Do] SEC-0044 Compatibility tests: Java client OIDC and unsecured modes, librdkafka OIDC
P0 · M4 · Task · deps: SEC-0039, SEC-0036, TEST-F08 · labels: test, security
Done when: Clients authenticate against a test identity provider; expired, wrong-issuer and wrong-audience tokens fail exactly as with Kafka 4.4.0.

### [To Do] SEC-0045 Document OAUTHBEARER configuration
P1 · M4 · Task · deps: SEC-0039 · labels: docs
Done when: Docs cover unsecured mode limits, JWKS file and HTTPS setups, claims and clock skew; doc build passes.

## [To Do] SEC-F06 SASL/GSSAPI
P3 · M4 · Feature · deps: SEC-F02
Scope: Spike: native Kerberos vs FFI exception; implementation if accepted.

### [To Do] SEC-0046 Spike: GSSAPI/Kerberos via native Rust, a reviewed FFI exception, or not supported
P3 · M4 · Spike · deps: none · labels: security
Done when: ADR compares a native acceptor, an FFI exception to `forbid(unsafe_code)` and non-support (with migration guidance), and decides; accepted ADR; architecture §10 updated.

### [To Do] SEC-0047 Handle GSSAPI configuration per the ADR, including Kafka's default `sasl.enabled.mechanisms=GSSAPI`
P3 · M4 · Task · deps: SEC-0046 · labels: security
Done when: If unsupported, enabling GSSAPI fails startup with a docs link and Alluvium's default enabled set excludes it (deviation recorded); if supported, this task only records the default; tests.

### [To Do] SEC-0048 Kerberos principal-to-local rules (`sasl.kerberos.principal.to.local.rules`)
P3 · M4 · Task · deps: SEC-0046, SEC-0004 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/kerberos/KerberosShortNamer.java, clients/src/main/java/org/apache/kafka/common/security/kerberos/KerberosRule.java @4.4.0
Done when: `RULE:[n:string](regexp)s/pattern/replacement/g[L|U]` and `DEFAULT` parse and apply as Kafka's KerberosShortNamer; tests ported from Kafka; Won't Do if the ADR rejects GSSAPI.

### [To Do] SEC-0049 GSSAPI server mechanism per the ADR
P3 · M4 · Task · deps: SEC-0048 · labels: security
Done when: Accepts Kerberos tokens for `sasl.kerberos.service.name` with the configured keytab and builds the principal through the short-namer; Won't Do if the ADR rejects GSSAPI.

### [To Do] SEC-0050 Compatibility test with a KDC container
P3 · M4 · Task · deps: SEC-0049, TEST-F08 · labels: test, security
Done when: Java client and librdkafka authenticate with Kerberos against Alluvium in the compat harness; Won't Do if the ADR rejects GSSAPI.

## [To Do] SEC-F07 ACL authorizer
P0 · M4 · Feature · deps: SEC-F01, META-F03
Scope: StandardAuthorizer semantics: literal, prefixed and wildcard patterns, deny precedence, super users, allow-if-no-ACL setting, host matching.

### [To Do] SEC-0051 ACL model: resource types, pattern types, operations, permission types and `StandardAcl` with Kafka codes
P0 · M4 · Task · deps: none · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAcl.java, clients/src/main/java/org/apache/kafka/common/acl/AclOperation.java @4.4.0
Done when: Enums carry Kafka's byte codes (operations up to TWO_PHASE_COMMIT, resource types up to USER); `StandardAcl` holds resource, pattern, principal, host, operation and permission; conversions from metadata records; unit tests.

### [To Do] SEC-0052 ACL binding validation rules shared with the controller
P0 · M4 · Task · deps: SEC-0051 · ref: metadata/src/main/java/org/apache/kafka/controller/AclControlManager.java validateNewAcl, validateHostPattern @4.4.0
Done when: Validation rejects ANY/UNKNOWN resource, pattern, operation and permission types, empty resource names, unparsable principals and bad host patterns, and returns UNSUPPORTED_VERSION for CIDR hosts below the supporting metadata version (KIP-1276), each with Kafka's message; tests per rule.

### [To Do] SEC-0053 Immutable ACL index built from the image with incremental add and remove
P0 · M4 · Task · deps: SEC-0051 · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizerData.java addAcl, removeAcl, metadata/src/main/java/org/apache/kafka/metadata/authorizer/AclCache.java @4.4.0
Done when: ACLs are indexed by resource type and name for literal, wildcard and prefix lookup; deltas add and remove by id; each image gets an immutable index; lookup cost is logarithmic in ACL count; tests.

### [To Do] SEC-0054 Literal ACLs match the exact resource name
P0 · M4 · Task · deps: SEC-0053 · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizerData.java checkSection @4.4.0
Done when: A LITERAL ACL matches only when the names are equal; tests including names that share prefixes.

### [To Do] SEC-0055 Wildcard `*` literal ACLs match every resource of the type
P0 · M4 · Task · deps: SEC-0053 · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizerData.java findAclRule (WILDCARD) @4.4.0
Done when: A LITERAL ACL named `*` matches any name of its resource type; tests.

### [To Do] SEC-0056 Prefixed ACLs match resource names that start with the prefix
P0 · M4 · Task · deps: SEC-0053 · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizerData.java matchesUpTo, checkSection @4.4.0
Done when: A PREFIXED ACL matches every name starting with its name; nested prefixes all apply; tests with overlapping prefixes.

### [To Do] SEC-0057 Principal matching: exact principal or `User:*`
P0 · M4 · Task · deps: SEC-0051 · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizerData.java matchingPrincipals @4.4.0
Done when: An ACL applies to the request principal or to the wildcard `User:*`; principal type is significant; tests.

### [To Do] SEC-0058 Host matching: `*`, exact address and CIDR ranges (KIP-1276)
P0 · M4 · Task · deps: SEC-0051 · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizerData.java findResult, clients/src/main/java/org/apache/kafka/server/authorizer/internals/CidrUtils.java isInRange @4.4.0
Done when: An ACL host matches `*`, the client IP string exactly, or a CIDR range containing it (IPv4 inclusive host count, IPv6); invalid CIDR never matches; tests for each form.

### [To Do] SEC-0059 Operation matching with ALL and implied DESCRIBE/DESCRIBE_CONFIGS for ALLOW ACLs only
P0 · M4 · Task · deps: SEC-0051 · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizerData.java findResult, IMPLIES_DESCRIBE, IMPLIES_DESCRIBE_CONFIGS @4.4.0
Done when: ALL matches any operation; an ALLOW for READ, WRITE, DELETE or ALTER implies DESCRIBE; an ALLOW for ALTER_CONFIGS implies DESCRIBE_CONFIGS; DENY ACLs imply nothing (DENY READ does not deny DESCRIBE); table tests.

### [To Do] SEC-0060 DENY takes precedence over ALLOW regardless of pattern specificity
P0 · M4 · Task · deps: SEC-0054, SEC-0055, SEC-0056, SEC-0057, SEC-0058, SEC-0059 · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizerData.java MatchingRuleBuilder.build @4.4.0
Done when: Any matching DENY (literal, wildcard or prefixed) wins over any matching ALLOW; tests combining patterns.

### [To Do] SEC-0061 `super.users` bypass all ACLs
P0 · M4 · Task · deps: SEC-0051, CFG-F07 · labels: security · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizer.java configure, metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizerData.java authorize @4.4.0
Done when: Principals listed in `super.users` (semicolon-separated) are allowed before any ACL lookup, even with matching DENY ACLs; tests.

### [To Do] SEC-0062 `allow.everyone.if.no.acl.found`: default result only when the resource has no ACLs at all
P0 · M4 · Task · deps: SEC-0060 · labels: security · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizerData.java MatchingRuleBuilder.build (hasResourceAcls) @4.4.0
Done when: With no matching-resource ACLs the result is the configured default (deny by default); if ACLs exist for the resource but none match the principal, host or operation, the result is DENY; tests for both.

### [To Do] SEC-0063 Cluster resource semantics (`kafka-cluster`)
P0 · M4 · Task · deps: SEC-0054 · ref: clients/src/main/java/org/apache/kafka/common/resource/Resource.java CLUSTER_NAME @4.4.0
Done when: CLUSTER actions are checked against the literal resource `kafka-cluster`; wildcard and prefixed cluster ACLs behave as Kafka; tests.

### [To Do] SEC-0064 `authorize_by_resource_type`: any-resource checks with deny-pattern precedence
P0 · M4 · Task · deps: SEC-0060, SEC-0061 · ref: clients/src/main/java/org/apache/kafka/server/authorizer/Authorizer.java authorizeByResourceType @4.4.0
Done when: Super users are allowed; otherwise allow only if some ALLOW pattern (literal, prefixed or wildcard) for the principal and host is not covered by a DENY pattern, as Kafka's default implementation; used for IDEMPOTENT_WRITE-style checks; table tests.

### [To Do] SEC-0065 Authorizer readiness: AUTHORIZER_NOT_READY before the initial load; early-start listeners not gated
P0 · M4 · Task · deps: SEC-0061, SEC-0066 · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizer.java start, completeInitialLoad @4.4.0
Done when: Before the first ACL image is loaded non-super users get AuthorizerNotReady; listeners in `early.start.listeners` open immediately while others wait for the initial load; tests.

### [To Do] SEC-0066 ACL publisher: apply ACL deltas and snapshots from the image
P0 · M4 · Task · deps: SEC-0053, SRV-F14 · ref: metadata/src/main/java/org/apache/kafka/metadata/publisher/AclPublisher.java @4.4.0
Done when: Image deltas add and remove ACLs, snapshots replace the set, and the initial load completes the readiness signal; publish order per SRV-F14; tests.

### [To Do] SEC-0067 Authorizer benchmark with 100k ACLs
P2 · M4 · Task · deps: SEC-0062 · labels: perf
Done when: Criterion bench measures authorize latency for literal, prefixed and wildcard-heavy ACL sets; p99 under the budget recorded in docs; regression threshold in CI.

### [To Do] SEC-0068 Document authorizer semantics
P1 · M4 · Task · deps: SEC-0062, SEC-0064 · labels: docs
Done when: Docs explain matching, precedence, defaults, super users, host and CIDR rules with examples; doc build passes.

## [To Do] SEC-F08 Authorization integration
P0 · M4 · Feature · deps: SEC-F07
Scope: Authorization helper used by all handlers, `authz_decision` events, implicit DESCRIBE rules.

### [To Do] SEC-0069 Authorization helper for handlers: single and batch checks, filter-by-authorized, by-resource-type
P0 · M4 · Task · deps: none · labels: security · ref: server/src/main/java/org/apache/kafka/server/AuthHelper.java authorize, filterByAuthorized, authorizeByResourceType @4.4.0
Done when: Handlers call one helper with request context, operation and resources (with reference counts and log flags); batch checks preserve order; `filter_authorized` returns permitted names; tests.

### [To Do] SEC-0070 `authz_decision` events: denials at info, allows at debug when requested, otherwise trace
P0 · M4 · Task · deps: SEC-0069, OBS-F07 · labels: security, nfr · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizerData.java logAuditMessage, buildAuditMessage @4.4.0
Done when: Each decision emits `authz_decision` under target `alluvium::authorizer` with principal, host, operation, resource type, name and pattern, api, correlation id and the matching rule (SuperUser, DefaultAllow, DefaultDeny or the ACL), at Kafka's levels; tests capture each level.

### [To Do] SEC-0071 Authorizer metrics: requests, allowed, denied and ACL count
P1 · M4 · Task · deps: SEC-0069, OBS-F04 · labels: nfr · ref: metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizer.java withPluginMetrics @4.4.0
Done when: `alluvium_authorizer_decisions_total{result}` and `alluvium_authorizer_acls` map StandardAuthorizer's plug-in metrics; tests.

### [To Do] SEC-0072 Authorization table: every API's operation and resource checks as in KafkaApis and ControllerApis
P0 · M4 · Task · deps: SEC-0069 · labels: security, docs · ref: core/src/main/scala/kafka/server/KafkaApis.scala, core/src/main/scala/kafka/server/ControllerApis.scala @4.4.0
Done when: A single table lists, per API and version range, the checks Kafka performs (with `// Parity:` sources) and is rendered into docs/security; doc check verifies every advertised API has an entry.

### [To Do] SEC-0073 Recording authorizer test harness to verify handlers against the table
P0 · M4 · Task · deps: SEC-0072 · labels: test, security
Done when: A test authorizer records every check; a harness runs a request per API and asserts the recorded checks equal the table entry; handlers adopt it in their tests.

### [To Do] SEC-0074 Authorization of forwarded requests uses the envelope's inner principal
P0 · M4 · Task · deps: SEC-0069, SRV-F06 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala handleEnvelopeRequest @4.4.0
Done when: Requests unwrapped from Envelope are authorized with the serialized client principal and client address, while Envelope itself requires CLUSTER_ACTION from a privileged listener; tests.

### [To Do] SEC-0075 Per-request authorization overhead benchmark
P2 · M4 · Task · deps: SEC-0069 · labels: perf
Done when: Benchmark of a produce with 100 partitions and 10k ACLs records the added latency; result in docs.

## [To Do] SEC-F09 ACL administration
P0 · M4 · Feature · deps: SEC-F07, CTRL-F13
Scope: Describe/Create/DeleteAcls filter semantics and results.

### [To Do] SEC-0076 ACL filter semantics: ANY, MATCH and null fields for resource patterns and entries
P0 · M4 · Task · deps: none · labels: security · ref: clients/src/main/java/org/apache/kafka/common/resource/ResourcePatternFilter.java matches, clients/src/main/java/org/apache/kafka/common/acl/AclBindingFilter.java @4.4.0
Done when: `ResourcePatternFilter` and `AccessControlEntryFilter` matching follow Kafka: ANY matches every pattern type, MATCH matches the literal name, `*` and prefixes that cover the name, null fields match anything; tests ported from Kafka.

### [To Do] SEC-0077 Filter validation errors
P0 · M4 · Task · deps: SEC-0076 · ref: metadata/src/main/java/org/apache/kafka/controller/AclControlManager.java validateFilter @4.4.0
Done when: Filters with UNKNOWN resource, pattern, operation or permission types fail with INVALID_REQUEST and Kafka's messages; tests.

### [To Do] SEC-0078 DescribeAcls semantics: filter the current ACL set and group results by resource pattern
P0 · M4 · Task · deps: SEC-0077 · ref: core/src/main/scala/kafka/server/AclApis.scala handleDescribeAcls, metadata/src/main/java/org/apache/kafka/metadata/authorizer/StandardAuthorizer.java acls @4.4.0
Done when: Describe returns matching bindings grouped by resource with entries, reading one image version; empty results are not errors; tests.

### [To Do] SEC-0079 CreateAcls semantics: per-creation results and idempotent duplicates
P0 · M4 · Task · deps: SEC-0052 · ref: metadata/src/main/java/org/apache/kafka/controller/AclControlManager.java createAcls @4.4.0
Done when: Each creation gets its own result; creating an ACL identical to an existing one succeeds without a new record; invalid bindings fail individually; tests.

### [To Do] SEC-0080 DeleteAcls semantics: per-filter matches, bounded deletions, matched bindings in results
P0 · M4 · Task · deps: SEC-0077 · ref: metadata/src/main/java/org/apache/kafka/controller/AclControlManager.java deleteAcls @4.4.0
Done when: Each filter removes all matching ACLs and returns them; a delete exceeding Kafka's per-request bound fails as Kafka does; invalid filters fail individually; tests.

### [To Do] SEC-0081 Differential tests of ACL administration against Kafka 4.4.0
P0 · M4 · Task · deps: SEC-0078, SEC-0079, SEC-0080, TEST-F09 · labels: test, security
Done when: A corpus of creations, describes and deletes with literal, prefixed, wildcard, ANY and MATCH filters returns identical results on both systems.

### [To Do] SEC-0082 Document ACL administration semantics
P2 · M4 · Task · deps: SEC-0078, SEC-0080 · labels: docs
Done when: Docs explain filter matching and the results of create, describe and delete with examples; doc build passes.

## [To Do] SEC-F10 Delegation tokens
P1 · M4 · Feature · deps: SEC-F04, CTRL-F15
Scope: Create, renew, expire and describe; token authentication via SCRAM; HMAC key.

### [To Do] SEC-0083 Token configuration: `delegation.token.secret.key`, max lifetime, expiry time and check interval
P1 · M4 · Task · deps: none · labels: security · ref: server-common/src/main/java/org/apache/kafka/server/config/DelegationTokenManagerConfigs.java @4.4.0
Done when: The secret key is a `Secret` and enables tokens; `delegation.token.max.lifetime.ms` (7 d), `delegation.token.expiry.time.ms` (24 h) and `delegation.token.expiry.check.interval.ms` (1 h) are consumed; without the key token requests fail with DELEGATION_TOKEN_AUTH_DISABLED; tests.

### [To Do] SEC-0084 Token HMAC: HmacSHA512 of the token id with the secret key
P1 · M4 · Task · deps: SEC-0083 · labels: security · ref: server-common/src/main/java/org/apache/kafka/security/DelegationTokenManager.java createHmac @4.4.0
Done when: HMACs match Kafka's for the same id and key (golden test) and are compared in constant time; tests.

### [To Do] SEC-0085 Token cache from the metadata image with SCRAM credentials per token
P1 · M4 · Task · deps: SEC-0084 · labels: security · ref: server-common/src/main/java/org/apache/kafka/security/DelegationTokenManager.java updateToken, removeToken, metadata/src/main/java/org/apache/kafka/metadata/publisher/DelegationTokenPublisher.java @4.4.0
Done when: Token records in the image update a cache of token information and per-mechanism SCRAM credentials derived from the HMAC; removed and expired tokens disappear; tests.

### [To Do] SEC-0086 Token authentication through SCRAM with the `tokenauth=true` extension
P1 · M4 · Task · deps: SEC-0085 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/scram/internals/ScramSaslServer.java evaluateResponse (tokenauth) @4.4.0
Done when: A SCRAM login with `tokenauth=true` looks up the token id as username and the HMAC as password; success yields the token owner as principal with the token-authenticated flag; unknown or expired tokens fail with Kafka's messages; tests.

### [To Do] SEC-0087 Token request permission rules used by handlers
P1 · M4 · Task · deps: SEC-0083 · labels: security · ref: core/src/main/scala/kafka/server/ControllerApis.scala allowTokenRequests @4.4.0
Done when: A helper rejects token requests from token-authenticated principals and from connections Kafka disallows (DELEGATION_TOKEN_REQUEST_NOT_ALLOWED), and checks CREATE_TOKENS on the owner for other-owner creation; tests.

### [To Do] SEC-0088 Describe visibility: owner, renewer, requester, DESCRIBE on the token or DESCRIBE_TOKENS on the owner
P1 · M4 · Task · deps: SEC-0085 · labels: security · ref: server-common/src/main/java/org/apache/kafka/security/DelegationTokenManager.java filterToken @4.4.0
Done when: Token filtering by owners list and visibility follows `DelegationTokenManager.filterToken`; tests per rule.

### [To Do] SEC-0089 End-to-end create: token creation, lifetime caps and authentication with the new token
P1 · M4 · Task · deps: SEC-0086, SEC-0087, TEST-F08 · labels: test, security
Done when: `kafka-delegation-tokens.sh --create` returns a token whose expiry and max lifetime respect the configs; a client authenticates with it immediately after creation; results match Kafka.

### [To Do] SEC-0090 End-to-end renew: renewer rules and max-lifetime cap
P1 · M4 · Task · deps: SEC-0089 · labels: test, security
Done when: Only owners and renewers can renew; renewal past max lifetime is capped; non-renewers get DELEGATION_TOKEN_OWNER_MISMATCH; matches Kafka.

### [To Do] SEC-0091 End-to-end expire: immediate expiry and authentication failure afterwards
P1 · M4 · Task · deps: SEC-0089 · labels: test, security
Done when: Expiring with a negative period removes the token; authentication with it then fails; new connections only; matches Kafka.

### [To Do] SEC-0092 End-to-end describe with owner filters and visibility
P1 · M4 · Task · deps: SEC-0088, SEC-0089 · labels: test, security
Done when: Describe returns exactly the tokens visible to the requester with and without owner filters; matches Kafka.

### [To Do] SEC-0093 Document delegation tokens
P2 · M4 · Task · deps: SEC-0089 · labels: docs
Done when: Docs cover enabling tokens, the shared secret across nodes, lifetimes and client usage; doc build passes.

## [To Do] SEC-F11 Secrets and sensitive configuration at rest
P0 · M2 · Feature · deps: TYPE-F08
Scope: Password-type configs, redaction everywhere, and how sensitive dynamic configs are stored in metadata (Kafka 4.x removed `password.encoder.*`; an ADR decides Alluvium's at-rest protection).

### [To Do] SEC-0094 Sensitive-key classification: one table of password-type keys, including listener and mechanism prefixes
P0 · M2 · Task · deps: CFG-F01 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/config/SslConfigs.java, clients/src/main/java/org/apache/kafka/common/config/SaslConfigs.java @4.4.0
Done when: A single function answers whether a key is sensitive (for example `ssl.key.password`, `ssl.keystore.password`, `ssl.keystore.key`, `ssl.truststore.password`, `sasl.jaas.config`, `delegation.token.secret.key`, `sasl.oauthbearer.client.credentials.client.secret`) including `listener.name.<l>.` and mechanism prefixes; CFG uses it for redaction; tests.

### [To Do] SEC-0095 Wrap sensitive values in `Secret` at parse time
P0 · M2 · Task · deps: SEC-0094 · labels: security
Done when: Values of sensitive keys never exist as plain `String` in config structs; `Debug` and `Display` show `Secret(***)`; a test prints every config struct and finds no planted value.

### [To Do] SEC-0096 Refuse secrets passed on the command line
P0 · M2 · Task · deps: SEC-0094, CFG-F02 · labels: security · ref: ADR-0006
Done when: Sensitive keys given through command-line overrides fail with a message pointing to files, environment or config providers; tests.

### [To Do] SEC-0097 Spike: sensitive dynamic configs at rest after Kafka removed `password.encoder.*`
P0 · M2 · Spike · deps: none · labels: security
Done when: ADR records that Kafka 4.x stores sensitive dynamic config values unencrypted in metadata records and removed the `password.encoder.*` keys, decides Alluvium's parity behavior, how configured `password.encoder.*` keys are reported, and links encryption at rest to X-SECX; accepted ADR.

### [To Do] SEC-0098 Owner-only permissions for files that hold secrets
P0 · M2 · Task · deps: SEC-0097 · labels: security
Done when: Metadata log segments, snapshots and other files containing credentials or sensitive configs are created with owner-only modes per the ADR; deviation documented; test checks modes.

### [To Do] SEC-0099 Document secret handling
P1 · M2 · Task · deps: SEC-0095, SEC-0097 · labels: docs, security
Done when: Docs cover where secrets come from, how they are redacted, how to rotate TLS, SASL and token secrets, and the at-rest behavior from the ADR; doc build passes.

## [To Do] SEC-F12 Re-authentication
P1 · M4 · Feature · deps: NET-F08
Scope: KIP-368 session expiry and re-authentication.

### [To Do] SEC-0100 Begin server re-authentication on a SaslHandshake over an authenticated connection
P1 · M4 · Task · deps: none · labels: security · ref: clients/src/main/java/org/apache/kafka/common/network/KafkaChannel.java maybeBeginServerReauthentication, clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java reauthenticate @4.4.0
Done when: A SaslHandshake from a client that negotiated SaslAuthenticate v1+ starts re-authentication instead of being handled as an API request; other clients get ILLEGAL_SASL_STATE; tests.

### [To Do] SEC-0101 Re-authentication must keep the mechanism
P1 · M4 · Task · deps: SEC-0100 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java ReauthInfo.saslMechanismUnchanged (REAUTH_BAD_MECHANISM) @4.4.0
Done when: A different mechanism goes through the bad-mechanism state and fails with Kafka's message, closing the connection; tests.

### [To Do] SEC-0102 Re-authentication must keep the principal
P1 · M4 · Task · deps: SEC-0100 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java ReauthInfo.ensurePrincipalUnchanged @4.4.0
Done when: A re-authentication that yields a different principal fails with 'Cannot change principals during re-authentication' and closes; tests.

### [To Do] SEC-0103 Recompute the session lifetime and expiry at each re-authentication
P1 · M4 · Task · deps: SEC-0100 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslServerAuthenticator.java ReauthInfo.reauthenticationLatencyMs @4.4.0
Done when: Completion sets a new session expiry and sends the new `session_lifetime_ms`; re-authentication latency is recorded; tests on the simulated clock.

### [To Do] SEC-0104 Re-authentication uses current credentials
P1 · M4 · Task · deps: SEC-0102, SEC-0103, SEC-F04, SEC-F10 · labels: security
Done when: Deleting a SCRAM credential, expiring a token or rotating a PLAIN password makes the next re-authentication fail and close the connection; tests per mechanism.

### [To Do] SEC-0105 Inter-node clients re-authenticate before session expiry
P1 · M4 · Task · deps: SEC-0103, SEC-F16 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslClientAuthenticator.java reauthenticate @4.4.0
Done when: Outbound SASL connections re-authenticate at a jittered 85-95% of the session lifetime, as Kafka clients do, without dropping in-flight requests; tests.

### [To Do] SEC-0106 Simulation of session expiry and re-authentication
P1 · M4 · Task · deps: SEC-0104, NET-F14 · labels: test, security
Done when: On the simulated transport, many clients with short `connections.max.reauth.ms` re-authenticate repeatedly; no request is served on an expired session; passes across 1000 seeds.

### [To Do] SEC-0107 Compatibility: Java client and librdkafka across repeated re-authentication
P1 · M4 · Task · deps: SEC-0104, TEST-F08 · labels: test, security
Done when: With `connections.max.reauth.ms=10000` both clients keep producing and consuming for 5 minutes with PLAIN, SCRAM and OAUTHBEARER; metrics show re-authentications.

## [To Do] SEC-F13 Security testing
P0 · M4 · Feature · deps: SEC-F07
Scope: Authorizer property tests, SASL negative tests, TLS conformance, fuzzing of authentication paths.

### [To Do] SEC-0108 Authorizer property tests against a naive reference model
P0 · M4 · Task · deps: none · labels: test, security
Done when: Random ACL sets (literal, prefixed, wildcard, DENY, hosts, CIDR, implied operations, super users) and random actions give the same decision from the authorizer and a straightforward scan model; regressions committed.

### [To Do] SEC-0109 Authorization differential tests against Kafka 4.4.0
P0 · M4 · Task · deps: SEC-0108, TEST-F09 · labels: test, security
Done when: A matrix of ACLs, principals, hosts, operations and resources is exercised with real requests on both systems; every allow/deny outcome and error code matches.

### [To Do] SEC-0110 Differential test: mTLS principals map identically to Kafka
P1 · M4 · Task · deps: SEC-0109 · labels: security, test
Done when: With the same certificates, `ssl.principal.mapping.rules` and a principal-specific ACL, clients are allowed or denied identically on Alluvium and Kafka 4.4.0.

### [To Do] SEC-0111 SASL negative test catalogue with expected errors and connection outcomes
P0 · M4 · Task · deps: SEC-F03, SEC-F04, SEC-F05, SEC-F12 · labels: test, security
Done when: Tests cover requests before handshake, repeated ApiVersions and handshakes, unknown mechanism, SaslAuthenticate before handshake, oversized tokens, malformed PLAIN, SCRAM and OAUTHBEARER messages, wrong nonce, bad proof, low iterations, expired tokens, mechanism and principal change on re-authentication; each outcome matches Kafka 4.4.0.

### [To Do] SEC-0112 Fuzz the pre-authentication path with arbitrary frame sequences
P0 · M4 · Task · deps: NET-F08, TEST-F07 · labels: test, security, decoder
Done when: A fuzz target drives the authenticator and mechanisms with arbitrary frames; no panic, hang or allocation above limits in 24 h nightly runs.

### [To Do] SEC-0113 TLS conformance checks of listeners
P1 · M4 · Task · deps: NET-F07 · labels: test, security
Done when: An external scanner run in CI confirms only TLS 1.2 and 1.3 with configured suites, no renegotiation and no compression on each TLS listener type.

### [To Do] SEC-0114 Authentication-path denial-of-service tests
P0 · M4 · Task · deps: NET-F09, SEC-0112 · labels: test, security, nfr
Done when: Slow handshakes, many unauthenticated connections and maximal pre-auth frames stay within memory and CPU bounds and are closed by the documented limits; existing clients keep working.

### [To Do] SEC-0115 Security test matrix mapping threats to tests
P1 · M4 · Task · deps: SEC-0109, SEC-0111, SEC-0114 · labels: docs, security
Done when: docs/security/test-matrix.md maps each threat in the security standard to the tests that cover it; gaps are filed as tasks.

## [To Do] SEC-F14 Security review process
P1 · M2 · Feature · deps: none
Scope: Threat model upkeep and a security review checklist at each milestone exit.

### [To Do] SEC-0116 Security review checklist and PR template section
P1 · M2 · Task · deps: none · labels: security, docs
Done when: docs/standards/security-review.md lists checks for input handling, authorization coverage, secrets, dependencies, unsafe, TLS settings and new network surfaces; the PR template links it.

### [To Do] SEC-0117 Full threat model per component
P1 · M2 · Task · deps: none · labels: security, docs
Done when: docs/security/threat-model.md expands the summary into per-component threats (network, storage, Raft, controller, coordinators, diagnostics) with defenses and owners; reviewed and linked from security.md.

### [To Do] SEC-0118 M2 exit security review
P1 · M2 · Task · deps: SEC-0116, SEC-0117 · labels: security
Done when: Review record docs/security/reviews/M2.md covers network listeners, framing and header decoders, TLS, diagnostics endpoint, storage and metadata decoders reading disk against the checklist; every finding is filed as a Bug task; all fuzz targets ran 24 h on the nightly job with no open crash; sign-off recorded.

### [To Do] SEC-0119 M3 exit security review
P1 · M3 · Task · deps: SEC-0116, SEC-0117 · labels: security
Done when: Review record docs/security/reviews/M3.md covers inter-node trust (Raft, replication, broker registration), simulated-fault findings, cluster id checks against the checklist; every finding is filed as a Bug task; all fuzz targets ran 24 h on the nightly job with no open crash; sign-off recorded.

### [To Do] SEC-0120 M4 exit security review
P1 · M4 · Task · deps: SEC-0116, SEC-0117 · labels: security
Done when: Review record docs/security/reviews/M4.md covers SASL mechanisms, ACL authorizer, delegation tokens, quotas, inter-node authentication against the checklist; every finding is filed as a Bug task; all fuzz targets ran 24 h on the nightly job with no open crash; sign-off recorded.

### [To Do] SEC-0121 M5 exit security review
P1 · M5 · Task · deps: SEC-0116, SEC-0117 · labels: security
Done when: Review record docs/security/reviews/M5.md covers group coordinator inputs (oversized member metadata, regex subscriptions), offsets, internal topic protection against the checklist; every finding is filed as a Bug task; all fuzz targets ran 24 h on the nightly job with no open crash; sign-off recorded.

### [To Do] SEC-0122 M6 exit security review
P1 · M6 · Task · deps: SEC-0116, SEC-0117 · labels: security
Done when: Review record docs/security/reviews/M6.md covers producer id and epoch spoofing, transaction markers and CLUSTER_ACTION paths, transactional id authorization against the checklist; every finding is filed as a Bug task; all fuzz targets ran 24 h on the nightly job with no open crash; sign-off recorded.

### [To Do] SEC-0123 M7 exit security review
P1 · M7 · Task · deps: SEC-0116, SEC-0117 · labels: security
Done when: Review record docs/security/reviews/M7.md covers admin APIs, dynamic config (including secrets), DeleteRecords, JBOD log dir moves against the checklist; every finding is filed as a Bug task; all fuzz targets ran 24 h on the nightly job with no open crash; sign-off recorded.

### [To Do] SEC-0124 M8 exit security review
P1 · M8 · Task · deps: SEC-0116, SEC-0117 · labels: security
Done when: Review record docs/security/reviews/M8.md covers share groups, streams groups and client telemetry payloads against the checklist; every finding is filed as a Bug task; all fuzz targets ran 24 h on the nightly job with no open crash; sign-off recorded.

### [To Do] SEC-0125 M9 exit security review
P1 · M9 · Task · deps: SEC-0116, SEC-0117 · labels: security
Done when: Review record docs/security/reviews/M9.md covers tiered storage plug-in boundary, release supply chain, full threat model re-check before parity GA against the checklist; every finding is filed as a Bug task; all fuzz targets ran 24 h on the nightly job with no open crash; sign-off recorded.

### [To Do] SEC-0126 External security assessment before parity GA
P2 · M9 · Task · deps: SEC-0124 · labels: security
Done when: An independent assessment of the network, authentication and authorization surfaces is completed; findings are filed and high-severity ones fixed before release.

## [To Do] SEC-F15 Release supply chain
P1 · M9 · Feature · deps: FND-F03
Scope: SBOM, signed artifacts, provenance attestation, vulnerability disclosure policy.

### [To Do] SEC-0127 Release SBOMs in CycloneDX for every binary
P1 · M9 · Task · deps: none · labels: security
Done when: The release job generates and publishes an SBOM per artifact; CI validates the SBOM schema and that every crate in Cargo.lock appears.

### [To Do] SEC-0128 Embed dependency metadata in binaries for `cargo audit bin`
P1 · M9 · Task · deps: none · labels: security
Done when: Release binaries are built with embedded dependency lists; `cargo audit bin` on a release artifact works in CI.

### [To Do] SEC-0129 Sign release artifacts and checksums with Sigstore keyless signing
P1 · M9 · Task · deps: SEC-0127 · labels: security
Done when: Binaries, checksum files and container images are signed in CI; verification commands in the docs succeed on the published artifacts.

### [To Do] SEC-0130 Build provenance attestations for binaries and images
P1 · M9 · Task · deps: SEC-0129 · labels: security
Done when: Each artifact has a SLSA provenance attestation from the release workflow; verification is documented and checked in CI.

### [To Do] SEC-0131 Harden the release pipeline
P1 · M9 · Task · deps: none · labels: security
Done when: Release jobs run with least-privilege tokens in a protected environment requiring two approvers; actions are pinned by SHA; a check fails on unpinned actions.

### [To Do] SEC-0132 Vulnerability disclosure policy and private reporting
P1 · M9 · Task · deps: none · labels: security, docs
Done when: SECURITY.md states the reporting channel, supported versions, response times and embargo process; GitHub private vulnerability reporting is enabled.

### [To Do] SEC-0133 Security advisory and patch release procedure
P1 · M9 · Task · deps: SEC-0132 · labels: security, docs
Done when: The release process documents triage, CVE requests, advisory publication and patch releases; a dry run is recorded.

### [To Do] SEC-0134 User docs for verifying downloads
P2 · M9 · Task · deps: SEC-0130 · labels: docs
Done when: Docs show how to verify signatures, provenance and SBOMs for binaries and images; commands tested in CI.

## [To Do] SEC-F16 Inter-node security
P0 · M4 · Feature · deps: NET-F07, SEC-F02
Scope: TLS and SASL on inter-broker and controller listeners; inter-node principals.

### [To Do] SEC-0135 Inter-broker security protocol and mechanism selection
P0 · M4 · Task · deps: none · labels: security · ref: core/src/main/scala/kafka/server/KafkaConfig.scala validateValues (interBrokerUsesSasl) @4.4.0
Done when: Brokers connect to each other with the inter-broker listener's protocol and `sasl.mechanism.inter.broker.protocol`, taking client credentials from that listener's JAAS options; misconfiguration fails startup with Kafka's messages; tests.

### [To Do] SEC-0136 Controller listener security and `sasl.mechanism.controller.protocol`
P0 · M4 · Task · deps: SEC-0135 · labels: security
Done when: Broker-to-controller and controller-to-controller connections use the controller listener's protocol and `sasl.mechanism.controller.protocol`; tests for SSL and SASL_SSL controller listeners.

### [To Do] SEC-0137 SASL client authenticator for outbound connections
P0 · M4 · Task · deps: SEC-0135, NET-F10, SEC-F03, SEC-F04 · labels: security · ref: clients/src/main/java/org/apache/kafka/common/security/authenticator/SaslClientAuthenticator.java @4.4.0
Done when: Outbound connections run ApiVersions, SaslHandshake v1 and SaslAuthenticate with the selected client mechanism, map errors to typed authentication failures with backoff, and record the session lifetime; tests against the server authenticator.

### [To Do] SEC-0138 Inter-node principals need CLUSTER_ACTION; failures match Kafka
P0 · M4 · Task · deps: SEC-0137, SEC-F08 · labels: security
Done when: A broker principal without CLUSTER_ACTION (and not a super user) gets CLUSTER_AUTHORIZATION_FAILED on inter-broker APIs as with Kafka; docs list the ACLs nodes need; tests.

### [To Do] SEC-0139 Three-node cluster with mTLS on inter-broker and controller listeners
P0 · M4 · Task · deps: SEC-0136 · labels: test, security
Done when: Replication, Raft and forwarding work over mTLS; a node with an untrusted certificate cannot join; integration test.

### [To Do] SEC-0140 Three-node cluster with SASL_SSL SCRAM on inter-broker and controller listeners
P0 · M4 · Task · deps: SEC-0137, SEC-0136 · labels: test, security
Done when: Replication, Raft and forwarding work with SCRAM; wrong credentials keep the broker from registering with a clear error; integration test.

### [To Do] SEC-0141 Document securing inter-node traffic
P1 · M4 · Task · deps: SEC-0138, SEC-0139 · labels: docs
Done when: Docs give example configurations for mTLS and SASL inter-node setups and the required ACLs; doc build passes.
