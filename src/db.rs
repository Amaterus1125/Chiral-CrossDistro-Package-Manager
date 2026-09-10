//! File tracking DB
//! Format for files likee  [pkgname=1.2.3|debian]
//! /usr/local/bin/foo

use std::collections::HashSet;
use std::fs::{self, File};
use std::path::PathBuf;

use crate::paths::{home, is_root};

fn db_dir() -> Result<PathBuf, String> {
    if is_root() { Ok(PathBuf::from("/var/lib/chiral")) }
    else         { Ok(home()?.join(".local").join("share").join("chiral")) }
}

fn db_file() -> Result<PathBuf, String> {
    Ok(db_dir()?.join("installed.db"))
}

fn db_ensure() -> Result<(), String> {
    let dir  = db_dir()?;
    let file = db_file()?;
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Cannot create DB dir {:?}: {}", dir, e))?;
    if !file.exists() {
        File::create(&file)
            .map_err(|e| format!("Cannot create DB file: {}", e))?;
    }
    Ok(())
}

fn db_read_all() -> Result<String, String> {
    db_ensure()?;
    fs::read_to_string(db_file()?).map_err(|e| e.to_string())
}

pub fn db_list() -> Result<Vec<(String, String, String)>, String> {
    let raw = db_read_all()?;
    let mut out = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            let inner = &line[1..line.len()-1];
            let mut eq   = inner.splitn(2, '=');
            let name     = eq.next().unwrap_or("").to_string();
            let rest     = eq.next().unwrap_or("unknown|unknown");
            let mut pipe = rest.splitn(2, '|');
            let version  = pipe.next().unwrap_or("unknown").to_string();
            let source   = pipe.next().unwrap_or("unknown").to_string();
            out.push((name, version, source));
        }
    }
    Ok(out)
}

pub fn db_files_for(package: &str) -> Result<Vec<PathBuf>, String> {
    let package = sanitize_db_field(package);
    let raw = db_read_all()?;
    let mut in_block = false;
    let mut files    = Vec::new();

    for line in raw.lines() {
        let line = line.trim();
        if line.starts_with(&format!("[{}=", package)) {
            in_block = true;
            continue;
        }
        if in_block {
            if line.starts_with('[') { break; }
            if !line.is_empty() { files.push(PathBuf::from(line)); }
        }
    }
    Ok(files)
}

/// All files tracked by every *other* installed package. Used before
/// deleting a package's files, so we never remove a path another package
/// still depends on (e.g. a shared file two packages both happened to ship).
pub fn db_files_owned_by_others(package: &str) -> Result<HashSet<PathBuf>, String> {
    let mut owned = HashSet::new();
    for (name, _, _) in db_list()? {
        if name == package { continue; }
        for f in db_files_for(&name)? {
            owned.insert(f);
        }
    }
    Ok(owned)
}

pub fn db_is_installed(package: &str) -> bool {
    let package = sanitize_db_field(package);
    db_list().unwrap_or_default().iter().any(|(n, _, _)| n == &package)
}

pub fn db_get_entry(package: &str) -> Option<(String, String)> {
    let package = sanitize_db_field(package);
    db_list().unwrap_or_default()
        .into_iter()
        .find(|(n, _, _)| n == &package)
        .map(|(_, v, s)| (v, s))
}

fn sanitize_db_field(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '[' | ']' | '=' | '|' | '\n' | '\r' => '_',
            other => other,
        })
        .collect()
}

fn db_write_atomic(content: &str) -> Result<(), String> {
    let dir  = db_dir()?;
    let file = db_file()?;
    let tmp  = dir.join(format!("installed.db.tmp.{}", std::process::id()));

    fs::write(&tmp, content).map_err(|e| format!("Cannot write DB temp file: {}", e))?;
    fs::rename(&tmp, &file).map_err(|e| format!("Cannot commit DB update: {}", e))
}

pub fn db_add(package: &str, version: &str, source: &str, files: &[PathBuf]) -> Result<(), String> {
    let package = sanitize_db_field(package);
    let version = sanitize_db_field(version);
    let source  = sanitize_db_field(source);

    let raw = db_read_all()?;
    let mut new_content = String::new();
    let mut skip = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("[{}=", package)) { skip = true; continue; }
        if skip && trimmed.starts_with('[') { skip = false; }
        if !skip { new_content.push_str(line); new_content.push('\n'); }
    }

    new_content.push_str(&format!("[{}={}|{}]\n", package, version, source));
    for f in files { new_content.push_str(&format!("{}\n", f.display())); }
    new_content.push('\n');
    db_write_atomic(&new_content)
}

pub fn db_remove_entry(package: &str) -> Result<(), String> {
    let package = sanitize_db_field(package);
    let raw = db_read_all()?;
    let mut new_content = String::new();
    let mut skip = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("[{}=", package)) { skip = true; continue; }
        if skip && trimmed.starts_with('[') { skip = false; }
        if !skip { new_content.push_str(line); new_content.push('\n'); }
    }
    db_write_atomic(&new_content)
}
