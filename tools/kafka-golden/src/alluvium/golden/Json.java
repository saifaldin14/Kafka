package alluvium.golden;

/** JSON text for values the dumps write by hand, so their output is plain ASCII. */
final class Json {
    private Json() {
    }

    /** {@code text} as a JSON string, or {@code null}; anything outside ASCII is escaped. */
    static String string(String text) {
        if (text == null) {
            return "null";
        }
        StringBuilder out = new StringBuilder("\"");
        for (char c : text.toCharArray()) {
            if (c == '"' || c == '\\') {
                out.append('\\').append(c);
            } else if (c < 0x20 || c > 0x7e) {
                out.append(String.format("\\u%04x", (int) c));
            } else {
                out.append(c);
            }
        }
        return out.append('"').toString();
    }
}
