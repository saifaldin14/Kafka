//! Rustdoc for generated items: Kafka's `about` text made safe for rustdoc and clippy,
//! and each field's versions.

use crate::message_spec::FieldSpec;

/// `about` as rustdoc text.
///
/// - `[`, `]`, `<` and `>` are escaped, so text isn't read as a link or HTML tag.
/// - Bare URLs are put in angle brackets.
/// - Words that look like code are put in backticks: `CamelCase` and `snake_case` names
///   and paths. Clippy's `doc_markdown` lint asks for that, and the generated code
///   suppresses no lints.
pub(crate) fn about(text: &str) -> String {
    let mut in_code = false;
    let mut words = Vec::new();
    for word in text.split_whitespace() {
        let backticks = word.matches('`').count();
        if in_code || backticks > 0 {
            // Inside a code span nothing is read as a link or tag.
            words.push(word.to_owned());
            in_code ^= backticks % 2 == 1;
            continue;
        }
        words.push(format_word(word));
    }
    words.join(" ")
}

/// One word of prose: a URL in angle brackets, a code-like word in backticks around its
/// core, anything else escaped.
fn format_word(word: &str) -> String {
    if word.starts_with("http://") || word.starts_with("https://") {
        let url = word.trim_end_matches(['.', ',', ';', ')']);
        let rest = word.get(url.len()..).unwrap_or_default();
        return format!("<{url}>{rest}");
    }
    let start = word
        .find(|character: char| character.is_alphanumeric() || character == '_')
        .unwrap_or(word.len());
    let end = word
        .rfind(|character: char| character.is_alphanumeric() || character == '_')
        .map_or(start, |end| end + 1);
    let (Some(lead), Some(core), Some(trail)) =
        (word.get(..start), word.get(start..end), word.get(end..))
    else {
        return escape(word);
    };
    if looks_like_code(core) {
        format!("{}`{core}`{}", escape(lead), escape(trail))
    } else {
        escape(word)
    }
}

/// Whether clippy's `doc_markdown` would take `word` for code: a name with an underscore
/// or `::`, or `CamelCase`: starting upper-case, with another upper-case letter and a
/// lower-case one, ignoring a plural `s` (so `TopicId` and `KRaft`, but not `IDs`).
// Parity with clippy 1.99's `doc_markdown` (`check_word`), the lint this avoids.
fn looks_like_code(word: &str) -> bool {
    let singular = word.strip_suffix('s').unwrap_or(word);
    let camel = word.starts_with(|character: char| character.is_uppercase())
        && singular.chars().all(char::is_alphanumeric)
        && singular
            .chars()
            .filter(|character| character.is_uppercase())
            .count()
            > 1
        && singular.chars().any(char::is_lowercase);
    camel || word.contains('_') || word.contains("::")
}

fn escape(text: &str) -> String {
    text.replace('[', "\\[")
        .replace(']', "\\]")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// `text` as comment lines starting with `prefix` (such as `    /// `), wrapped before
/// 92 columns. A word longer than a line, such as a URL, gets a line of its own.
pub(crate) fn wrap(text: &str, prefix: &str) -> String {
    const WIDTH: usize = 92;
    let mut lines = String::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && prefix.len() + line.len() + 1 + word.len() > WIDTH {
            lines.push_str(prefix);
            lines.push_str(&line);
            lines.push('\n');
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    lines.push_str(prefix);
    lines.push_str(&line);
    lines.push('\n');
    lines
}

/// A field's versions as one doc line: `Versions 3+, nullable 3+, tagged 2+ (tag 0).`
pub(crate) fn field_versions(field: &FieldSpec) -> String {
    let mut parts = vec![format!("Versions {}", field.versions)];
    let nullable = field.nullable_versions.intersect(field.versions);
    if !nullable.is_empty() {
        parts.push(format!("nullable {nullable}"));
    }
    let tagged = field.tagged_versions.intersect(field.versions);
    if let (false, Some(tag)) = (tagged.is_empty(), field.tag) {
        parts.push(format!("tagged {tagged} (tag {tag})"));
    }
    if field.ignorable {
        parts.push("ignorable".to_owned());
    }
    format!("{}.", parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::{about, wrap};

    #[test]
    fn long_text_is_wrapped_with_its_prefix() {
        let text = "word ".repeat(30);
        let wrapped = wrap(&text, "    /// ");
        assert!(
            wrapped
                .lines()
                .all(|line| line.len() <= 92 && line.starts_with("    /// "))
        );
        assert_eq!(
            wrapped
                .split_whitespace()
                .filter(|word| *word == "word")
                .count(),
            30
        );
    }

    #[test]
    fn prose_is_escaped_and_code_is_backticked() {
        assert_eq!(
            about("The TopicId, or [null] if <unset>."),
            "The `TopicId`, or \\[null\\] if &lt;unset&gt;."
        );
        assert_eq!(
            about("See https://kafka.apache.org."),
            "See <https://kafka.apache.org>."
        );
        assert_eq!(about("Use `a [b]` here"), "Use `a [b]` here");
        assert_eq!(about("the max_bytes limit"), "the `max_bytes` limit");
        assert_eq!(about("The ID of the broker."), "The ID of the broker.");
        assert_eq!(about("KRaft controllers' IDs"), "`KRaft` controllers' IDs");
    }
}
