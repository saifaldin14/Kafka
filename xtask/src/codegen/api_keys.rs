//! `ApiKey`: one variant per Kafka API, rendered from the request and response schemas,
//! with the two flags the schemas don't state read from Kafka's own `ApiKeys`.
//!
//! As Kafka's `ApiMessageTypeGenerator` does, the registry takes each API's versions,
//! deprecated versions, listeners and unstable flag from its request schema, includes
//! APIs whose schemas have no valid versions (their keys stay reserved), and fails on
//! two requests or two responses with one key. It also fails when an API lacks a request
//! or a response, or when the two disagree on valid or flexible versions: header version
//! selection reads one flexible range per API, which is right only when they agree.
//!
//! `clusterAction` and `forwardable` are arguments of `ApiKeys`' constructors in
//! `ApiKeys.java`, not schema properties. They are read from the registry `cargo xtask
//! golden api-keys` recorded from Kafka's jar, which must list exactly the schemas' API
//! keys, under the same names, so a new or renamed API fails generation until the
//! recording is refreshed.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use super::docs;
use super::structs::versions;
use crate::message_spec::{Listener, MessageKind, MessageSpec, Versions};

/// One API, as the generated table describes it.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Api {
    pub(super) id: i16,
    /// The request's name without `Request`: `Produce`.
    pub(super) name: String,
    pub(super) versions: Versions,
    pub(super) flexible_versions: Versions,
    pub(super) deprecated_versions: Versions,
    pub(super) listeners: Vec<Listener>,
    pub(super) latest_version_unstable: bool,
    pub(super) cluster_action: bool,
    pub(super) forwardable: bool,
}

/// `clusterAction` and `forwardable` for one API, as Kafka's `ApiKeys` sets them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Flags {
    name: String,
    cluster_action: bool,
    forwardable: bool,
}

/// The flags of every API key in the registry recorded at `path`, by key.
pub(super) fn flags(path: &Path) -> Result<BTreeMap<i16, Flags>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
    let entries = json
        .as_array()
        .ok_or_else(|| format!("{}: not an array", path.display()))?;
    let mut flags = BTreeMap::new();
    for (index, entry) in entries.iter().enumerate() {
        let problem = |what: &str| format!("{}: entry {index}: {what}", path.display());
        let id = entry["id"]
            .as_i64()
            .and_then(|id| i16::try_from(id).ok())
            .ok_or_else(|| problem("no int16 `id`"))?;
        let flag = |name: &str| {
            entry[name]
                .as_bool()
                .ok_or_else(|| problem(&format!("no boolean `{name}`")))
        };
        let value = Flags {
            name: entry["name"]
                .as_str()
                .ok_or_else(|| problem("no `name`"))?
                .to_owned(),
            cluster_action: flag("cluster_action")?,
            forwardable: flag("forwardable")?,
        };
        if flags.insert(id, value).is_some() {
            return Err(problem(&format!("API key {id} is listed twice")));
        }
    }
    Ok(flags)
}

/// Every API in `messages`, by key, with its flags from `flags`.
// Parity: generator/src/main/java/org/apache/kafka/message/ApiMessageTypeGenerator.java@4.4.0 ApiMessageTypeGenerator.registerMessageType
pub(super) fn collect(
    messages: &[MessageSpec],
    flags: &BTreeMap<i16, Flags>,
) -> Result<Vec<Api>, String> {
    let mut requests: BTreeMap<i16, &MessageSpec> = BTreeMap::new();
    let mut responses: BTreeMap<i16, &MessageSpec> = BTreeMap::new();
    for message in messages {
        let (Some(key), table, kind) = (
            message.api_key,
            match message.kind {
                MessageKind::Request => &mut requests,
                MessageKind::Response => &mut responses,
                _ => continue,
            },
            if message.kind == MessageKind::Request {
                "request"
            } else {
                "response"
            },
        ) else {
            continue;
        };
        if table.insert(key, message).is_some() {
            return Err(format!("Found more than one {kind} with API key {key}"));
        }
    }
    let mut apis = Vec::new();
    for (&id, request) in &requests {
        let response = responses
            .get(&id)
            .ok_or_else(|| format!("{} has no response with API key {id}", request.name))?;
        if response.valid_versions != request.valid_versions
            || response.flexible_versions != request.flexible_versions
        {
            return Err(format!(
                "{} and {} must have the same valid and flexible versions",
                request.name, response.name
            ));
        }
        let name = request
            .name
            .strip_suffix("Request")
            .unwrap_or(&request.name)
            .to_owned();
        let flag = flags.get(&id).ok_or_else(|| {
            format!(
                "Kafka's recorded ApiKeys have no API key {id} ({name}); run `cargo xtask golden \
                 api-keys`"
            )
        })?;
        if flag.name != name {
            return Err(format!(
                "API key {id} is {name} in the schemas but {} in Kafka's recorded ApiKeys",
                flag.name
            ));
        }
        apis.push(Api {
            id,
            name,
            versions: request.valid_versions,
            flexible_versions: request.flexible_versions,
            deprecated_versions: request.deprecated_versions,
            listeners: request.listeners.clone(),
            latest_version_unstable: request.latest_version_unstable,
            cluster_action: flag.cluster_action,
            forwardable: flag.forwardable,
        });
    }
    if let Some((id, response)) = responses.iter().find(|(id, _)| !requests.contains_key(id)) {
        return Err(format!(
            "{} has no request with API key {id}",
            response.name
        ));
    }
    if let Some((id, flag)) = flags.iter().find(|(id, _)| !requests.contains_key(id)) {
        return Err(format!(
            "Kafka's recorded ApiKeys list API key {id} ({}), which no schema defines",
            flag.name
        ));
    }
    Ok(apis)
}

/// `docs/protocol/apis.md`: every API with its versions, listeners and flags.
pub(super) fn page(apis: &[Api], sources: &str) -> String {
    let mut out = format!(
        "<!-- @generated by `cargo xtask codegen` from {sources}. Do not edit: change the \
         generator in xtask/src/codegen/api_keys.rs. -->\n\n\
         # Kafka APIs\n\n\
         Every API of Kafka 4.4.0, as its request and response schemas and its `ApiKeys` \
         enum define it. In code, each is a variant of `alluvium_protocol::ApiKey` \
         ([codegen](codegen.md#api-keys)). Which of them Alluvium implements, and when, is in \
         [parity.md](../parity.md) and the backlog.\n\n\
         - **Versions**: the versions Kafka implements. \"Removed\" marks an API Kafka 4.0 \
         removed, whose key stays reserved.\n\
         - **Flexible**: the versions with the flexible encoding: compact lengths, tagged \
         fields, and request header v2 and response header v1.\n\
         - **Listeners**: where Kafka serves the API. `broker` is a broker's client and \
         inter-broker listeners; `controller` is a controller's.\n\
         - **Notes**: an unstable latest version, served only with \
         `unstable.api.versions.enable`; deprecated versions; `clusterAction`, an API only \
         other cluster nodes may call; and `forwardable`, an API a broker forwards to the \
         controller.\n\n\
         | Key | API | Versions | Flexible | Listeners | Notes |\n\
         |---:|---|---|---|---|---|\n"
    );
    let range = |versions: Versions| {
        if versions.is_empty() {
            "none".to_owned()
        } else if versions.highest() == i16::MAX {
            format!("{}+", versions.lowest())
        } else if versions.lowest() == versions.highest() {
            versions.lowest().to_string()
        } else {
            format!("{}-{}", versions.lowest(), versions.highest())
        }
    };
    for api in apis {
        let listeners: Vec<&str> = api
            .listeners
            .iter()
            .map(|listener| match listener {
                Listener::Broker => "broker",
                Listener::Controller => "controller",
            })
            .collect();
        let mut notes = Vec::new();
        if api.latest_version_unstable {
            notes.push(format!("v{} unstable", api.versions.highest()));
        }
        if !api.deprecated_versions.is_empty() {
            notes.push(format!("deprecated {}", range(api.deprecated_versions)));
        }
        if api.cluster_action {
            notes.push("clusterAction".to_owned());
        }
        if api.forwardable {
            notes.push("forwardable".to_owned());
        }
        let (versions, flexible) = if api.versions.is_empty() {
            ("removed".to_owned(), "—".to_owned())
        } else {
            (range(api.versions), range(api.flexible_versions))
        };
        let _ = writeln!(
            out,
            "| {} | `{}` | {versions} | {flexible} | {} | {} |",
            api.id,
            api.name,
            if listeners.is_empty() {
                "—".to_owned()
            } else {
                listeners.join(", ")
            },
            notes.join(", ")
        );
    }
    out
}

/// The `api_key` module: the enum, its facts table, and accessors over it.
pub(super) fn module(apis: &[Api], runtime: &str) -> String {
    let mut out = String::new();
    render_enum(&mut out, apis);
    render_facts(&mut out, apis, runtime);
    let _ = write!(
        out,
        "impl ApiKey {{\n    /// Every API, in API key order.\n    pub const ALL: [ApiKey; {}] = [\n",
        apis.len()
    );
    for api in apis {
        let _ = writeln!(out, "        ApiKey::{},", api.name);
    }
    out.push_str("    ];\n");
    out.push_str(ACCESSORS);
    out.push_str("}\n");
    out.replace("crate::", &format!("{runtime}::"))
}

fn render_enum(out: &mut String, apis: &[Api]) {
    out.push_str(
        "//! `ApiKey`, one variant per API in Kafka's schemas, with what Kafka's `ApiKeys` \
         knows about each.\n\n\
         /// A Kafka API, named as Kafka names it and identified on the wire by its API key.\n\
         ///\n/// Variants are in API key order, and include APIs Kafka has removed, whose \
         keys stay\n/// reserved and which have no versions.\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]\npub enum ApiKey {\n",
    );
    for api in apis {
        let _ = writeln!(
            out,
            "    /// `{}`, API key {}.\n    {},",
            api.name, api.id, api.name
        );
    }
    out.push_str("}\n\n");
}

fn render_facts(out: &mut String, apis: &[Api], runtime: &str) {
    out.push_str(&docs::wrap(
        "What the schemas and Kafka's `ApiKeys` say about one API. `FACTS` has one per \
         variant, in declaration order, so a variant's position (`self as usize`) is its \
         index.",
        "/// ",
    ));
    let _ = write!(
        out,
        "struct Facts {{\n    id: i16,\n    name: &'static str,\n    versions: \
         crate::Versions,\n    flexible_versions: crate::Versions,\n    \
         deprecated_versions: crate::Versions,\n    listeners: &'static \
         [crate::Listener],\n    latest_version_unstable: bool,\n    cluster_action: \
         bool,\n    forwardable: bool,\n}}\n\nconst FACTS: [Facts; {}] = [\n",
        apis.len()
    );
    for api in apis {
        let listeners: Vec<&str> = api
            .listeners
            .iter()
            .map(|listener| match listener {
                Listener::Broker => "crate::Listener::Broker",
                Listener::Controller => "crate::Listener::Controller",
            })
            .collect();
        let _ = writeln!(
            out,
            "    Facts {{ id: {}, name: \"{}\", versions: {}, flexible_versions: {}, \
             deprecated_versions: {}, listeners: &[{}], latest_version_unstable: {}, \
             cluster_action: {}, forwardable: {} }},",
            api.id,
            api.name,
            versions(api.versions, runtime),
            versions(api.flexible_versions, runtime),
            versions(api.deprecated_versions, runtime),
            listeners.join(", "),
            api.latest_version_unstable,
            api.cluster_action,
            api.forwardable,
        );
    }
    out.push_str("];\n\n");
}

/// The accessors over `FACTS`, the same for every registry.
const ACCESSORS: &str = "
    fn facts(self) -> &'static Facts {
        match FACTS.get(self as usize) {
            Some(facts) => facts,
            None => unreachable!(\"invariant: FACTS has one entry per ApiKey variant\"),
        }
    }

    /// The API key on the wire.
    pub fn id(self) -> i16 {
        self.facts().id
    }

    /// Kafka's name for the API: `Produce`.
    pub fn name(self) -> &'static str {
        self.facts().name
    }

    /// The versions Kafka 4.4.0 implements, including an unstable latest version: its
    /// request schema's valid versions. None for a removed API.
    pub fn versions(self) -> crate::Versions {
        self.facts().versions
    }

    /// The versions that use the flexible encoding, the same for the request and the
    /// response.
    pub fn flexible_versions(self) -> crate::Versions {
        self.facts().flexible_versions
    }

    /// The versions Kafka has deprecated.
    pub fn deprecated_versions(self) -> crate::Versions {
        self.facts().deprecated_versions
    }

    /// The listeners that serve the API.
    pub fn listeners(self) -> &'static [crate::Listener] {
        self.facts().listeners
    }

    /// Whether the latest version is still unstable, and served only when unstable
    /// versions are enabled.
    pub fn latest_version_unstable(self) -> bool {
        self.facts().latest_version_unstable
    }

    /// Whether only other cluster nodes may send the request: Kafka's `clusterAction`,
    /// which authorizes it as `CLUSTER_ACTION` on the cluster.
    pub fn is_cluster_action(self) -> bool {
        self.facts().cluster_action
    }

    /// Whether a broker forwards the request to the controller: Kafka's `forwardable`.
    pub fn is_forwardable(self) -> bool {
        self.facts().forwardable
    }
";

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{Flags, collect};
    use crate::message_spec::{self, MessageSpec};

    fn schema(kind: &str, name: &str, key: i16, valid: &str, flexible: &str) -> MessageSpec {
        message_spec::load(
            "fixture.json",
            &format!(
                r#"{{ "apiKey": {key}, "type": "{kind}", "name": "{name}{}",
                    "validVersions": "{valid}", "flexibleVersions": "{flexible}",
                    "fields": [] }}"#,
                if kind == "request" {
                    "Request"
                } else {
                    "Response"
                }
            ),
        )
        .unwrap()
    }

    fn api(name: &str, key: i16) -> Vec<MessageSpec> {
        vec![
            schema("request", name, key, "0-3", "2+"),
            schema("response", name, key, "0-3", "2+"),
        ]
    }

    fn flags(entries: &[(i16, &str)]) -> BTreeMap<i16, Flags> {
        entries
            .iter()
            .map(|&(id, name)| {
                let flags = Flags {
                    name: name.to_owned(),
                    cluster_action: id == 1,
                    forwardable: false,
                };
                (id, flags)
            })
            .collect()
    }

    #[test]
    fn apis_take_the_requests_facts_and_the_recorded_flags() {
        let mut messages = api("Second", 1);
        messages.extend(api("First", 0));
        let apis = collect(&messages, &flags(&[(0, "First"), (1, "Second")])).unwrap();
        let names: Vec<_> = apis.iter().map(|api| (api.id, api.name.as_str())).collect();
        assert_eq!(names, [(0, "First"), (1, "Second")]);
        assert!(apis[1].cluster_action && !apis[0].cluster_action);
        assert_eq!(apis[0].flexible_versions.lowest(), 2);
    }

    // Parity: generator/src/main/java/org/apache/kafka/message/ApiMessageTypeGenerator.java@4.4.0 ApiMessageTypeGenerator.registerMessageType
    #[test]
    fn two_requests_or_responses_with_one_key_fail() {
        let mut messages = api("First", 0);
        messages.push(schema("request", "Other", 0, "0-3", "2+"));
        let error = collect(&messages, &flags(&[(0, "First")])).unwrap_err();
        assert_eq!(error, "Found more than one request with API key 0");
        let mut messages = api("First", 0);
        messages.push(schema("response", "Other", 0, "0-3", "2+"));
        let error = collect(&messages, &flags(&[(0, "First")])).unwrap_err();
        assert_eq!(error, "Found more than one response with API key 0");
    }

    #[test]
    fn an_api_needs_a_request_and_a_response_that_agree_on_versions() {
        let lone = [schema("request", "First", 0, "0-3", "2+")];
        assert!(collect(&lone, &flags(&[(0, "First")])).is_err());
        let lone = [schema("response", "First", 0, "0-3", "2+")];
        assert!(collect(&lone, &flags(&[(0, "First")])).is_err());
        for (valid, flexible) in [("0-4", "2+"), ("0-3", "3+")] {
            let messages = [
                schema("request", "First", 0, "0-3", "2+"),
                schema("response", "First", 0, valid, flexible),
            ];
            let error = collect(&messages, &flags(&[(0, "First")])).unwrap_err();
            assert!(
                error.contains("same valid and flexible versions"),
                "{error}"
            );
        }
    }

    #[test]
    fn the_recorded_registry_must_list_exactly_the_schemas_apis() {
        let messages = api("First", 0);
        assert!(
            collect(&messages, &flags(&[]))
                .unwrap_err()
                .contains("no API key 0")
        );
        assert!(
            collect(&messages, &flags(&[(0, "Renamed")]))
                .unwrap_err()
                .contains("in Kafka's recorded ApiKeys")
        );
        assert!(
            collect(&messages, &flags(&[(0, "First"), (7, "Extra")]))
                .unwrap_err()
                .contains("no schema defines")
        );
    }
}
