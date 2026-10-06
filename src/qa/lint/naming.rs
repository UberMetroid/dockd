//! Naming lint: no drawer-named files (AGENT.md §2).
//!
//! Enforces descriptive, domain-specific naming across all source files.

use std::fs;
use std::path::Path;

const BANNED_STEMS: &[&str] = &[
    "util", "utils", "helper", "helpers", "common", "misc", "shared", "base", "core",
];

#[test]
fn no_drawer_named_rs_files() {
    let mut violations = Vec::new();
    visit_rs(Path::new("src"), &mut |p| {
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if BANNED_STEMS.contains(&stem) {
            violations.push(format!(
                "{} — name the function, not the drawer",
                p.display()
            ));
        }
    });
    assert!(
        violations.is_empty(),
        "drawer-named file(s) under src/:\n{}",
        violations.join("\n")
    );
}

fn visit_rs(dir: &Path, cb: &mut dyn FnMut(&Path)) {
    if let Ok(rd) = fs::read_dir(dir) {
        for ent in rd.flatten() {
            let p = ent.path();
            if p.is_dir() {
                visit_rs(&p, cb);
            } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
                cb(&p);
            }
        }
    }
}
