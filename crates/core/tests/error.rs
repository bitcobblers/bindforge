//! The module-mirrored suite for `crates/core/src/error.rs`'s echo renderer
//! (ADR-004: this file is the unit test of that module).
//!
//! proves: design 024's Test strategy #7 + Chosen design (Shared grammar and bound —
//! `error.rs`: `one_line` becomes `pub fn escape_and_truncate(raw: &str) -> String`
//! and is re-exported from `lib.rs`) — the one implementation ADR-024's `024-L1` echo
//! discipline names: control characters come back as `\uXXXX` escapes, a long value
//! is truncated with `…`, and a short printable value is echoed byte-for-byte.
//!
//! Why this half matters for ADR-024: the CLI-boundary refusal for a game whose slug
//! violates the shared grammar or bound echoes the offending value, and that value
//! arrives from a third-party plugin — untrusted input at a new boundary. The renderer
//! is what makes the echo unable to forge a second stderr line or inflate log volume
//! (`schemas/README.md`: rejections are "a single bounded line" that "escapes control
//! characters and truncates"). Which values reach an echo at all is the boundary's
//! business; what an echo can *look like* is this file's.
//!
//! Unit-level: pure function over strings, no filesystem, no network (repo practice).

/// proves: design 024's Test strategy #7 — a control character inside an echoed value
/// is rendered as a `\uXXXX` escape rather than emitted: the exact spelling the
/// design states, so a refusal stays one line and a reader can see what was refused.
#[test]
fn renders_a_control_character_as_a_unicode_escape() {
    assert_eq!(
        bindforge_core::escape_and_truncate("ys-ix\u{7}monstrum"),
        "ys-ix\\u0007monstrum",
        "a bell inside an echoed value must come back as its `\\uXXXX` escape"
    );
}

/// proves: design 024's Test strategy #7 + `schemas/README.md` (single bounded line)
/// — no control character survives the renderer raw, so a value carrying a forged
/// line break can never render a second line in a refusal.
#[test]
fn renders_every_control_character_escaped_so_a_value_cannot_forge_a_second_line() {
    for control in ['\u{0}', '\u{7}', '\t', '\n', '\r', '\u{1b}', '\u{85}'] {
        let raw = format!("slug-before{control}slug-after");
        let echoed = bindforge_core::escape_and_truncate(&raw);

        assert!(
            !echoed.contains(control),
            "`{control:?}` must not appear raw in an echoed value, got {echoed:?}"
        );
        assert!(
            echoed.chars().all(|ch| !ch.is_control()),
            "an echoed value may carry no control character at all, got {echoed:?}"
        );
        assert_eq!(
            echoed.lines().count(),
            1,
            "one refused value is one line, whatever it carried, got {echoed:?}"
        );
    }
}

/// proves: design 024's Test strategy #7 — the truncation half: a long value is cut,
/// with the `…` marker the design names, instead of pasting the whole hostile value
/// into the log; the tail it drops stays dropped.
#[test]
fn truncates_a_long_value_with_an_ellipsis() {
    let filler = "x".repeat(4096);
    let tail = "TAIL-OF-THE-ECHOED-VALUE";
    let raw = format!("{filler}{tail}");

    let echoed = bindforge_core::escape_and_truncate(&raw);

    assert!(
        echoed.chars().count() < raw.chars().count(),
        "an echo must be shorter than the 4 KiB value it refused, got {} chars for {} chars",
        echoed.chars().count(),
        raw.chars().count()
    );
    assert!(
        echoed.ends_with('…'),
        "a truncated echo must say so with the `…` marker, got {echoed:?}"
    );
    assert!(
        !echoed.contains(tail),
        "the part the renderer dropped must not reappear in the echo, got {echoed:?}"
    );
}

/// proves: design 024's Test strategy #7 + Chosen design (CLI boundary and output) —
/// the echo is still useful: a grammar-valid-shaped or near-bound value that is short
/// and printable is echoed unchanged, which is what lets
/// `error: refused game with invalid slug: <value>` name the value it refused.
#[test]
fn echoes_a_short_printable_value_unchanged() {
    for raw in [
        "ys-ix-monstrum-nox",
        "unknown/elden-ring",
        "Elden-Ring",
        "a".repeat(64).as_str(),
    ] {
        assert_eq!(
            bindforge_core::escape_and_truncate(raw),
            raw,
            "a printable value inside the truncation budget must come back byte-for-byte"
        );
    }
}
