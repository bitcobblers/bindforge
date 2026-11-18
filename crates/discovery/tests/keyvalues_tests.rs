//! Unit tests for the hand-rolled KeyValues reader (`crates/discovery/src/keyvalues.rs`).
//!
//! These tests encode the accepted/rejected grammar subset of Chosen design §4
//! for both `appmanifest_*.acf` and `libraryfolders.vdf` input.

use discovery::keyvalues::{Value, parse};

/// Returns a slice of the keys of a section's children, in declaration order.
fn keys(children: &[(String, Value)]) -> Vec<&str> {
    children.iter().map(|(k, _)| k.as_str()).collect()
}

// proves: Test strategy #1 (Chosen design §4) — quoted and bare keys/values and
// nested `{}` sections parse in declaration order; bare values are strings.
#[test]
fn parses_quoted_and_bare_tokens_in_declaration_order() {
    let text = "\"AppState\"\n\
                {\n\
                \t\"appid\"\t\t900001\n\
                \t\"name\"\t\t\"Example Game\"\n\
                \t\"UserConfig\"\n\
                \t{\n\
                \t\t\"language\"\t\t\"english\"\n\
                \t}\n\
                }\n";
    let doc = match parse(text) {
        Ok(doc) => doc,
        Err(e) => panic!("well-formed KeyValues must parse (line {}: {})", e.line, e.message),
    };
    assert_eq!(keys(&doc.children), ["AppState"]);
    assert!(matches!(doc.children[0].1, Value::Section(_)));

    let app = doc.get_section("AppState").expect("AppState must be a section");
    assert_eq!(keys(&app.children), ["appid", "name", "UserConfig"]);
    assert_eq!(app.get_str("appid"), Some("900001"));
    assert_eq!(app.get_str("name"), Some("Example Game"));
    // A key whose value is a section has no string value.
    assert!(app.get_str("UserConfig").is_none(), "a section value has no string value");

    let user = app.get_section("UserConfig").expect("UserConfig must be a nested section");
    assert_eq!(user.get_str("language"), Some("english"));
}

// proves: Test strategy #1 (Chosen design §4) — `children` keeps declaration order.
#[test]
fn preserves_declaration_order_of_top_level_children() {
    let text = "\"gamma\" \"3\"\n\"alpha\" \"1\"\n\"beta\" \"2\"\n";
    let doc = match parse(text) {
        Ok(doc) => doc,
        Err(e) => panic!("top-level pairs must parse (line {}: {})", e.line, e.message),
    };
    assert_eq!(keys(&doc.children), ["gamma", "alpha", "beta"]);
}

// proves: Test strategy #1 (A5, Chosen design §4) — `get_str` matches `name`,
// `NAME` and `Name` alike.
#[test]
fn get_str_matches_keys_case_insensitively() {
    let text = "\"name\" \"Example Game\"\n";
    let doc = match parse(text) {
        Ok(doc) => doc,
        Err(e) => panic!("a single pair must parse (line {}: {})", e.line, e.message),
    };
    assert_eq!(doc.get_str("name"), Some("Example Game"));
    assert_eq!(doc.get_str("NAME"), Some("Example Game"));
    assert_eq!(doc.get_str("Name"), Some("Example Game"));
}

// proves: Test strategy #1 (A5, Chosen design §4/§5) — section lookup is
// case-insensitive too (`AppState` / `libraryfolders` roots are taken case-insensitively).
#[test]
fn get_section_matches_keys_case_insensitively() {
    let text = "\"AppState\"\n{\n\t\"installdir\"\t\t\"Example Game\"\n}\n";
    let doc = match parse(text) {
        Ok(doc) => doc,
        Err(e) => panic!("a single section must parse (line {}: {})", e.line, e.message),
    };
    for query in ["AppState", "appstate", "APPSTATE"] {
        let section = doc
            .get_section(query)
            .unwrap_or_else(|| panic!("`{query}` must resolve the AppState section"));
        assert_eq!(section.get_str("installdir"), Some("Example Game"));
    }
}

// proves: Test strategy #1 (Chosen design §4) — a duplicate key resolves to its first occurrence.
#[test]
fn get_str_returns_first_of_duplicate_keys() {
    let text = "\"name\" \"First\"\n\"name\" \"Second\"\n";
    let doc = match parse(text) {
        Ok(doc) => doc,
        Err(e) => panic!("duplicate keys must parse (line {}: {})", e.line, e.message),
    };
    assert_eq!(doc.get_str("name"), Some("First"));
}

// proves: Test strategy #1 (Chosen design §4) — `//` line comments parse.
#[test]
fn parses_line_comments() {
    let text = "// a leading comment\n\
                \"AppState\" // a trailing comment\n\
                {\n\
                \t// an inner comment\n\
                \t\"name\"\t\t\"Example Game\"\n\
                }\n";
    let doc = match parse(text) {
        Ok(doc) => doc,
        Err(e) => panic!("line comments must parse (line {}: {})", e.line, e.message),
    };
    let app = doc.get_section("AppState").expect("AppState must be a section");
    assert_eq!(app.get_str("name"), Some("Example Game"));
}

// proves: Test strategy #1 (Chosen design §4) — a leading UTF-8 BOM parses.
#[test]
fn parses_leading_utf8_bom() {
    let text = "\u{feff}\"name\" \"BOM Game\"\n";
    let doc = match parse(text) {
        Ok(doc) => doc,
        Err(e) => panic!("a leading BOM must parse (line {}: {})", e.line, e.message),
    };
    assert_eq!(doc.get_str("name"), Some("BOM Game"));
}

// proves: Test strategy #1 (Chosen design §4) — CRLF line endings parse.
#[test]
fn parses_crlf_line_endings() {
    let text = "\"AppState\"\r\n{\r\n\t\"name\"\t\t\"CRLF Game\"\r\n}\r\n";
    let doc = match parse(text) {
        Ok(doc) => doc,
        Err(e) => panic!("CRLF input must parse (line {}: {})", e.line, e.message),
    };
    let app = doc.get_section("AppState").expect("AppState must be a section");
    assert_eq!(app.get_str("name"), Some("CRLF Game"));
}

// proves: Test strategy #1 (A6, Chosen design §4) — `\\` and `\"` unescape to
// `\` and `"` inside quoted values.
#[test]
fn unescapes_backslash_and_quote_in_quoted_values() {
    // File bytes: "path"\t\t"a\\b\"c"  and  "win"\t\t"C:\\Program Files (x86)\\Steam"
    let text = "\"path\"\t\t\"a\\\\b\\\"c\"\n\"win\"\t\t\"C:\\\\Program Files (x86)\\\\Steam\"\n";
    let doc = match parse(text) {
        Ok(doc) => doc,
        Err(e) => panic!("escaped values must parse (line {}: {})", e.line, e.message),
    };
    assert_eq!(doc.get_str("path"), Some(r#"a\b"c"#));
    assert_eq!(doc.get_str("win"), Some(r"C:\Program Files (x86)\Steam"));
}

// proves: Test strategy #1 (Chosen design §4) — an unbalanced brace returns a
// positioned `KeyValueError { line, message }` at or after the unclosed `{`.
#[test]
fn rejects_unbalanced_braces_with_positioned_error() {
    let text = "\"AppState\"\n{\n\t\"name\" \"Example Game\"\n";
    match parse(text) {
        Ok(_) => panic!("unbalanced braces must be rejected"),
        Err(e) => {
            assert!(
                (2..=4).contains(&e.line),
                "error must point at or after the unclosed `{{` on line 2, got line {}",
                e.line
            );
            assert!(!e.message.is_empty(), "the error must carry a message");
        }
    }
}

// proves: Test strategy #1 (Chosen design §4) — an unterminated string returns a
// positioned `KeyValueError` carrying the offending line.
#[test]
fn rejects_unterminated_string_with_positioned_error() {
    // The unterminated string opens on line 3.
    let text = "\"AppState\"\n{\n\t\"name\" \"Example Game\n\t\"installdir\" \"Example Game\"\n}\n";
    match parse(text) {
        Ok(_) => panic!("an unterminated string must be rejected"),
        Err(e) => {
            assert_eq!(e.line, 3, "the offending line is where the string opens");
            assert!(!e.message.is_empty(), "the error must carry a message");
        }
    }
}

// proves: Test strategy #1 (A7, Chosen design §4) — a `[` conditional is rejected
// with a positioned `KeyValueError`.
#[test]
fn rejects_conditional_with_positioned_error() {
    let text = "\"AppState\"\n{\n\t[$WIN32]\n\t\"name\" \"Example Game\"\n}\n";
    match parse(text) {
        Ok(_) => panic!("a `[$WIN32]` conditional must be rejected"),
        Err(e) => {
            assert_eq!(e.line, 3, "the offending line is the conditional line");
            assert!(!e.message.is_empty(), "the error must carry a message");
        }
    }
}

// proves: Test strategy #1 (A7, Chosen design §4) — a `#base` line is rejected
// with a positioned `KeyValueError`.
#[test]
fn rejects_base_directive_with_positioned_error() {
    let text = "#base \"gameinfo.txt\"\n\"AppState\"\n{\n\t\"name\" \"Example Game\"\n}\n";
    match parse(text) {
        Ok(_) => panic!("a `#base` directive must be rejected"),
        Err(e) => {
            assert_eq!(e.line, 1, "the offending line is the `#base` line");
            assert!(!e.message.is_empty(), "the error must carry a message");
        }
    }
}

// proves: Test strategy #1 (Chosen design §4) — a section nested 17 levels deep
// is rejected, while 16 levels is the accepted cap.
#[test]
fn enforces_sixteen_level_nesting_cap() {
    let nested = |levels: usize| {
        let mut text = String::new();
        for i in 0..levels {
            text.push_str(&format!("\"lvl{i}\"\n{{\n"));
        }
        text.push_str("\"leaf\" \"value\"\n");
        for _ in 0..levels {
            text.push_str("}\n");
        }
        text
    };
    assert!(parse(&nested(16)).is_ok(), "16 nesting levels is the cap and must be accepted");
    match parse(&nested(17)) {
        Ok(_) => panic!("17 nesting levels must be rejected"),
        Err(e) => assert!(e.line > 0, "the error must carry a line number"),
    }
}
