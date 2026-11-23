# Version negotiation

How an Alluvium node learns which versions of each API another node serves, and picks the version of each request it sends. This happens when it connects to another Alluvium node, or to a Kafka broker or controller. It mirrors Kafka 4.4.0's client: `NodeApiVersions` and the `ApiVersions` exchange in `NetworkClient`. The code is in [`negotiation.rs`](../../crates/alluvium-protocol/src/negotiation.rs). The network layer sends the requests and logs the outcome (NET-F10).

## The exchange

The first request on a connection is `ApiVersions`, at this node's highest version. The response header is always v0, since the client can't know which version the peer answers with ([KIP-511](https://cwiki.apache.org/confluence/display/KAFKA/KIP-511%3A+Collect+and+Expose+Client%27s+Name+and+Version+in+the+Brokers)). `ApiVersionsHandshake::on_response` takes the body and returns one of three steps:

| The response | Step |
|---|---|
| No error | `Negotiated`: the peer's versions, as `NodeApiVersions` |
| `UNSUPPORTED_VERSION` to a version above 0 | `Retry` at the highest `ApiVersions` version the error lists, or at 0 if it lists none, as Kafka retries, even if that version is the one sent or a higher one |
| `UNSUPPORTED_VERSION` naming a version this node doesn't have, such as a negative one | `Failed` (`UnusableRetry`); Kafka's client fails to build that request |
| `UNSUPPORTED_VERSION` after as many requests as this node has `ApiVersions` versions | `Failed` (`UnusableRetry`) |
| Any other error, or any error to version 0 | `Failed` (`ErrorResponse`); Kafka disconnects |
| A body that decodes neither at the version sent nor at version 0 | `Failed` (`Malformed`) |

A peer that doesn't serve the version sent answers with a version 0 body. So a body that doesn't decode at the version sent is decoded again at version 0, as Kafka's `ApiVersionsResponse.parse` does. Since Kafka 2.4, the error lists the peer's own `ApiVersions` range, so one retry is enough. An older peer lists nothing, and the retry is at version 0.

**One deviation.** A peer that keeps answering `UNSUPPORTED_VERSION` while naming versions this node has would make Kafka's client retry without end. Here this node sends at most one request per `ApiVersions` version it has, then fails, so the exchange always ends. A sequence of retries that ends within that bound gives the same requests as Kafka's.

`REBOOTSTRAP_REQUIRED` is a failure here too. A Kafka client configured for it rebootstraps; what to do here is the caller's decision.

## Choosing a version

`NodeApiVersions::latest_usable_version(api, allowed)` returns the highest version in all three of:
- this node's versions of the API (`ApiKey::versions`, the unstable last version included, as Kafka's `ApiKeys.latestVersion()` includes it);
- the peer's range;
- `allowed`.

Pass `api.versions()` to allow any.

The two sides apply the same rule: each node advertises only the versions it has implemented and tested. Alluvium's [`advertised_api_versions`](../../crates/alluvium-protocol/src/api_versions.rs) narrows Kafka's ranges to the table of implemented versions the server passes it (engineering rule 15). So a version chosen here is one the peer has implemented, never just one its schema lists. With no version in all three, the result is `UnsupportedVersion { api, local, peer }`. `local` is this node's versions within `allowed`, and `peer` is `None` when the peer didn't list the API.

The peer's list is read as Kafka reads it:
- a key listed twice keeps its last entry;
- a key Kafka 4.4.0 doesn't define is kept apart, in `unknown_apis`, since a newer peer may serve APIs this node doesn't know.

Supported and finalized features in the response aren't read yet; feature negotiation will add them.

## The negotiation event

A `Negotiation` holds the peer's node id, the `ApiVersions` version the exchange ended at, and the peer's versions. Its `Display` is the network layer's log line:

```
node 1 via ApiVersions v4: (Produce(0): 0 to 13 [usable: 13], Fetch(1): 4 to 18 [usable: 18], ...)
```

The list after the colon is exactly what Kafka's `NodeApiVersions.toString()` prints, so logs read the same as a Kafka client's:
- each API the peer listed, with its range and the version this node would use;
- `[unusable: node too new]` or `[unusable: node too old]` where no version is in common;
- `UNSUPPORTED` for each broker API the peer didn't list;
- `UNKNOWN(<key>)` for keys Kafka 4.4.0 doesn't define.

`cargo xtask golden api-keys` records Kafka's own text for each broker and controller list it records. A test checks that `Display` gives the same text for all of them.
