package alluvium.golden;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.stream.Stream;

import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

/**
 * Kafka's message schemas, read as far as the {@code messages} dump needs them: each
 * message's kind, versions and fields, and the structs it defines.
 *
 * <p>Structs are registered as Kafka's generator registers them: every common struct,
 * and the inline struct of the first field naming each struct type.
 */
final class Schemas {
    private Schemas() {
    }

    /** An inclusive version range; {@code lowest > highest} when empty. */
    record Versions(short lowest, short highest) {
        static final Versions NONE = new Versions((short) 0, (short) -1);

        // Parity: generator/src/main/java/org/apache/kafka/message/Versions.java@4.4.0 Versions.parse
        static Versions parse(String text) {
            if (text == null || text.isEmpty() || text.equals("none")) {
                return NONE;
            }
            if (text.endsWith("+")) {
                return new Versions(Short.parseShort(text.substring(0, text.length() - 1)), Short.MAX_VALUE);
            }
            int dash = text.indexOf('-');
            if (dash < 0) {
                short version = Short.parseShort(text);
                return new Versions(version, version);
            }
            return new Versions(Short.parseShort(text.substring(0, dash)),
                Short.parseShort(text.substring(dash + 1)));
        }

        boolean contains(short version) {
            return lowest <= version && version <= highest;
        }

        boolean isEmpty() {
            return lowest > highest;
        }
    }

    record Field(String name, String type, Versions versions, Versions nullableVersions,
                 List<Field> fields) {
        /** The key {@code JsonConverter} reads the field from: its name, first letter lower. */
        String jsonName() {
            return Character.toLowerCase(name.charAt(0)) + name.substring(1);
        }

        /** The element type of an array, or null. */
        String elementType() {
            return type.startsWith("[]") ? type.substring(2) : null;
        }
    }

    record Message(String name, String kind, Versions validVersions, Versions flexibleVersions,
                   List<Field> fields, Map<String, List<Field>> structs) {
        /** The class Kafka generates for it: {@code <Name>Data} for APIs and headers. */
        // Parity: generator/src/main/java/org/apache/kafka/message/MessageSpec.java@4.4.0 MessageSpec.dataClassName
        String dataClassName() {
            return switch (kind) {
                case "request", "response", "header" -> name + "Data";
                default -> name;
            };
        }
    }

    /** Every schema in {@code dir} with valid versions, sorted by file name. */
    static List<Message> load(Path dir) throws IOException {
        ObjectMapper mapper = new ObjectMapper();
        mapper.configure(JsonParser.Feature.ALLOW_COMMENTS, true);
        List<Path> files;
        try (Stream<Path> listing = Files.list(dir)) {
            files = listing.filter(path -> path.toString().endsWith(".json")).sorted().toList();
        }
        List<Message> messages = new ArrayList<>();
        for (Path file : files) {
            JsonNode node = mapper.readTree(file.toFile());
            Versions valid = Versions.parse(node.path("validVersions").asText(null));
            if (valid.isEmpty()) {
                continue;
            }
            Map<String, List<Field>> structs = new HashMap<>();
            for (JsonNode common : node.path("commonStructs")) {
                structs.put(common.get("name").asText(), fields(common.path("fields"), structs));
            }
            List<Field> fields = fields(node.path("fields"), structs);
            messages.add(new Message(node.get("name").asText(), node.get("type").asText(), valid,
                Versions.parse(node.path("flexibleVersions").asText(null)), fields, structs));
        }
        return messages;
    }

    private static List<Field> fields(JsonNode array, Map<String, List<Field>> structs) {
        List<Field> fields = new ArrayList<>();
        for (JsonNode node : array) {
            String type = node.get("type").asText();
            List<Field> inline = fields(node.path("fields"), structs);
            String struct = type.startsWith("[]") ? type.substring(2) : type;
            if (node.has("fields")) {
                structs.putIfAbsent(struct, inline);
            }
            fields.add(new Field(node.get("name").asText(), type,
                Versions.parse(node.path("versions").asText(null)),
                Versions.parse(node.path("nullableVersions").asText(null)), inline));
        }
        return fields;
    }
}
