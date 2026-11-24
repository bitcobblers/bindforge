//! `read::open_capped` and `read::decode` — the size-capped, BOM-sniffing
//! reader (§Read discipline, A4, A19).
//!
//! proves: Test strategy #4 (the `MAX_BINDING_FILE_OCTETS` seam) and #5.

mod common;

use bindforge_codecs::MAX_BINDING_FILE_OCTETS;
use bindforge_codecs::codec::ExtractionRefusal;
use bindforge_codecs::read::{decode, open_capped};
use common::TempRoot;

fn refusal_name(refusal: &ExtractionRefusal) -> String {
    format!("{refusal:?}")
}

// proves: Test strategy #4 (§Read discipline, `017-B1`) — `open_capped` reads a
// file of exactly `MAX_BINDING_FILE_OCTETS` (1 MiB): the bound is inclusive.
#[test]
fn open_capped_reads_a_file_at_the_budget() {
    let root = TempRoot::new("read-at-cap");
    let path = root.write_bytes("at-cap.ini", &vec![b'a'; MAX_BINDING_FILE_OCTETS]);
    let bytes = open_capped(&path).expect("a file exactly at the budget must read");
    assert_eq!(bytes.len(), MAX_BINDING_FILE_OCTETS);
}

// proves: Test strategy #4 (§Read discipline: "a metadata length check against
// `MAX_BINDING_FILE_OCTETS` before allocation, then a bounded read") — a
// `MAX_BINDING_FILE_OCTETS + 1` file is refused as `FileOverBudget` and no
// contents come back.
#[test]
fn open_capped_refuses_an_over_budget_file_without_contents() {
    let root = TempRoot::new("read-over-cap");
    let path = root.write_bytes("over.ini", &vec![b'a'; MAX_BINDING_FILE_OCTETS + 1]);
    let outcome = open_capped(&path);
    match outcome {
        Err(refusal) => {
            assert!(
                matches!(refusal, ExtractionRefusal::FileOverBudget { .. }),
                "the named variant must be FileOverBudget, got {refusal:?}"
            );
        }
        Ok(_) => panic!("a file over MAX_BINDING_FILE_OCTETS must be refused, never buffered"),
    }
}

// proves: Test strategy #5 (§Read discipline: "no BOM means strict UTF-8") —
// a plain UTF-8 body decodes unchanged.
#[test]
fn utf8_without_a_bom_decodes() {
    let text = "[/Script/Engine.PlayerInput]\n+Bindings=(Name=\"W\",Command=\"MoveForward\")\n";
    let decoded = decode(text.as_bytes()).expect("strict UTF-8 must decode");
    assert_eq!(decoded, text);
}

// proves: Test strategy #5 (§Read discipline: "a UTF-8 BOM is stripped") — the
// BOM is not part of the decoded text.
#[test]
fn a_utf8_bom_is_stripped() {
    let text = "hello";
    let decoded = decode(&common::utf8_with_bom(text)).expect("a UTF-8 BOM body must decode");
    assert_eq!(decoded, text, "the BOM must not reach the parse");
}

// proves: Test strategy #5 (§Read discipline: "a UTF-16LE/BE BOM decodes to
// UTF-8"; A19 — UE may write either endianness) — both BOM spellings decode.
#[test]
fn utf16_bodies_decode_through_their_boms() {
    let text = "ÅÄÖ — TurnRight MouseX";
    let le = decode(&common::utf16le_with_bom(text)).expect("UTF-16LE + BOM must decode");
    assert_eq!(le, text);
    let be = decode(&common::utf16be_with_bom(text)).expect("UTF-16BE + BOM must decode");
    assert_eq!(be, text);
}

// proves: Test strategy #5 (§Read discipline: "an unpaired surrogate
// refused") — a UTF-16 body whose units carry a lone surrogate is
// `UndecodableText`.
#[test]
fn an_unpaired_surrogate_is_undecodable_text() {
    // BOM(FF FE) + the single UTF-16LE code unit D8 00 — a high surrogate with
    // no low half.
    let le = [0xFF_u8, 0xFE, 0x00, 0xD8];
    let outcome = decode(&le);
    match outcome {
        Err(refusal) => assert!(
            matches!(refusal, ExtractionRefusal::UndecodableText { .. }),
            "expected UndecodableText, got {}",
            refusal_name(&refusal)
        ),
        Ok(text) => panic!("an unpaired surrogate must not decode, got {text:?}"),
    }
}

// proves: Test strategy #5 (§Read discipline: "no BOM means strict UTF-8") — a
// BOM-less body that is not valid UTF-8 is `UndecodableText`, not a skip or a
// lossy replacement.
#[test]
fn a_bomless_invalid_utf8_body_is_undecodable_text() {
    let bytes = [b'h', b'i', 0xC3, 0x28, b'!']; // 0xC3 expects a continuation byte
    let outcome = decode(&bytes);
    match outcome {
        Err(refusal) => assert!(
            matches!(refusal, ExtractionRefusal::UndecodableText { .. }),
            "expected UndecodableText, got {}",
            refusal_name(&refusal)
        ),
        Ok(text) => panic!("invalid UTF-8 must be refused, got {text:?}"),
    }
}
