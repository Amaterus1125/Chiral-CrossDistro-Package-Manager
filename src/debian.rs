//! Debian stable backend — resolves a `.deb` download URL, verifies it
//! against the `Packages.gz` index, and repacks its data as a `.tar.gz` so
//! the rest of the pipeline can treat it like any other package tarball.

use flate2::read::GzDecoder;
use std::fs;
use std::io::Read;
use std::path::Path;

use crate::checksum::verify_file_sha256;
use crate::download::download;

pub fn debian_fetch_sha256(pool_path: &str) -> Result<String, String> {
    let idx_url = "https://deb.debian.org/debian/dists/stable/main/binary-amd64/Packages.gz";
    let mut resp = reqwest::blocking::get(idx_url)
        .map_err(|e| format!("Debian index fetch error: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("Debian index not reachable (HTTP {})", resp.status()));
    }

    let mut bytes = Vec::new();
    resp.copy_to(&mut bytes).map_err(|e| e.to_string())?;

    let mut text = String::new();
    GzDecoder::new(&bytes[..]).read_to_string(&mut text)
        .map_err(|e| format!("Cannot decompress Packages.gz: {}", e))?;

    for stanza in text.split("\n\n") {
        let is_match = stanza.lines().any(|l| {
            l.strip_prefix("Filename: ").map(|f| f.trim() == pool_path).unwrap_or(false)
        });
        if is_match {
            return stanza.lines()
                .find_map(|l| l.strip_prefix("SHA256: "))
                .map(|s| s.trim().to_string())
                .ok_or_else(|| "Package stanza has no SHA256 field".to_string());
        }
    }
    Err(format!("'{}' not found in Debian package index", pool_path))
}

fn debian_relative_path(url: &str) -> Option<String> {
    url.find("pool/").map(|i| url[i..].to_string())
}

fn debian_find_deb(package: &str) -> Result<(String, String, String), String> {
    let client = reqwest::blocking::Client::new();
    let page = client
        .get(&format!("https://packages.debian.org/stable/amd64/{}/download", package))
        .header("User-Agent", "chiral-package-manager")
        .send()
        .map_err(|e| format!("Debian search error: {}", e))?
        .text()
        .map_err(|e| e.to_string())?;

    for line in page.lines() {
        if line.contains("deb.debian.org") && line.contains(".deb") {
            if let Some(start) = line.find("href=\"") {
                let rest = &line[start + 6..];
                if let Some(end) = rest.find('"') {
                    let url = &rest[..end];
                    if url.ends_with(".deb") {
                        let filename = url.split('/').last().unwrap_or("");
                        let version = filename
                            .trim_end_matches("_amd64.deb")
                            .splitn(2, '_').nth(1)
                            .unwrap_or("unknown").to_string();
                        let pool_path = debian_relative_path(url).unwrap_or_default();
                        return Ok((url.to_string(), version, pool_path));
                    }
                }
            }
        }
    }
    Err(format!("Could not find '{}' in Debian stable", package))
}

fn extract_deb(deb_path: &Path, dest: &Path) -> Result<(), String> {
    let tmp_dir = deb_path.parent().unwrap_or(Path::new("/tmp")).join("deb_extracted");
    fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;

    let status = std::process::Command::new("ar")
        .args(["x", deb_path.to_str().unwrap()])
        .current_dir(&tmp_dir)
        .status()
        .map_err(|e| format!("ar not found: {}", e))?;

    if !status.success() { return Err("Failed to extract .deb with ar".to_string()); }

    let data_tar = fs::read_dir(&tmp_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok()).map(|e| e.path())
        .find(|p| p.file_name().and_then(|n| n.to_str())
            .map(|n| n.starts_with("data.tar")).unwrap_or(false))
        .ok_or("No data.tar.* found inside .deb")?;

    let stage = tmp_dir.join("stage");
    fs::create_dir_all(&stage).map_err(|e| e.to_string())?;

    let status = std::process::Command::new("tar")
        .args(["xf", data_tar.to_str().unwrap(), "-C", stage.to_str().unwrap()])
        .status().map_err(|e| format!("tar failed: {}", e))?;

    if !status.success() { return Err("Failed to extract data.tar from .deb".to_string()); }

    let status = std::process::Command::new("tar")
        .args(["czf", dest.to_str().unwrap(), "-C", stage.to_str().unwrap(), "."])
        .status().map_err(|e| format!("tar repack failed: {}", e))?;

    if !status.success() { return Err("Failed to repack .deb data as .tar.gz".to_string()); }

    let _ = fs::remove_dir_all(&tmp_dir);
    Ok(())
}

pub fn try_debian(package: &str, dest: &Path) -> Result<String, String> {
    let (deb_url, version, pool_path) = debian_find_deb(package)?;
    let deb_tmp = dest.parent().unwrap_or(Path::new("/tmp"))
        .join(format!("chiral-{}.deb", package));
    download(&deb_url, &deb_tmp)?;

    if pool_path.is_empty() {
        eprintln!("  ⚠ Could not determine pool path for '{}' — installing unverified.", package);
    } else {
        match debian_fetch_sha256(&pool_path) {
            Ok(expected) => verify_file_sha256(&deb_tmp, &expected, &format!("Debian package '{}'", package))?,
            Err(e) => eprintln!(
                "  ⚠ Could not verify Debian checksum for '{}' ({}) — installing unverified.", package, e
            ),
        }
    }

    extract_deb(&deb_tmp, dest)?;
    let _ = fs::remove_file(&deb_tmp);
    Ok(version)
}
