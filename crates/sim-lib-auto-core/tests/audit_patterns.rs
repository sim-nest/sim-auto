use std::{fs, path::Path};

use regex::Regex;

#[test]
fn no_committed_value_matches_real_pii_shape() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crate lives under crates/sim-lib-auto-core");
    let hits = scan_fixture_tree_for_patterns(
        &repo_root.join("crates/sim-lib-auto-core/tests/fixtures"),
        &pii_patterns(),
    );
    assert!(hits.is_empty(), "committed PII/secret shapes: {hits:?}");
}

fn pii_patterns() -> Vec<(&'static str, Regex)> {
    vec![
        (
            "VIN_17",
            Regex::new(r"\b[A-HJ-NPR-Z0-9]{17}\b").expect("valid VIN pattern"),
        ),
        (
            "SE_PLATE",
            Regex::new(r"\b[A-Z]{3}[0-9]{2}[A-Z0-9]\b").expect("valid plate pattern"),
        ),
        (
            "DEALER_COOKIE",
            Regex::new(
                r"(?i)\b(?:JSESSIONID|XENTRYSESSION|dealer[_-]?cookie)=[A-Za-z0-9._~+/=-]{12,}\b",
            )
            .expect("valid cookie pattern"),
        ),
        (
            "VENDOR_TOKEN",
            Regex::new(
                r"(?i)\b(?:bearer|vendor[_-]?token|api[_-]?key)[ :=]+[A-Za-z0-9._~+/=-]{20,}\b",
            )
            .expect("valid token pattern"),
        ),
    ]
}

fn scan_fixture_tree_for_patterns(root: &Path, patterns: &[(&str, Regex)]) -> Vec<String> {
    let mut hits = Vec::new();
    scan_dir(root, patterns, &mut hits);
    hits
}

fn scan_dir(path: &Path, patterns: &[(&str, Regex)], hits: &mut Vec<String>) {
    let entries = fs::read_dir(path).unwrap_or_else(|err| {
        panic!("read fixture dir {}: {err}", path.display());
    });
    for entry in entries {
        let entry = entry.expect("read fixture entry");
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, patterns, hits);
            continue;
        }
        let text = fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("read fixture {}: {err}", path.display());
        });
        for (name, pattern) in patterns {
            for found in pattern.find_iter(&text) {
                hits.push(format!("{}:{}:{name}", path.display(), found.start()));
            }
        }
    }
}
