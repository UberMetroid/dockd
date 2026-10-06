//! File-size lint: every .rs file is 16–256 lines (comments and blank lines count).
//!
//! Module shims (`mod.rs` / `lib.rs`) whose code lines solely declare modules
//! or re-exports are exempt from the 16-line floor only.

use std::fs;
use std::path::Path;

const FLOOR: usize = 16;
const CAP: usize = 256;

#[test]
fn every_rs_file_within_16_to_256_lines() {
    let src = Path::new("src");
    let mut violations = Vec::new();
    visit_rs(src, &mut |p, content| {
        let n = content.lines().count();
        if n > CAP {
            violations.push(format!("{}: {} lines (cap {})", p.display(), n, CAP));
        } else if n < FLOOR && !is_shim(p, content) {
            violations.push(format!("{}: {} lines (floor {})", p.display(), n, FLOOR));
        }
    });
    assert!(
        violations.is_empty(),
        "file-size violation, want 16–256 lines:\n{}",
        violations.join("\n")
    );
}

fn is_shim(path: &Path, content: &str) -> bool {
    let is_wiring = path
        .file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|f| f == "mod.rs" || f == "lib.rs");
    if !is_wiring {
        return false;
    }
    let mut saw_decl = false;
    for line in content.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("//") || t.starts_with('#') {
            continue;
        }
        if t.starts_with("mod ") || t.starts_with("use ") || t.starts_with("pub ") {
            saw_decl = true;
            continue;
        }
        if t == "}" || t == "};" {
            continue;
        }
        return false;
    }
    saw_decl
}

fn visit_rs(dir: &Path, cb: &mut dyn FnMut(&Path, &str)) {
    if let Ok(rd) = fs::read_dir(dir) {
        for ent in rd.flatten() {
            let p = ent.path();
            if p.is_dir() {
                visit_rs(&p, cb);
            } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(t) = fs::read_to_string(&p) {
                    cb(&p, &t);
                }
            }
        }
    }
}
