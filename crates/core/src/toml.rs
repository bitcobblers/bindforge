//! A TOML reader for the grammar the `schemas/` format specs allow.
//!
//! Contract documents are TOML (ADR-009), and every document this reader
//! accepts is a legal TOML v1.1.0 document (A20) — but the v1 grammar each
//! `spec.md` ships is a documented *subset* of TOML v1.1.0: comments,
//! `key = value` entries, basic strings, integers, booleans, and arrays
//! written on one line. Table headers, dotted keys, multi-line arrays, floats,
//! and hex integers are legal TOML and outside that subset, so this reader
//! refuses them with a structured `CoreError`: an adversarial document can
//! never reach the model through a syntax the contract does not define.
//! Bare keys follow TOML's rule (`[A-Za-z0-9_-]+`), quoted keys are accepted,
//! and integer literals follow TOML's — optional sign, `_` only between
//! digits, no leading zero.
//!
//! Keeping the reader in-tree (rather than pulling in `toml`/`serde`) also
//! keeps the workspace dependency-free at scaffold time; `toml_edit`'s first
//! consumer remains ADR-012, and when it lands this grammar is what its output
//! must be narrowed to.
//!
//! The adversarial parse budget (Chosen design -> Adversarial guard, budget
//! documented in `schemas/README.md`): documents larger than
//! `MAX_DOCUMENT_BYTES` and arrays nested deeper than `MAX_NESTING_DEPTH`
//! levels are rejected with typed errors before any model value is built.
//! `read_file` and `read_reader` apply the size cap *before* allocating, so a
//! caller handling an untrusted tree or body never has to hold more than the
//! budget in memory.

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::Path;

use crate::error::CoreError;

/// Parse budget: maximum document size, in bytes (`schemas/README.md`).
pub const MAX_DOCUMENT_BYTES: usize = 1024 * 1024;

/// Parse budget: maximum table/array nesting depth (`schemas/README.md`).
pub const MAX_NESTING_DEPTH: usize = 16;

/// A TOML value the accepted grammar can produce.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    String(String),
    Integer(i64),
    Boolean(bool),
    Array(Vec<Value>),
}

/// Read one contract file, applying the size budget before reading it: a file
/// longer than `MAX_DOCUMENT_BYTES` is refused without being allocated, which
/// is what lets a caller that serves or syncs a contract tree enforce the
/// budget instead of discovering it after the whole file is in memory.
pub fn read_file(path: &Path) -> Result<String, CoreError> {
    let metadata = fs::metadata(path).map_err(|err| io_error(path, err))?;
    if metadata.len() > MAX_DOCUMENT_BYTES as u64 {
        return Err(CoreError::DocumentTooLarge {
            limit: MAX_DOCUMENT_BYTES,
        });
    }
    fs::read_to_string(path).map_err(|err| io_error(path, err))
}

/// A failed read of one contract file: the path stays on the variant, never
/// in its message, so a server-side log can name it.
fn io_error(path: &Path, err: std::io::Error) -> CoreError {
    CoreError::Io {
        path: path.display().to_string(),
        message: err.to_string(),
    }
}

/// Read one stream — an upload body, a pipe — under the same budget: at most
/// `MAX_DOCUMENT_BYTES + 1` bytes are taken from it, and if more is available
/// the document is refused. A hub's upload policy still checks
/// `Content-Length` first (`schemas/README.md`); this is what makes the check
/// unskippable.
pub fn read_reader(source: impl Read) -> Result<String, CoreError> {
    let budget = MAX_DOCUMENT_BYTES as u64 + 1;
    let mut bytes = Vec::new();
    source
        .take(budget)
        .read_to_end(&mut bytes)
        .map_err(|err| CoreError::Io {
            path: "<stream>".to_string(),
            message: err.to_string(),
        })?;
    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(CoreError::DocumentTooLarge {
            limit: MAX_DOCUMENT_BYTES,
        });
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| CoreError::NotUtf8)?;
    Ok(text.to_owned())
}

/// Read one contract document: an ordered map of top-level entries.
pub fn read_document(text: &str) -> Result<BTreeMap<String, Value>, CoreError> {
    if text.len() > MAX_DOCUMENT_BYTES {
        return Err(CoreError::DocumentTooLarge {
            limit: MAX_DOCUMENT_BYTES,
        });
    }

    let mut entries = BTreeMap::new();
    for (index, raw) in text.lines().enumerate() {
        let number = index + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') {
            return Err(CoreError::Malformed {
                line: number,
                message: "table headers are outside the grammar this format allows".to_string(),
            });
        }

        let chars: Vec<char> = trimmed.chars().collect();
        let mut cursor = Cursor {
            chars: &chars,
            pos: 0,
            line: number,
        };
        let key = cursor.parse_key()?;
        cursor.skip_inline_space();
        if cursor.peek() != Some('=') {
            return Err(cursor.error("expected `=` after the key"));
        }
        cursor.pos += 1;
        let value = cursor.parse_value(0)?;
        cursor.finish()?;

        if entries.insert(key.clone(), value).is_some() {
            return Err(CoreError::Malformed {
                line: number,
                message: format!("duplicate key `{key}`"),
            });
        }
    }
    Ok(entries)
}

/// Position-tracked reader over one trimmed line.
struct Cursor<'a> {
    chars: &'a [char],
    pos: usize,
    line: usize,
}

impl Cursor<'_> {
    fn error(&self, message: impl Into<String>) -> CoreError {
        CoreError::Malformed {
            line: self.line,
            message: message.into(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn skip_inline_space(&mut self) {
        while self.pos < self.chars.len() && matches!(self.chars[self.pos], ' ' | '\t') {
            self.pos += 1;
        }
    }

    /// TOML's bare-key rule (`[A-Za-z0-9_-]+`), plus a quoted basic string
    /// used as a key. A dotted key stops here with "expected `=` after the
    /// key", which is how the grammar refuses dotted keys.
    fn parse_key(&mut self) -> Result<String, CoreError> {
        self.skip_inline_space();
        if self.peek() == Some('"') {
            return self.parse_string();
        }
        let mut key = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                key.push(c);
                self.pos += 1;
            } else {
                break;
            }
        }
        if key.is_empty() {
            return Err(self.error("expected a key before `=`"));
        }
        Ok(key)
    }

    /// `depth` is the number of containers enclosing the value's position;
    /// an array opened at `depth` is itself at nesting level `depth + 1`.
    fn parse_value(&mut self, depth: usize) -> Result<Value, CoreError> {
        self.skip_inline_space();
        match self.peek() {
            None => Err(self.error("expected a value")),
            Some('"') => Ok(Value::String(self.parse_string()?)),
            Some('[') => self.parse_array(depth + 1),
            Some('t') | Some('f') => self.parse_boolean(),
            Some(c) if c == '+' || c == '-' || c.is_ascii_digit() => self.parse_integer(),
            Some(c) => Err(self.error(format!("unexpected `{c}` at the start of a value"))),
        }
    }

    fn parse_string(&mut self) -> Result<String, CoreError> {
        // Precondition: peek() == '"'.
        self.pos += 1;
        let mut out = String::new();
        loop {
            let Some(c) = self.peek() else {
                return Err(self.error("unterminated string"));
            };
            self.pos += 1;
            match c {
                '"' => return Ok(out),
                '\\' => {
                    let Some(escape) = self.peek() else {
                        return Err(self.error("unterminated escape in string"));
                    };
                    self.pos += 1;
                    match escape {
                        'b' => out.push('\u{8}'),
                        'f' => out.push('\u{c}'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        'u' => out.push(self.parse_hex_chars(4)?),
                        'U' => out.push(self.parse_hex_chars(8)?),
                        other => {
                            return Err(self.error(format!(
                                "`\\{other}` is not an escape this grammar allows"
                            )));
                        }
                    }
                }
                c if (c as u32) < 0x20 => {
                    return Err(self.error("control character inside a string"));
                }
                other => out.push(other),
            }
        }
    }

    /// The `\uXXXX` and `\UXXXXXXXX` escapes TOML spells the same way. A code
    /// point that is not a character (a surrogate half, or past the last
    /// scalar value) is refused.
    fn parse_hex_chars(&mut self, digits: usize) -> Result<char, CoreError> {
        let mut value: u32 = 0;
        for _ in 0..digits {
            let Some(digit) = self.peek() else {
                return Err(self.error("unterminated unicode escape in string"));
            };
            let Some(nibble) = digit.to_digit(16) else {
                return Err(self.error(format!("`{digit}` is not a hexadecimal digit")));
            };
            value = value * 16 + nibble;
            self.pos += 1;
        }
        value
            .try_into()
            .map_err(|_| self.error(format!("`\\u{value:04x}` is not a character")))
    }

    fn parse_boolean(&mut self) -> Result<Value, CoreError> {
        for (word, value) in [("true", true), ("false", false)] {
            if word
                .chars()
                .eq(self.chars[self.pos..].iter().copied().take(word.len()))
            {
                self.pos += word.len();
                return Ok(Value::Boolean(value));
            }
        }
        Err(self.error("expected `true` or `false`"))
    }

    /// TOML's decimal-integer rule: an optional sign, then digits with `_`
    /// only between digits and no leading zero. So `1_`, `1__0`, and `01` are
    /// refused, and `0x`/`0o`/`0b` prefixes fall out of the grammar.
    fn parse_integer(&mut self) -> Result<Value, CoreError> {
        let negative = self.peek() == Some('-');
        if matches!(self.peek(), Some('+') | Some('-')) {
            self.pos += 1;
        }
        let mut digits = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || (c == '_' && !digits.is_empty()) {
                digits.push(c);
                self.pos += 1;
            } else {
                break;
            }
        }
        if digits.is_empty() {
            return Err(self.error("expected an integer"));
        }
        if digits.starts_with('_') || digits.ends_with('_') || digits.contains("__") {
            return Err(self.error("`_` may only separate digits in an integer"));
        }
        let plain = digits.replace('_', "");
        if plain != "0" && plain.starts_with('0') {
            return Err(self.error("an integer may not have a leading zero"));
        }
        let text = if negative { format!("-{plain}") } else { plain };
        let Ok(value) = text.parse::<i64>() else {
            return Err(self.error(format!("`{text}` is not an integer this grammar allows")));
        };
        Ok(Value::Integer(value))
    }

    fn parse_array(&mut self, level: usize) -> Result<Value, CoreError> {
        if level > MAX_NESTING_DEPTH {
            return Err(CoreError::NestingTooDeep {
                limit: MAX_NESTING_DEPTH,
            });
        }
        // Precondition: peek() == '['.
        self.pos += 1;
        let mut items = Vec::new();
        loop {
            self.skip_inline_space();
            if self.peek() == Some(']') {
                self.pos += 1;
                return Ok(Value::Array(items));
            }
            items.push(self.parse_value(level)?);
            self.skip_inline_space();
            match self.peek() {
                Some(',') => self.pos += 1,
                Some(']') => {
                    self.pos += 1;
                    return Ok(Value::Array(items));
                }
                None => return Err(self.error("unterminated array")),
                Some(other) => {
                    return Err(
                        self.error(format!("expected `,` or `]` in an array, found `{other}`"))
                    );
                }
            }
        }
    }

    /// After a complete entry: only inline space and a trailing comment may remain.
    fn finish(&mut self) -> Result<(), CoreError> {
        self.skip_inline_space();
        match self.peek() {
            None => Ok(()),
            Some('#') => Ok(()),
            Some(c) => Err(self.error(format!("unexpected `{c}` after the value"))),
        }
    }
}
