// Prints Kafka's Errors.values() as JSON, for `cargo xtask schemas dump-errors`
// (xtask/src/errors_dump.rs): name, code, default message, the exception class, and
// whether the exception is a RetriableException or an InvalidMetadataException.
import org.apache.kafka.common.protocol.Errors;
import org.apache.kafka.common.errors.InvalidMetadataException;
import org.apache.kafka.common.errors.RetriableException;

String json(String text) {
    if (text == null) return "null";
    StringBuilder out = new StringBuilder("\"");
    for (char c : text.toCharArray()) {
        if (c == '"' || c == '\\') out.append('\\').append(c);
        else if (c < 0x20 || c > 0x7e) out.append(String.format("\\u%04x", (int) c));
        else out.append(c);
    }
    return out.append('"').toString();
}

StringBuilder out = new StringBuilder("[\n");
Errors[] all = Errors.values();
for (int i = 0; i < all.length; i++) {
    Errors error = all[i];
    Exception exception = error.exception();
    out.append(String.format(
        "  {\"name\": %s, \"code\": %d, \"message\": %s, \"exception\": %s, \"retriable\": %b, \"invalidates_metadata\": %b}%s\n",
        json(error.name()), error.code(),
        json(exception == null ? null : exception.getMessage()),
        json(exception == null ? null : exception.getClass().getSimpleName()),
        exception instanceof RetriableException,
        exception instanceof InvalidMetadataException,
        i + 1 < all.length ? "," : ""));
}
out.append("]\n");
java.nio.file.Files.writeString(java.nio.file.Path.of("/work/errors.json"), out.toString());
/exit
