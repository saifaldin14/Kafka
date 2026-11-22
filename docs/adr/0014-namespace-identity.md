# ADR-0014: Namespace identity

- **Status:** Accepted
- **Date:** 2026-11-21

## Context

Multi-tenancy (X-TENANT) gives each tenant a namespace: its own topics, groups and transactional ids, isolated from other tenants', while Kafka clients keep using plain Kafka names. The full tenancy model (principal-to-namespace resolution, limits, fairness, metering) is X-TENANT-F01's decision, in M14. Two things are needed much earlier, because metadata records (META-F07, M2) and the id types store them:

- what a namespace id is, and which strings are valid ones;
- how a name qualified by a namespace relates to the plain Kafka name it stands for.

Kafka 4.4.0 has no namespaces. Its names are flat: a topic name is 1 to 249 characters of `[a-zA-Z0-9._-]` (`Topic.detectInvalidTopic`), group and transactional ids are any string. Some multi-tenant Kafka services emulate namespaces with name prefixes (`tenant.orders`), which takes characters from the 249-character limit, makes prefixes a convention that ACLs must enforce, and leaks the prefix to clients.

## Decision

1. **A namespace id is a short DNS-label-like name.** `NamespaceId` is 1 to 63 characters of `[a-z0-9-]`, starting with a letter and not ending with `-`. Lower case only, so two ids never differ only by case; the DNS-label shape lets a namespace appear in a host name (for example SNI-based routing) or a metric label without escaping. Errors name the first rule broken, in this order: empty, longer than 63, an illegal character (with its position), not starting with a letter, ending with `-`.

2. **The default namespace is `default`.** `NamespaceId::DEFAULT` holds every resource of a cluster without tenancy, and every resource Kafka clients name without a namespace. It is implicit: metadata never stores it (META-0104), so a cluster that never enables tenancy has no namespace records at all.

3. **A qualified name is a pair, never a concatenation.** `NamespacedName<N>` is `(NamespaceId, N)` for `N` one of `TopicName`, `GroupId` and `TransactionalId`. The Kafka name inside keeps its own rules and its full length: a topic in any namespace may still have a 249-character name, because the namespace is never prefixed into it.

4. **Every valid Kafka name is valid, unchanged, in the default namespace.** `NamespacedName::from_kafka(name)` is `(DEFAULT, name)`, and `into_kafka` gives the name back without loss. Converting a name in any other namespace to a plain Kafka name is an explicit call that returns a typed error, so code can't drop a namespace by accident. A default-namespace name displays exactly as the Kafka name; any other displays as `<namespace>/<name>`, for people and logs only, never to be parsed.

5. **X-TENANT-F01 owns the tenancy model.** It may extend these types or supersede this ADR, for example to add hierarchy. Until a namespace other than `default` exists, which only X-TENANT can create and only when tenancy is enabled, nothing here is visible to Kafka clients ([ADR-0010](0010-alluvium-is-its-own-platform.md)).

## Consequences

- META-F07 can define its records with a `NamespaceId` string field and a topic binding keyed by topic id, without changing Kafka's `TopicRecord`.
- Code that handles names must carry the namespace explicitly from the start of X-TENANT; the typed error on `into_kafka` marks every place that would otherwise lose it.
- Kafka clients of a tenant see unqualified names: the broker qualifies them on the way in and strips the namespace on the way out (X-TENANT-0003 lists every name-bearing field).

## Alternatives considered

- **Name prefixes (`tenant.orders`).** No new types, but each tenant loses part of the 249-character limit, `.` and `_` collisions (`Topic.hasCollision`) cross tenants, and isolation depends on every ACL using the prefix correctly.
- **Namespace ids as UUIDs.** Unique without coordination, but unreadable in logs, metrics and CLI output; uniqueness of a short name is easy for the controller to enforce.
- **Allowing the topic-name alphabet (`.`, `_`, upper case).** Case-only and `.`/`_` differences would make look-alike tenants, and the ids couldn't go in host names.
- **Storing the default namespace explicitly.** Every non-tenant cluster would carry records it doesn't need, and a missing record would become a failure mode.
