package alluvium.golden;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HexFormat;
import java.util.List;
import java.util.Locale;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.JsonNodeFactory;
import com.fasterxml.jackson.databind.node.ObjectNode;

import org.apache.kafka.common.protocol.ByteBufferAccessor;
import org.apache.kafka.common.protocol.Message;
import org.apache.kafka.common.protocol.MessageUtil;
import org.apache.kafka.common.protocol.types.RawTaggedField;

/**
 * The {@code messages} dump: Kafka's bytes for every message in the schemas, at every
 * version, as Kafka 4.4.0's generated classes write them.
 *
 * <p>For each message and version it writes these cases, each a {@code .bin} file of
 * Kafka's bytes and a {@code .json} file of Kafka's {@code JsonConverter} output:
 * <ul>
 * <li>{@code default}: a new instance, every field at its default;</li>
 * <li>{@code populated}: every field the version has, with values derived from the
 * field's path ({@link Values});</li>
 * <li>{@code nulls}: the same with every field that is nullable in the version null, for
 * versions where something is;</li>
 * <li>{@code inner_nulls}: the same, but with a nullable field kept when something below
 * it is nullable, so that nested nulls are written under present parents; only where it
 * differs from {@code nulls};</li>
 * <li>{@code unknown_tags}: in flexible versions, the populated message with an unknown
 * tagged field set through {@code RawTaggedField} at the root and in the first element of
 * each struct field below it.</li>
 * </ul>
 *
 * <p>Then each case built at the highest version is written at every lower version: a
 * downgrade. Where Kafka writes it, the bytes are a case of their own; where Kafka
 * throws, the exception's class and message are recorded instead. These are the
 * negative vectors: a non-default field the lower version lacks, a null where it is not
 * nullable, tagged fields where there are none.
 *
 * <p>{@code utf8.json} records how Kafka decodes invalid UTF-8 in a string field and
 * writes it back; {@code headers.json}, the request and response headers of every API
 * ({@link HeadersDump}). {@code cases.json} indexes every case.
 */
final class MessagesDump {
    private static final JsonNodeFactory JSON = JsonNodeFactory.instance;
    private static final String PACKAGE = "org.apache.kafka.common.message.";
    /** The unknown tag set at the root, and the one set one level below. */
    private static final int ROOT_TAG = 10_000;
    private static final int NESTED_TAG = 10_001;

    private final Path out;
    private final ObjectMapper mapper = new ObjectMapper().enable(SerializationFeature.INDENT_OUTPUT);
    private final ArrayNode cases = JSON.arrayNode();

    private MessagesDump(Path out) {
        this.out = out;
    }

    static void write(Path out, Path schemas) throws Exception {
        MessagesDump dump = new MessagesDump(out);
        for (Schemas.Message message : Schemas.load(schemas)) {
            dump.message(message);
        }
        Files.createDirectories(out);
        Files.writeString(out.resolve("cases.json"), dump.mapper.writeValueAsString(dump.cases) + "\n");
        Files.writeString(out.resolve("utf8.json"), dump.mapper.writeValueAsString(Utf8Cases.all()) + "\n");
        Files.writeString(out.resolve("headers.json"), dump.mapper.writeValueAsString(HeadersDump.all()) + "\n");
    }

    private void message(Schemas.Message message) throws Exception {
        Class<?> data = Class.forName(PACKAGE + message.dataClassName());
        Class<?> converter = Class.forName(PACKAGE + message.dataClassName() + "JsonConverter");
        Method read = converter.getMethod("read", JsonNode.class, short.class);
        Method toJson = converter.getMethod("write", data, short.class);
        Schemas.Versions valid = message.validVersions();
        short highest = valid.highest();
        for (short version = valid.lowest(); version <= highest; version++) {
            for (String variant : List.of("default", "populated", "nulls", "inner_nulls", "unknown_tags")) {
                Message built = build(message, variant, version, data, read);
                if (built != null) {
                    record(message, variant, version, version, built, toJson);
                }
            }
        }
        for (String variant : List.of("populated", "nulls", "inner_nulls", "unknown_tags")) {
            Message built = build(message, variant, highest, data, read);
            if (built == null) {
                continue;
            }
            for (short version = valid.lowest(); version < highest; version++) {
                record(message, variant, highest, version, built, null);
            }
        }
    }

    /** The {@code variant} case of {@code message} at {@code version}, or null if it has none. */
    private Message build(Schemas.Message message, String variant, short version, Class<?> data,
                          Method read) throws Exception {
        return switch (variant) {
            case "default" -> (Message) data.getConstructor().newInstance();
            case "populated" -> (Message) read.invoke(null, Values.populated(message, version), version);
            case "nulls" -> {
                ObjectNode json = Values.nulls(message, version);
                yield json == null ? null : (Message) read.invoke(null, json, version);
            }
            case "inner_nulls" -> {
                ObjectNode json = Values.innerNulls(message, version);
                yield json == null ? null : (Message) read.invoke(null, json, version);
            }
            case "unknown_tags" -> {
                if (!message.flexibleVersions().contains(version)) {
                    yield null;
                }
                Message built = (Message) read.invoke(null, Values.populated(message, version), version);
                addUnknownTags(built, message, version);
                yield built;
            }
            default -> throw new IllegalArgumentException(variant);
        };
    }

    private static void addUnknownTags(Message built, Schemas.Message message, short version)
            throws Exception {
        built.unknownTaggedFields().add(new RawTaggedField(ROOT_TAG, "root".getBytes(StandardCharsets.UTF_8)));
        for (Schemas.Field field : message.fields()) {
            if (!field.versions().contains(version)) {
                continue;
            }
            Object value = built.getClass().getMethod(field.jsonName()).invoke(built);
            // Struct arrays are Lists, or keyed collections that are only Iterable.
            if (value instanceof Iterable<?> elements) {
                java.util.Iterator<?> first = elements.iterator();
                value = first.hasNext() ? first.next() : null;
            }
            if (value instanceof Message nested) {
                nested.unknownTaggedFields().add(
                    new RawTaggedField(NESTED_TAG, field.name().getBytes(StandardCharsets.UTF_8)));
            }
        }
    }

    /**
     * Writes {@code built}, made at {@code builtAt}, at {@code version}, and indexes the
     * case. {@code toJson} is null for a downgrade, which has no JSON of its own.
     */
    private void record(Schemas.Message message, String variant, short builtAt, short version,
                        Message built, Method toJson) throws Exception {
        String file = file(message, variant, builtAt, version);
        ObjectNode entry = cases.addObject();
        entry.put("message", message.name());
        entry.put("variant", variant);
        if (builtAt != version) {
            entry.put("built_at", builtAt);
        }
        entry.put("version", version);
        byte[] bytes;
        try {
            ByteBuffer buffer = MessageUtil.toByteBufferAccessor(built, version).buffer();
            bytes = MessageUtil.byteBufferToArray(buffer);
        } catch (RuntimeException error) {
            ObjectNode recorded = entry.putObject("error");
            recorded.put("exception", error.getClass().getSimpleName());
            recorded.put("message", error.getMessage());
            return;
        }
        entry.put("file", file + ".bin");
        Path path = out.resolve(file + ".bin");
        Files.createDirectories(path.getParent());
        Files.write(path, bytes);
        if (toJson != null) {
            JsonNode json;
            try {
                json = (JsonNode) toJson.invoke(null, built, version);
            } catch (InvocationTargetException error) {
                throw new IllegalStateException(file + ": JsonConverter.write failed", error.getCause());
            }
            Files.writeString(out.resolve(file + ".json"), mapper.writeValueAsString(json) + "\n");
        }
    }

    /**
     * {@code <group>/<base>_v<N>_<variant>}, or {@code ..._as_v<M>} for a downgrade:
     * requests and responses are grouped by API, as {@code produce/request_v13_populated};
     * headers under {@code header/} and data types under {@code data/}, by name.
     */
    private static String file(Schemas.Message message, String variant, short builtAt, short version) {
        String name = message.name();
        String group;
        String base;
        switch (message.kind()) {
            case "request" -> {
                group = snake(name.substring(0, name.length() - "Request".length()));
                base = "request";
            }
            case "response" -> {
                group = snake(name.substring(0, name.length() - "Response".length()));
                base = "response";
            }
            case "header" -> {
                group = "header";
                base = snake(name);
            }
            default -> {
                group = "data";
                base = snake(name);
            }
        }
        String file = group + "/" + base + "_v" + builtAt + "_" + variant;
        return builtAt == version ? file : file + "_as_v" + version;
    }

    /** {@code AddPartitionsToTxn} as {@code add_partitions_to_txn}. */
    static String snake(String name) {
        StringBuilder out = new StringBuilder();
        for (int i = 0; i < name.length(); i++) {
            char c = name.charAt(i);
            if (Character.isUpperCase(c) && i > 0) {
                out.append('_');
            }
            out.append(c);
        }
        return out.toString().toLowerCase(Locale.ROOT);
    }

    /** Kafka's decoding of invalid UTF-8 in a string field, and its encoding of the result. */
    static final class Utf8Cases {
        private Utf8Cases() {
        }

        private static final String[][] CASES = {
            {"valid_multibyte", "61c3a9e282acf09f9880"},
            {"lone_continuation", "6180"},
            {"truncated_two_byte", "61c3"},
            {"truncated_three_byte", "61e282"},
            {"truncated_four_byte", "61f09f98"},
            {"truncated_then_ascii", "e28261"},
            {"overlong_nul", "c080"},
            {"overlong_slash", "c0af"},
            {"overlong_three_byte", "e080af"},
            {"overlong_four_byte", "f08080af"},
            {"surrogate_high", "eda080"},
            {"surrogate_low", "edbfbf"},
            {"surrogate_pair", "eda0bdedb880"},
            {"surrogate_truncated", "61eda0"},
            {"surrogate_then_ascii", "eda061"},
            {"surrogate_lead_alone", "ed61"},
            {"above_unicode", "f4908080"},
            {"five_byte_form", "f888808080"},
            {"six_byte_form", "fc8480808080"},
            {"invalid_bytes", "fffe61"},
            {"mixed", "41ff42e2824343eda08044"},
        };

        static ArrayNode all() {
            ArrayNode array = JSON.arrayNode();
            HexFormat hex = HexFormat.of();
            for (String[] utf8Case : CASES) {
                byte[] input = hex.parseHex(utf8Case[1]);
                // Parity: clients/src/main/java/org/apache/kafka/common/protocol/Readable.java@4.4.0 Readable.readString
                String decoded = new ByteBufferAccessor(ByteBuffer.wrap(input)).readString(input.length);
                ObjectNode entry = array.addObject();
                entry.put("name", utf8Case[0]);
                entry.put("input", utf8Case[1]);
                entry.put("kafka", hex.formatHex(decoded.getBytes(StandardCharsets.UTF_8)));
            }
            return array;
        }
    }
}
