//! Printing client-supplied strings without letting them forge log lines.
//!
//! Group ids, client ids, transactional ids and other strings that clients choose are
//! logged, as Kafka logs them (docs/standards/debuggability.md#printing-values). A raw
//! newline in one would start a fake log line, and a bidirectional override would make
//! the rest of the line read differently from what it says. [`LogSafe`] prints such
//! characters as escapes and everything else unchanged.

use std::fmt::{self, Write};

/// Displays a string with the characters that could forge or disguise a log line
/// escaped.
///
/// Escaped, as `\n`, `\r`, `\t`, or `\u{hex}` for the rest:
///
/// - control characters (Unicode `Cc`: U+0000–U+001F, U+007F–U+009F);
/// - the line and paragraph separators U+2028 and U+2029, which some log viewers break
///   lines on;
/// - the bidirectional formatting characters U+061C, U+200E, U+200F, U+202A–U+202E and
///   U+2066–U+2069, which reorder how the rest of a line is displayed.
///
/// Every other character, including `\` and all printable Unicode, prints unchanged.
/// The output is for reading, not parsing: a literal `\n` in the input looks like an
/// escaped newline.
///
/// ```
/// use alluvium_types::LogSafe;
///
/// assert_eq!(LogSafe("orders\nERROR forged").to_string(), r"orders\nERROR forged");
/// assert_eq!(LogSafe("café-group").to_string(), "café-group");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct LogSafe<'a>(pub &'a str);

impl fmt::Display for LogSafe<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for character in self.0.chars() {
            match character {
                '\n' => f.write_str(r"\n")?,
                '\r' => f.write_str(r"\r")?,
                '\t' => f.write_str(r"\t")?,
                other if needs_escape(other) => write!(f, "\\u{{{:x}}}", u32::from(other))?,
                other => f.write_char(other)?,
            }
        }
        Ok(())
    }
}

fn needs_escape(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{2028}'
                | '\u{2029}'
                | '\u{061c}'
                | '\u{200e}'
                | '\u{200f}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2066}'..='\u{2069}'
        )
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};

    use super::{LogSafe, needs_escape};

    #[test]
    fn line_breaks_and_controls_are_escaped() {
        let cases = [
            ("a\nb", r"a\nb"),
            ("a\r\nb", r"a\r\nb"),
            ("tab\there", r"tab\there"),
            ("nul\u{0}", r"nul\u{0}"),
            ("bell\u{7}", r"bell\u{7}"),
            ("del\u{7f}", r"del\u{7f}"),
            ("c1\u{85}", r"c1\u{85}"),
            ("line\u{2028}sep", r"line\u{2028}sep"),
            ("para\u{2029}sep", r"para\u{2029}sep"),
            ("rlo\u{202e}txt.exe", r"rlo\u{202e}txt.exe"),
            ("isolate\u{2066}x\u{2069}", r"isolate\u{2066}x\u{2069}"),
            ("\n", r"\n"),
        ];
        for (input, expected) in cases {
            assert_eq!(LogSafe(input).to_string(), expected, "{input:?}");
        }
    }

    #[test]
    fn printable_text_is_unchanged() {
        for input in [
            "",
            "payments",
            r#"back\slash "quoted" 'single'"#,
            "café 日本語 emoji 🚀",
            "zero\u{200b}width",
        ] {
            assert_eq!(LogSafe(input).to_string(), input);
        }
    }

    proptest! {
        #[test]
        fn output_never_contains_a_character_that_needs_escaping(input in any::<String>()) {
            let shown = LogSafe(&input).to_string();
            prop_assert!(!shown.chars().any(needs_escape), "{:?}", shown);
            prop_assert!(!shown.contains('\n') && !shown.contains('\r'));
        }

        #[test]
        fn strings_without_such_characters_print_unchanged(input in any::<String>()) {
            let clean: String = input.chars().filter(|&c| !needs_escape(c)).collect();
            prop_assert_eq!(LogSafe(&clean).to_string(), clean);
        }
    }
}
