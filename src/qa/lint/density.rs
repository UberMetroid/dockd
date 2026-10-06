//! Directory density lint: at most 8 .rs files per directory (AGENT.md §2).
//!
//! Enforces modular subdivision of crowded source subdirectories.

use std::fs;
use std::path::Path;

const MAX_DENSITY: usize = 8;

#[test]
fn directory_density_within_limit() {
    let mut violations = Vec::new();
    check_dir(Path::new("src"), &mut violations);
    assert!(
        violations.is_empty(),
        "directory density violation, want <= {} .rs files per directory:\n{}",
        MAX_DENSITY,
        violations.join("\n")
    );
}

fn check_dir(dir: &Path, violations: &mut Vec<String>) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    let mut count = 0;
    let mut subdirs = Vec::new();

    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            subdirs.push(p);
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            count += 1;
        }
    }

    if count > MAX_DENSITY {
        violations.push(format!(
            "{}: {} files (cap {})",
            dir.display(),
            count,
            MAX_DENSITY
        ));
    }

    for sub in subdirs {
        check_dir(&sub, violations);
    }
}
