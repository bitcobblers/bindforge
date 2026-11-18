//! A hand-rolled reader for the subset of the Valve KeyValues ("VDF") grammar
//! that Steam's `appmanifest_*.acf` and `libraryfolders.vdf` files use
//! (Chosen design §4).
//!
//! The module is platform-neutral and carries no Steam knowledge, so a later
//! Windows plugin can reuse it as a property of the code.
//!
//! Accepted: whitespace-separated tokens, quoted and bare keys and values,
//! `{}`-nested sections, `//` line comments, a leading UTF-8 BOM, LF and CRLF,
//! and `\\` / `\"` unescapes inside quoted values.
//!
//! Rejected, as a positioned [`KeyValueError`]: unbalanced braces, an
//! unterminated string, a `[` conditional or `#base` line, and any section
//! nested deeper than [`MAX_NESTING_DEPTH`].

/// Largest document this reader will parse.
///
/// Restates the repo's shared parse budget (`schemas/reach-classes/v1/spec.md`)
/// as a plugin-local constant: `crates/discovery` takes no `bindforge-core`
/// edge (Chosen design §4, A16).
pub const MAX_DOCUMENT_BYTES: usize = 1024 * 1024;

/// Deepest `{}` nesting this reader accepts.
pub const MAX_NESTING_DEPTH: usize = 16;

/// A grammar refusal, positioned at the 1-based line that triggered it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyValueError {
    pub line: usize,
    pub message: &'static str,
}

/// The value side of a KeyValues pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Str(String),
    Section(Section),
}

/// A KeyValues section: its children, in declaration order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub children: Vec<(String, Value)>,
}

impl Section {
    /// The first child whose key matches `key` case-insensitively (A5).
    fn find(&self, key: &str) -> Option<&Value> {
        self.children
            .iter()
            .find(|(child, _)| child.eq_ignore_ascii_case(key))
            .map(|(_, value)| value)
    }

    /// The string value of the first case-insensitive match; `None` when the
    /// key is absent or its value is a section.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        match self.find(key)? {
            Value::Str(text) => Some(text),
            Value::Section(_) => None,
        }
    }

    /// The section value of the first case-insensitive match; `None` when the
    /// key is absent or its value is a string.
    pub fn get_section(&self, key: &str) -> Option<&Section> {
        match self.find(key)? {
            Value::Section(section) => Some(section),
            Value::Str(_) => None,
        }
    }
}

/// Parses `text` into the top-level pseudo-section.
pub fn parse(text: &str) -> Result<Section, KeyValueError> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    if text.len() > MAX_DOCUMENT_BYTES {
        return Err(KeyValueError {
            line: 1,
            message: "document is larger than the read cap",
        });
    }
    Reader {
        bytes: text.as_bytes(),
        pos: 0,
        line: 1,
    }
    .parse_document()
}

/// The byte cursor behind [`parse`]; `bytes` is valid UTF-8, and every token
/// boundary is an ASCII delimiter, so byte slices are always char boundaries.
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
    line: usize,
}

impl Reader<'_> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    /// Moves the cursor to `target`, keeping `line` in step.
    fn skip_to(&mut self, target: usize) {
        self.line += self.bytes[self.pos..target]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count();
        self.pos = target;
    }

    fn error(&self, message: &'static str) -> KeyValueError {
        KeyValueError {
            line: self.line,
            message,
        }
    }

    /// Advances past whitespace and `//` line comments.
    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(byte) if byte.is_ascii_whitespace() => self.skip_to(self.pos + 1),
                Some(b'/') if self.bytes.get(self.pos + 1) == Some(&b'/') => {
                    let mut end = self.pos;
                    while let Some(byte) = self.bytes.get(end) {
                        end += 1;
                        if *byte == b'\n' {
                            break;
                        }
                    }
                    self.skip_to(end);
                }
                _ => return,
            }
        }
    }

    /// Reads the next key or value token, unescaping `\\` and `\"` inside
    /// quoted values (A6). `Ok(None)` is end of input.
    fn read_token(&mut self) -> Result<Option<String>, KeyValueError> {
        self.skip_trivia();
        let Some(first) = self.peek() else {
            return Ok(None);
        };
        match first {
            b'{' | b'}' => Err(self.error("expected a key or a value, found a brace")),
            b'[' => Err(self.error("conditionals such as `[$WIN32]` are not supported")),
            b'#' => Err(self.error("directives such as `#base` are not supported")),
            b'"' => self.read_quoted_token(),
            _ => Ok(Some(self.read_bare_token())),
        }
    }

    /// Reads a bare (unquoted) token: it runs to whitespace, a quote or a brace.
    fn read_bare_token(&mut self) -> String {
        let start = self.pos;
        while let Some(byte) = self.peek() {
            if byte.is_ascii_whitespace() || byte == b'"' || byte == b'{' || byte == b'}' {
                break;
            }
            self.skip_to(self.pos + 1);
        }
        String::from_utf8_lossy(&self.bytes[start..self.pos]).into_owned()
    }

    /// Reads a `"…"` token. A newline before the closing quote means the string
    /// is unterminated, so the error names the line the string opened on.
    fn read_quoted_token(&mut self) -> Result<Option<String>, KeyValueError> {
        let open_line = self.line;
        self.skip_to(self.pos + 1);
        let mut bytes: Vec<u8> = Vec::new();
        loop {
            match self.peek() {
                None | Some(b'\n') | Some(b'\r') => {
                    return Err(KeyValueError {
                        line: open_line,
                        message: "unterminated string",
                    });
                }
                Some(b'"') => {
                    self.skip_to(self.pos + 1);
                    return Ok(Some(String::from_utf8_lossy(&bytes).into_owned()));
                }
                Some(b'\\') => match self.bytes.get(self.pos + 1) {
                    Some(b'"') => {
                        bytes.push(b'"');
                        self.skip_to(self.pos + 2);
                    }
                    Some(b'\\') => {
                        bytes.push(b'\\');
                        self.skip_to(self.pos + 2);
                    }
                    // Any other escape is kept verbatim; the next turn copies
                    // the escaped byte(s) as they are.
                    Some(_) => {
                        bytes.push(b'\\');
                        self.skip_to(self.pos + 1);
                    }
                    None => {
                        return Err(KeyValueError {
                            line: open_line,
                            message: "unterminated string",
                        });
                    }
                },
                Some(_) => {
                    bytes.push(self.bytes[self.pos]);
                    self.skip_to(self.pos + 1);
                }
            }
        }
    }

    /// Parses the whole document, which is the top-level pseudo-section.
    fn parse_document(&mut self) -> Result<Section, KeyValueError> {
        let document = self.parse_children(0, None)?;
        self.skip_trivia();
        match self.peek() {
            None => Ok(document),
            Some(_) => Err(self.error("unbalanced braces")),
        }
    }

    /// Reads `key value` and `key {}` pairs until the closing brace of the
    /// enclosing section. `open_line` is `Some` for a nested section, and its
    /// brace line is what an unclosed section is reported against.
    fn parse_children(
        &mut self,
        depth: usize,
        open_line: Option<usize>,
    ) -> Result<Section, KeyValueError> {
        let mut children: Vec<(String, Value)> = Vec::new();
        loop {
            self.skip_trivia();
            match self.peek() {
                None => {
                    return match open_line {
                        Some(line) => Err(KeyValueError {
                            line,
                            message: "unbalanced braces",
                        }),
                        None => Ok(Section { children }),
                    };
                }
                Some(b'}') => {
                    return match open_line {
                        Some(_) => {
                            self.skip_to(self.pos + 1);
                            Ok(Section { children })
                        }
                        None => Err(self.error("unbalanced braces")),
                    };
                }
                _ => {}
            }

            let Some(key) = self.read_token()? else {
                return Err(self.error("expected a key"));
            };
            self.skip_trivia();
            match self.peek() {
                Some(b'{') => {
                    if depth + 1 > MAX_NESTING_DEPTH {
                        return Err(self.error("section nested deeper than the maximum depth"));
                    }
                    self.skip_to(self.pos + 1);
                    let section = self.parse_children(depth + 1, Some(self.line))?;
                    children.push((key, Value::Section(section)));
                }
                Some(b'}') | None => return Err(self.error("expected a value")),
                _ => {
                    let Some(value) = self.read_token()? else {
                        return Err(self.error("expected a value"));
                    };
                    children.push((key, Value::Str(value)));
                }
            }
        }
    }
}
