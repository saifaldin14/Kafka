//! Kafka's error codes, read from the vendored `Errors.java` and exception classes, and
//! rendered as `ErrorCode` and docs/errors.md.
//!
//! [`extract`] reads the `Errors` enum's constants (name, code, default message and the
//! exception its builder creates) and resolves each exception's superclass chain through
//! the vendored classes to `java.lang.RuntimeException`. An exception is retriable when
//! the chain reaches `RetriableException`, and invalidates metadata when it reaches
//! `InvalidMetadataException`, as Kafka's clients test with `instanceof`. A class the
//! chain can't resolve fails extraction rather than defaulting to "not retriable".
//!
//! The Java is read with a small tokenizer (comments, string and character literals,
//! identifiers, numbers and punctuation), so formatting and comments in the source don't
//! matter; the grammar of each constant is checked exactly, and anything else fails.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use super::docs;

/// One of Kafka's error codes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ErrorEntry {
    /// The `Errors` constant: `NOT_LEADER_OR_FOLLOWER`.
    pub(crate) name: String,
    pub(crate) code: i16,
    /// The default message, which is null only for `NONE`.
    pub(crate) message: Option<String>,
    /// The exception class the code maps to, which is none only for `NONE`.
    pub(crate) exception: Option<String>,
    pub(crate) retriable: bool,
    pub(crate) invalidates_metadata: bool,
}

/// The file in the errors directory that defines the codes.
const ERRORS_FILE: &str = "Errors.java";

/// Where every exception's superclass chain ends.
const ROOT_CLASS: &str = "RuntimeException";

/// Every error code in `dir`'s `Errors.java`, in code order. The codes must be contiguous,
/// so a code's position in the list is its distance from the lowest.
// Parity: clients/src/main/java/org/apache/kafka/common/protocol/Errors.java@4.4.0 Errors
pub(crate) fn extract(dir: &Path) -> Result<Vec<ErrorEntry>, String> {
    let read = |name: &str| {
        std::fs::read_to_string(dir.join(name))
            .map_err(|error| format!("could not read {}: {error}", dir.join(name).display()))
    };
    let superclasses = superclasses(dir)?;
    let constants = constants(&tokens(&read(ERRORS_FILE)?)?)?;
    let mut entries = Vec::with_capacity(constants.len());
    for constant in constants {
        let ancestors = match &constant.exception {
            Some(class) => ancestors(class, &superclasses)?,
            None => Vec::new(),
        };
        entries.push(ErrorEntry {
            retriable: ancestors.contains(&"RetriableException"),
            invalidates_metadata: ancestors.contains(&"InvalidMetadataException"),
            name: constant.name,
            code: constant.code,
            message: constant.message,
            exception: constant.exception,
        });
    }
    entries.sort_by_key(|entry| entry.code);
    let mut names = BTreeMap::new();
    for (index, entry) in entries.iter().enumerate() {
        if let Some(previous) = names.insert(entry.name.as_str(), entry.code) {
            return Err(format!(
                "{ERRORS_FILE}: {} is defined twice (codes {previous} and {})",
                entry.name, entry.code
            ));
        }
        let expected = entries
            .first()
            .and_then(|first| i16::try_from(index).ok()?.checked_add(first.code));
        if expected != Some(entry.code) {
            return Err(format!(
                "{ERRORS_FILE}: the codes are not contiguous at {} ({})",
                entry.name, entry.code
            ));
        }
    }
    Ok(entries)
}

/// A token of Java source.
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Identifier(String),
    Number(String),
    /// A string literal's value, escapes decoded.
    Text(String),
    Punct(&'static str),
}

/// The tokens of `source`, without whitespace and comments.
fn tokens(source: &str) -> Result<Vec<Token>, String> {
    const PUNCT: &[&str] = &[
        "->", "::", "(", ")", "{", "}", "[", "]", "<", ">", ",", ";", ".", "+", "-", "=", "@", "?",
        "!", "&", "|", "*", "/", "%", ":", "^", "~",
    ];
    let mut tokens = Vec::new();
    let mut rest = source;
    while let Some(first) = rest.chars().next() {
        if first.is_whitespace() {
            rest = rest.trim_start();
        } else if let Some(comment) = rest.strip_prefix("//") {
            rest = comment.split_once('\n').map_or("", |(_, after)| after);
        } else if let Some(comment) = rest.strip_prefix("/*") {
            rest = comment
                .split_once("*/")
                .ok_or("a /* comment is not closed")?
                .1;
        } else if first == '"' || first == '\'' {
            let (value, after) = literal(rest, first)?;
            if first == '"' {
                tokens.push(Token::Text(value));
            }
            rest = after;
        } else if first.is_ascii_alphabetic() || first == '_' || first == '$' {
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '$'))
                .unwrap_or(rest.len());
            let (word, after) = rest.split_at(end);
            tokens.push(Token::Identifier(word.to_owned()));
            rest = after;
        } else if first.is_ascii_digit() {
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            let (number, after) = rest.split_at(end);
            tokens.push(Token::Number(number.to_owned()));
            rest = after;
        } else if let Some(punct) = PUNCT.iter().find(|punct| rest.starts_with(**punct)) {
            tokens.push(Token::Punct(punct));
            rest = rest.get(punct.len()..).unwrap_or_default();
        } else {
            return Err(format!("unexpected character {first:?}"));
        }
    }
    Ok(tokens)
}

/// The value of the string or character literal at the start of `text`, quoted by
/// `quote`, and the text after it.
fn literal(text: &str, quote: char) -> Result<(String, &str), String> {
    let mut value = String::new();
    let mut chars = text.char_indices().skip(1);
    while let Some((index, c)) = chars.next() {
        match c {
            _ if c == quote => {
                let after = text.get(index + c.len_utf8()..).unwrap_or_default();
                return Ok((value, after));
            }
            '\n' => break,
            '\\' => {
                let escaped = match chars.next().map(|(_, escaped)| escaped) {
                    Some('n') => '\n',
                    Some('t') => '\t',
                    Some('r') => '\r',
                    Some('b') => '\u{8}',
                    Some('f') => '\u{c}',
                    Some('s') => ' ',
                    Some(c @ ('"' | '\'' | '\\')) => c,
                    Some('u') => {
                        let hex: String = chars.by_ref().take(4).map(|(_, c)| c).collect();
                        u32::from_str_radix(&hex, 16)
                            .ok()
                            .and_then(char::from_u32)
                            .ok_or_else(|| format!("bad escape \\u{hex}"))?
                    }
                    other => return Err(format!("unsupported escape \\{other:?}")),
                };
                value.push(escaped);
            }
            _ => value.push(c),
        }
    }
    Err("a literal is not closed on its line".to_owned())
}

/// An `Errors` constant as written.
struct Constant {
    name: String,
    code: i16,
    message: Option<String>,
    exception: Option<String>,
}

/// The constants of `enum Errors`: each is `NAME(code, message, builder)`, where the
/// message is `null` or string literals joined by `+`, and the builder is
/// `Exception::new`, `message -> new Exception(message)` or `message -> null`.
fn constants(tokens: &[Token]) -> Result<Vec<Constant>, String> {
    let start = tokens
        .windows(3)
        .position(|window| {
            window
                == [
                    Token::Identifier("enum".to_owned()),
                    Token::Identifier("Errors".to_owned()),
                    Token::Punct("{"),
                ]
        })
        .ok_or(format!("{ERRORS_FILE}: no `enum Errors {{`"))?;
    let mut parser = Parser {
        tokens: tokens.get(start + 3..).unwrap_or_default(),
    };
    let mut constants: Vec<Constant> = Vec::new();
    loop {
        let name = parser.identifier().map_err(|problem| {
            let after = constants
                .last()
                .map_or_else(String::new, |last| format!(" after {}", last.name));
            format!("{ERRORS_FILE}: a constant{after}: {problem}")
        })?;
        let at = |problem: String| format!("{ERRORS_FILE}: {name}: {problem}");
        parser.punct("(").map_err(at)?;
        let negative = parser.eat_punct("-");
        let digits = parser.number().map_err(at)?;
        let code = format!("{}{digits}", if negative { "-" } else { "" })
            .parse::<i16>()
            .map_err(|error| at(format!("code {digits}: {error}")))?;
        parser.punct(",").map_err(at)?;
        let message = if parser.eat_identifier("null") {
            None
        } else {
            let mut message = parser.text().map_err(at)?;
            while parser.eat_punct("+") {
                message.push_str(&parser.text().map_err(at)?);
            }
            Some(message)
        };
        parser.punct(",").map_err(at)?;
        let exception = parser.builder().map_err(at)?;
        parser.punct(")").map_err(at)?;
        constants.push(Constant {
            name,
            code,
            message,
            exception,
        });
        if parser.eat_punct(";") {
            return Ok(constants);
        }
        parser
            .punct(",")
            .map_err(|problem| format!("{ERRORS_FILE}: after code {code}: {problem}"))?;
    }
}

struct Parser<'a> {
    tokens: &'a [Token],
}

impl Parser<'_> {
    fn next(&mut self) -> Option<&Token> {
        let (first, rest) = self.tokens.split_first()?;
        self.tokens = rest;
        Some(first)
    }

    fn expected(&self, what: &str) -> String {
        format!("expected {what}, found {:?}", self.tokens.first())
    }

    fn identifier(&mut self) -> Result<String, String> {
        match self.tokens.first() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.next();
                Ok(name)
            }
            _ => Err(self.expected("an identifier")),
        }
    }

    fn number(&mut self) -> Result<String, String> {
        match self.tokens.first() {
            Some(Token::Number(digits)) => {
                let digits = digits.clone();
                self.next();
                Ok(digits)
            }
            _ => Err(self.expected("a number")),
        }
    }

    fn text(&mut self) -> Result<String, String> {
        match self.tokens.first() {
            Some(Token::Text(text)) => {
                let text = text.clone();
                self.next();
                Ok(text)
            }
            _ => Err(self.expected("a string")),
        }
    }

    fn eat_punct(&mut self, punct: &str) -> bool {
        let found = matches!(self.tokens.first(), Some(Token::Punct(found)) if *found == punct);
        if found {
            self.next();
        }
        found
    }

    fn punct(&mut self, punct: &str) -> Result<(), String> {
        if self.eat_punct(punct) {
            Ok(())
        } else {
            Err(self.expected(&format!("`{punct}`")))
        }
    }

    fn eat_identifier(&mut self, word: &str) -> bool {
        let found = matches!(self.tokens.first(), Some(Token::Identifier(found)) if found == word);
        if found {
            self.next();
        }
        found
    }

    /// An exception builder, as the class it creates, or none for `message -> null`.
    fn builder(&mut self) -> Result<Option<String>, String> {
        let first = self.identifier()?;
        if self.eat_punct("::") {
            return if self.eat_identifier("new") {
                Ok(Some(first))
            } else {
                Err(self.expected("`new`"))
            };
        }
        self.punct("->")?;
        if self.eat_identifier("null") {
            return Ok(None);
        }
        if !self.eat_identifier("new") {
            return Err(self.expected("`new` or `null`"));
        }
        let class = self.identifier()?;
        self.punct("(")?;
        if self.identifier()? != first {
            return Err(format!("the builder must pass `{first}` on"));
        }
        self.punct(")")?;
        Ok(Some(class))
    }
}

/// The superclass of every class declared in `dir`'s `.java` files, by simple name.
fn superclasses(dir: &Path) -> Result<BTreeMap<String, String>, String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|error| format!("could not read {}: {error}", dir.display()))?;
    let mut superclasses = BTreeMap::new();
    for entry in entries {
        let path = entry
            .map_err(|error| format!("could not read {}: {error}", dir.display()))?
            .path();
        if path.extension().is_none_or(|extension| extension != "java") {
            continue;
        }
        let file = path
            .file_name()
            .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("could not read {file}: {error}"))?;
        let tokens = tokens(&text).map_err(|problem| format!("{file}: {problem}"))?;
        if let Some((class, superclass)) =
            declaration(&tokens).map_err(|problem| format!("{file}: {problem}"))?
        {
            superclasses.insert(class, superclass);
        }
    }
    Ok(superclasses)
}

/// The class `tokens` declare first, and its superclass's simple name if it has
/// `extends`.
fn declaration(tokens: &[Token]) -> Result<Option<(String, String)>, String> {
    // `Errors.class` is a class literal, not a declaration.
    let Some(start) = tokens.iter().enumerate().position(|(index, token)| {
        *token == Token::Identifier("class".to_owned())
            && index
                .checked_sub(1)
                .and_then(|previous| tokens.get(previous))
                .is_none_or(|previous| *previous != Token::Punct("."))
    }) else {
        return Ok(None);
    };
    let mut parser = Parser {
        tokens: tokens.get(start + 1..).unwrap_or_default(),
    };
    let class = parser.identifier()?;
    if parser.eat_punct("<") {
        while !parser.eat_punct(">") {
            parser.next().ok_or("unclosed type parameters")?;
        }
    }
    if !parser.eat_identifier("extends") {
        return Ok(None);
    }
    let mut superclass = parser.identifier()?;
    while parser.eat_punct(".") {
        superclass = parser.identifier()?;
    }
    Ok(Some((class, superclass)))
}

/// `class` and its superclasses up to `RuntimeException`.
fn ancestors<'a>(
    class: &'a str,
    superclasses: &'a BTreeMap<String, String>,
) -> Result<Vec<&'a str>, String> {
    let mut chain = vec![class];
    let mut current = class;
    while current != ROOT_CLASS {
        current = superclasses.get(current).ok_or_else(|| {
            format!(
                "cannot resolve the superclass of {current}, in the chain of {class}: it is \
                 not declared with `extends` in a vendored file"
            )
        })?;
        if chain.contains(&current) {
            return Err(format!("{class}'s superclasses loop at {current}"));
        }
        chain.push(current);
    }
    Ok(chain)
}

/// `NOT_LEADER_OR_FOLLOWER` as a Rust variant: `NotLeaderOrFollower`.
pub(crate) fn variant(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_string() + &chars.as_str().to_ascii_lowercase()
            })
        })
        .collect()
}

/// The `ErrorCode` module's source, before formatting.
pub(crate) fn module(entries: &[ErrorEntry]) -> String {
    let mut out = docs::wrap(
        "[`ErrorCode`]: every error code of Kafka 4.4.0, generated from `Errors.java`. \
         `from_code` and `Display` are written by hand in `error_code.rs`.",
        "//! ",
    );
    out.push('\n');
    out.push_str(&docs::wrap(
        "A Kafka error code, as the `ErrorCode` fields of responses carry it.",
        "/// ",
    ));
    out.push_str("///\n");
    out.push_str(&docs::wrap(
        "Each variant is one constant of Kafka's `Errors` enum, documented with its wire \
         code and Kafka's default message. docs/errors.md lists them all.",
        "/// ",
    ));
    out.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\npub enum ErrorCode {\n");
    for entry in entries {
        let message = entry
            .message
            .as_deref()
            .map_or_else(|| "No error.".to_owned(), docs::about);
        out.push_str(&docs::wrap(
            &format!("`{}` ({}): {message}", entry.name, entry.code),
            "    /// ",
        ));
        // Writing to a String can't fail; here and below.
        let _ = writeln!(out, "    {},", variant(&entry.name));
    }
    out.push_str("}\n\n");
    out.push_str(&docs::wrap(
        "What Kafka says about one code. `FACTS` has one per variant, in declaration \
         order, so a variant's position (`self as usize`) is its index.",
        "/// ",
    ));
    out.push_str(
        "struct Facts {\n    code: i16,\n    name: &'static str,\n    \
         default_message: Option<&'static str>,\n    retriable: bool,\n    \
         invalidates_metadata: bool,\n}\n\n",
    );
    let _ = writeln!(out, "const FACTS: [Facts; {}] = [", entries.len());
    for entry in entries {
        let message = entry
            .message
            .as_ref()
            .map_or_else(|| "None".to_owned(), |message| format!("Some({message:?})"));
        let _ = writeln!(
            out,
            "    Facts {{ code: {}, name: {:?}, default_message: {message}, retriable: {}, \
             invalidates_metadata: {} }},",
            entry.code, entry.name, entry.retriable, entry.invalidates_metadata
        );
    }
    out.push_str("];\n\nimpl ErrorCode {\n");
    let _ = writeln!(
        out,
        "    /// Every error code, in code order.\n    pub const ALL: [ErrorCode; {}] = [",
        entries.len()
    );
    for entry in entries {
        let _ = writeln!(out, "        ErrorCode::{},", variant(&entry.name));
    }
    out.push_str("    ];\n\n");
    out.push_str(
        "    fn facts(self) -> &'static Facts {\n        \
         match FACTS.get(self as usize) {\n            Some(facts) => facts,\n            \
         None => unreachable!(\"invariant: FACTS has one entry per ErrorCode variant\"),\n        \
         }\n    }\n\n",
    );
    render_accessors(&mut out);
    out.push_str("}\n");
    out
}

/// One method per fact, each reading it from `FACTS`.
fn render_accessors(out: &mut String) {
    for (doc, name, output, field) in [
        ("The code on the wire.", "code", "i16", "code"),
        (
            "Kafka's name for the code: `NOT_LEADER_OR_FOLLOWER`.",
            "name",
            "&'static str",
            "name",
        ),
        (
            "Kafka's default message for the code, which is none only for `NONE`.",
            "default_message",
            "Option<&'static str>",
            "default_message",
        ),
        (
            "Whether the request may succeed if retried: Kafka's exception for the code \
             extends `RetriableException`.",
            "is_retriable",
            "bool",
            "retriable",
        ),
        (
            "Whether a client should refresh its metadata before retrying: Kafka's \
             exception for the code extends `InvalidMetadataException`.",
            "invalidates_metadata",
            "bool",
            "invalidates_metadata",
        ),
    ] {
        out.push_str(&docs::wrap(doc, "    /// "));
        let _ = writeln!(
            out,
            "    pub fn {name}(self) -> {output} {{\n        self.facts().{field}\n    }}\n"
        );
    }
}

/// docs/errors.md: one row per code, each with an anchor that other docs link to.
pub(crate) fn page(entries: &[ErrorEntry], source: &str) -> String {
    let retriable = entries.iter().filter(|entry| entry.retriable).count();
    let mut out = format!(
        "<!-- @generated by `cargo xtask codegen` from {source}. Do not edit: change the \
         generator in xtask/src/codegen/errors.rs. -->\n\n\
         # Error codes\n\n\
         Every error code of Kafka 4.4.0, from \
         [`Errors.java`](https://github.com/apache/kafka/blob/4.4.0/clients/src/main/java/org/apache/kafka/common/protocol/Errors.java). \
         Alluvium returns the same code, with the same meaning, wherever Kafka does. In code, \
         each is a variant of `alluvium_protocol::ErrorCode`.\n\n\
         - **Retriable**: the request may succeed if retried. Kafka's exception for the code \
         extends `RetriableException`; {retriable} of the {} codes do.\n\
         - **Refresh metadata**: a client should refresh its metadata before retrying. The \
         exception extends `InvalidMetadataException`.\n\
         - **Message**: Kafka's default message, which a response's error message field \
         carries when nothing more specific applies.\n\n\
         Each code has an anchor named after it in lower case, such as \
         `#not_leader_or_follower`.\n\n\
         | Code | Name | Retriable | Refresh metadata | Message |\n\
         |---:|---|---|---|---|\n",
        entries.len()
    );
    for entry in entries {
        let yes_no = |flag: bool| if flag { "yes" } else { "no" };
        let message = entry
            .message
            .as_deref()
            .unwrap_or("No error.")
            .replace('|', "\\|");
        let _ = writeln!(
            out,
            "| {} | <a id=\"{}\"></a>`{}` | {} | {} | {message} |",
            entry.code,
            entry.name.to_ascii_lowercase(),
            entry.name,
            yes_no(entry.retriable),
            yes_no(entry.invalidates_metadata),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{ErrorEntry, extract, variant};

    fn vendored() -> PathBuf {
        crate::repo_root().join("schemas/kafka-4.4.0/errors")
    }

    #[test]
    fn the_vendored_errors_have_kafkas_138_contiguous_codes() {
        let entries = extract(&vendored()).unwrap();
        assert_eq!(entries.len(), 138);
        assert_eq!(entries.first().map(|entry| entry.code), Some(-1));
        assert_eq!(entries.last().map(|entry| entry.code), Some(136));
        assert_eq!(entries.iter().filter(|entry| entry.retriable).count(), 32);
        let none = &entries[1];
        assert_eq!((none.name.as_str(), &none.message), ("NONE", &None));
        let not_leader = &entries[7];
        assert_eq!(not_leader.name, "NOT_LEADER_OR_FOLLOWER");
        assert!(not_leader.retriable && not_leader.invalidates_metadata);
        assert!(
            not_leader
                .message
                .as_deref()
                .unwrap()
                .ends_with("not a replica of the topic partition.")
        );
    }

    /// The table `cargo xtask schemas dump-errors` read from Kafka's own `Errors.values()`.
    fn golden() -> Vec<ErrorEntry> {
        let path = crate::repo_root().join("xtask/tests/fixtures/kafka-4.4.0-errors.json");
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        json.as_array()
            .unwrap()
            .iter()
            .map(|entry| {
                let text = |key: &str| entry[key].as_str().map(str::to_owned);
                ErrorEntry {
                    name: text("name").unwrap(),
                    code: i16::try_from(entry["code"].as_i64().unwrap()).unwrap(),
                    message: text("message"),
                    exception: text("exception"),
                    retriable: entry["retriable"].as_bool().unwrap(),
                    invalidates_metadata: entry["invalidates_metadata"].as_bool().unwrap(),
                }
            })
            .collect()
    }

    #[test]
    fn the_extracted_table_equals_kafkas_runtime_table() {
        let mut golden = golden();
        golden.sort_by_key(|entry| entry.code);
        let extracted = extract(&vendored()).unwrap();
        for (ours, kafkas) in extracted.iter().zip(&golden) {
            assert_eq!(ours, kafkas);
        }
        assert_eq!(extracted.len(), golden.len());
    }

    fn fixture(name: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xtask-errors-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for (file, text) in files {
            std::fs::write(dir.join(file), text).unwrap();
        }
        dir
    }

    fn extract_fixture(
        name: &str,
        errors: &str,
        classes: &[(&str, &str)],
    ) -> Result<Vec<ErrorEntry>, String> {
        let mut files = vec![("Errors.java", errors)];
        files.extend_from_slice(classes);
        let dir = fixture(name, &files);
        let result = extract(&dir);
        std::fs::remove_dir_all(&dir).unwrap();
        result
    }

    const ROOTS: [(&str, &str); 2] = [
        (
            "ApiException.java",
            "public class ApiException extends RuntimeException {}",
        ),
        (
            "RetriableException.java",
            "/* retriable */ public abstract class RetriableException extends ApiException {}",
        ),
    ];

    #[test]
    fn every_builder_form_and_concatenated_messages_are_read() {
        let mut classes = ROOTS.to_vec();
        classes.push((
            "LateException.java",
            "public class LateException extends\n    org.example.RetriableException {}",
        ));
        let entries = extract_fixture(
            "forms",
            r#"public enum Errors {
                // A comment between constants.
                NONE(0, null, message -> null),
                LATE(1, "Too " + "late: \"now\"\u0021", message -> new LateException(message)),
                WRONG(2, "Wrong.", ApiException::new);
                private final short code;
            }"#,
            &classes,
        )
        .unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[1].message.as_deref(), Some("Too late: \"now\"!"));
        assert_eq!(entries[1].exception.as_deref(), Some("LateException"));
        assert!(entries[1].retriable && !entries[2].retriable);
        assert_eq!(entries[0].exception, None);
    }

    #[test]
    fn an_unresolvable_superclass_fails_with_the_class_name() {
        let problem = extract_fixture(
            "unresolved",
            r#"enum Errors { LOST(0, "Lost.", LostException::new); }"#,
            &[(
                "LostException.java",
                "class LostException extends MissingException {}",
            )],
        )
        .unwrap_err();
        assert!(
            problem.contains("superclass of MissingException"),
            "{problem}"
        );
        assert!(problem.contains("LostException"), "{problem}");
    }

    #[test]
    fn duplicate_names_and_gaps_in_codes_fail() {
        let gap = extract_fixture(
            "gap",
            r#"enum Errors { A(0, "A.", ApiException::new), B(2, "B.", ApiException::new); }"#,
            &ROOTS,
        )
        .unwrap_err();
        assert!(gap.contains("not contiguous at B (2)"), "{gap}");
        let twice = extract_fixture(
            "twice",
            r#"enum Errors { A(0, "A.", ApiException::new), A(1, "A.", ApiException::new); }"#,
            &ROOTS,
        )
        .unwrap_err();
        assert!(twice.contains("A is defined twice"), "{twice}");
    }

    #[test]
    fn names_become_rust_variants() {
        assert_eq!(variant("NOT_LEADER_OR_FOLLOWER"), "NotLeaderOrFollower");
        assert_eq!(variant("NONE"), "None");
        assert_eq!(variant("KAFKA_STORAGE_ERROR"), "KafkaStorageError");
    }
}
