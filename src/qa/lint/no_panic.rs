//! Zero-panic invariant lint: no bare .unwrap() or .expect() in production code.
//!
//! Enforces resilient error propagation via Result throughout the daemon codebase.

use std::fs;
use std::path::Path;

#[test]
fn no_bare_unwrap_or_expect_in_prod() {
    let mut violations = Vec::new();
    visit_rs(Path::new("src"), &mut |p, content| {
        let path_str = p.to_string_lossy().replace('\\', "/");
        if path_str.contains("/qa/") {
            return;
        }

        let mut in_test = false;
        for (idx, line) in content.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("#[cfg(test)]") || t.starts_with("mod test") {
                in_test = true;
            }
            if in_test {
                continue;
            }
            if t.starts_with("//") {
                continue;
            }
            if t.contains(".unwrap()") {
                violations.push(format!(
                    "{}:{}: bare .unwrap() in production code",
                    path_str,
                    idx + 1
                ));
            } else if t.contains(".expect(") {
                violations.push(format!(
                    "{}:{}: bare .expect() in production code",
                    path_str,
                    idx + 1
                ));
            }
        }
    });
    assert!(
        violations.is_empty(),
        "Zero-panic invariant violations:\n{}",
        violations.join("\n")
    );
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
