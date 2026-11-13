//! Reach-class tables loaded from the `schemas/` contract tree.
//!
//! The reach data lives in `schemas/reach-classes/`, not in Rust literals
//! (ADR-008 clause 3): one TOML table file per hand, sitting directly in the
//! version directory. The class *content* is ADR-012's: `v2` ships the
//! five-class taxonomy — `home`, `thumb`, `stretch`, `far`, `off-reach`, in
//! that severity order — and each hand's grade for every graded input, while
//! `v1` keeps the placeholder ids it shipped with. Both majors load through
//! this one loader, which is what keeps a `v2` document from answering a `v1`
//! reader. The loader's job, pinned by Test strategy #4, is still to file each
//! table under the hand it belongs to and to keep the two hands as two
//! distinct tables.
//!
//! The load goes through `document::parse_file`, so the parse budget binds
//! before a table file is read into memory, and the file-name/hand agreement
//! rule `spec.md` states is expressed as a pure function callers can check
//! without touching a filesystem. A refusal never names a local path: the
//! file name is enough to say what is wrong, and the contract tree's absolute
//! location is not something an uploader needs told.

use std::path::Path;

use crate::document::{Document, check_dir_version, parse_file};
use crate::error::CoreError;
use crate::input::InputId;

/// Which hand a reach-class table belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hand {
    Left,
    Right,
}

impl Hand {
    /// The contract-tree spelling of this hand: the value of a table's `hand`
    /// entry, and the stem of the file that table is stored in.
    pub fn name(self) -> &'static str {
        match self {
            Hand::Left => "left",
            Hand::Right => "right",
        }
    }

    /// The hand one of those spellings names, or `None` when it names
    /// neither — the closed hand set `spec.md` states.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Hand> {
        match name {
            "left" => Some(Hand::Left),
            "right" => Some(Hand::Right),
            _ => None,
        }
    }
}

/// The file-name/hand agreement rule `spec.md` states: a table is stored as
/// `<hand>.toml` and its `hand` entry must name the same hand. A pure
/// function, so a caller can check a name before reading it and the rule can
/// be exercised without a filesystem.
#[must_use]
pub fn hand_matches_file_name(hand: Hand, file_stem: &str) -> bool {
    Hand::from_name(file_stem) == Some(hand)
}

/// One reach class: this design carries its id, and the v2 taxonomy makes that
/// id name one of five postures a hand takes from its home-row anchor
/// (ADR-012, Decision Two).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReachClass {
    pub id: String,
}

/// One graded input: a v2 table's `"<input-id> <class-id>"` pair, parsed into
/// an id and the class this document grades it. A `v1` table grades nothing,
/// which is exactly what ADR-012 calls the missing half of the shipped
/// contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputGrade {
    pub input: InputId,
    pub class: ReachClass,
}

/// The reach classes for one hand, and the grade that hand gives each input it
/// grades.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReachTable {
    hand: Hand,
    classes: Vec<ReachClass>,
    grades: Vec<InputGrade>,
}

impl ReachTable {
    /// The hand this table was loaded for and filed under.
    pub fn hand(&self) -> Hand {
        self.hand
    }

    pub fn classes(&self) -> &[ReachClass] {
        &self.classes
    }

    /// The class this hand grades one input with, or `None` when this table
    /// does not grade it: `v1` tables grade nothing, and an id outside the
    /// graded set has no grade to look up.
    #[must_use]
    pub fn grade(&self, input: &InputId) -> Option<&ReachClass> {
        let graded = self.grades.iter().find(|entry| entry.input == *input);
        graded.map(|entry| &entry.class)
    }

    /// The severity rank of one class: its index in this table's `classes`
    /// order, which the v2 spec pins as the shared severity order, worst last.
    /// Rank arithmetic over that order is what a placement bound is measured
    /// in, so the taxonomy's ordinality stays table data.
    #[must_use]
    pub fn rank(&self, class: &ReachClass) -> Option<usize> {
        let classes = &self.classes;
        classes.iter().position(|other| other.id == class.id)
    }
}

/// The per-hand reach tables, kept distinct: one table per hand, never one
/// table answering under both addresses.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReachTables {
    tables: Vec<ReachTable>,
}

impl ReachTables {
    /// The table filed under one hand.
    pub fn hand(&self, hand: Hand) -> Option<&ReachTable> {
        self.tables.iter().find(|table| table.hand == hand)
    }
}

/// Load the reach-class tables from the TOML data sitting directly in one
/// `schemas/reach-classes/v<N>/` version directory.
///
/// Each hand's data is the file `<hand>.toml` in that directory; the `hand`
/// entry of the document it holds must name the same hand as its file name,
/// and its declared `version` must name the major of the directory holding it.
/// That second check runs for every reach table, `v1` directories included: a
/// `v2` table in a `v1` directory would otherwise answer a `v1` reader with
/// grades that reader has no grammar for.
pub fn load_reach_tables(dir: &Path) -> Result<ReachTables, CoreError> {
    let mut tables = Vec::new();
    for hand in [Hand::Left, Hand::Right] {
        let file_stem = hand.name();
        let file_name = format!("{file_stem}.toml");
        let document = parse_file(&dir.join(&file_name))?;
        match document {
            Document::ReachClasses(table) => {
                if table.hand != hand {
                    let declared = table.hand.name();
                    return Err(CoreError::InvalidValue {
                        key: "hand".to_string(),
                        message: format!("`{file_name}` holds the {declared} hand's table"),
                    });
                }
                check_dir_version(dir, &table.version, &file_name)?;
                tables.push(ReachTable {
                    hand,
                    classes: table.classes,
                    grades: table.keys,
                });
            }
            _ => {
                return Err(CoreError::InvalidValue {
                    key: "format".to_string(),
                    message: format!("`{file_name}` must hold a reach-classes document"),
                });
            }
        }
    }
    Ok(ReachTables { tables })
}
