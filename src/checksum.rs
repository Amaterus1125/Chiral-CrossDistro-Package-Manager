//! SHA-256 hashing and verification helpers shared by the Arch, Debian, and
//! GitHub-packages download paths.

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use crate::constants::SERVER;

pub fn sha256_hex_of_file(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};

    let mut f = File::open(path)
        .map_err(|e| format!("Cannot open {:?} for hashing: {}", path, e))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = f.read(&mut buf)
            .map_err(|e| format!("Read error while hashing {:?}: {}", path, e))?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }

    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for b in digest {
        hex.push_str(&format!("{:02x}", b));
    }
    Ok(hex)
}

/// Case- and whitespace-insensitive comparison — upstreams are inconsistent
/// about hex casing, and sidecar files often have trailing newlines.
fn checksum_matches(actual_hex: &str, expected_hex: &str) -> bool {
    actual_hex.trim().eq_ignore_ascii_case(expected_hex.trim())
}

pub fn verify_file_sha256(path: &Path, expected_hex: &str, what: &str) -> Result<(), String> {
    let actual = sha256_hex_of_file(path)?;
    if !checksum_matches(&actual, expected_hex) {
        let _ = fs::remove_file(path);
        return Err(format!(
            "Checksum mismatch for {}: expected {}, got {}. File deleted, refusing to use it.",
            what, expected_hex.trim(), actual
        ));
    }
    Ok(())
}

pub fn verify_chpm_package_checksum(package: &str, tarball: &Path) -> Result<(), String> {
    let sidecar_url = format!("{}/{}.tar.gz.sha256", SERVER, package);

    let resp = match reqwest::blocking::get(&sidecar_url) {
        Ok(r) if r.status().is_success() => r,
        _ => {
            eprintln!("  ⚠ No published checksum for '{}' yet — installing unverified.", package);
            return Ok(());
        }
    };

    let body = match resp.text() {
        Ok(b) => b,
        Err(_) => {
            eprintln!("  ⚠ Could not read checksum file for '{}' — installing unverified.", package);
            return Ok(());
        }
    };

    let expected = body.split_whitespace().next().unwrap_or("");
    if expected.len() != 64 || !expected.chars().all(|c| c.is_ascii_hexdigit()) {
        eprintln!("  ⚠ Checksum file for '{}' is malformed — installing unverified.", package);
        return Ok(());
    }

    verify_file_sha256(tarball, expected, &format!("package '{}'", package))
}
