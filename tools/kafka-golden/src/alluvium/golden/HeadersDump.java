package alluvium.golden;

import java.nio.ByteBuffer;
import java.util.HexFormat;

import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.JsonNodeFactory;
import com.fasterxml.jackson.databind.node.ObjectNode;

import org.apache.kafka.common.protocol.ApiKeys;
import org.apache.kafka.common.protocol.MessageUtil;
import org.apache.kafka.common.requests.RequestHeader;
import org.apache.kafka.common.requests.ResponseHeader;

/**
 * {@code headers.json}: Kafka's request and response headers for every API with
 * versions, at its oldest version, its latest (unstable included) and one past it, each
 * with a null, an empty and a non-ASCII client id.
 *
 * <p>Each entry has the request header Kafka writes ({@code new RequestHeader(...)} at the
 * version it selects), the client id {@code RequestHeader.parse} reads back from it, and
 * the response header {@code toResponseHeader()} writes, as hex.
 */
final class HeadersDump {
    private static final JsonNodeFactory JSON = JsonNodeFactory.instance;
    private static final String[] CLIENT_IDS = {null, "", "client-\u00e9-\u2713"};

    private HeadersDump() {
    }

    static ArrayNode all() {
        ArrayNode array = JSON.arrayNode();
        HexFormat hex = HexFormat.of();
        for (ApiKeys key : ApiKeys.values()) {
            if (!key.hasValidVersion()) {
                continue;
            }
            short latest = key.latestVersion(true);
            for (short version : new short[] {key.oldestVersion(), latest, (short) (latest + 1)}) {
                for (String clientId : CLIENT_IDS) {
                    int correlationId = key.id * 1000 + version;
                    RequestHeader request = new RequestHeader(key, version, clientId, correlationId);
                    byte[] requestBytes = bytes(request.data(), request.headerVersion());
                    RequestHeader parsed = RequestHeader.parse(ByteBuffer.wrap(requestBytes));
                    ResponseHeader response = request.toResponseHeader();
                    ObjectNode entry = array.addObject();
                    entry.put("api_key", key.id);
                    entry.put("version", version);
                    entry.put("client_id", clientId);
                    entry.put("correlation_id", correlationId);
                    entry.put("request_header_version", request.headerVersion());
                    entry.put("request", hex.formatHex(requestBytes));
                    entry.put("parsed_client_id", parsed.clientId());
                    entry.put("parsed_size", parsed.size());
                    entry.put("response_header_version", response.headerVersion());
                    entry.put("response", hex.formatHex(bytes(response.data(), response.headerVersion())));
                }
            }
        }
        return array;
    }

    private static byte[] bytes(org.apache.kafka.common.protocol.Message data, short version) {
        return MessageUtil.byteBufferToArray(MessageUtil.toByteBufferAccessor(data, version).buffer());
    }
}
