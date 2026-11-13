//! The input vocabulary of ADR-012: the `<namespace>:<member>` id grammar in
//! `crates/core/src/input.rs`, the three closed member sets, and the message
//! shape every refusal keeps.
//!
//! proves: Test strategy #1 — the id grammar, one case per rule: the accepted
//! spellings (`keyboard:w`, `keyboard:1`, `keyboard:;`, `keyboard:\`,
//! `keyboard:space`, `mouse:side-1`, `mouse:wheel-up`, `mouse:middle`,
//! `gamepad:dpad-up`, `gamepad:lt`) and one refusal per rule — an empty member,
//! an uppercase member, a member outside ASCII, a member with a space or a second
//! colon, `keyboard:semicolon`, `keyboard:sp`, an unknown namespace, and a member
//! outside a closed set (`mouse:thumb`, `gamepad:guide`).
//!
//! The canonical statement is `schemas/reach-classes/v2/spec.md` (Chosen design
//! → The input id): an id is lowercase, carries no space, and holds exactly one
//! colon; the member grammars are **closed** in v1 of the vocabulary, so a
//! member the set does not list is a refusal rather than an unregistered name —
//! "growing any member set widens what validators accept, so it rides a new
//! vocabulary major". The keyboard members are exactly one glyph of the ANSI-US
//! set plus the one reserved word `space` (the space bar has no glyph), which is
//! why no id can ever contain the space the `"<id> <class>"` pair encoding
//! splits on (A10). That set is written `{a-z}`, `{0-9}`, so the grammar's
//! lowercase is **ASCII** lowercase: a member that reads as lowercase only to a
//! Unicode-aware reader (`é`) is outside the set, and a suite that never asks
//! leaves a second reader free to accept what this one refuses.
//!
//! Every refusal is an existing `CoreError` variant — `InvalidValue` — echoing
//! the id escaped to printable characters, in the single-line, path-free message
//! shape the contract already promises (A11); no new variant enters the error
//! contract. Unit-level: a pure function over a `&str`, no filesystem and no
//! network (ADR-004, repo practice).

use bindforge_core::{CoreError, InputId, InputNamespace};

/// The 48 `keyboard:` members the vocabulary closes over: 26 letters, 10
/// digits, the 11 ANSI-US punctuation glyphs, and the one reserved word — the
/// set A10 pins as "the keys a movement cluster binds in v1".
const KEYBOARD_MEMBERS: [&str; 48] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
    "t", "u", "v", "w", "x", "y", "z", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "`", "-",
    "=", "[", "]", "\\", ";", "'", ",", ".", "/", "space",
];

/// The closed `mouse:` member set: five named buttons plus the wheel's two
/// directions. The wheel press is `middle` because Pointer Events button 1 *is*
/// the wheel press, and the wheel itself is an event, not a button (A7).
const MOUSE_MEMBERS: [&str; 7] = [
    "left",
    "right",
    "middle",
    "side-1",
    "side-2",
    "wheel-up",
    "wheel-down",
];

/// The closed `gamepad:` member set: the 16 W3C standard-layout button names
/// plus the two stick presses (A7).
const GAMEPAD_MEMBERS: [&str; 18] = [
    "a",
    "b",
    "x",
    "y",
    "lb",
    "rb",
    "lt",
    "rt",
    "back",
    "start",
    "l3",
    "r3",
    "dpad-up",
    "dpad-down",
    "dpad-left",
    "dpad-right",
    "left-stick",
    "right-stick",
];

/// proves: Test strategy #1 — every accepted spelling the proposal lists parses,
/// reports its namespace, and keeps its spelling exactly: an id is a name, and a
/// canonicalized name would let two spellings name one input.
#[test]
fn accepts_a_well_formed_id_in_each_namespace_and_keeps_its_spelling() {
    for (raw, namespace) in [
        ("keyboard:w", InputNamespace::Keyboard),
        ("keyboard:1", InputNamespace::Keyboard),
        ("keyboard:;", InputNamespace::Keyboard),
        ("keyboard:\\", InputNamespace::Keyboard),
        ("keyboard:space", InputNamespace::Keyboard),
        ("mouse:side-1", InputNamespace::Mouse),
        ("mouse:wheel-up", InputNamespace::Mouse),
        ("mouse:middle", InputNamespace::Mouse),
        ("gamepad:dpad-up", InputNamespace::Gamepad),
        ("gamepad:lt", InputNamespace::Gamepad),
    ] {
        let id = InputId::parse(raw)
            .unwrap_or_else(|err| panic!("`{raw}` is a well-formed id and must parse: {err}"));
        assert_eq!(id.as_str(), raw, "`{raw}` must be kept exactly as written");
        assert_eq!(
            id.namespace(),
            namespace,
            "`{raw}` must report the namespace its prefix names"
        );
    }
}

/// proves: Test strategy #1 — one case per shape rule: an id with no colon, an
/// empty namespace, an empty member, an uppercase member, a member outside the
/// ASCII range the closed set is written in, a member carrying a space, a second
/// colon, and a namespace the vocabulary does not carry. Each is a refusal, never
/// a canonicalization, and the refusal is `InvalidValue` (A11: no new error
/// variant enters the contract).
#[test]
fn rejects_an_id_that_breaks_the_namespace_or_member_grammar() {
    for (raw, fault) in [
        ("", "the empty string"),
        (
            "keyboard",
            "an id with no `:` separating namespace and member",
        ),
        (":a", "an empty namespace"),
        ("keyboard:", "an empty member"),
        (
            "keyboard:W",
            "an uppercase member: the grammar is exact, not case-folded",
        ),
        (
            "keyboard:é",
            "a member outside ASCII: the closed set is ASCII glyphs",
        ),
        (
            "mouse:side 1",
            "a member carrying a space, which the pair encoding splits on",
        ),
        (
            "keyboard:a:b",
            "a second colon, which would make the member two fields",
        ),
        (
            "joystick:left",
            "a namespace the closed vocabulary does not carry",
        ),
    ] {
        let Err(err) = InputId::parse(raw) else {
            panic!("`{raw}` — {fault} — must be refused");
        };
        assert!(
            matches!(err, CoreError::InvalidValue { .. }),
            "a malformed id is refused as `InvalidValue`, got {err:?}"
        );
    }
}

/// proves: Test strategy #1 — a keyboard member is exactly one ANSI-US glyph or
/// the single reserved word `space`: a spelled word for a glyph
/// (`keyboard:semicolon`, `keyboard:comma`) or an abbreviation of the reserved
/// word (`keyboard:sp`) is refused. The spelled word is the case the id grammar
/// exists to forbid — Options considered settles the literal glyph, and a
/// multi-character word member is what would make a space in an id thinkable.
#[test]
fn rejects_a_keyboard_member_that_is_neither_a_glyph_nor_the_reserved_word() {
    for raw in [
        "keyboard:semicolon",
        "keyboard:sp",
        "keyboard:comma",
        "keyboard:Space",
    ] {
        let Err(err) = InputId::parse(raw) else {
            panic!(
                "`{raw}` is not one glyph of the ANSI-US set and not the reserved word `space`, so \
                 it must be refused"
            );
        };
        assert!(
            matches!(err, CoreError::InvalidValue { .. }),
            "a non-glyph keyboard member is refused as `InvalidValue`, got {err:?}"
        );
    }
}

/// proves: Test strategy #1 — the member sets are closed, so a member the set
/// does not list is a refusal rather than a name the registry has not registered
/// yet (`mouse:thumb`, `gamepad:guide`). `mouse:wheel` is the same rule from the
/// other side: the wheel is an event, so only its two named directions are
/// members and its press is `middle` (A7); per-stick-direction naming is
/// deferred, so `gamepad:left-stick-up` is refused too (A10, Deferred).
#[test]
fn rejects_a_member_outside_a_closed_member_set() {
    for raw in [
        "mouse:thumb",
        "mouse:wheel",
        "gamepad:guide",
        "gamepad:dpad-center",
        "gamepad:left-stick-up",
    ] {
        let Err(err) = InputId::parse(raw) else {
            panic!("`{raw}` is outside a closed member set and must be refused");
        };
        assert!(
            matches!(err, CoreError::InvalidValue { .. }),
            "a member outside a closed set is refused as `InvalidValue`, got {err:?}"
        );
    }
}

/// proves: Test strategy #1 (the closed sets, positively) — every member the
/// three sets list parses under its own namespace, so "closed" means exactly
/// these members and no more: a set that also refused one of its own members
/// would make the shipped tables unloadable.
#[test]
fn every_member_of_a_closed_set_parses_as_that_namespace() {
    for (prefix, namespace, members) in [
        (
            "keyboard",
            InputNamespace::Keyboard,
            KEYBOARD_MEMBERS.as_slice(),
        ),
        ("mouse", InputNamespace::Mouse, MOUSE_MEMBERS.as_slice()),
        (
            "gamepad",
            InputNamespace::Gamepad,
            GAMEPAD_MEMBERS.as_slice(),
        ),
    ] {
        for member in members {
            let raw = format!("{prefix}:{member}");
            let id = InputId::parse(&raw)
                .unwrap_or_else(|err| panic!("`{raw}` is a member of a closed set: {err}"));
            assert_eq!(id.namespace(), namespace, "`{raw}` names that namespace");
            assert_eq!(id.as_str(), raw, "`{raw}` keeps its spelling");
        }
    }
}

/// proves: Test strategy #1 + Adversarial guard (message shape, A11) — a
/// malformed member is refused with the id echoed escaped to printable
/// characters, so one hostile id cannot forge an extra log line in whatever
/// reports the refusal.
#[test]
fn a_refusal_echoes_the_id_escaped_to_printable_characters() {
    let hostile = "keyboard:a\u{7}b";
    let Err(err) = InputId::parse(hostile) else {
        panic!("an id carrying a control character must be refused");
    };
    let detail = err.to_string();

    assert!(
        !detail.is_empty(),
        "the refusal must say what is wrong, got `{detail}`"
    );
    assert!(
        !detail.contains('\n') && !detail.contains('\r'),
        "a refusal stays one line whatever the id carried, got {detail:?}"
    );
    assert!(
        detail.chars().all(|ch| !ch.is_control()),
        "the echoed id must be escaped to printable characters, got {detail:?}"
    );
}
