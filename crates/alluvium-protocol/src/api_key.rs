//! [`ApiKey`]'s hand-written half: finding an API by its key, and the version rules
//! Kafka's `ApiKeys` applies on top of the schemas. The enum and its per-API table are
//! generated from the request and response schemas into `generated::api_key`.

pub use crate::generated::api_key::ApiKey;

/// The kind of listener a Kafka node serves an API on, from each request schema's
/// `listeners`.
// Parity: generator/src/main/java/org/apache/kafka/message/RequestListenerType.java@4.4.0 RequestListenerType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Listener {
    /// A broker's client and inter-broker listeners.
    Broker,
    /// A controller's listener.
    Controller,
}

impl ApiKey {
    /// The API with key `id`, or `None` for a key Kafka 4.4.0 doesn't define.
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java@4.4.0 ApiKeys.forId, ApiKeys.hasId
    pub fn from_id(id: i16) -> Option<ApiKey> {
        ApiKey::ALL
            .binary_search_by_key(&id, |api| api.id())
            .ok()
            .and_then(|index| ApiKey::ALL.get(index))
            .copied()
    }

    /// The highest version served: the latest, unless it is unstable and
    /// `enable_unstable` (`unstable.api.versions.enable`) is off. `None` when no version
    /// is served, as for a removed API, or one whose only version is unstable.
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java@4.4.0 ApiKeys.latestVersion(boolean)
    pub fn latest_version(self, enable_unstable: bool) -> Option<i16> {
        let versions = self.versions();
        let highest = versions.highest()?;
        let latest = if self.latest_version_unstable() && !enable_unstable {
            highest.checked_sub(1)?
        } else {
            highest
        };
        versions.contains(latest).then_some(latest)
    }

    /// Whether a request at `version` is served: within the API's versions, up to
    /// [`ApiKey::latest_version`]. Every version of `ApiVersions` is, because a client
    /// sends its highest and the server answers an unknown one with its own range
    /// (KIP-511).
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java@4.4.0 ApiKeys.isVersionEnabled
    pub fn is_version_enabled(self, version: i16, enable_unstable: bool) -> bool {
        if self == ApiKey::ApiVersions {
            return true;
        }
        self.latest_version(enable_unstable)
            .is_some_and(|latest| self.versions().contains(version) && version <= latest)
    }

    /// Whether Kafka has deprecated `version`, for the deprecated-request metric.
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java@4.4.0 ApiKeys.isVersionDeprecated
    pub fn is_version_deprecated(self, version: i16) -> bool {
        self.deprecated_versions().contains(version)
    }

    /// Whether `listener` serves the API.
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java@4.4.0 ApiKeys.inScope
    pub fn is_served_on(self, listener: Listener) -> bool {
        self.listeners().contains(&listener)
    }

    /// The APIs `listener` serves, in API key order.
    // Parity: clients/src/main/java/org/apache/kafka/common/protocol/ApiKeys.java@4.4.0 ApiKeys.apisForListener
    pub fn served_on(listener: Listener) -> impl Iterator<Item = ApiKey> {
        ApiKey::ALL
            .into_iter()
            .filter(move |api| api.is_served_on(listener))
    }
}

#[cfg(test)]
mod tests {
    use super::{ApiKey, Listener};

    #[test]
    fn from_id_finds_every_api_and_nothing_else() {
        for api in ApiKey::ALL {
            assert_eq!(ApiKey::from_id(api.id()), Some(api));
        }
        assert_eq!(ApiKey::from_id(-1), None);
        assert_eq!(ApiKey::from_id(i16::MAX), None);
    }

    #[test]
    fn an_unstable_latest_version_is_served_only_when_enabled() {
        let api = ApiKey::InitProducerId;
        assert!(api.latest_version_unstable());
        assert_eq!(api.latest_version(true), Some(6));
        assert_eq!(api.latest_version(false), Some(5));
        assert!(api.is_version_enabled(6, true));
        assert!(!api.is_version_enabled(6, false));
        assert!(api.is_version_enabled(5, false));
    }

    #[test]
    fn a_removed_api_serves_no_version() {
        let api = ApiKey::LeaderAndIsr;
        assert_eq!(api.latest_version(true), None);
        assert!(!api.is_version_enabled(0, true));
    }

    #[test]
    fn every_api_versions_version_is_enabled() {
        for version in [-1, 0, 4, 127, i16::MAX] {
            assert!(ApiKey::ApiVersions.is_version_enabled(version, false));
        }
        assert!(!ApiKey::Produce.is_version_enabled(2, false));
        assert!(!ApiKey::Produce.is_version_enabled(14, true));
    }

    #[test]
    fn served_on_lists_the_apis_a_listener_serves() {
        assert!(ApiKey::served_on(Listener::Broker).any(|api| api == ApiKey::Produce));
        assert!(!ApiKey::served_on(Listener::Controller).any(|api| api == ApiKey::Produce));
        assert!(ApiKey::Vote.is_served_on(Listener::Controller));
    }
}
