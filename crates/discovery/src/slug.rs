//! The canonical slug derivation ADR-024 gives discovery (its `024-C2` rule).
//!
//! One rule, in one place: a game's display name (or, failing that, its install
//! directory) becomes a single path component that satisfies the shared slug
//! grammar and can be adopted byte-for-byte as a manifest `id`. The grammar
//! itself — and the length bound the callers check the result against — lives in
//! `bindforge-core` (`is_slug_name`, `MAX_SLUG_OCTETS`); this module only
//! produces candidates and never truncates, so a name that derives past the bound
//! arrives whole at the caller's named refusal instead of being quietly cut into
//! a different identity.

/// The lower-case ASCII base of a precomposed Latin letter, or `None` when the
/// character has no single-letter canonical base.
///
/// The table covers the precomposed letters of Latin-1 Supplement and Latin
/// Extended-A whose Unicode canonical decomposition is one ASCII Latin letter
/// plus combining marks, and maps each to its lower-case base (`Å`→`a`, `É`→`e`).
/// Nothing here expands to more than one character: `ß`, `æ`, `ø`, `þ`, `ð`, `Œ`
/// and the stroke letters (`ł`, `đ`, `ħ`, `ŧ`, `ł`) have no single-letter base, so
/// they answer `None` and the caller turns them into a `-` — the same outcome
/// NFKD-with-dropped-combining-marks yields for them and for every non-Latin
/// script. That is the smallest reading under which ADR-024's "NFKD, drop
/// combining marks" and "std-only" clauses both hold; code points outside these
/// two blocks (Latin Extended Additional, for example) fold to `-`.
fn fold_latin_base(c: char) -> Option<char> {
    let base = match c {
        // Latin-1 Supplement. `Æ æ Ð ð × Ø ø ÷ Þ þ ß` are deliberately absent:
        // none has a single-letter canonical base.
        '\u{00C0}'..='\u{00C5}' | '\u{00E0}'..='\u{00E5}' => 'a',
        '\u{00C7}' | '\u{00E7}' => 'c',
        '\u{00C8}'..='\u{00CB}' | '\u{00E8}'..='\u{00EB}' => 'e',
        '\u{00CC}'..='\u{00CF}' | '\u{00EC}'..='\u{00EF}' => 'i',
        '\u{00D1}' | '\u{00F1}' => 'n',
        '\u{00D2}'..='\u{00D6}' | '\u{00F2}'..='\u{00F6}' => 'o',
        '\u{00D9}'..='\u{00DC}' | '\u{00F9}'..='\u{00FC}' => 'u',
        '\u{00DD}' | '\u{00FD}' | '\u{00FF}' => 'y',
        // Latin Extended-A. Strokes, ligatures, `ı`, `ĸ`, `ŉ`, `Ŋ ŋ`, `Œ œ`, `ſ`
        // and `Ŀ ŀ` are absent for the same reason as above.
        '\u{0100}'..='\u{0105}' => 'a',
        '\u{0106}'..='\u{010D}' => 'c',
        '\u{010E}' | '\u{010F}' => 'd',
        '\u{0112}'..='\u{011B}' => 'e',
        '\u{011C}'..='\u{0123}' => 'g',
        '\u{0124}' | '\u{0125}' => 'h',
        '\u{0128}'..='\u{012F}' | '\u{0130}' => 'i',
        '\u{0134}' | '\u{0135}' => 'j',
        '\u{0136}' | '\u{0137}' => 'k',
        '\u{0139}'..='\u{013E}' => 'l',
        '\u{0143}'..='\u{0148}' => 'n',
        '\u{014C}'..='\u{0151}' => 'o',
        '\u{0154}'..='\u{0159}' => 'r',
        '\u{015A}'..='\u{0161}' => 's',
        '\u{0162}'..='\u{0165}' => 't',
        '\u{0168}'..='\u{0173}' => 'u',
        '\u{0174}' | '\u{0175}' => 'w',
        '\u{0176}'..='\u{0178}' => 'y',
        '\u{0179}'..='\u{017E}' => 'z',
        _ => return None,
    };
    Some(base)
}

/// Derives the ADR-024 slug of `name`, or `None` when the name holds nothing a
/// slug can be built from.
///
/// In order: fold precomposed Latin to its ASCII base with [`fold_latin_base`],
/// ASCII case-fold the rest, map every remaining character outside `[a-z0-9]` to
/// `-`, collapse runs of `-`, trim `-` at both ends, and prefix a leading digit
/// with `g-` so a title such as `7 Days to Die` stays grammar-valid. An empty
/// result is a refusal rather than a bare separator.
///
/// The output is pure ASCII — so octets and characters are equal — and it is
/// never truncated: length is the caller's check against
/// [`bindforge_core::MAX_SLUG_OCTETS`], and a candidate past that bound must
/// arrive whole to be refused by name.
pub fn derive_slug(name: &str) -> Option<String> {
    let mut folded = String::with_capacity(name.len());
    for c in name.chars() {
        folded.push(match fold_latin_base(c) {
            Some(base) => base,
            None if c.is_ascii_digit() || c.is_ascii_lowercase() => c,
            None if c.is_ascii_uppercase() => c.to_ascii_lowercase(),
            None => '-',
        });
    }

    let mut collapsed = String::with_capacity(folded.len());
    for c in folded.chars() {
        if c == '-' && collapsed.ends_with('-') {
            continue;
        }
        collapsed.push(c);
    }

    let slug = collapsed.trim_matches('-');
    if slug.is_empty() {
        return None;
    }
    Some(if slug.as_bytes()[0].is_ascii_digit() {
        format!("g-{slug}")
    } else {
        slug.to_string()
    })
}
