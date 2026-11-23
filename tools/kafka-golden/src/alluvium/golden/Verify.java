package alluvium.golden;

import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.stream.Stream;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.JsonNodeFactory;
import com.fasterxml.jackson.databind.node.ObjectNode;

import org.apache.kafka.common.protocol.ByteBufferAccessor;
import org.apache.kafka.common.protocol.Message;
import org.apache.kafka.common.protocol.MessageUtil;
import org.apache.kafka.common.protocol.Readable;

/**
 * The reverse check: Kafka 4.4.0 reads the messages Alluvium encoded and writes them
 * back, which must give the same bytes.
 *
 * <p>Each input file is {@code <Message>_v<version>.bin}: messages encoded by Alluvium at
 * that version, each preceded by its length as an {@code int32}. Each message is read
 * with the generated class's {@code (Readable, short)} constructor, must use all of its
 * bytes, and is written with {@code MessageUtil.toByteBufferAccessor}. {@code report.json}
 * gives the number checked and the first mismatches, each with the message, version,
 * index in its file, and the first differing byte or the exception.
 */
final class Verify {
    private static final int SHOWN_MISMATCHES = 50;
    private static final JsonNodeFactory JSON = JsonNodeFactory.instance;

    private Verify() {
    }

    static void write(Path out, Path encoded, Path schemas) throws Exception {
        Map<String, String> classes = new HashMap<>();
        for (Schemas.Message message : Schemas.load(schemas)) {
            classes.put(message.name(), message.dataClassName());
        }
        List<Path> files;
        try (Stream<Path> listing = Files.list(encoded)) {
            files = listing.filter(path -> path.toString().endsWith(".bin")).sorted().toList();
        }
        long checked = 0;
        long mismatched = 0;
        ArrayNode shown = JSON.arrayNode();
        for (Path file : files) {
            String stem = file.getFileName().toString().replace(".bin", "");
            int at = stem.lastIndexOf("_v");
            String name = stem.substring(0, at);
            short version = Short.parseShort(stem.substring(at + 2));
            String dataClass = classes.get(name);
            if (dataClass == null) {
                throw new IllegalArgumentException(file + ": no schema named " + name);
            }
            Constructor<?> read = Class.forName("org.apache.kafka.common.message." + dataClass)
                .getConstructor(Readable.class, short.class);
            ByteBuffer input = ByteBuffer.wrap(Files.readAllBytes(file));
            for (int index = 0; input.hasRemaining(); index++) {
                byte[] bytes = new byte[input.getInt()];
                input.get(bytes);
                checked++;
                String problem = problem(read, bytes, version);
                if (problem != null) {
                    mismatched++;
                    if (shown.size() < SHOWN_MISMATCHES) {
                        ObjectNode entry = shown.addObject();
                        entry.put("message", name);
                        entry.put("version", version);
                        entry.put("index", index);
                        entry.put("problem", problem);
                    }
                }
            }
        }
        ObjectNode report = JSON.objectNode();
        report.put("checked", checked);
        report.put("mismatched", mismatched);
        report.set("mismatches", shown);
        Files.createDirectories(out);
        Files.writeString(out.resolve("report.json"),
            new ObjectMapper().enable(SerializationFeature.INDENT_OUTPUT).writeValueAsString(report) + "\n");
    }

    /** What is wrong with Kafka's reading and writing of {@code bytes}, or null. */
    private static String problem(Constructor<?> read, byte[] bytes, short version) {
        try {
            ByteBufferAccessor accessor = new ByteBufferAccessor(ByteBuffer.wrap(bytes));
            Message message = (Message) read.newInstance(accessor, version);
            if (accessor.remaining() != 0) {
                return "Kafka read " + (bytes.length - accessor.remaining()) + " of " + bytes.length + " bytes";
            }
            byte[] written = MessageUtil.byteBufferToArray(
                MessageUtil.toByteBufferAccessor(message, version).buffer());
            int length = Math.min(bytes.length, written.length);
            for (int offset = 0; offset < length; offset++) {
                if (bytes[offset] != written[offset]) {
                    return "first difference at byte " + offset;
                }
            }
            return bytes.length == written.length ? null
                : "Kafka wrote " + written.length + " bytes for " + bytes.length;
        } catch (InvocationTargetException error) {
            return "Kafka threw " + error.getCause();
        } catch (ReflectiveOperationException | RuntimeException error) {
            return "Kafka threw " + error;
        }
    }
}
