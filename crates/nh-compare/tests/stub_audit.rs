//! Stub & TODO Audit: Scans nh-core for stubs and TODO markers.
//!
//! Prints a dashboard of TODO counts per file and detects stub patterns
//! (functions returning early with placeholder behavior).

use std::collections::BTreeMap;
use std::fs;

const NH_CORE_SRC: &str = "/Users/pierre/src/games/nethack-rs/crates/nh-core/src";

fn walk_rust_files(dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    walk_dir(dir, &mut files);
    files.sort();
    files
}

fn walk_dir(dir: &str, files: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_dir(path.to_str().unwrap(), files);
        } else if path.extension().is_some_and(|e| e == "rs") {
            files.push(path.to_string_lossy().to_string());
        }
    }
}

fn relative_path(full: &str) -> &str {
    full.strip_prefix(NH_CORE_SRC)
        .map(|s| s.strip_prefix('/').unwrap_or(s))
        .unwrap_or(full)
}

#[test]
fn todo_audit_dashboard() {
    let files = walk_rust_files(NH_CORE_SRC);

    let mut by_file: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_todos = 0;
    let mut total_lines = 0;

    for path in &files {
        let content = fs::read_to_string(path).unwrap_or_default();
        let todo_count = content.matches("TODO").count();
        let line_count = content.lines().count();
        total_todos += todo_count;
        total_lines += line_count;
        if todo_count > 0 {
            by_file.insert(relative_path(path).to_string(), todo_count);
        }
    }

    println!("\n=== TODO Audit Dashboard ===");
    println!("Total files scanned: {}", files.len());
    println!("Total lines: {}", total_lines);
    println!("Total TODOs: {}", total_todos);
    println!();

    // Sort by count descending
    let mut sorted: Vec<_> = by_file.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    println!("{:<50} {:>6}", "File", "TODOs");
    println!("{}", "-".repeat(58));

    for (file, count) in &sorted {
        println!("{:<50} {:>6}", file, count);
    }

    println!("{}", "-".repeat(58));
    println!("{:<50} {:>6}", "TOTAL", total_todos);
}

#[test]
fn stub_pattern_audit() {
    let files = walk_rust_files(NH_CORE_SRC);

    let stub_patterns = [
        "not yet implemented",
        "// stub",
        "// placeholder",
        "// PLACEHOLDER",
        "unimplemented!",
        "todo!(",
    ];

    let mut stubs: BTreeMap<String, Vec<(usize, String)>> = BTreeMap::new();
    let mut total_stubs = 0;

    for path in &files {
        let content = fs::read_to_string(path).unwrap_or_default();
        let rel = relative_path(path).to_string();

        for (line_num, line) in content.lines().enumerate() {
            let line_lower = line.to_lowercase();
            for pattern in &stub_patterns {
                if line_lower.contains(&pattern.to_lowercase()) {
                    let entry = stubs.entry(rel.clone()).or_default();
                    entry.push((line_num + 1, line.trim().to_string()));
                    total_stubs += 1;
                    break; // count each line only once
                }
            }
        }
    }

    println!("\n=== Stub Pattern Audit ===");
    println!("Total stub indicators found: {}", total_stubs);
    println!();

    println!("{:<50} {:>6}", "File", "Stubs");
    println!("{}", "-".repeat(58));

    let mut sorted: Vec<_> = stubs.iter().collect();
    sorted.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    for (file, locations) in &sorted {
        println!("{:<50} {:>6}", file, locations.len());
        for (line_num, text) in locations.iter().take(3) {
            let display = if text.len() > 70 {
                format!("{}...", &text[..67])
            } else {
                text.clone()
            };
            println!("  L{}: {}", line_num, display);
        }
        if locations.len() > 3 {
            println!("  ... and {} more", locations.len() - 3);
        }
    }

    println!("{}", "-".repeat(58));
    println!("{:<50} {:>6}", "TOTAL", total_stubs);
}

#[test]
fn early_return_audit() {
    // Detect functions that return default/empty/false immediately
    // These are likely shallow ports
    let files = walk_rust_files(NH_CORE_SRC);

    let early_return_patterns = [
        "return Vec::new()",
        "return None",
        "return false",
        "return String::new()",
        "return 0",
        "Vec::new()\n}",
        "false\n}",
        "None\n}",
    ];

    let mut suspicious: BTreeMap<String, usize> = BTreeMap::new();
    let mut total = 0;

    for path in &files {
        let content = fs::read_to_string(path).unwrap_or_default();
        let rel = relative_path(path).to_string();
        let mut count = 0;

        for pattern in &early_return_patterns {
            count += content.matches(pattern).count();
        }

        if count > 5 {
            // Threshold to avoid noise
            suspicious.insert(rel, count);
            total += count;
        }
    }

    println!("\n=== Early Return Audit (potential shallow stubs) ===");
    println!(
        "Files with >5 early-return patterns: {}",
        suspicious.len()
    );
    println!();

    let mut sorted: Vec<_> = suspicious.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    println!("{:<50} {:>6}", "File", "Count");
    println!("{}", "-".repeat(58));

    for (file, count) in &sorted {
        println!("{:<50} {:>6}", file, count);
    }

    println!("{}", "-".repeat(58));
    println!("{:<50} {:>6}", "TOTAL", total);
    println!("\nNote: High counts in large files may be normal. Focus on small files with many early returns.");
}
