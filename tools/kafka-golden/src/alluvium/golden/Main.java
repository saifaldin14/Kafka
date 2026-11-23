package alluvium.golden;

/**
 * The Kafka golden harness: runs Kafka 4.4.0's own classes to record what Kafka does,
 * as files Alluvium's tests compare against.
 *
 * <p>{@code cargo xtask golden <dump>} compiles it and runs it in a pinned JDK image
 * under Docker, without network access, with the Kafka client jar and its runtime
 * dependencies on the class path (xtask/src/golden.rs). Each dump writes under the
 * output directory it is given; xtask moves the result into the repository.
 */
public final class Main {
    private Main() {
    }

    public static void main(String[] args) throws Exception {
        if (args.length < 2) {
            usage();
        }
        java.nio.file.Path out = java.nio.file.Path.of(args[1]);
        switch (args[0]) {
            case "errors" -> ErrorsDump.write(out);
            case "api-keys" -> ApiKeysDump.write(out);
            case "messages" -> {
                if (args.length != 3) {
                    usage();
                }
                MessagesDump.write(out, java.nio.file.Path.of(args[2]));
            }
            default -> usage();
        }
    }

    private static void usage() {
        System.err.println("usage: Main <dump> <output-directory> [<schemas-directory>]; dumps: "
            + "errors, api-keys, messages <schemas-directory>");
        System.exit(2);
    }
}
