package alluvium.golden;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;

import org.apache.kafka.common.message.ApiMessageType;
import org.apache.kafka.common.message.ApiMessageType.ListenerType;
import org.apache.kafka.common.message.ApiVersionsResponseData;
import org.apache.kafka.common.message.ApiVersionsResponseData.ApiVersionCollection;
import org.apache.kafka.common.protocol.ApiKeys;
import org.apache.kafka.common.protocol.MessageUtil;
import org.apache.kafka.common.requests.ApiVersionsResponse;

/**
 * The {@code api-keys} dump: Kafka's API key registry, for the generated {@code ApiKey}
 * and the version negotiation tests.
 *
 * <p>{@code api_keys.json} lists every {@code ApiKeys} constant in id order: its name,
 * oldest and latest versions with and without the unstable last version, deprecated
 * range, listeners, {@code clusterAction} and {@code forwardable}, and the request and
 * response header version for each version from 0 to one past the latest, and for
 * {@code Short.MAX_VALUE}.
 *
 * <p>{@code api_versions_<listener>[_unstable][_telemetry].bin} is an
 * ApiVersionsResponse at its latest stable version carrying only the API version list
 * a Kafka server of that listener type advertises. {@code ApiVersionsResponse.filterApis}
 * builds the broker lists, as a broker without a controller connection does;
 * {@code collectApis} builds the controller lists, as Kafka's controller does. Supported
 * and finalized features are left out: they come from the server module, not the
 * client jar.
 */
final class ApiKeysDump {
    private ApiKeysDump() {
    }

    static void write(Path out) throws IOException {
        Files.createDirectories(out);
        Files.writeString(out.resolve("api_keys.json"), apiKeys());
        for (boolean unstable : new boolean[] {false, true}) {
            for (boolean telemetry : new boolean[] {false, true}) {
                apiVersions(out, "broker", unstable, telemetry,
                    ApiVersionsResponse.filterApis(ListenerType.BROKER, unstable, telemetry));
            }
            apiVersions(out, "controller", unstable, false,
                ApiVersionsResponse.collectApis(ListenerType.CONTROLLER,
                    ApiKeys.apisForListener(ListenerType.CONTROLLER), unstable));
        }
    }

    private static String apiKeys() {
        List<String> entries = new ArrayList<>();
        for (ApiKeys key : ApiKeys.values()) {
            ApiMessageType type = key.messageType;
            String listeners = type.listeners().stream()
                .map(listener -> Json.string(listener.name()))
                .collect(Collectors.joining(", ", "[", "]"));
            entries.add(String.format(
                "  {\"id\": %d, \"name\": %s, \"oldest\": %d, \"latest\": %d, \"latest_stable\": %d, "
                    + "\"latest_version_unstable\": %b, \"deprecated\": [%d, %d], \"listeners\": %s, "
                    + "\"cluster_action\": %b, \"forwardable\": %b, "
                    + "\"request_header_versions\": %s, \"response_header_versions\": %s}",
                key.id, Json.string(key.name), key.oldestVersion(), key.latestVersion(true),
                key.latestVersion(false), type.latestVersionUnstable(),
                type.lowestDeprecatedVersion(), type.highestDeprecatedVersion(), listeners,
                key.clusterAction, key.forwardable,
                headerVersions(key, true), headerVersions(key, false)));
        }
        return "[\n" + String.join(",\n", entries) + "\n]\n";
    }

    /** {@code [[version, header version], ...]}, or null for an API with no versions. */
    private static String headerVersions(ApiKeys key, boolean request) {
        if (!key.hasValidVersion()) {
            return "null";
        }
        List<Short> versions = new ArrayList<>();
        for (short version = 0; version <= key.latestVersion(true) + 1; version++) {
            versions.add(version);
        }
        versions.add(Short.MAX_VALUE);
        return versions.stream()
            .map(version -> String.format("[%d, %d]", version,
                request ? key.requestHeaderVersion(version) : key.responseHeaderVersion(version)))
            .collect(Collectors.joining(", ", "[", "]"));
    }

    private static void apiVersions(Path out, String listener, boolean unstable, boolean telemetry,
                                    ApiVersionCollection versions) throws IOException {
        ApiVersionsResponseData data = new ApiVersionsResponseData().setApiKeys(versions);
        short version = ApiKeys.API_VERSIONS.latestVersion(false);
        ByteBuffer bytes = MessageUtil.toByteBufferAccessor(data, version).buffer();
        String name = "api_versions_" + listener + (unstable ? "_unstable" : "")
            + (telemetry ? "_telemetry" : "") + "_v" + version + ".bin";
        Files.write(out.resolve(name), MessageUtil.byteBufferToArray(bytes));
    }
}
