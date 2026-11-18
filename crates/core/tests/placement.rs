//! The placement half of ADR-012 in `crates/core`: the `keyboard-geometry/v1`
//! table format, `load_geometry`, and the placement-legality predicate — the pure
//! function over loaded data that ADR-015 preconditions its transforms on (A5).
//!
//! proves: Test strategy #4 — geometry: `load_geometry(dir, "ansi-us")` returns 48
//! distinct coordinates covering exactly the graded id set, and a `name` outside
//! the grammar — `..`, a separator — is refused before any file is read; a
//! `geometry` entry contradicting its file stem is refused without naming a local
//! path; a coordinate outside `0..=999`, and a `keys` pair that is not a triple,
//! are refused.
//!
//! proves: Test strategy #5 — the predicate, each case naming its `(from, to)`
//! hands: WASD→IJKL `left`→`right`, WASD→OKL `left`→`right`, and WASD→ESDF
//! `left`→`left` are legal with uniform deltas `(6,0)`, `(7,0)`, `(1,0)` at
//! `GradeBound(0)`; a source-`home` to `off-reach` move is illegal at
//! `GradeBound(1)`; a one-rank degradation (`home`→`thumb`, a home key to
//! `keyboard:space`) is legal at `(1)` and illegal at `(0)`; an improving move
//! never blocks; a non-translation move set (a swap or rotation of two WASD keys)
//! is illegal at every bound and the check reports per-move deltas; a move naming
//! an ungraded or uncoordinated id is a structured refusal, not a verdict.
//!
//! proves: Test strategy #7 (the shipped-data agreement) — every graded id appears
//! in `ansi-us.toml` and every coordinated id is graded, so the predicate's
//! definedness rule can never fire on shipped data.
//!
//! The three predicate clauses are the design's, in order: definedness (every id
//! `keyboard:*`, present in the geometry, graded by both hands — a structured
//! refusal, not a verdict); shape, `coordinate(target) − coordinate(source)` the
//! same `(dx, dy)` for every move over bounded signed integers, targets distinct —
//! so a rotation, reflection, or shear is illegal whatever the grades say; and the
//! bound, rank being the class's index in the shared `classes` order, legal when
//! every degradation is at most the bound, a negative degradation never blocking.
//! The folklore rationale for the IJKL cluster comes out of the table data here,
//! not from a special case.
//!
//! Unit-level: loaded contract data and scratch directories, no network (ADR-004,
//! repo practice).

mod common;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use bindforge_core::{GradeBound, Hand, InputId, KeyboardGeometry, ReachTables};
use common::{ScratchDir, fixture_text, quoted_array, schemas_root};

/// The 48 graded inputs (A10), which are also the ids `ansi-us.toml` places.
const GRADED_IDS: [&str; 48] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
    "t", "u", "v", "w", "x", "y", "z", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "`", "-",
    "=", "[", "]", "\\", ";", "'", ",", ".", "/", "space",
];

/// WASD as `from` keys, in one fixed order, for the three worked examples.
const WASD: [&str; 4] = ["keyboard:w", "keyboard:a", "keyboard:s", "keyboard:d"];

/// The shipped geometry directory, and the shipped reach tables, as one call.
fn shipped_geometry_dir() -> PathBuf {
    schemas_root().join("keyboard-geometry").join("v1")
}

/// The shipped v2 tables and the shipped ANSI-US geometry — the pair the
/// predicate is meant to run over.
fn shipped_inputs() -> (ReachTables, KeyboardGeometry) {
    let reach_dir = schemas_root().join("reach-classes").join("v2");
    let tables = bindforge_core::load_reach_tables(&reach_dir)
        .unwrap_or_else(|err| panic!("the shipped v2 tables must load: {err}"));
    let geometry = bindforge_core::load_geometry(&shipped_geometry_dir(), "ansi-us")
        .unwrap_or_else(|err| panic!("the shipped ANSI-US geometry must load: {err}"));
    (tables, geometry)
}

/// An input id, or a test failure naming the spelling.
fn id(raw: &str) -> InputId {
    InputId::parse(raw).unwrap_or_else(|err| panic!("`{raw}` is a well-formed id: {err}"))
}

/// A move set from `from`/`to` spellings, paired positionally.
fn move_set(pairs: &[(&str, &str)]) -> Vec<(InputId, InputId)> {
    pairs.iter().map(|(from, to)| (id(from), id(to))).collect()
}

/// The three-key shift of WASD onto another cluster, as `(from, to)` pairs.
fn cluster_shift(target: &[&str; 4]) -> Vec<(InputId, InputId)> {
    move_set(
        &WASD
            .iter()
            .zip(target.iter())
            .map(|(from, to)| (*from, *to))
            .collect::<Vec<_>>(),
    )
}

/// A `keyboard-geometry` document placing the given `"<id> <col> <row>"` triples.
fn geometry_doc(geometry: &str, keys: &[&str]) -> String {
    let list = keys
        .iter()
        .map(|triple| format!("\"{triple}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "format = \"keyboard-geometry\"\nversion = \"v1\"\ngeometry = \"{geometry}\"\nkeys = \
         [{list}]\n"
    )
}

/// proves: Test strategy #4 + The model — `ansi-us.toml` pins the 48 graded ids on
/// the ANSI rows: `load_geometry(dir, "ansi-us")` answers a coordinate for every
/// graded input, all 48 distinct, and the rows the design pins by name are in the
/// right place (`q` = `0 1`, `a` = `0 2`, `z` = `0 3`, space = `3 4`).
#[test]
fn the_ansi_us_geometry_places_every_graded_input_on_its_own_square() {
    let geometry = bindforge_core::load_geometry(&shipped_geometry_dir(), "ansi-us")
        .expect("the shipped geometry loads");

    assert_eq!(
        geometry.name(),
        "ansi-us",
        "the geometry's name is the `geometry` entry its file declares"
    );

    let mut coordinates = BTreeSet::new();
    for member in GRADED_IDS {
        let raw = format!("keyboard:{member}");
        let placed = geometry.coordinate(&id(&raw)).unwrap_or_else(|| {
            panic!(
                "`{raw}` is one of the 48 graded ids, so the geometry places \
                                      it"
            )
        });
        assert!(
            coordinates.insert(placed),
            "`{raw}` shares the square {placed:?} with another key; the grid places each key once"
        );
    }
    assert_eq!(
        coordinates.len(),
        GRADED_IDS.len(),
        "the grid must hold 48 distinct squares"
    );

    for (member, expected) in [
        ("q", (0, 1)),
        ("a", (0, 2)),
        ("z", (0, 3)),
        ("space", (3, 4)),
    ] {
        assert_eq!(
            geometry.coordinate(&id(&format!("keyboard:{member}"))),
            Some(expected),
            "`keyboard:{member}` sits at `{expected:?}` on the ANSI rows"
        );
    }
}

/// proves: Test strategy #7 — the shipped tables and the shipped geometry agree:
/// every graded id has a coordinate and vice versa, so the predicate's
/// definedness rule can never fire on shipped data.
#[test]
fn every_graded_input_is_coordinated_and_every_coordinated_input_is_graded() {
    let (tables, geometry) = shipped_inputs();
    let dir = schemas_root().join("reach-classes").join("v2");

    let vocabulary: BTreeSet<String> = GRADED_IDS
        .iter()
        .map(|member| format!("keyboard:{member}"))
        .collect();

    for file in ["left.toml", "right.toml"] {
        let text = fixture_text(&dir.join(file));
        let pairs = quoted_array(&text, "keys").unwrap_or_else(|| {
            panic!("reach-classes/v2/{file}: `keys` must be one one-line string array")
        });
        let graded: BTreeSet<String> = pairs
            .iter()
            .map(|pair| {
                pair.replace("\\\\", "\\")
                    .split(' ')
                    .next()
                    .unwrap_or_default()
                    .to_string()
            })
            .collect();
        assert_eq!(
            graded, vocabulary,
            "reach-classes/v2/{file} grades exactly the closed keyboard set"
        );
    }

    let geometry_text = fixture_text(&shipped_geometry_dir().join("ansi-us.toml"));
    let triples = quoted_array(&geometry_text, "keys").unwrap_or_else(|| {
        panic!(
            "keyboard-geometry/v1/ansi-us.toml: `keys` must be one one-line \
                                  string array"
        )
    });
    let coordinated: BTreeSet<String> = triples
        .iter()
        .map(|triple| {
            triple
                .replace("\\\\", "\\")
                .split(' ')
                .next()
                .unwrap_or_default()
                .to_string()
        })
        .collect();
    assert_eq!(
        coordinated, vocabulary,
        "`ansi-us.toml` places exactly the graded set — no key is graded without a square, and none \
         is placed without a grade"
    );

    for (hand, label) in [(Hand::Left, "left"), (Hand::Right, "right")] {
        let table = tables
            .hand(hand)
            .unwrap_or_else(|| panic!("the v2 data declares the `{label}` hand table"));
        for member in GRADED_IDS {
            let input = id(&format!("keyboard:{member}"));
            assert!(
                table.grade(&input).is_some(),
                "the `{label}` table grades `keyboard:{member}`"
            );
            assert!(
                geometry.coordinate(&input).is_some(),
                "the geometry places `keyboard:{member}`, which the `{label}` table grades"
            );
        }
    }
}

/// proves: Test strategy #4 + the security finding this design carries —
/// `load_geometry` matches `name` against the `[a-z][a-z0-9-]*` grammar *before*
/// joining it, so a traversal name is refused without reading anything: the file a
/// traversal would reach is a perfectly valid geometry document here, and the call
/// still refuses. The refusal is an `InvalidValue` that names no path the loader
/// resolved and no file it opened, and it keeps A11's one-line printable shape.
#[test]
fn a_geometry_name_outside_the_grammar_is_refused_before_any_file_is_read() {
    let scratch = ScratchDir::new("geometry-name-grammar");
    let dir = scratch.path().join("grid");
    scratch.write(
        "grid/ansi-us.toml",
        &geometry_doc("ansi-us", &["keyboard:a 0 2"]),
    );
    // A valid geometry document one step *outside* the directory the caller names:
    // if the name reached the join, this file would load cleanly.
    scratch.write(
        "outside.toml",
        &geometry_doc("outside", &["keyboard:a 0 2"]),
    );
    // The same file addressed absolutely, which `Path::join` would take over `dir`.
    let absolute = scratch.path().join("outside");

    for name in [
        "..",
        ".",
        "",
        "../outside",
        "grid/../outside",
        "./outside",
        "outside/../../outside",
        "ANSI-US",
        "ansi us",
    ] {
        let Err(err) = bindforge_core::load_geometry(&dir, name) else {
            panic!(
                "`{name}` is outside the `[a-z][a-z0-9-]*` geometry grammar and must be refused \
                 before any file is read"
            );
        };
        assert!(
            matches!(err, bindforge_core::CoreError::InvalidValue { .. }),
            "a name outside the grammar is refused as `InvalidValue`, got {err:?}"
        );
        let detail = err.to_string();
        assert!(
            !detail.contains(&scratch.path().display().to_string()),
            "a refusal for a bad `name` must not disclose the local path the loader resolved \
             (A11), got `{detail}`"
        );
        assert!(
            !detail.contains("outside.toml"),
            "the refusal must not name a file the loader read: the grammar check runs *before* \
             the join, so no file is opened at all, got `{detail}`"
        );
        assert!(
            detail.chars().all(|ch| !ch.is_control()),
            "the echoed name takes the same printable-escape path as an echoed id (A11), got \
             {detail:?}"
        );
    }

    // The absolute case is the hazard the accepted finding names: `Path::join` with
    // an absolute argument replaces `dir` outright, so a caller forwarding a
    // user-selected name could read any `.toml` on the machine. The grammar check
    // runs first, so the read never happens. What this case pins is the refusal,
    // its one-line printable shape, and that no resolved file is named; the
    // caller's own string appears in the echo, which is not the disclosure A11 is
    // about — that is the loader's path, pinned above for the traversal names.
    let Err(err) = bindforge_core::load_geometry(&dir, absolute.to_str().unwrap_or("outside"))
    else {
        panic!(
            "an absolute `name` must be refused by the grammar check: `Path::join` would replace \
             the directory entirely and read any `.toml` on the machine"
        );
    };
    assert!(
        matches!(err, bindforge_core::CoreError::InvalidValue { .. }),
        "an absolute `name` is outside the grammar, so it is `InvalidValue`, got {err:?}"
    );
    let detail = err.to_string();
    assert!(
        !detail.contains('\n') && detail.chars().all(|ch| !ch.is_control()),
        "a refusal stays one bounded printable line whatever `name` carried (A11), got {detail:?}"
    );
    assert!(
        !detail.contains("outside.toml"),
        "the refusal must not name the file the traversal would have read, got `{detail}`"
    );
}

/// proves: design 024's Test strategy #14 + Chosen design (Shared grammar and
/// bound) — `load_geometry` checks the name against `MAX_SLUG_OCTETS` beside the
/// `is_slug_name` check, before the join, because `024-B1` enumerates a keyboard
/// geometry name among the bound's riders and no scope narrowing may drop one:
/// this is the case the branch previously pinned as a past-bound name *loading*,
/// re-pinned to the refusal by the escalation repair. A grammar-valid name one
/// octet past the bound is refused as `InvalidValue` even while the matching
/// geometry document sits in the directory — the bound check runs before the
/// join, like the grammar check beside it — the refusal keeps A11's one-line
/// printable shape and names no local path, and a name at exactly the bound
/// still loads.
#[test]
fn a_geometry_name_past_the_shared_slug_bound_is_refused_beside_the_grammar() {
    let scratch = ScratchDir::new("geometry-name-past-bound");
    let dir = scratch.path().join("grid");
    let past_bound = "g".repeat(bindforge_core::MAX_SLUG_OCTETS + 1);
    let at_bound = "g".repeat(bindforge_core::MAX_SLUG_OCTETS);
    // The file a past-bound name would reach is a perfectly valid geometry
    // document: if the name reached the join, it would load cleanly.
    scratch.write(
        &format!("grid/{past_bound}.toml"),
        &geometry_doc(&past_bound, &["keyboard:a 0 2"]),
    );

    let Err(err) = bindforge_core::load_geometry(&dir, &past_bound) else {
        panic!(
            "a grammar-valid name of {} octets must be refused: `024-B1` enumerates a \
             geometry name among the ids riding `MAX_SLUG_OCTETS`, so the bound applies \
             wherever the grammar does",
            bindforge_core::MAX_SLUG_OCTETS + 1
        )
    };
    assert!(
        matches!(err, bindforge_core::CoreError::InvalidValue { .. }),
        "a name past the shared bound is refused beside the grammar refusal, as \
         `InvalidValue`, got {err:?}"
    );
    let detail = err.to_string();
    assert!(
        !detail.contains(&scratch.path().display().to_string()),
        "a refusal for a past-bound `name` must not disclose the local path the loader \
         resolved (A11), got `{detail}`"
    );
    assert!(
        !detail.contains("keyboard:a"),
        "the refusal must not echo document content: the bound check runs before the \
         join, so no file is opened, got `{detail}`"
    );
    assert!(
        !detail.contains('\n') && detail.chars().all(|ch| !ch.is_control()),
        "a refusal stays one bounded printable line whatever `name` carried (A11), got \
         {detail:?}"
    );

    // The boundary itself stays open: the bound is *at most* `MAX_SLUG_OCTETS`,
    // so a name at exactly the bound rides grammar and bound alike and loads.
    scratch.write(
        &format!("grid/{at_bound}.toml"),
        &geometry_doc(&at_bound, &["keyboard:a 0 2"]),
    );
    let geometry = bindforge_core::load_geometry(&dir, &at_bound).unwrap_or_else(|err| {
        panic!(
            "a name of exactly {} octets is inside the shared bound and must load: {err}",
            bindforge_core::MAX_SLUG_OCTETS
        )
    });
    assert_eq!(
        geometry.name(),
        at_bound.as_str(),
        "the name at exactly the bound is kept whole: refusal starts one octet past it"
    );
}

/// proves: Test strategy #4 — the file-name rule mirrors the hand rule: a
/// `geometry` entry contradicting its file stem is refused, and the refusal keeps
/// the local path out of the message shown to whoever supplied the file.
#[test]
fn a_geometry_entry_contradicting_its_file_stem_is_refused_without_naming_a_path() {
    let scratch = ScratchDir::new("geometry-name-contradiction");
    let dir = scratch.path().join("grid");
    scratch.write(
        "grid/ansi-us.toml",
        &geometry_doc("dvorak-us", &["keyboard:a 0 2"]),
    );

    let Err(err) = bindforge_core::load_geometry(&dir, "ansi-us") else {
        panic!(
            "`grid/ansi-us.toml` declares `geometry = \"dvorak-us\"`; the file is `<geometry>.toml` \
             and its entry must name the same value"
        );
    };
    let detail = err.to_string();
    assert!(
        !detail.contains(&scratch.path().display().to_string()),
        "a refusal shown to whoever sent the file must not disclose a local path, got `{detail}`"
    );
    assert!(
        detail.contains("ansi-us") || detail.contains("dvorak-us"),
        "the refusal must name the name it refused, got `{detail}`"
    );
}

/// proves: Test strategy #4 — coordinates are bounded `0..=999` through checked
/// integer conversion: `999` is legal, `1000`, a negative, and an oversized
/// literal are grammar refusals rather than a wrapped value or a panic
/// (`bad-coordinate.toml` is the corpus pin).
#[test]
fn a_coordinate_outside_the_documented_bound_is_refused() {
    for triple in [
        "keyboard:a 1000 0",
        "keyboard:a 0 1000",
        "keyboard:a -1 0",
        "keyboard:a 999999999999999999999999 0",
        "keyboard:a nine 0",
    ] {
        let scratch = ScratchDir::new("geometry-coordinate-bound");
        let dir = scratch.path().join("grid");
        scratch.write("grid/tiny.toml", &geometry_doc("tiny", &[triple]));
        let result = bindforge_core::load_geometry(&dir, "tiny");
        if result.is_ok() {
            panic!("the coordinate `{triple}` is outside `0..=999` and must be refused");
        }
    }

    let scratch = ScratchDir::new("geometry-coordinate-edge");
    let dir = scratch.path().join("grid");
    scratch.write(
        "grid/tiny.toml",
        &geometry_doc("tiny", &["keyboard:a 0 0", "keyboard:w 999 999"]),
    );
    let geometry = bindforge_core::load_geometry(&dir, "tiny")
        .expect("0 and 999 are inside the documented bound, so the grid loads");
    assert_eq!(geometry.coordinate(&id("keyboard:w")), Some((999, 999)));
}

/// proves: Test strategy #4 — a `keys` element is exactly one
/// `"<keyboard-input-id> <col> <row>"` triple: two fields or four are refused
/// (`pair-two-fields.toml` is the corpus pin).
#[test]
fn a_geometry_keys_entry_that_is_not_a_triple_is_refused() {
    for triple in [
        "keyboard:a",
        "keyboard:a 0",
        "keyboard:a 0 0 0",
        "\"keyboard:a\" 0 0",
    ] {
        let scratch = ScratchDir::new("geometry-pair-shape");
        let dir = scratch.path().join("grid");
        scratch.write("grid/tiny.toml", &geometry_doc("tiny", &[triple]));
        if bindforge_core::load_geometry(&dir, "tiny").is_ok() {
            panic!("`{triple}` is not a `<id> <col> <row>` triple and must be refused");
        }
    }
}

/// proves: The model (`geometry_matches_file_name`) — the pure post-load name rule
/// beside `hand_matches_file_name`: a geometry matches its own lowercase stem and
/// nothing else, so one geometry cannot be filed from two files, which on a
/// case-insensitive filesystem is the same file twice.
#[test]
fn a_geometry_matches_only_its_own_lowercase_file_stem() {
    for (name, stem, expected) in [
        ("ansi-us", "ansi-us", true),
        ("ansi-us", "ANSI-US", false),
        ("ansi-us", "ansi_us", false),
        ("ansi-us", "ansi-us-2", false),
        ("ansi-us", "us", false),
        ("ansi-us", "dvorak-us", false),
    ] {
        let matched = bindforge_core::geometry_matches_file_name(name, stem);
        assert_eq!(
            matched,
            expected,
            "`{stem}.toml` must {}be the file for the `{name}` geometry",
            if expected { "" } else { "not " }
        );
    }
}

/// proves: Test strategy #5 — the ADR's named placement, WASD→IJKL with `(from,
/// to)` = `left`→`right`: one common delta `(6, 0)`, no degradation for the right
/// hand, legal at `GradeBound(0)`. The folklore rationale for the IJKL cluster
/// comes out of the table data — every landing key is `home` or `stretch` for the
/// right hand — not from a special case.
#[test]
fn wasd_to_ijkl_is_legal_left_to_right_at_bound_zero_with_delta_six() {
    let (tables, geometry) = shipped_inputs();
    let check = bindforge_core::placement_is_legal(
        Hand::Left,
        Hand::Right,
        &cluster_shift(&["keyboard:i", "keyboard:j", "keyboard:k", "keyboard:l"]),
        GradeBound(0),
        &tables,
        &geometry,
    )
    .expect("every id is graded and placed, so this is a verdict, not a refusal");

    assert!(check.shape_ok, "the four moves are one translation");
    assert_eq!(check.delta, Some((6, 0)), "the cluster shifts six columns");
    assert_eq!(
        check.move_deltas,
        vec![(6, 0); 4],
        "every move carries the same delta"
    );
    assert_eq!(
        check.degradations,
        vec![0; 4],
        "WASD's grades carry over to the right hand unchanged: W is `stretch` and A, S, D are \
         `home` for the left hand, and I is `stretch` while J, K, L are `home` for the right"
    );
    assert!(check.legal, "WASD→IJKL is legal at bound 0");
}

/// proves: Test strategy #5 — the second named placement, WASD→OKL with `(from,
/// to)` = `left`→`right`: the same legal shape one column further right, delta
/// `(7, 0)`, legal at `GradeBound(0)`.
#[test]
fn wasd_to_okl_is_legal_left_to_right_at_bound_zero_with_delta_seven() {
    let (tables, geometry) = shipped_inputs();
    let check = bindforge_core::placement_is_legal(
        Hand::Left,
        Hand::Right,
        &cluster_shift(&["keyboard:o", "keyboard:k", "keyboard:l", "keyboard:;"]),
        GradeBound(0),
        &tables,
        &geometry,
    )
    .expect("every id is graded and placed, so this is a verdict, not a refusal");

    assert!(check.shape_ok, "the four moves are one translation");
    assert_eq!(check.delta, Some((7, 0)));
    assert!(
        check.legal,
        "WASD→OKL is legal at bound 0: O, K, L and `;` are all at least as comfortable for the \
         right hand as W, A, S, D are for the left; degradations {:?} bound 0",
        check.degradations
    );
}

/// proves: Test strategy #5 — the same-hand cluster shift, WASD→ESDF with `(from,
/// to)` = `left`→`left`: delta `(1, 0)`, legal at `GradeBound(0)`.
#[test]
fn wasd_to_esdf_is_legal_left_to_left_at_bound_zero_with_delta_one() {
    let (tables, geometry) = shipped_inputs();
    let check = bindforge_core::placement_is_legal(
        Hand::Left,
        Hand::Left,
        &cluster_shift(&["keyboard:e", "keyboard:s", "keyboard:d", "keyboard:f"]),
        GradeBound(0),
        &tables,
        &geometry,
    )
    .expect("every id is graded and placed, so this is a verdict, not a refusal");

    assert!(check.shape_ok, "the four moves are one translation");
    assert_eq!(
        check.delta,
        Some((1, 0)),
        "the cluster shifts one column right"
    );
    assert!(
        check.legal,
        "a same-hand one-column shift is legal at bound 0"
    );
}

/// proves: Test strategy #5 — a `home` source key moving onto a key that is
/// `off-reach` for the placement's `to` hand is illegal at `GradeBound(1)`, and at
/// every bound below the four-step class gap; the class gap is where it turns
/// legal, which is what "degraded beyond a declared bound" means
/// (rank(`off-reach`) − rank(`home`) = 4).
#[test]
fn a_home_key_moving_onto_an_off_reach_key_is_illegal_below_the_class_gap() {
    let (tables, geometry) = shipped_inputs();
    let moves = move_set(&[("keyboard:a", "keyboard:;")]);

    for bound in [0, 1, 2, 3] {
        let check = bindforge_core::placement_is_legal(
            Hand::Left,
            Hand::Left,
            &moves,
            GradeBound(bound),
            &tables,
            &geometry,
        )
        .expect("both ids are graded and placed, so this is a verdict, not a refusal");
        assert_eq!(
            check.degradations,
            vec![4],
            "`keyboard:a` is `home` (rank 0) and `keyboard:;` is `off-reach` (rank 4) for the left \
             hand, and the bound is {bound}"
        );
        assert!(
            !check.legal,
            "a four-step degradation is illegal at `GradeBound({bound})`"
        );
    }

    let at_the_gap = bindforge_core::placement_is_legal(
        Hand::Left,
        Hand::Left,
        &moves,
        GradeBound(4),
        &tables,
        &geometry,
    )
    .expect("a verdict");
    assert!(
        at_the_gap.legal,
        "at exactly the class gap the declared bound is met: degradations {:?}",
        at_the_gap.degradations
    );
}

/// proves: Test strategy #5 — a one-rank degradation, `home`→`thumb`: a home key to
/// `keyboard:space` (rank 1 − rank 0 = 1) is legal at `GradeBound(1)` and illegal
/// at `GradeBound(0)`. The pinned order puts `thumb` directly above `home`, so this
/// really is the one-step case (A9).
#[test]
fn a_one_rank_home_to_thumb_move_is_legal_at_bound_one_and_illegal_at_zero() {
    let (tables, geometry) = shipped_inputs();
    let moves = move_set(&[("keyboard:a", "keyboard:space")]);

    let zero = bindforge_core::placement_is_legal(
        Hand::Left,
        Hand::Left,
        &moves,
        GradeBound(0),
        &tables,
        &geometry,
    )
    .expect("both ids are graded and placed, so this is a verdict, not a refusal");
    assert_eq!(
        zero.degradations,
        vec![1],
        "`keyboard:space` is the `thumb` rank, one step above `home`"
    );
    assert!(
        !zero.legal,
        "a one-rank degradation is illegal at `GradeBound(0)`"
    );

    let one = bindforge_core::placement_is_legal(
        Hand::Left,
        Hand::Left,
        &moves,
        GradeBound(1),
        &tables,
        &geometry,
    )
    .expect("a verdict");
    assert!(
        one.legal,
        "the same move is legal at `GradeBound(1)`; degradations {:?}",
        one.degradations
    );
}

/// proves: Test strategy #5 — a negative degradation never blocks: moving an action
/// from `keyboard:;` onto `keyboard:a` improves reach for the `to` hand, and the
/// placement is legal at the tightest bound.
#[test]
fn an_improving_move_never_blocks() {
    let (tables, geometry) = shipped_inputs();
    let moves = move_set(&[("keyboard:;", "keyboard:a")]);

    let check = bindforge_core::placement_is_legal(
        Hand::Left,
        Hand::Left,
        &moves,
        GradeBound(0),
        &tables,
        &geometry,
    )
    .expect("both ids are graded and placed, so this is a verdict, not a refusal");
    assert!(
        check.degradations[0] < 0,
        "the move is an improvement, got {:?}",
        check.degradations
    );
    assert!(
        check.legal,
        "an improving move is legal at `GradeBound(0)`; degradations {:?}",
        check.degradations
    );
}

/// proves: Test strategy #5 + predicate clause (a) — a move set that is not one
/// translation is illegal at every bound whatever the grades say: a swap of two
/// WASD keys and a rotation both break shape, and the check reports the per-move
/// deltas so a caller can explain the refusal instead of seeing a bool.
#[test]
fn a_non_translation_move_set_is_illegal_at_every_bound_and_reports_its_deltas() {
    let (tables, geometry) = shipped_inputs();

    let swap = move_set(&[("keyboard:a", "keyboard:d"), ("keyboard:d", "keyboard:a")]);
    let rotation = move_set(&[("keyboard:w", "keyboard:d"), ("keyboard:a", "keyboard:w")]);

    for (label, moves, expected_deltas) in [
        (
            "a swap of two WASD keys",
            swap.clone(),
            vec![(2, 0), (-2, 0)],
        ),
        (
            "a rotation of two WASD keys",
            rotation.clone(),
            vec![(1, 1), (1, -1)],
        ),
    ] {
        for bound in [0, 1, 4, 999] {
            let check = bindforge_core::placement_is_legal(
                Hand::Left,
                Hand::Left,
                &moves,
                GradeBound(bound),
                &tables,
                &geometry,
            )
            .expect("every id is graded and placed, so this is a verdict, not a refusal");
            assert!(
                !check.shape_ok,
                "{label} is not a translation, so clause (a) fails at bound {bound}"
            );
            assert!(
                !check.legal,
                "{label} is illegal at `GradeBound({bound})` however comfortable the targets are"
            );
            assert_eq!(
                check.move_deltas, expected_deltas,
                "{label}: the check reports each move's own delta"
            );
            assert_eq!(
                check.delta, None,
                "{label}: there is no single common delta to report"
            );
        }
    }
}

/// proves: Test strategy #5 + predicate clause (a) — the distinct-targets half of
/// clause (a), which the common-delta rule cannot reach. `a → s` and `d → s`, the
/// set this case used to run, fails clause (a) on its two different deltas, so it
/// never asks whether the targets are distinct and a reader cannot tell whether
/// the distinctness rule exists at all. The set that does ask is the one that
/// really shares a delta and repeats a target — repeating a source id is the only
/// such shape — where the check must still refuse: `delta` is `Some((1, 0))`, so
/// the common-delta half passes, and `shape_ok` is false on distinctness alone.
/// The differing-delta set stays as the pin for the other half.
#[test]
fn two_moves_onto_the_same_target_key_are_illegal() {
    let (tables, geometry) = shipped_inputs();
    let a_to_s = ("keyboard:a", "keyboard:s");
    let d_to_s = ("keyboard:d", "keyboard:s");
    let repeated_target = move_set(&[a_to_s, a_to_s]);
    let differing_deltas = move_set(&[a_to_s, d_to_s]);

    let check = bindforge_core::placement_is_legal(
        Hand::Left,
        Hand::Left,
        &repeated_target,
        GradeBound(4),
        &tables,
        &geometry,
    )
    .expect("every id is graded and placed, so this is a verdict, not a refusal");
    assert_eq!(
        check.move_deltas,
        vec![(1, 0), (1, 0)],
        "both moves share one delta"
    );
    assert_eq!(
        check.delta,
        Some((1, 0)),
        "the common-delta half of clause (a) passes, so only distinctness can fail"
    );
    assert!(
        !check.shape_ok,
        "both moves land on `keyboard:s`, which is not an assignment of the cluster"
    );
    assert!(
        !check.legal,
        "duplicate targets are illegal even at a generous bound; degradations {:?}",
        check.degradations
    );

    let check = bindforge_core::placement_is_legal(
        Hand::Left,
        Hand::Left,
        &differing_deltas,
        GradeBound(4),
        &tables,
        &geometry,
    )
    .expect("a verdict");
    assert_eq!(
        check.delta, None,
        "`a → s` and `d → s` share no delta, so the common-delta rule refuses them"
    );
    assert!(
        !check.shape_ok,
        "the common-delta half already refuses this set"
    );
    assert!(!check.legal, "and the placement is illegal all the same");
}

/// proves: Test strategy #5 + predicate clause 1 — a move naming an input the
/// placement cannot be computed over — an ungraded `mouse:*` or `gamepad:*` id, or
/// an id the geometry does not place — is a structured refusal, never a verdict.
#[test]
fn a_move_naming_an_ungraded_or_uncoordinated_input_is_a_structured_refusal() {
    let (tables, geometry) = shipped_inputs();

    for (from, to) in [
        ("mouse:left", "keyboard:a"),
        ("keyboard:a", "mouse:left"),
        ("gamepad:a", "keyboard:w"),
        ("keyboard:w", "gamepad:dpad-up"),
    ] {
        let moves = move_set(&[(from, to)]);
        let result = bindforge_core::placement_is_legal(
            Hand::Left,
            Hand::Right,
            &moves,
            GradeBound(0),
            &tables,
            &geometry,
        );
        let Err(err) = result else {
            panic!(
                "a move `{from}` → `{to}` names an input that is not graded and placed; definedness \
                 must answer with a refusal, not a verdict"
            );
        };
        assert!(
            !err.to_string().is_empty(),
            "the definedness refusal must say what is wrong"
        );
    }
}

/// proves: The model (`load_geometry`) — the predicate's inputs are loaded data,
/// so the geometry is read from the contract tree by name: a directory holding
/// another geometry is one more file, and `load_geometry` answers it with no code
/// change ("new table files, not new logic").
#[test]
fn a_geometry_is_loaded_by_name_from_its_directory() {
    let scratch = ScratchDir::new("geometry-by-name");
    let dir: &Path = &scratch.path().join("grid");
    scratch.write(
        "grid/ansi-us.toml",
        &geometry_doc("ansi-us", &["keyboard:a 0 2", "keyboard:w 1 1"]),
    );
    scratch.write(
        "grid/dvorak-us.toml",
        &geometry_doc("dvorak-us", &["keyboard:a 0 2", "keyboard:w 2 1"]),
    );

    let ansi = bindforge_core::load_geometry(dir, "ansi-us").expect("both files are valid");
    let dvorak =
        bindforge_core::load_geometry(dir, "dvorak-us").expect("one more file, same format");
    assert_eq!(ansi.coordinate(&id("keyboard:w")), Some((1, 1)));
    assert_eq!(dvorak.coordinate(&id("keyboard:w")), Some((2, 1)));
    assert_eq!(dvorak.name(), "dvorak-us");
}

// ---------------------------------------------------------------------------
// The shared slug grammar and its one exported bound (design 024 / ADR-024)
// ---------------------------------------------------------------------------

/// proves: design 024's Test strategy #2 + Chosen design (Shared grammar and bound)
/// — the bound has exactly one exported home and ADR-024's number: `MAX_SLUG_OCTETS`
/// is reachable through `lib.rs`, the crate's only export list, beside
/// `is_slug_name` in this module, and its value is the 128 that amends `013-L5`.
#[test]
fn the_shared_slug_bound_is_exported_at_one_hundred_twenty_eight_octets() {
    assert_eq!(
        bindforge_core::MAX_SLUG_OCTETS,
        128,
        "ADR-024 raises the shared slug bound from 63 to 128 octets and states it once, so every \
         consumer reads this constant instead of a private copy"
    );
}

/// proves: design 024's Test strategy #2 + Chosen design (Shared grammar and bound)
/// — the exported predicate accepts every spelling a product id can take, including
/// the `g-`-prefixed spellings and the suffixed candidates ADR-024's derivation and
/// collision rule emit, up to exactly the shared bound.
#[test]
fn is_slug_name_accepts_every_slug_shaped_spelling_up_to_the_bound() {
    let at_bound = "a".repeat(bindforge_core::MAX_SLUG_OCTETS);

    for slug in [
        "a",
        "g-7-days-to-die",
        "g-8bitboy",
        "g-8-bit-armies",
        "ys-ix-monstrum-nox",
        "ys-ix-monstrum-nox-2",
        "warhammer-40-000-darktide-sisters-novitiate-class-deluxe-edition",
        at_bound.as_str(),
    ] {
        assert!(
            bindforge_core::is_slug_name(slug),
            "`{slug}` is `[a-z][a-z0-9-]*` and inside the shared bound, so the exported grammar \
             must accept it"
        );
    }
}

/// proves: design 024's Test strategy #2 + Chosen design (Shared grammar and bound)
/// — the spellings that can never be a product id are refused by the same predicate a
/// discovered game's slug is checked against: the empty string, an uppercase spelling,
/// a leading digit, the `unknown/<name>` spelling ADR-024 retires, and the `.` / `..`
/// and separator shapes that must never ask the filesystem for a path.
#[test]
fn is_slug_name_refuses_every_spelling_that_cannot_be_one_path_component() {
    for slug in [
        "",
        "Elden-Ring",
        "ELDEN",
        "7-days-to-die",
        "unknown/elden-ring",
        "elden/ring",
        "/",
        ".",
        "..",
        "./elden",
        "../elden",
        "elden ring",
        "elden_ring",
        "-elden",
        "elden*",
        "эльден",
    ] {
        assert!(
            !bindforge_core::is_slug_name(slug),
            "`{slug}` is outside `[a-z][a-z0-9-]*`, so the shared predicate must refuse it \
             before it can name a directory or an id"
        );
    }
}
