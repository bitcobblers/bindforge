//! The size-capped, BOM-sniffing reader (§Read discipline, A4, A19).
//!
//! The posture is the one `steam_common` and `catalog` keep: a metadata
//! length check against the budget **before** allocation, then a bounded
//! read, then a decode that never lossily replaces what it cannot read.

use std::io::Read;
use std::path::Path;

use crate::codec::ExtractionRefusal;

/// One file a codec opens (`017-B1`).
pub const MAX_BINDING_FILE_OCTETS: usize = 1_048_576;

/// Opens `path` strictly for reading — `OpenOptions::new().read(true)` and
/// nothing else (`017-L1`) — after a metadata length check against
/// [`MAX_BINDING_FILE_OCTETS`] applied *before* the contents are allocated
/// (A4). A length past the bound is refused as
/// [`ExtractionRefusal::FileOverBudget`] with no contents returned; the read
/// itself stays bounded even if the file grows behind the check.
pub fn open_capped(path: &Path) -> Result<Vec<u8>, ExtractionRefusal> {
    let over_budget = |octets: u64| ExtractionRefusal::FileOverBudget {
        path: path.display().to_string(),
        octets,
        limit: MAX_BINDING_FILE_OCTETS,
    };
    let meta = std::fs::metadata(path).map_err(|_| ExtractionRefusal::UnreadableFile {
        path: path.display().to_string(),
    })?;
    if meta.len() > MAX_BINDING_FILE_OCTETS as u64 {
        return Err(over_budget(meta.len()));
    }
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|_| ExtractionRefusal::UnreadableFile {
            path: path.display().to_string(),
        })?;
    // The bound is applied to the read itself: at most one octet past the
    // budget is ever buffered, and an over-budget result is refused unread.
    let mut bytes = Vec::new();
    file.by_ref()
        .take(MAX_BINDING_FILE_OCTETS as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ExtractionRefusal::UnreadableFile {
            path: path.display().to_string(),
        })?;
    if bytes.len() > MAX_BINDING_FILE_OCTETS {
        return Err(over_budget(bytes.len() as u64));
    }
    Ok(bytes)
}

/// BOM sniffing (A19): a UTF-8 BOM is stripped, a UTF-16LE/BE BOM decodes to
/// UTF-8 with an unpaired surrogate refused, and no BOM means strict UTF-8.
/// Nothing lossy happens on any path: undecodable bytes are
/// [`ExtractionRefusal::UndecodableText`]. This borrowed slice is the pure
/// seam the suites call directly; a caller that owns the buffer from
/// [`open_capped`] hands it to [`decode_owned`] so the strict-UTF-8 path
/// moves it into the `String` instead of copying it a second time.
pub fn decode(bytes: &[u8]) -> Result<String, ExtractionRefusal> {
    decode_owned(bytes.to_vec())
}

/// [`decode`] over an owned buffer: the no-BOM path moves `bytes` into the
/// decoded `String` (`String::from_utf8`) with no second full-size copy.
pub fn decode_owned(bytes: Vec<u8>) -> Result<String, ExtractionRefusal> {
    let undecodable = || ExtractionRefusal::UndecodableText {
        path: String::new(),
    };
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8(bytes[3..].to_vec()).map_err(|_| undecodable());
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return from_utf16(&bytes[2..], true).ok_or_else(undecodable);
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return from_utf16(&bytes[2..], false).ok_or_else(undecodable);
    }
    String::from_utf8(bytes).map_err(|_| undecodable())
}

/// Decodes UTF-16 code units of the given endianness; `None` for an odd
/// octet count or any unpaired surrogate (`String::from_utf16` refuses
/// those).
fn from_utf16(body: &[u8], little_endian: bool) -> Option<String> {
    if body.len() % 2 != 0 {
        return None;
    }
    let units: Vec<u16> = body
        .chunks(2)
        .map(|pair| {
            if little_endian {
                u16::from_le_bytes([pair[0], pair[1]])
            } else {
                u16::from_be_bytes([pair[0], pair[1]])
            }
        })
        .collect();
    String::from_utf16(&units).ok()
}
