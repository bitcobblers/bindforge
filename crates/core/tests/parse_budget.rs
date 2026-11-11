//! The parse budget of the adversarial guard: the input limits this design
//! decides and publishes for third-party readers in `schemas/README.md` —
//! a document larger than `MAX_DOCUMENT_BYTES` (1 MiB) and nesting deeper than
//! `MAX_NESTING_DEPTH` (16 levels) are refused with typed errors before any
//! value is built. Profile, patch, policy and manifest documents arrive from
//! disk, from another user, or from the network, so these guards are the
//! precondition for the hub validating uploads with the same compiled types.
//!
//! proves: Chosen design → Adversarial guard (input budget) — the boundary, on
//! both sides, written against the exported constants rather than the literals
//! 1 MiB and 16, the way `chain_guard.rs` treats `MAX_CHAIN_DEPTH`. The
//! language-neutral corpus carries the nesting case (`nesting-past-limit.toml`);
//! a 1 MiB fixture would bloat the contract tree, so the size half is asserted
//! in memory here, on each entry point the contract publishes for it: the text
//! path, the file path where the cap must bind *before* the read, and the stream
//! path a network upload reaches — `schemas/README.md` names `parse_file` and
//! `parse_reader` as the readers that apply the check themselves.
//!
//! The same two readers carry the error-API half of the guard contract, asserted
//! here because no fixture can: a stream of non-UTF-8 bytes is refused as
//! `NotUtf8` rather than decoded through a lossy fallback, and an I/O refusal
//! names its file through `source_path` and never through `Display` — the pair
//! `reach_tables.rs` leaves only half-pinned, since a corpus case can show a
//! path is absent from a message but not that it is present in the channel meant
//! to carry it.

mod common;

use std::io::Cursor;

use bindforge_core::CoreError;
use common::{ScratchDir, fixture_text, fixtures, profile_doc_declaring, version_dir};

/// The byte budget as a `usize`, so the boundary arithmetic below stays
/// comparable with `String::len()`.
fn document_budget() -> usize {
    usize::try_from(bindforge_core::MAX_DOCUMENT_BYTES)
        .expect("MAX_DOCUMENT_BYTES is a byte count, so it fits in a usize")
}

/// The nesting budget as a `usize`, for the same reason.
fn nesting_limit() -> usize {
    usize::try_from(bindforge_core::MAX_NESTING_DEPTH)
        .expect("MAX_NESTING_DEPTH is an array level count, so it fits in a usize")
}

/// A valid profile padded with a comment line — legal in the v1 grammar — to
/// exactly `bytes` long.
fn document_of_exactly(bytes: usize) -> String {
    let mut text = profile_doc_declaring("size-boundary", Some("v1"), &[]);
    text.push_str(&"#".repeat(bytes - text.len() - 1));
    text.push('\n');
    assert_eq!(
        text.len(),
        bytes,
        "the padding arithmetic is part of this test: the document must sit exactly on the budget"
    );
    text
}

/// A profile whose `derives` value nests arrays `levels` deep, counting the
/// outer array `derives = [` is written as.
fn document_with_array_nesting(levels: usize) -> String {
    let inner = format!("{}\"deep\"{}", "[".repeat(levels - 1), "]".repeat(levels - 1));
    format!("format = \"profile\"\nversion = \"v1\"\nid = \"nested\"\nderives = [{inner}]\n")
}

/// proves: Adversarial guard (input budget) — the cap is a limit, not an
/// off-by-one wall: a document of exactly `MAX_DOCUMENT_BYTES` is parsed.
#[test]
fn parses_a_document_of_exactly_max_document_bytes() {
    let text = document_of_exactly(document_budget());

    if let Err(err) = bindforge_core::parse(&text) {
        panic!("a document of exactly MAX_DOCUMENT_BYTES must parse; the reader refused: {err}");
    }
}

/// proves: Adversarial guard (input budget) — one byte past the budget is
/// refused with the typed `DocumentTooLarge` error naming the budget, before
/// any document value is built.
#[test]
fn refuses_a_document_one_byte_past_max_document_bytes() {
    let text = document_of_exactly(document_budget() + 1);

    let err = bindforge_core::parse(&text)
        .err()
        .expect("a document larger than MAX_DOCUMENT_BYTES must be refused, not parsed");
    match err {
        CoreError::DocumentTooLarge { limit, .. } => assert_eq!(
            limit,
            bindforge_core::MAX_DOCUMENT_BYTES,
            "the refusal must name the budget it applied"
        ),
        other => panic!("expected the `DocumentTooLarge` variant, got {other:?}"),
    }
}

/// proves: Adversarial guard (input budget) — the cap binds on the file path
/// too, which is where a caller has to be able to refuse a document it has not
/// read into memory yet (`schemas/README.md`: "The cap binds *before* the read,
/// not after it"). An over-budget file is refused by `parse_file` with the same
/// typed error.
#[test]
fn refuses_a_profile_file_larger_than_max_document_bytes() {
    let scratch = ScratchDir::new("parse-budget-file");
    let oversized = scratch.write("oversized.toml", &document_of_exactly(document_budget() + 1));

    let err = bindforge_core::parse_file(&oversized)
        .err()
        .expect("parse_file must refuse a file larger than MAX_DOCUMENT_BYTES");
    match err {
        CoreError::DocumentTooLarge { limit, .. } => assert_eq!(
            limit,
            bindforge_core::MAX_DOCUMENT_BYTES,
            "the file refusal must name the same budget as the text refusal"
        ),
        other => panic!("expected the `DocumentTooLarge` variant, got {other:?}"),
    }
}

/// proves: Adversarial guard (input budget) — a file inside the budget still
/// loads through `parse_file`, so the pre-read check is a guard and not a cap
/// on legitimate input.
#[test]
fn parses_a_profile_file_within_max_document_bytes() {
    let scratch = ScratchDir::new("parse-budget-file-ok");
    let path = scratch.write("basic.toml", &profile_doc_declaring("basic", Some("v1"), &[]));

    bindforge_core::parse_file(&path)
        .unwrap_or_else(|err| panic!("a small valid profile file must parse: {err}"));
}

/// proves: Adversarial guard (input budget) — the stream path applies the same
/// cap. A reader cannot stat what it is being handed, so `parse_reader` takes at
/// most `MAX_DOCUMENT_BYTES` from it: an over-budget stream is refused with the
/// same typed error rather than allocated whole and then refused, which is what
/// `schemas/README.md` requires of the hub's upload path (ADR-014).
#[test]
fn refuses_a_stream_one_byte_past_max_document_bytes() {
    let text = document_of_exactly(document_budget() + 1);

    let err = bindforge_core::parse_reader(Cursor::new(text.as_bytes()))
        .err()
        .expect("parse_reader must refuse a stream larger than MAX_DOCUMENT_BYTES");
    match err {
        CoreError::DocumentTooLarge { limit, .. } => assert_eq!(
            limit,
            bindforge_core::MAX_DOCUMENT_BYTES,
            "the stream refusal must name the same budget as the text and file refusals"
        ),
        other => panic!("expected the `DocumentTooLarge` variant, got {other:?}"),
    }
}

/// proves: Adversarial guard (input budget) — a stream inside the budget still
/// parses through `parse_reader`, so the cap on the unread path is a guard and
/// not a limit on legitimate input.
#[test]
fn parses_a_stream_within_max_document_bytes() {
    let text = profile_doc_declaring("stream", Some("v1"), &[]);

    bindforge_core::parse_reader(Cursor::new(text.as_bytes()))
        .unwrap_or_else(|err| panic!("a small valid profile stream must parse: {err}"));
}

/// proves: Adversarial guard (input budget) — nesting one level past
/// `MAX_NESTING_DEPTH` is refused with the typed `NestingTooDeep` error naming
/// the bound, instead of the reader recursing further.
#[test]
fn refuses_arrays_nested_one_level_past_max_nesting_depth() {
    let text = document_with_array_nesting(nesting_limit() + 1);

    let err = bindforge_core::parse(&text)
        .err()
        .expect("arrays nested past MAX_NESTING_DEPTH must be refused, not parsed");
    match err {
        CoreError::NestingTooDeep { limit, .. } => assert_eq!(
            limit,
            bindforge_core::MAX_NESTING_DEPTH,
            "the refusal must name the nesting bound it applied"
        ),
        other => panic!("expected the `NestingTooDeep` variant, got {other:?}"),
    }
}

/// proves: Adversarial guard (input budget) — the nesting bound is a limit, not
/// an off-by-one wall: at exactly `MAX_NESTING_DEPTH` array levels the budget
/// does not fire. (The value itself is not a legal `derives` array, so the
/// reader may still refuse it — as anything except the nesting budget.)
#[test]
fn the_nesting_budget_does_not_fire_at_exactly_max_nesting_depth() {
    let text = document_with_array_nesting(nesting_limit());

    if let Err(err) = bindforge_core::parse(&text) {
        assert!(
            !matches!(err, CoreError::NestingTooDeep { .. }),
            "nesting at exactly MAX_NESTING_DEPTH must not trip the budget; got {err:?}"
        );
    }
}

/// proves: Adversarial guard (input budget) — the nesting bound also ships in
/// the language-neutral corpus under `schemas/profile/v1/fixtures/invalid/`,
/// rejected as `NestingTooDeep`, so every implementer refuses it the same way
/// (the invalid-fixture corpus is the contract for rejection behavior).
#[test]
fn the_invalid_corpus_ships_the_nesting_boundary() {
    let profile_v1 = version_dir("profile", "v1");

    let nested = fixtures(&profile_v1, "invalid").into_iter().any(|path| {
        matches!(
            bindforge_core::parse(&fixture_text(&path)),
            Err(CoreError::NestingTooDeep { .. })
        )
    });

    assert!(
        nested,
        "{}'s fixtures/invalid/ must contain a document nested past MAX_NESTING_DEPTH and \
         rejected as `NestingTooDeep`",
        profile_v1.path.display()
    );
}

/// proves: Adversarial guard (input budget) — the stream path refuses bytes it
/// cannot decode as text with the typed `NotUtf8` error instead of reading them
/// through a lossy fallback. `parse_reader` is the entry point a hub upload
/// reaches, and a document decoded with replacement characters is a document the
/// sender never wrote: validating it is validating an attack surface, not a
/// profile.
#[test]
fn refuses_a_stream_that_is_not_utf8() {
    let err = bindforge_core::parse_reader(Cursor::new(&[0xff_u8, 0xfe, 0x00, 0x00]))
        .err()
        .expect("parse_reader must refuse non-UTF-8 bytes, not decode them lossily");
    assert!(
        matches!(err, CoreError::NotUtf8 { .. }),
        "expected the `NotUtf8` variant for a stream of non-UTF-8 bytes, got {err:?}"
    );
}

/// proves: Chosen design → Adversarial guard (rejection messages) — the two
/// halves of the I/O refusal, pinned together: `Display` keeps the local path
/// out of the message, because rejection text is what a hub or CLI shows whoever
/// supplied the document (`reach_tables.rs` pins that absence for a table file),
/// and `source_path` is the channel that puts it back for the operator who has
/// to find the file. Pin either half alone and the other can drift: a refusal
/// with no path anywhere names a failure nobody can locate, and a path in the
/// message discloses one machine's filesystem to another's user.
#[test]
fn an_io_refusal_names_its_file_only_through_source_path() {
    let scratch = ScratchDir::new("parse-io-source-path");
    let absent = scratch.path().join("absent.toml");
    let path_text = absent.to_str().expect("a temp-directory path is valid UTF-8");

    let err = bindforge_core::parse_file(&absent)
        .err()
        .expect("a file that is not there must be refused, not read as an empty document");
    let message = err.to_string();
    let named = err.source_path().unwrap_or_default();

    assert_eq!(
        named, path_text,
        "`source_path` is the documented channel for the path `Display` deliberately omits, so the \
         I/O refusal must carry it"
    );
    assert!(
        !message.contains(path_text),
        "the path must stay out of a rejection that may be shown to whoever sent the document: \
         {message}"
    );
}
