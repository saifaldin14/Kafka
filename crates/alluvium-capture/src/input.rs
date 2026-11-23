//! A frame's bytes from the forms people copy them in (PROTO-0143):
//! - [`InputFormat::Raw`]: the bytes themselves, such as a file a tool wrote;
//! - [`InputFormat::Hex`]: hex digit pairs, separated by whitespace or not, which
//!   covers Wireshark's "Copy as Hex Stream" and `xxd -p`;
//! - [`InputFormat::Xxd`]: `xxd`'s default dump: an offset, a colon, groups of hex
//!   digits, then the text column, which is ignored.
//!
//! [`detect`] picks the format of an input; [`parse`] reads it.

use std::fmt;

/// The form of a dump.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputFormat {
    /// The bytes as they are.
    Raw,
    /// Hex digit pairs, with any whitespace between them.
    Hex,
    /// `xxd`'s dump: `00000000: 0000 0012 0012 0004  ........`.
    Xxd,
}

impl fmt::Display for InputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            InputFormat::Raw => "raw",
            InputFormat::Hex => "hex",
            InputFormat::Xxd => "xxd",
        })
    }
}

/// Where and why a dump doesn't parse. Lines and columns count from 1, columns in
/// characters.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("line {line}, column {column}: {problem}")]
pub struct InputError {
    /// The line.
    pub line: usize,
    /// The column.
    pub column: usize,
    /// What is wrong there.
    pub problem: InputProblem,
}

/// What is wrong in a dump.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InputProblem {
    /// The dump isn't text.
    #[error("a {format} dump must be UTF-8 text")]
    NotText {
        /// The format expected.
        format: InputFormat,
    },
    /// A character that isn't a hex digit where one must be.
    #[error("{found:?} is not a hex digit")]
    NotHex {
        /// The character found.
        found: char,
    },
    /// A hex digit without the second of its pair.
    #[error("a hex digit without the second of its pair")]
    OddDigits,
    /// An `xxd` line without its `offset:`.
    #[error("an xxd line starts with its offset and a colon")]
    NoOffset,
    /// An `xxd` line whose offset isn't the count of the bytes before it.
    #[error("the offset is {found:#x}, but {expected:#x} bytes come before this line")]
    WrongOffset {
        /// The offset the line has.
        found: u64,
        /// The bytes before it.
        expected: u64,
    },
}

/// The format `input` looks like: [`InputFormat::Xxd`] if its first line is an `xxd`
/// line, [`InputFormat::Hex`] if it is hex digits and whitespace only, and otherwise
/// [`InputFormat::Raw`].
pub fn detect(input: &[u8]) -> InputFormat {
    let Ok(text) = std::str::from_utf8(input) else {
        return InputFormat::Raw;
    };
    let first = text.lines().find(|line| !line.trim().is_empty());
    if first.is_some_and(|line| xxd_offset(line).is_some()) {
        InputFormat::Xxd
    } else if first.is_some()
        && text
            .chars()
            .all(|character| character.is_ascii_hexdigit() || character.is_whitespace())
    {
        InputFormat::Hex
    } else {
        InputFormat::Raw
    }
}

/// The bytes `input` holds in `format`.
///
/// # Errors
///
/// [`InputError`] naming the line and column where a hex or `xxd` dump goes wrong.
pub fn parse(input: &[u8], format: InputFormat) -> Result<Vec<u8>, InputError> {
    let text = || {
        std::str::from_utf8(input).map_err(|error| {
            let (line, column) = position(input.get(..error.valid_up_to()).unwrap_or_default());
            InputError {
                line,
                column,
                problem: InputProblem::NotText { format },
            }
        })
    };
    match format {
        InputFormat::Raw => Ok(input.to_vec()),
        InputFormat::Hex => {
            let mut bytes = Vec::new();
            for (number, line) in text()?.lines().enumerate() {
                hex_pairs(line, number.saturating_add(1), 1, &mut bytes, true)?;
            }
            Ok(bytes)
        }
        InputFormat::Xxd => xxd(text()?),
    }
}

/// The line and column just after `text`, which is valid UTF-8 up to its end.
fn position(text: &[u8]) -> (usize, usize) {
    let text = String::from_utf8_lossy(text);
    let line = text.matches('\n').count().saturating_add(1);
    let column = text
        .rsplit('\n')
        .next()
        .map_or(0, |last| last.chars().count())
        .saturating_add(1);
    (line, column)
}

/// The value of a hex digit.
fn digit(character: char) -> Option<u8> {
    character
        .to_digit(16)
        .and_then(|value| u8::try_from(value).ok())
}

/// Appends the hex digit pairs of `text`, the part of line `line` that starts at column
/// `first_column`, to `bytes`. With `spaced`, whitespace may come between any two pairs;
/// without it, only between groups, and a group ends at whitespace.
fn hex_pairs(
    text: &str,
    line: usize,
    first_column: usize,
    bytes: &mut Vec<u8>,
    spaced: bool,
) -> Result<(), InputError> {
    let error = |column: usize, problem| InputError {
        line,
        column,
        problem,
    };
    let mut high: Option<(u8, usize)> = None;
    for (index, character) in text.chars().enumerate() {
        let column = first_column.saturating_add(index);
        if character.is_whitespace() {
            if let Some((_, at)) = high {
                return Err(error(at, InputProblem::OddDigits));
            }
            if !spaced && index == 0 {
                return Err(error(column, InputProblem::NotHex { found: character }));
            }
            continue;
        }
        let value =
            digit(character).ok_or(error(column, InputProblem::NotHex { found: character }))?;
        match high.take() {
            Some((high, _)) => bytes.push((high << 4) | value),
            None => high = Some((value, column)),
        }
    }
    match high {
        Some((_, at)) => Err(error(at, InputProblem::OddDigits)),
        None => Ok(()),
    }
}

/// The offset an `xxd` line starts with, and the column just after its colon, if it has
/// one.
fn xxd_offset(line: &str) -> Option<(u64, usize)> {
    let (offset, _) = line.split_once(':')?;
    let digits = offset.trim_start();
    if digits.is_empty()
        || !digits
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return None;
    }
    let value = u64::from_str_radix(digits, 16).ok()?;
    Some((value, offset.chars().count().saturating_add(2)))
}

/// The bytes of an `xxd` dump: on each line, the offset, then hex groups up to the two
/// spaces before the text column.
// The format of xxd(1): `xxd` writes the offset, ": ", the groups separated by one
// space, then two spaces and the text column; a short last line is padded to align it.
fn xxd(text: &str) -> Result<Vec<u8>, InputError> {
    let mut bytes = Vec::new();
    for (number, line) in text.lines().enumerate() {
        let number = number.saturating_add(1);
        if line.trim().is_empty() {
            continue;
        }
        let (offset, column) = xxd_offset(line).ok_or(InputError {
            line: number,
            column: 1,
            problem: InputProblem::NoOffset,
        })?;
        let expected = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if offset != expected {
            return Err(InputError {
                line: number,
                column: 1,
                problem: InputProblem::WrongOffset {
                    found: offset,
                    expected,
                },
            });
        }
        let after_colon = line.split_once(':').map_or("", |(_, rest)| rest);
        let (hex, column) = match after_colon.strip_prefix(' ') {
            Some(rest) => (rest, column.saturating_add(1)),
            None => (after_colon, column),
        };
        let hex = hex.split_once("  ").map_or(hex, |(groups, _)| groups);
        for (group, at) in groups(hex, column) {
            hex_pairs(group, number, at, &mut bytes, false)?;
        }
    }
    Ok(bytes)
}

/// The space-separated groups of `hex`, with the column each starts at, given that
/// `hex` starts at `column`.
fn groups(hex: &str, column: usize) -> impl Iterator<Item = (&str, usize)> {
    let mut at = column;
    hex.split(' ').map(move |group| {
        let start = at;
        at = at.saturating_add(group.chars().count()).saturating_add(1);
        (group, start)
    })
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use proptest::prelude::*;

    use super::{InputError, InputFormat, InputProblem, detect, parse};

    /// The first bytes of a frame: a size, then an `ApiVersions` v3 request header.
    const BYTES: [u8; 20] = [
        0x00, 0x00, 0x00, 0x10, 0x00, 0x12, 0x00, 0x03, 0x00, 0x00, 0x00, 0x2a, 0x00, 0x06, b'a',
        b'd', b'm', b'i', b'n', 0x00,
    ];

    fn error(line: usize, column: usize, problem: InputProblem) -> Result<Vec<u8>, InputError> {
        Err(InputError {
            line,
            column,
            problem,
        })
    }

    #[test]
    fn raw_input_is_the_bytes() {
        assert_eq!(detect(&BYTES), InputFormat::Raw);
        assert_eq!(parse(&BYTES, InputFormat::Raw), Ok(BYTES.to_vec()));
    }

    #[test]
    fn hex_reads_pairs_with_or_without_spaces() {
        // Wireshark's "Copy as Hex Stream".
        let stream = b"00000010001200030000002a000661646d696e00";
        // xxd -p, and pairs separated by spaces.
        let plain = b"00000010001200030000002a000661646d696e\n00\n";
        let spaced = b"00 00 00 10 00 12 00 03\n00 00 00 2A 00 06 61 64 6d 69 6e 00";
        for input in [&stream[..], plain, spaced] {
            assert_eq!(detect(input), InputFormat::Hex);
            assert_eq!(parse(input, InputFormat::Hex), Ok(BYTES.to_vec()));
        }
    }

    #[test]
    fn xxd_reads_its_groups_and_ignores_the_text_column() {
        // `xxd`, and `xxd -g1`; the text column holds hex-looking characters.
        let grouped = "00000000: 0000 0010 0012 0003 0000 002a 0006 6164  ...........*..ad\n\
                       00000010: 6d69 6e00                                min.\n";
        let single = "00000000: 00 00 00 10 00 12 00 03 00 00 00 2a 00 06 61 64  ...........*..ad\n\
                      00000010: 6d 69 6e 00                                      min.\n";
        for input in [grouped, single] {
            assert_eq!(detect(input.as_bytes()), InputFormat::Xxd);
            assert_eq!(
                parse(input.as_bytes(), InputFormat::Xxd),
                Ok(BYTES.to_vec())
            );
        }
    }

    #[test]
    fn malformed_dumps_name_the_line_and_column() {
        let hex = |text: &str| parse(text.as_bytes(), InputFormat::Hex);
        let xxd = |text: &str| parse(text.as_bytes(), InputFormat::Xxd);
        assert_eq!(
            hex("00 0g"),
            error(1, 5, InputProblem::NotHex { found: 'g' })
        );
        assert_eq!(hex("00\n0 10"), error(2, 1, InputProblem::OddDigits));
        assert_eq!(hex("00\n001"), error(2, 3, InputProblem::OddDigits));
        assert_eq!(
            parse(b"00\n\xff", InputFormat::Hex),
            error(
                2,
                1,
                InputProblem::NotText {
                    format: InputFormat::Hex
                }
            )
        );
        assert_eq!(xxd("0000"), error(1, 1, InputProblem::NoOffset));
        assert_eq!(
            xxd("00000000: 0000\n00000004: 00"),
            error(
                2,
                1,
                InputProblem::WrongOffset {
                    found: 4,
                    expected: 2
                }
            )
        );
        assert_eq!(
            xxd("00000000: 00z0"),
            error(1, 13, InputProblem::NotHex { found: 'z' })
        );
        assert_eq!(
            xxd("00000000: 000 0"),
            error(1, 13, InputProblem::OddDigits)
        );
        assert_eq!(
            error(2, 1, InputProblem::NoOffset).unwrap_err().to_string(),
            "line 2, column 1: an xxd line starts with its offset and a colon"
        );
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().fold(String::new(), |mut text, byte| {
            write!(text, "{byte:02x}").unwrap();
            text
        })
    }

    proptest! {
        // No input makes parsing panic, in any format.
        #[test]
        fn no_input_panics(input in proptest::collection::vec(any::<u8>(), 0..200)) {
            for format in [InputFormat::Raw, InputFormat::Hex, InputFormat::Xxd] {
                let _ = parse(&input, format);
            }
            let _ = detect(&input);
        }

        // Bytes written as hex or as xxd would write them read back the same.
        #[test]
        fn written_dumps_read_back(bytes in proptest::collection::vec(any::<u8>(), 0..100)) {
            let stream = hex(&bytes);
            prop_assert_eq!(parse(stream.as_bytes(), InputFormat::Hex), Ok(bytes.clone()));
            let mut xxd = String::new();
            for (line, chunk) in bytes.chunks(16).enumerate() {
                let groups: Vec<String> = chunk
                    .chunks(2)
                    .map(hex)
                    .collect();
                let offset = line.checked_mul(16).unwrap();
                writeln!(xxd, "{offset:08x}: {:<40} ....", groups.join(" ")).unwrap();
            }
            prop_assert_eq!(parse(xxd.as_bytes(), InputFormat::Xxd), Ok(bytes));
        }
    }
}
