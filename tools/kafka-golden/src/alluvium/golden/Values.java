package alluvium.golden;

import java.nio.charset.StandardCharsets;
import java.util.Base64;
import java.util.List;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.JsonNodeFactory;
import com.fasterxml.jackson.databind.node.ObjectNode;

import org.apache.kafka.common.Uuid;

/**
 * The JSON a message's {@code JsonConverter} reads for the {@code populated} and
 * {@code nulls} cases: every field the version has, with a value derived from the field's
 * path, so that rerunning the dump writes the same bytes.
 *
 * <p>Arrays have two elements. Strings are the field's path; byte and records fields are
 * the path's UTF-8 bytes; numbers, booleans and UUIDs come from a 64-bit FNV-1a hash of
 * the path. In the {@code nulls} case, every field nullable in the version is null; in
 * the {@code inner_nulls} case, only those with nothing nullable below them.
 */
final class Values {
    private static final JsonNodeFactory JSON = JsonNodeFactory.instance;
    private static final int ARRAY_LENGTH = 2;

    private final Schemas.Message message;
    private final short version;
    private final boolean nulls;
    private final boolean innermost;
    private boolean placedNull;
    private boolean keptNullable;

    private Values(Schemas.Message message, short version, boolean nulls, boolean innermost) {
        this.message = message;
        this.version = version;
        this.nulls = nulls;
        this.innermost = innermost;
    }

    /** The JSON for {@code message} at {@code version}. */
    static ObjectNode populated(Schemas.Message message, short version) {
        return new Values(message, version, false, false).struct(message.fields(), message.name());
    }

    /** The JSON with nulls where nullable, or null when nothing is nullable in the version. */
    static ObjectNode nulls(Schemas.Message message, short version) {
        Values values = new Values(message, version, true, false);
        ObjectNode node = values.struct(message.fields(), message.name());
        return values.placedNull ? node : null;
    }

    /**
     * The JSON with nulls in the nullable fields with nothing nullable below them, or null
     * when that is the same as {@link #nulls}: when no nullable field has a nullable field
     * below it.
     */
    static ObjectNode innerNulls(Schemas.Message message, short version) {
        Values values = new Values(message, version, true, true);
        ObjectNode node = values.struct(message.fields(), message.name());
        return values.keptNullable ? node : null;
    }

    private ObjectNode struct(List<Schemas.Field> fields, String path) {
        ObjectNode node = JSON.objectNode();
        for (Schemas.Field field : fields) {
            if (field.versions().contains(version)) {
                node.set(field.jsonName(), field(field, path + "." + field.name()));
            }
        }
        return node;
    }

    private JsonNode field(Schemas.Field field, String path) {
        if (nulls && field.nullableVersions().contains(version)) {
            if (innermost && hasNullableBelow(field)) {
                keptNullable = true;
            } else {
                placedNull = true;
                return JSON.nullNode();
            }
        }
        String element = field.elementType();
        if (element != null) {
            ArrayNode array = JSON.arrayNode();
            for (int index = 0; index < ARRAY_LENGTH; index++) {
                array.add(value(element, field, path + "[" + index + "]"));
            }
            return array;
        }
        return value(field.type(), field, path);
    }

    private JsonNode value(String type, Schemas.Field field, String path) {
        long hash = fnv1a(path);
        return switch (type) {
            case "bool" -> JSON.booleanNode((hash & 1) == 1);
            case "int8" -> JSON.numberNode((byte) hash);
            case "int16" -> JSON.numberNode((short) hash);
            case "uint16" -> JSON.numberNode(hash & 0xffff);
            case "int32" -> JSON.numberNode((int) hash);
            case "uint32" -> JSON.numberNode(hash & 0xffffffffL);
            case "int64" -> JSON.numberNode(hash);
            case "float64" -> JSON.numberNode((hash % 1_000_000) / 100.0);
            case "uuid" -> JSON.textNode(new Uuid(hash, fnv1a(path + "#")).toString());
            case "string" -> JSON.textNode(path);
            case "bytes", "records" -> JSON.textNode(
                Base64.getEncoder().encodeToString(path.getBytes(StandardCharsets.UTF_8)));
            default -> struct(structFields(type, field), path);
        };
    }

    /** Whether a field of the struct {@code field} holds, at any depth, is nullable. */
    private boolean hasNullableBelow(Schemas.Field field) {
        String type = field.elementType() != null ? field.elementType() : field.type();
        for (Schemas.Field inner : structFields(type, field)) {
            if (inner.versions().contains(version)
                && (inner.nullableVersions().contains(version) || hasNullableBelow(inner))) {
                return true;
            }
        }
        return false;
    }

    private List<Schemas.Field> structFields(String type, Schemas.Field field) {
        if (!field.fields().isEmpty()) {
            return field.fields();
        }
        return message.structs().getOrDefault(type, List.of());
    }

    /** 64-bit FNV-1a of {@code text}'s UTF-8 bytes. */
    static long fnv1a(String text) {
        long hash = 0xcbf29ce484222325L;
        for (byte b : text.getBytes(StandardCharsets.UTF_8)) {
            hash ^= b & 0xff;
            hash *= 0x100000001b3L;
        }
        return hash;
    }
}
