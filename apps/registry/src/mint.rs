//! The mint rule (`027-C5`, §Issued entries): a store display name and an appid in,
//! one candidate `game_id` out — or the named reason no identity was available.
//!
//! This is the retired `024-C2` derivation re-pinned as `apps/registry` product
//! code, which the retirement gate allows (A9: the gate scans the two discovery
//! trees, and this is neither). The behavioral record it is pinned against is
//! `crates/discovery/tests/slug_tests.rs`, re-expressed through the serving face in
//! `apps/registry/tests/mint_rule.rs`, and the same table is pinned here against
//! the seam itself.
//!
//! Three things the shipped client's derivation did *not* have to do, and this one
//! does:
//!
//! - **Terminate on a held identity.** A client mints locally, where a collision is
//!   two files arguing; the registry mints into the one document every other record
//!   is indexed against, so every spelling is checked against the corpus, the issued
//!   document, and the retired list (§Issued entries' step 6), and there is no
//!   fourth spelling to invent. `024-L2`'s ordinal `-2` is not revived: an ordinal
//!   depends on which records existed when, where the appid suffix is record-stable.
//! - **Stay inside the id bound by construction.** The fold never exceeds the name's
//!   octets, but the `g-` prefix adds 2 and the appid suffix adds 7 plus the appid's
//!   digits, onto a name the snapshot loader admits at 128 octets. So no mint output
//!   is in budget by construction and the retired "length is never truncated"
//!   posture is not re-pinned here: step 5 falls back to the appid spelling, and an
//!   appid spelling that is *still* over bound is the named
//!   `mint identity over bound`.
//! - **Never produce an empty or separator-only id.** The client's caller already
//!   had a name; a store display name may be CJK-only, Arabic-only, or emoji, which
//!   the research says exist in volume — hence step 3's `steam-<appid>`.

use bindforge_core::{MAX_SLUG_OCTETS, is_slug_name};

/// Why the rule could not hand back an identity. Both arms are a named `503`: an
/// identity that cannot be named is not an identity, and `[]` is never the answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MintRefusal {
    /// Every spelling the rule is allowed to use — plain, appid-suffixed, appid — is
    /// already held or retired by a different record (§Issued entries' step 6).
    IdentityHeld,
    /// The appid spelling itself is over `MAX_SLUG_OCTETS` (step 5's last rung),
    /// which a decimal appid cannot reach but an over-long external id can.
    IdentityOverBound,
}

/// The derived part of the id: steps 1 and 2, with no knowledge of what is held.
/// Empty for a name with no Latin base letter at all.
#[must_use]
pub fn derive(display_name: &str) -> String {
    let mut folded = String::with_capacity(display_name.len());
    for character in display_name.chars() {
        let mapped = if character.is_ascii_alphanumeric() {
            Some(character.to_ascii_lowercase())
        } else {
            fold_latin(character)
        };
        match mapped {
            Some(letter) => folded.push(letter),
            // Every code point outside the covered alphabet is exactly one `-`,
            // which the collapse below then joins with its neighbours.
            None => folded.push('-'),
        }
    }
    let folded = collapse_and_trim(&folded);
    // Step 2: the shipped grammar needs a lowercase first byte (A10), so a leading
    // digit buys the `g-` prefix the retired suite's `g-7-days-to-die` records.
    if folded
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_digit())
    {
        return format!("g-{folded}");
    }
    folded
}

/// `mint_id` — §Issued entries' steps 1–6 for one store record.
///
/// `external_id` is the queried key's external id spelled as it arrived (a decimal
/// appid for every request the serving face can route here); the appid spellings are
/// built from that spelling, so the identity is tied to the key that was asked about
/// and not to a re-rendered number.
///
/// `held` answers whether a spelling is already a record's id in the corpus, the
/// issued document, or the retired list — the union §Issued entries calls the held
/// set. It is called once per spelling the rule tries, in order.
///
/// # Errors
///
/// Returns [`MintRefusal::IdentityHeld`] when every spelling is held, and
/// [`MintRefusal::IdentityOverBound`] when the appid spelling is over the id bound.
pub fn mint_id(
    display_name: &str,
    external_id: &str,
    held: &dyn Fn(&str) -> bool,
) -> Result<String, MintRefusal> {
    let derived = derive(display_name);
    let appid_id = format!("steam-{external_id}");
    // Step 3: nothing derived means the appid spelling *is* the mint id, so the
    // chain starts there rather than at an empty string.
    let mut chain: Vec<String> = Vec::with_capacity(3);
    if !derived.is_empty() {
        chain.push(derived.clone());
        // Step 4's collision spelling, earned the same held check before it is
        // served: a corpus id is an operator-chosen grammar string, so one can
        // already spell this form.
        chain.push(format!("{derived}-{appid_id}"));
    }
    // Step 5's bound fallback and step 3's empty fallback are the same spelling.
    chain.push(appid_id.clone());

    for candidate in &chain {
        if candidate.len() > MAX_SLUG_OCTETS || !is_slug_name(candidate) {
            // The plain and suffixed spellings are grammar-valid by construction, so
            // this rung is only ever reached by an over-long appid spelling — the
            // named refusal, never a truncated identity.
            if candidate == &appid_id {
                return Err(MintRefusal::IdentityOverBound);
            }
            continue;
        }
        if !held(candidate) {
            return Ok(candidate.clone());
        }
    }
    Err(MintRefusal::IdentityHeld)
}

/// Hyphen runs collapse to one, and leading and trailing hyphens are trimmed — the
/// same normalisation the retired derivation ran, which is what makes
/// `"Game--Name"` and `"  Hello World!!!  "` come out as `game-name` and
/// `hello-world`.
fn collapse_and_trim(folded: &str) -> String {
    let mut out = String::with_capacity(folded.len());
    let mut in_run = false;
    for character in folded.chars() {
        if character == '-' {
            in_run = true;
            continue;
        }
        if in_run && !out.is_empty() {
            out.push('-');
        }
        in_run = false;
        out.push(character);
    }
    out
}

/// The finite fold table: `Latin-1 Supplement` and `Latin Extended-A`, single base
/// letter only. Everything the two blocks do not spell with one base letter — `ø`,
/// `æ`, `ß`, `ł`, `Đ`, `Œ`, `ı`, `ŧ` — is outside it and becomes `-`, and so is every
/// code point in no block at all, including `Latin Extended Additional` (`ḗ`) and
/// `General Punctuation` (`’`, `—`).
///
/// This is the boundary A9's record drew, kept as a table rather than as a
/// normalisation call: no NFKD dependency, and no behaviour that changes with the
/// Unicode version the standard library happens to carry.
#[must_use]
fn fold_latin(character: char) -> Option<char> {
    let folded = match character {
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
        'Ç' | 'ç' => 'c',
        'È' | 'É' | 'Ê' | 'Ë' | 'è' | 'é' | 'ê' | 'ë' => 'e',
        'Ì' | 'Í' | 'Î' | 'Ï' | 'ì' | 'í' | 'î' | 'ï' => 'i',
        'Ñ' | 'ñ' => 'n',
        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
        'Ù' | 'Ú' | 'Û' | 'Ü' | 'ù' | 'ú' | 'û' | 'ü' => 'u',
        'Ý' | 'ý' | 'Ÿ' | 'ÿ' => 'y',

        'Ā' | 'ā' | 'Ă' | 'ă' | 'Ą' | 'ą' => 'a',
        'Ć' | 'ć' | 'Ĉ' | 'ĉ' | 'Ċ' | 'ċ' | 'Č' | 'č' => 'c',
        'Ď' | 'ď' => 'd',
        'Ē' | 'ē' | 'Ĕ' | 'ĕ' | 'Ė' | 'ė' | 'Ę' | 'ę' | 'Ě' | 'ě' => 'e',
        'Ĝ' | 'ĝ' | 'Ğ' | 'ğ' | 'Ġ' | 'ġ' | 'Ģ' | 'ģ' => 'g',
        'Ĥ' | 'ĥ' => 'h',
        'Ĩ' | 'ĩ' | 'Ī' | 'ī' | 'Ĭ' | 'ĭ' | 'Į' | 'į' | 'İ' | 'i' => 'i',
        'Ĵ' | 'ĵ' => 'j',
        'Ķ' | 'ķ' => 'k',
        'Ĺ' | 'ĺ' | 'Ļ' | 'ļ' | 'Ľ' | 'ľ' => 'l',
        'Ń' | 'ń' | 'Ņ' | 'ņ' | 'Ň' | 'ň' => 'n',
        'Ō' | 'ō' | 'Ŏ' | 'ŏ' | 'Ő' | 'ő' => 'o',
        'Ŕ' | 'ŕ' | 'Ŗ' | 'ŗ' | 'Ř' | 'ř' => 'r',
        'Ś' | 'ś' | 'Ŝ' | 'ŝ' | 'Ş' | 'ş' | 'Š' | 'š' => 's',
        'Ţ' | 'ţ' | 'Ť' | 'ť' => 't',
        'Ũ' | 'ũ' | 'Ū' | 'ū' | 'Ŭ' | 'ŭ' | 'Ů' | 'ů' | 'Ű' | 'ű' | 'Ų' | 'ų' => 'u',
        'Ŵ' | 'ŵ' => 'w',
        'Ŷ' | 'ŷ' => 'y',
        'Ź' | 'ź' | 'Ż' | 'ż' | 'Ž' | 'ž' => 'z',

        _ => return None,
    };
    Some(folded)
}

#[cfg(test)]
mod tests {
    use super::{derive, mint_id};

    /// Nothing is held, so the rule's answer is the derivation itself.
    fn nobody_holds() -> impl Fn(&str) -> bool {
        |_| false
    }

    /// Only the exact spellings named are held.
    fn those_hold(names: &[&str]) -> impl Fn(&str) -> bool {
        let owned: Vec<String> = names.iter().map(|name| (*name).to_string()).collect();
        move |candidate: &str| owned.iter().any(|held| held == candidate)
    }

    #[test]
    fn folds_the_surveyed_titles_the_retired_table_recorded() {
        for (name, slug) in [
            (
                "Warhammer 40,000: Darktide - Sisters Novitiate Class Deluxe Edition",
                "warhammer-40-000-darktide-sisters-novitiate-class-deluxe-edition",
            ),
            ("7 Days to Die", "g-7-days-to-die"),
            ("8BitBoy™", "g-8bitboy"),
            ("8-Bit Armies", "g-8-bit-armies"),
            ("0 A.D.", "g-0-a-d"),
            ("1000 Heads", "g-1000-heads"),
            ("3D Print Miniatures", "g-3d-print-miniatures"),
            ("NieR:Automata™", "nier-automata"),
            (
                "S.T.A.L.K.E.R.: Shadow of Chernobyl",
                "s-t-a-l-k-e-r-shadow-of-chernobyl",
            ),
            ("Assassin’s Creed III", "assassin-s-creed-iii"),
            ("COUNTER-STRIKE 2", "counter-strike-2"),
            ("HALF-LIFE 2: Episode One", "half-life-2-episode-one"),
            ("Super Mario Odyssey!", "super-mario-odyssey"),
            ("Game--Name", "game-name"),
            ("  Hello World!!!  ", "hello-world"),
        ] {
            assert_eq!(derive(name), slug, "`{name}` derives the recorded id");
        }
    }

    #[test]
    fn folds_the_covered_blocks_and_never_transliterates_outside_them() {
        for (name, slug) in [
            ("Café du Monde", "cafe-du-monde"),
            ("Kühlstraße Depot", "kuhlstra-e-depot"),
            ("El Niño", "el-nino"),
            ("Åre Mountain Pass", "are-mountain-pass"),
            ("Vårgården", "vargarden"),
            ("Éclair Station", "eclair-station"),
            ("Über Driver", "uber-driver"),
            ("Zażółć Gęślą Jaźń", "zazo-c-gesla-jazn"),
            ("Mølly", "m-lly"),
            ("ænigmatix", "nigmatix"),
            ("straße", "stra-e"),
            ("Doppelgänger ßß", "doppelganger"),
            ("Cḗadme", "c-adme"),
        ] {
            assert_eq!(derive(name), slug, "`{name}` derives the recorded id");
        }
    }

    #[test]
    fn derives_nothing_for_a_name_with_no_latin_base_letter() {
        for name in [
            "ドラゴンズドグマ",
            "بازی",
            "🙂🙂",
            "!!!",
            "---",
            "ø",
            "ḗ",
            "  — · ,  ",
        ] {
            assert_eq!(derive(name), "", "`{name}` derives nothing");
        }
    }

    #[test]
    fn an_empty_derivation_mints_the_appid_spelling() {
        assert_eq!(
            mint_id("ドラゴンズドグマ", "9220000", &nobody_holds()).expect("a free id"),
            "steam-9220000"
        );
    }

    #[test]
    fn a_holder_pushes_the_chain_through_the_suffix_to_the_appid_spelling() {
        let name = "Example Game";
        assert_eq!(
            mint_id(name, "9230001", &those_hold(&["example-game"])).expect("a free id"),
            "example-game-steam-9230001"
        );
        assert_eq!(
            mint_id(
                name,
                "9230001",
                &those_hold(&["example-game", "example-game-steam-9230001"])
            )
            .expect("a free id"),
            "steam-9230001"
        );
        // Step 6: three held spellings is a refusal, never a fourth invention.
        assert_eq!(
            mint_id(
                name,
                "9230002",
                &those_hold(&[
                    "example-game",
                    "example-game-steam-9230002",
                    "steam-9230002"
                ])
            ),
            Err(super::MintRefusal::IdentityHeld)
        );
    }

    #[test]
    fn an_appid_spelling_over_bound_is_named_not_truncated() {
        let external = "9".repeat(200);
        // Nothing derived, so the chain starts at the over-bound appid spelling.
        assert_eq!(
            mint_id("🙂", &external, &nobody_holds()),
            Err(super::MintRefusal::IdentityOverBound)
        );
        // And a derived name whose plain spelling is held and whose suffixed spelling
        // is over bound falls the same way: the rule never truncates to fit.
        let long_name = "a".repeat(128);
        let plain_held = |candidate: &str| candidate == long_name;
        assert_eq!(
            mint_id(&long_name, &external, &plain_held),
            Err(super::MintRefusal::IdentityOverBound)
        );
    }
}
