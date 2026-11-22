//! The registry's whole key rule, in one place (design 026, Chosen design:
//! §The serving face step 1).
//!
//! The shipped pair — [`bindforge_core::MAX_DISCOVERY_SLUG_OCTETS`] and
//! [`discovery::game::Source::spells_lookup_key`] — *is* the rule (A3, A16), applied
//! to the **decoded** bytes because a query-string slug arrives percent-encoded
//! (A19) and the cap rides the key, not its wire spelling. Path safety is the only
//! extra refusal, and it is scoped to the component after the source token: keys are
//! opaque in-memory strings that are never joined onto a path, so the refusal exists
//! to keep `../`, `\`, and control bytes out of a document or a refusal line.
//!
//! `is_slug_name`'s dot ban is `game_id`-only (A4), which is why the ADR's
//! `local/<dir>` case — `local/Steam.SE` — is admitted here and a dotted `game_id`
//! is refused by `corpus.rs`.

use bindforge_core::MAX_DISCOVERY_SLUG_OCTETS;
use discovery::game::Source;

/// The separator between a source token and an external id.
const SEPARATOR: char = '/';

/// Percent-decodes one query-string or path component, strictly: a `%` that is not
/// followed by two hex digits is not a decoding, and decoded bytes that are not
/// valid UTF-8 are not a key.
#[must_use]
pub fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                let hex = bytes.get(index + 1..index + 3)?;
                let text = std::str::from_utf8(hex).ok()?;
                let decoded = u8::from_str_radix(text, 16).ok()?;
                out.push(decoded);
                index += 3;
            }
            other => {
                out.push(other);
                index += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

// `percent_encode` (the copied shipped-client escape-everything rule, A19) retired
// with `upstream.rs`: it built the outbound request target, and this registry builds
// no outbound requests now. Inbound decoding — [`percent_decode`] — is the half the
// serving face still needs.

/// Whether `key` is a lookup key this registry will act on: inside `025-B1`'s cap,
/// spelled `<token>/<external-id>` against the closed source-token set, carrying one
/// path component after the token, and free of control bytes.
///
/// This governs candidate queries, admin upserts, and cache loads alike
/// (Chosen design: §The serving face step 1).
#[must_use]
pub fn key_is_admissible(key: &str) -> bool {
    if key.is_empty() || key.len() > MAX_DISCOVERY_SLUG_OCTETS {
        return false;
    }
    if key
        .bytes()
        .any(|byte| byte.is_ascii_control() || byte == 0x7f)
    {
        return false;
    }
    // The shipped predicate, called rather than reconstructed: this is the closed
    // source-token set and the `<token>/<external-id>` shape, exactly as the plugins
    // and the client agree on them (A16).
    if !Source::spells_lookup_key(key) {
        return false;
    }
    let Some((_, external_id)) = key.split_once(SEPARATOR) else {
        return false;
    };
    // Path safety, scoped to the component after the token: no further separator of
    // either spelling, and no `.`/`..` component on its own.
    !external_id.contains('/')
        && !external_id.contains('\\')
        && external_id != "."
        && external_id != ".."
}

#[cfg(test)]
mod tests {
    //! The key rule as the design states it, one row per spelling it names: the cap
    //! and `spells_lookup_key` ride the decoded bytes, path safety is the only extra
    //! refusal, and `is_slug_name`'s dot ban stays `game_id`-only.
    use super::{key_is_admissible, percent_decode};

    #[test]
    fn the_admitted_alphabet_is_the_shipped_pair_plus_one_component() {
        for key in [
            "steam/900001",
            "epic/55",
            "local/Steam.SE",
            "local/100%",
            "local/two words",
        ] {
            assert!(key_is_admissible(key), "`{key:?}` is admitted");
        }
        let on_cap = format!("steam/{}", "a".repeat(122));
        assert_eq!(on_cap.len(), 128, "the cap is 128 octets");
        assert!(key_is_admissible(&on_cap), "on the cap is admitted");
    }

    #[test]
    fn every_spelling_the_design_names_is_refused() {
        for key in [
            "gog/900001",
            "steam-900001",
            "steam/",
            "/900001",
            "",
            "steam/900001/eu",
            "steam\\900001",
            "local/../etc",
            "local/./x",
            "../steam/900001",
            "steam/\n730",
            "steam/\t730",
            "steam/.",
            "local/..",
            format!("steam/{}", "9".repeat(130)).as_str(),
        ] {
            assert!(!key_is_admissible(key), "`{key:?}` is refused");
        }
    }

    #[test]
    fn decoding_is_strict_and_the_cap_rides_decoded_bytes() {
        assert_eq!(
            percent_decode("steam%2F900001").as_deref(),
            Some("steam/900001")
        );
        assert_eq!(
            percent_decode("local%2F100%25").as_deref(),
            Some("local/100%")
        );
        assert_eq!(
            percent_decode("a%2"),
            None,
            "a truncated escape is not a key"
        );
        assert_eq!(percent_decode("%zz"), None, "a non-hex escape is not a key");
        assert_eq!(
            percent_decode("%C3"),
            None,
            "half a UTF-8 sequence is not a key"
        );
        let wire = format!("steam/{}", "%61".repeat(122));
        assert!(
            wire.len() > 2 * 128,
            "the encoded form is several times the cap it decodes to"
        );
        let decoded = percent_decode(&wire).expect("decodes");
        assert!(key_is_admissible(&decoded), "the cap rides the decoded key");
    }

    // `percent_encode`'s own check retired with `upstream.rs`, the only product code
    // that built an outbound request target; the suites' encoder is `tests/common`.
}
