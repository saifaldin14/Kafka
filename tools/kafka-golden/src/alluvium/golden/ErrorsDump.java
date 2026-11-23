package alluvium.golden;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

import org.apache.kafka.common.errors.InvalidMetadataException;
import org.apache.kafka.common.errors.RetriableException;
import org.apache.kafka.common.protocol.Errors;

/**
 * The {@code errors} dump: Kafka's {@code Errors.values()} as JSON, the golden table
 * the generated {@code ErrorCode} is tested against. Each entry has the name, the code,
 * the default message, the exception class, and whether the exception is a
 * {@code RetriableException} or an {@code InvalidMetadataException}.
 */
final class ErrorsDump {
    private ErrorsDump() {
    }

    static void write(Path out) throws IOException {
        StringBuilder json = new StringBuilder("[\n");
        Errors[] all = Errors.values();
        for (int i = 0; i < all.length; i++) {
            Errors error = all[i];
            Exception exception = error.exception();
            json.append(String.format(
                "  {\"name\": %s, \"code\": %d, \"message\": %s, \"exception\": %s, \"retriable\": %b, \"invalidates_metadata\": %b}%s\n",
                Json.string(error.name()), error.code(),
                Json.string(exception == null ? null : exception.getMessage()),
                Json.string(exception == null ? null : exception.getClass().getSimpleName()),
                exception instanceof RetriableException,
                exception instanceof InvalidMetadataException,
                i + 1 < all.length ? "," : ""));
        }
        json.append("]\n");
        Files.createDirectories(out);
        Files.writeString(out.resolve("errors.json"), json.toString());
    }
}
