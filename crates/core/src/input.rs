//! The OS-neutral physical-input vocabulary (ADR-012, decision three).
//!
//! An input id names one physical input without naming one OS's key code:
//! `<namespace>:<member>`, where `keyboard:<glyph>` is the US legend at that
//! position of the ANSI-US reference geometry. Every mature key registry the
//! 012 research surveyed (USB HID, evdev, SDL, W3C `code`, macOS kVK) names
//! physical keys from that same reference geometry, so this design states the
//! geometry where readers see it (`schemas/reach-classes/v2/spec.md`, the
//! canonical home of the grammar) rather than inventing a sixth scheme.
//!
//! The member sets are closed. An id outside one is a refusal rather than an
//! unknown member a validator happens to tolerate, because accepting new
//! members widens what every validator accepts and therefore rides a new
//! vocabulary major — the posture `schemas/README.md` already states. The id
//! *shape* is what reserves the deferred physical-position naming. The sets are
//! ASCII and compared byte-exact, so a non-ASCII letter is a refusal, never a
//! fold onto a member: a reader that case-folds would put two spellings on one
//! physical key.
//!
//! No member carries a space, which is what makes the contract's pair encoding
//! (`"<input-id> <class-id>"` in a v2 reach table) split unambiguously. Reach
//! grades attach to `keyboard:*` only: this module checks the grammar, and the
//! callers in `reach` and `placement` check the namespace a grade needs.

use crate::error::CoreError;

/// The namespace half of an input id: which device a member belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InputNamespace {
    Keyboard,
    Mouse,
    Gamepad,
}

impl InputNamespace {
    /// The contract spelling of this namespace: the part before the `:`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            InputNamespace::Keyboard => "keyboard",
            InputNamespace::Mouse => "mouse",
            InputNamespace::Gamepad => "gamepad",
        }
    }

    /// The namespace one spelling names, or `None` when it names none of the
    /// three — the closed namespace set `spec.md` states. A fourth spelling is
    /// a fourth device the vocabulary does not name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<InputNamespace> {
        match name {
            "keyboard" => Some(InputNamespace::Keyboard),
            "mouse" => Some(InputNamespace::Mouse),
            "gamepad" => Some(InputNamespace::Gamepad),
            _ => None,
        }
    }
}

/// The id shape, stated apart so a refusal names the rule the id broke.
const ID_SHAPE_RULE: &str = "an input id is `<namespace>:<member>`";

/// The shape's edge condition: exactly one colon, so the member can never hold
/// one and the pair encoding always has one space to split on.
const COLON_RULE: &str = "an input id holds exactly one `:`";

/// The closed namespace set, named in the refusal rather than guessed at.
const NAMESPACE_RULE: &str = "the namespace is `keyboard`, `mouse`, or `gamepad`";

/// The closed `keyboard:` member sets, stated one rule per namespace because
/// the bounded message cannot carry a 48-member list; `spec.md` is the list.
const KEYBOARD_MEMBER_RULE: &str =
    "a keyboard member is one lowercase ASCII ANSI-US glyph, or `space`";
const MOUSE_MEMBER_RULE: &str = "a mouse member is a named button or a wheel direction";
const GAMEPAD_MEMBER_RULE: &str = "a gamepad member is a standard-layout button name";

/// The one reserved keyboard member: the space bar is the key with no glyph to
/// name it, and a word cannot collide with a single-character glyph member.
const SPACE_MEMBER: &str = "space";

/// The eleven punctuation glyphs of the ANSI-US reference geometry. With the
/// letters, the digits, and `space` these are the 48 members the graded set
/// covers; modifiers and named keys are outside the closed set.
const KEYBOARD_GLYPHS: &[char] = &['`', '-', '=', '[', ']', '\\', ';', '\'', ',', '.', '/'];

/// The closed `mouse:` set: five named buttons plus the wheel's two
/// directions. The wheel press is `middle` because Pointer Events button 1 *is*
/// that press, and a second id for one physical input invites drift.
const MOUSE_MEMBERS: &[&str] = &[
    "left",
    "middle",
    "right",
    "side-1",
    "side-2",
    "wheel-up",
    "wheel-down",
];

/// The closed `gamepad:` set: the 16 W3C standard-layout button names plus the
/// two stick presses. Per-stick-direction naming is deferred, so the sticks
/// themselves are the members.
const GAMEPAD_MEMBERS: &[&str] = &[
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

/// One physical input id, held exactly as written. Comparison is byte-exact, as
/// it is for action ids: two spellings never name one physical input, and a
/// member is never case-folded or canonicalized into a different key.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InputId {
    text: String,
    namespace: InputNamespace,
}

impl InputId {
    /// Validates one id against the closed namespace and member sets. A
    /// malformed member is an `InvalidValue` echoing the id as written, as
    /// action ids do; `CoreError`'s rendering is what escapes it to printable
    /// characters and truncates it.
    pub fn parse(text: &str) -> Result<InputId, CoreError> {
        let Some((namespace, member)) = text.split_once(':') else {
            return Err(invalid_id(text, ID_SHAPE_RULE));
        };
        if member.contains(':') {
            return Err(invalid_id(text, COLON_RULE));
        }
        let Some(kind) = InputNamespace::from_name(namespace) else {
            return Err(invalid_id(text, NAMESPACE_RULE));
        };
        if !is_member(kind, member) {
            return Err(invalid_id(text, member_rule(kind)));
        }
        Ok(InputId {
            text: text.to_string(),
            namespace: kind,
        })
    }

    /// Which closed member set this id sits in — the namespace a reach grade
    /// needs in order to exist at all.
    #[must_use]
    pub fn namespace(&self) -> InputNamespace {
        self.namespace
    }

    /// The id exactly as written, never canonicalized.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

/// Whether one member is in the closed set of its namespace.
fn is_member(namespace: InputNamespace, member: &str) -> bool {
    match namespace {
        InputNamespace::Keyboard => is_keyboard_member(member),
        InputNamespace::Mouse => MOUSE_MEMBERS.contains(&member),
        InputNamespace::Gamepad => GAMEPAD_MEMBERS.contains(&member),
    }
}

/// A keyboard member is exactly one ANSI-US glyph — a lowercase ASCII letter, a
/// digit, or one of the eleven punctuation keys of the reference geometry — or
/// the reserved word `space`. A spelled word (`keyboard:semicolon`) is the
/// refusal this rule exists to make: the ADR's `keyboard:<key>` is a glyph, not
/// a name. The set is ASCII, not Unicode-lowercase: `é` is a letter to a
/// case-folding check and is still refused, because a fold would put two
/// spellings on one physical key.
fn is_keyboard_member(member: &str) -> bool {
    if member == SPACE_MEMBER {
        return true;
    }
    let mut glyphs = member.chars();
    let Some(glyph) = glyphs.next() else {
        return false;
    };
    if glyphs.next().is_some() {
        return false;
    }
    glyph.is_ascii_lowercase() || glyph.is_ascii_digit() || KEYBOARD_GLYPHS.contains(&glyph)
}

/// Which closed set a rejected member missed. The rule names the set, not the
/// membership list, because the rejection is a single bounded line and `spec.md`
/// is where the list lives.
fn member_rule(namespace: InputNamespace) -> &'static str {
    match namespace {
        InputNamespace::Keyboard => KEYBOARD_MEMBER_RULE,
        InputNamespace::Mouse => MOUSE_MEMBER_RULE,
        InputNamespace::Gamepad => GAMEPAD_MEMBER_RULE,
    }
}

/// The one refusal shape for a malformed id: `InvalidValue`, echoing the id as
/// written. `CoreError`'s rendering escapes control characters and truncates,
/// so a hostile id can neither forge a log line nor inflate one.
fn invalid_id(id: &str, fault: &str) -> CoreError {
    CoreError::InvalidValue {
        key: "id".to_string(),
        message: format!("`{id}` is not a valid input id: {fault}"),
    }
}
