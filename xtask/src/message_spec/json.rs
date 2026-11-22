//! Kafka's schema files: JSON with `//` and `/* */` comments.
//!
//! Kafka reads them with Jackson's `ALLOW_COMMENTS` and `FAIL_ON_TRAILING_TOKENS`. Here
//! the comments are blanked out byte for byte, keeping line breaks, so `serde_json`
//! parses the rest and its line and column numbers still point into the original
//! file. A comment marker inside a string literal is part of the string.

// Parity: generator/src/main/java/org/apache/kafka/message/MessageGenerator.java@4.4.0 MessageGenerator.JSON_SERDE

use serde_json::Value;

/// A syntax error at a 1-based line and column (in bytes) of the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SyntaxError {
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) message: String,
}

/// Parses a schema file's text into a JSON value.
pub(crate) fn parse(text: &str) -> Result<Value, SyntaxError> {
    let stripped = strip_comments(text)?;
    serde_json::from_str(&stripped).map_err(|error| SyntaxError {
        line: error.line(),
        column: error.column(),
        message: without_position(&error.to_string()),
    })
}

/// `text` with every comment's bytes replaced by spaces, except line breaks.
fn strip_comments(text: &str) -> Result<String, SyntaxError> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    let mut in_string = false;
    while let Some(&byte) = bytes.get(index) {
        let next = bytes.get(index + 1).copied();
        if in_string {
            out.push(byte);
            match byte {
                b'\\' => {
                    if let Some(escaped) = next {
                        out.push(escaped);
                        index += 1;
                    }
                }
                b'"' => in_string = false,
                _ => {}
            }
            index += 1;
            continue;
        }
        match (byte, next) {
            (b'"', _) => {
                in_string = true;
                out.push(byte);
                index += 1;
            }
            (b'/', Some(b'/')) => {
                let end = bytes
                    .get(index..)
                    .and_then(|rest| rest.iter().position(|&b| b == b'\n'))
                    .map_or(bytes.len(), |offset| index + offset);
                out.resize(out.len() + (end - index), b' ');
                index = end;
            }
            (b'/', Some(b'*')) => {
                let end = bytes
                    .get(index + 2..)
                    .and_then(|rest| rest.windows(2).position(|pair| pair == b"*/"))
                    .map(|offset| index + 2 + offset + 2)
                    .ok_or_else(|| {
                        let (line, column) = position(text, index);
                        SyntaxError {
                            line,
                            column,
                            message: "unterminated `/*` comment".to_owned(),
                        }
                    })?;
                for &blanked in bytes.get(index..end).unwrap_or_default() {
                    out.push(if blanked == b'\n' { b'\n' } else { b' ' });
                }
                index = end;
            }
            _ => {
                out.push(byte);
                index += 1;
            }
        }
    }
    // Only whole comments, which may contain multi-byte characters, were replaced, and
    // always by ASCII, so the result is still UTF-8.
    String::from_utf8(out).map_err(|_| SyntaxError {
        line: 1,
        column: 1,
        message: "the file is not UTF-8".to_owned(),
    })
}

/// The 1-based line and byte column of byte `offset` in `text`.
fn position(text: &str, offset: usize) -> (usize, usize) {
    let before = text.as_bytes().get(..offset).unwrap_or_default();
    let line = before.split(|&byte| byte == b'\n').count();
    let line_start = before
        .iter()
        .rposition(|&byte| byte == b'\n')
        .map_or(0, |at| at + 1);
    (line, offset - line_start + 1)
}

/// `serde_json`'s message without its " at line L column C" suffix, which the error
/// already carries as numbers.
fn without_position(message: &str) -> String {
    message
        .rfind(" at line ")
        .and_then(|at| message.get(..at))
        .unwrap_or(message)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{SyntaxError, parse};

    #[test]
    fn comments_are_ignored_but_markers_inside_strings_are_kept() {
        let text = "// Licensed to the ASF.\n{\n  \"about\": \"a // b /* c */\", /* note */\n  \"n\": 1 // end\n}\n";
        assert_eq!(
            parse(text),
            Ok(json!({ "about": "a // b /* c */", "n": 1 }))
        );
        let escaped = "{ \"about\": \"quote \\\" // still a string\" }";
        assert_eq!(
            parse(escaped),
            Ok(json!({ "about": "quote \" // still a string" }))
        );
    }

    #[test]
    fn an_unterminated_block_comment_is_reported_where_it_starts() {
        assert_eq!(
            parse("{\n  \"a\": 1 /* never closed\n}\n"),
            Err(SyntaxError {
                line: 2,
                column: 10,
                message: "unterminated `/*` comment".to_owned()
            })
        );
    }

    #[test]
    fn trailing_tokens_and_bad_json_keep_the_original_position() {
        let error = parse("/* a\n multi-line comment */ { \"a\": 1 }\n{ }\n").unwrap_err();
        assert_eq!((error.line, error.column), (3, 1));
        assert_eq!(error.message, "trailing characters");
        let error = parse("{\n  \"a\": /* é */ ,\n}").unwrap_err();
        assert_eq!(error.line, 2);
    }
}
