//! Arch Linux repo backend — package lookup, checksum verification via the
//! sync db, and .pkg.tar.zst extraction.

use flate2::read::GzDecoder;
use std::fs;
use std::io::Read;
use std::path::Path;
use tar::Archive;

use crate::checksum::verify_file_sha256;
use crate::constants::ARCH_MIRROR;
use crate::deps::strip_ver;
use crate::download::download;

pub struct ArchPkg {
    pub repo:    String,
    pub pkgname: String,
    pub version: String,
    pub arch:    String,
    pub depends: Vec<String>,
}

pub fn arch_query(package: &str) -> Result<ArchPkg, String> {
    let api = format!("https://archlinux.org/packages/search/json/?name={}", package);

    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(&api)
        .header("User-Agent", "chiral-package-manager")
        .send()
        .map_err(|e| format!("Arch API error: {}", e))?
        .text()
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = serde_json::from_str(&resp)
        .map_err(|e| format!("Arch API parse error: {}", e))?;

    let results = json["results"].as_array().ok_or("No results from Arch API")?;
    if results.is_empty() {
        return Err(format!("'{}' not found in Arch repos", package));
    }

    let pkg = results.iter()
        .find(|r| { let repo = r["repo"].as_str().unwrap_or(""); repo == "core" || repo == "extra" })
        .or_else(|| results.first())
        .ok_or("No suitable Arch package found")?;

    let repo    = pkg["repo"].as_str().unwrap_or("extra").to_string();
    let arch_str= pkg["arch"].as_str().unwrap_or("x86_64").to_string();
    let pkgname = pkg["pkgname"].as_str().unwrap_or(package).to_string();
    let pkgver  = pkg["pkgver"].as_str().unwrap_or("").to_string();
    let pkgrel  = pkg["pkgrel"].as_str().unwrap_or("1").to_string();
    let version = format!("{}-{}", pkgver, pkgrel);

    let depends: Vec<String> = pkg["depends"]
        .as_array().unwrap_or(&vec![])
        .iter()
        .filter_map(|d| d.as_str())
        .map(|d| strip_ver(d))
        .filter(|d| !d.is_empty())
        .collect();

    Ok(ArchPkg { repo, pkgname, version, arch: arch_str, depends })
}

pub fn arch_download_url(pkg: &ArchPkg) -> String {
    let filename = format!("{}-{}-{}.pkg.tar.zst", pkg.pkgname, pkg.version, pkg.arch);
    format!("{}/{}/os/x86_64/{}", ARCH_MIRROR, pkg.repo, filename)
}

pub fn arch_fetch_sha256(pkg: &ArchPkg) -> Result<String, String> {
    let db_url = format!("{}/{}/os/x86_64/{}.db", ARCH_MIRROR, pkg.repo, pkg.repo);
    let mut resp = reqwest::blocking::get(&db_url)
        .map_err(|e| format!("Arch db fetch error: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("Arch db not reachable (HTTP {})", resp.status()));
    }

    let mut bytes = Vec::new();
    resp.copy_to(&mut bytes).map_err(|e| e.to_string())?;

    let entry_name = format!("{}-{}/desc", pkg.pkgname, pkg.version);
    let mut archive = Archive::new(GzDecoder::new(&bytes[..]));

    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?
            .to_string_lossy().trim_end_matches('/').to_string();
        if path != entry_name { continue; }

        let mut content = String::new();
        entry.read_to_string(&mut content).map_err(|e| e.to_string())?;

        let mut lines = content.lines();
        while let Some(line) = lines.next() {
            if line.trim() == "%SHA256SUM%" {
                if let Some(hash) = lines.next() {
                    return Ok(hash.trim().to_string());
                }
            }
        }
        return Err(format!("No %SHA256SUM% field for {} in {} db", pkg.pkgname, pkg.repo));
    }

    Err(format!("'{}' not found in {} sync db (version mismatch?)", pkg.pkgname, pkg.repo))
}

fn extract_pkg_zst(pkg_path: &Path, dest: &Path) -> Result<(), String> {
    let tmp_dir = pkg_path.parent().unwrap_or(Path::new("/tmp")).join("arch_extracted");
    fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;

    let status = std::process::Command::new("tar")
        .args([
            "xf", pkg_path.to_str().unwrap(),
            "-C", tmp_dir.to_str().unwrap(),
            "--exclude=.PKGINFO", "--exclude=.BUILDINFO",
            "--exclude=.MTREE",  "--exclude=.INSTALL",
        ])
        .status()
        .map_err(|e| format!("tar failed: {}", e))?;

    if !status.success() {
        return Err("Failed to extract .pkg.tar.zst — is zstd installed?".to_string());
    }

    let status = std::process::Command::new("tar")
        .args(["czf", dest.to_str().unwrap(), "-C", tmp_dir.to_str().unwrap(), "."])
        .status()
        .map_err(|e| format!("tar repack failed: {}", e))?;

    if !status.success() {
        return Err("Failed to repack Arch package as .tar.gz".to_string());
    }

    let _ = fs::remove_dir_all(&tmp_dir);
    Ok(())
}

pub fn try_arch(package: &str, dest: &Path) -> Result<(String, Vec<String>), String> {
    let pkg     = arch_query(package)?;
    let version = pkg.version.clone();
    let deps    = pkg.depends.clone();
    let url     = arch_download_url(&pkg);

    let pkg_tmp = dest.parent().unwrap_or(Path::new("/tmp"))
        .join(format!("chiral-{}.pkg.tar.zst", package));

    download(&url, &pkg_tmp)?;

    match arch_fetch_sha256(&pkg) {
        Ok(expected) => verify_file_sha256(&pkg_tmp, &expected, &format!("Arch package '{}'", package))?,
        Err(e) => eprintln!(
            "  ⚠ Could not verify Arch checksum for '{}' ({}) — installing unverified.", package, e
        ),
    }

    extract_pkg_zst(&pkg_tmp, dest)?;
    let _ = fs::remove_file(&pkg_tmp);
    Ok((version, deps))
}
