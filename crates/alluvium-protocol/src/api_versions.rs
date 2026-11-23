//! The API version list an `ApiVersions` response advertises: for each API a listener
//! serves, the versions the node both knows and implements.
//!
//! The rule is Kafka's: an API is listed with its oldest version and its latest served
//! version ([`ApiKey::latest_version`]), and left out when that range is empty. On top of
//! Kafka's rule, the range is narrowed to the versions the caller says are implemented
//! and tested, and an API the caller doesn't list is left out: Alluvium advertises only
//! what works (engineering rule 15). The table of implemented versions belongs to the
//! server that handles the requests, so it is an argument here.

use crate::api_key::{ApiKey, Listener};
use crate::generated::api_versions_response::ApiVersion;
use crate::versions::Versions;

/// The lowest Produce version a broker advertises, though it serves only 3 and above.
///
/// Kafka 4.0 removed Produce versions 0-2, but librdkafka refuses a broker whose range
/// doesn't start at 0, so brokers keep advertising 0 (KAFKA-18659). A request at a
/// removed version still fails as unsupported.
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java@4.4.0 ApiKeys.PRODUCE_API_VERSIONS_RESPONSE_MIN_VERSION
const PRODUCE_MIN_ADVERTISED_ON_BROKER: i16 = 0;

/// The APIs and versions `listener` advertises, in API key order.
///
/// `implemented` lists the versions the node implements and has tested for each API it
/// handles. Produce on a broker is the one exception to narrowing: it is advertised from
/// version 0 whatever its implemented range. `enable_unstable` is `unstable.api.versions.enable`. The telemetry APIs
/// (`GetTelemetrySubscriptions`, `PushTelemetry`) are listed only when
/// `client_telemetry_enabled`, as Kafka lists them only when a telemetry exporter is
/// configured.
// Parity: clients/src/main/java/org/apache/kafka/common/requests/ApiVersionsResponse.java@4.4.0 ApiVersionsResponse.filterApis, ApiVersionsResponse.collectApis
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java@4.4.0 ApiKeys.toApiVersion
pub fn advertised_api_versions(
    listener: Listener,
    enable_unstable: bool,
    client_telemetry_enabled: bool,
    implemented: &[(ApiKey, Versions)],
) -> Vec<ApiVersion> {
    ApiKey::served_on(listener)
        .filter(|api| {
            client_telemetry_enabled
                || !matches!(
                    api,
                    ApiKey::GetTelemetrySubscriptions | ApiKey::PushTelemetry
                )
        })
        .filter_map(|api| {
            let oldest = api.versions().lowest()?;
            let latest = api.latest_version(enable_unstable)?;
            let (implemented_lowest, implemented_highest) = implemented
                .iter()
                .find(|(key, _)| *key == api)
                .and_then(|(_, versions)| Some((versions.lowest()?, versions.highest()?)))?;
            let min_version = oldest.max(implemented_lowest);
            let max_version = latest.min(implemented_highest);
            if min_version > max_version {
                return None;
            }
            let min_version = if api == ApiKey::Produce && listener == Listener::Broker {
                PRODUCE_MIN_ADVERTISED_ON_BROKER
            } else {
                min_version
            };
            Some(ApiVersion {
                api_key: api.id(),
                min_version,
                max_version,
                ..ApiVersion::default()
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::advertised_api_versions;
    use crate::api_key::{ApiKey, Listener};
    use crate::versions::Versions;

    fn ranges(listener: Listener, implemented: &[(ApiKey, Versions)]) -> Vec<(i16, i16, i16)> {
        advertised_api_versions(listener, false, false, implemented)
            .into_iter()
            .map(|api| (api.api_key, api.min_version, api.max_version))
            .collect()
    }

    #[test]
    fn only_implemented_apis_are_advertised_within_their_implemented_versions() {
        assert_eq!(ranges(Listener::Broker, &[]), []);
        let implemented = [
            (ApiKey::ApiVersions, Versions::new(0, 3)),
            (ApiKey::Metadata, Versions::new(10, 20)),
        ];
        assert_eq!(
            ranges(Listener::Broker, &implemented),
            [(3, 10, 13), (18, 0, 3)]
        );
    }

    #[test]
    fn an_api_with_no_implemented_version_in_range_is_left_out() {
        let implemented = [(ApiKey::Fetch, Versions::new(0, 3))];
        assert_eq!(ranges(Listener::Broker, &implemented), []);
        let implemented = [(ApiKey::Fetch, Versions::NONE)];
        assert_eq!(ranges(Listener::Broker, &implemented), []);
    }

    #[test]
    fn produce_starts_at_0_on_the_broker_listener_only() {
        let implemented = [(ApiKey::Produce, Versions::new(3, 13))];
        assert_eq!(ranges(Listener::Broker, &implemented), [(0, 0, 13)]);
        let implemented = [(ApiKey::Produce, Versions::new(5, 13))];
        assert_eq!(ranges(Listener::Broker, &implemented), [(0, 0, 13)]);
    }

    #[test]
    fn telemetry_apis_are_advertised_only_when_enabled() {
        let implemented = [(ApiKey::PushTelemetry, Versions::new(0, 0))];
        let listed =
            |enabled| advertised_api_versions(Listener::Broker, false, enabled, &implemented).len();
        assert_eq!((listed(false), listed(true)), (0, 1));
    }
}
