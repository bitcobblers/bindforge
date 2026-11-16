use discovery::slug::normalize_slug;

#[test]
fn test_normalize_slug_basic() {
    assert_eq!(normalize_slug("Elden Ring"), "unknown/elden-ring");
}

#[test]
fn test_normalize_slug_special_chars() {
    assert_eq!(
        normalize_slug("Super Mario Odyssey!"),
        "unknown/super-mario-odyssey"
    );
}

#[test]
fn test_normalize_slug_casing() {
    assert_eq!(
        normalize_slug("COUNTER-STRIKE 2"),
        "unknown/counter-strike-2"
    );
}

#[test]
fn test_normalize_slug_consecutive_hyphens() {
    assert_eq!(normalize_slug("Game--Name"), "unknown/game-name");
}

#[test]
fn test_normalize_slug_complex() {
    assert_eq!(normalize_slug("  Hello World!!!  "), "unknown/hello-world");
}
