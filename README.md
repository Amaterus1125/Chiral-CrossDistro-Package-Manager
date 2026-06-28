<div align="center">

# ⚗️ Chiral Package Manager

**A fast, dependency-aware, cross-distro package manager built in Rust**

*Born from a custom Linux distribution built entirely from scratch using LFS/BLFS*

[![Release](https://img.shields.io/github/v/release/Amaterus1125/Chiral-CrossDistro-Package-Manager?style=flat-square&color=orange)](https://github.com/Amaterus1125/Chiral-CrossDistro-Package-Manager/releases/latest)
[![Rust](https://img.shields.io/badge/built%20with-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Linux%20x86__64-lightgrey?style=flat-square&logo=linux)](https://kernel.org/)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)


</div>

---

## What is Chiral?

Chiral is a binary package manager that works on **any Linux system** — including custom distros, LFS/BLFS builds, Arch, Debian, and anything in between. It requires no package manager to install itself, no Rust runtime, and no dependencies of any kind. Just copy the binary and run it.

Instead of locking you into one distro's ecosystem, Chiral uses a **3-way fallback chain** to find packages:

```
1. Your own GitHub packages/ repo  →  fastest, custom packages
         ↓ not found?
2. Debian stable repos             →  huge selection, stable
         ↓ not found?
3. Arch Linux repos                →  bleeding edge, massive AUR coverage
```

If a package exists anywhere in that chain, Chiral finds it, resolves all its dependencies, and installs everything in the right order automatically.

---

## Features

| Feature | Description |
|---|---|
| 🔗 **Dependency resolution** | Full recursive BFS + topological sort — deps install before dependents, always |
| 🌐 **3-way fallback** | GitHub → Debian → Arch means near-universal package coverage |
| 🧠 **Smart system detection** | Checks pacman, dpkg, pkg-config, ldconfig, PATH, rustup, and filesystem before downloading anything already present |
| 📦 **Clean installs and removes** | Every installed file is tracked — `chiral remove` leaves nothing behind |
| 🔢 **Real version pinning** | Stores actual version strings from Debian/Arch APIs |
| 👤 **Root and user modes** | System-wide as root, or into `~/.local` as a regular user |
| 🔄 **Weekly auto-sync** | GitHub Actions keeps your package repo updated every Sunday automatically |
| ⬆️ **Self-updating** | `chiral self-update` downloads and replaces the binary from GitHub releases |
| 🦀 **100% static binary** | Built with musl — zero runtime dependencies, runs on any Linux |

---

## Quick Install

```bash
curl -L https://github.com/Amaterus1125/Chiral-CrossDistro-Package-Manager/releases/latest/download/chiral -o chiral
chmod +x chiral
sudo mv chiral /usr/local/bin/chiral
```

That's it. No Rust, no cargo, no package manager required. The binary is fully static.

---

## Usage

```
chiral install <package>     Install a package and all its dependencies
chiral remove  <package>     Remove an installed package cleanly
chiral update  <package>     Update a package to the latest version
chiral upgrade               Update all installed packages
chiral search  <query>       Search available packages
chiral list                  List installed packages with version and source
chiral info    <package>     Show version, source, deps, and installed files
chiral deps    <package>     Preview what would be installed — dry run
chiral self-update           Update chiral itself to the latest version
```

### Examples

```bash
# See what installing ffmpeg would pull in without installing anything
chiral deps ffmpeg

# Install nano — chiral resolves and installs all dependencies first
chiral install nano

# Check info on any package — even ones not installed by chiral
chiral info rust
# Output: "installed via rustup, not managed by chiral"

# Install steam — chiral finds it in Arch repos automatically
chiral install steam

# Remove a package — every file it installed gets cleaned up
chiral remove nano

# Update chiral itself when a new version is available
sudo chiral self-update
```

---

## How it Works

### 1. Dependency Resolution

When you run `chiral install gtk3`, Chiral:

1. Queries the Arch Linux API for gtk3's full dependency list
2. Walks the entire tree recursively (BFS) — deps of deps of deps
3. Checks each one against your system — skips anything already present
4. Builds a topological install order so deepest deps go first
5. Installs each missing dep through the fallback chain, then installs gtk3 last

```
gtk3
 ├── glib2       ← installed 1st
 ├── cairo
 │    └── pixman ← installed before cairo
 ├── pango
 └── gdk-pixbuf2
```

Circular dependencies are automatically detected and broken.

### 2. System Detection

Before downloading anything, Chiral checks if a dep is already present via:

| Check | Catches |
|---|---|
| `pacman -Q` / `dpkg -s` / `rpm -q` | System package manager installs |
| `command -v` | Any binary on PATH |
| `ldconfig -p` | Shared libraries |
| `pkg-config --exists` | Manually compiled libraries |
| Direct filesystem scan | `/usr/bin`, `/usr/lib`, `/usr/include`, `.pc` files |
| rustup | Rust toolchain installs |

This means even packages you compiled and installed manually from source are detected and skipped — chiral won't re-download something that's already there.

### 3. Fallback Chain

```
chiral install <pkg>
      │
      ▼
GitHub packages/<pkg>.tar.gz  ──found──▶  install
      │
   not found
      │
      ▼
Debian stable (<pkg> amd64)   ──found──▶  extract .deb → repack → install
      │
   not found
      │
      ▼
Arch Linux repos (<pkg>)      ──found──▶  extract .pkg.tar.zst → repack → install
      │
   not found
      │
      ▼
   ❌ Error: not found anywhere
```

### 4. File Tracking DB

Every installed file is recorded:

```
# Root installs:  /var/lib/chiral/installed.db
# User installs:  ~/.local/share/chiral/installed.db

[steam=3.2.0-1|arch]
/usr/local/bin/steam
/usr/local/lib/steam/...
...
```

`chiral remove` reads this list and deletes exactly those files — nothing more, nothing less.

### 5. Root vs User Mode

| | `sudo chiral` | `chiral` |
|---|---|---|
| Install prefix | `/usr/local` | `~/.local` |
| DB location | `/var/lib/chiral/` | `~/.local/share/chiral/` |
| Runs ldconfig | ✅ | ❌ |
| System-wide | ✅ | current user only |

---

## Building from Source

Requires Rust 1.70+:

```bash
git clone https://github.com/Amaterus1125/Chiral-CrossDistro-Package-Manager
cd Chiral-CrossDistro-Package-Manager

# Standard build
cargo build --release

# Fully static build (recommended — runs on any Linux)
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl

sudo cp target/x86_64-unknown-linux-musl/release/chiral /usr/local/bin/chiral
```

---

## Releasing a New Version

Just push a tag — GitHub Actions builds and publishes everything automatically:

```bash
git tag v3.2.0
git push origin v3.2.0
```

The workflow:
1. Reads the version from the tag
2. Updates `Cargo.toml` automatically
3. Builds a fully static musl binary
4. Strips debug symbols
5. Creates a GitHub release with the binary attached

Users then get the update via:
```bash
sudo chiral self-update
```

---

## Auto-Sync

The `packages/` folder is automatically kept up to date every Sunday at midnight UTC via GitHub Actions:

1. Checks Arch Linux repos for newer versions of every package
2. Downloads and repacks updated packages as `.tar.gz`
3. Falls back to Debian for packages not in Arch
4. Commits only if something actually changed

You can also trigger it manually from the Actions tab.

---

## Adding Your Own Packages

Drop a `.tar.gz` into `packages/` with a standard `usr/` layout:

```
mypackage.tar.gz
└── usr/
    ├── bin/mypackage
    ├── lib/libmypackage.so
    └── include/mypackage/
```

Chiral finds it automatically on the next `chiral install mypackage`. The auto-sync workflow will keep it updated from Arch/Debian if a matching package exists there.

---

## Supported Platforms

| Platform | Status | Dep detection method |
|---|---|---|
| Arch Linux | ✅ Full | pacman + ldconfig + PATH |
| Debian / Ubuntu | ✅ Full | dpkg + ldconfig + PATH |
| LFS / BLFS | ✅ Full | ldconfig + pkg-config + filesystem |
| Any Linux x86_64 | ✅ Full | PATH + ldconfig + filesystem |
| macOS / Windows | ❌ | Linux only |

---

## Origin

Chiral was built as the package manager for a custom Linux distribution assembled entirely from scratch using [Linux From Scratch (LFS)](https://www.linuxfromscratch.org/) and [Beyond LFS (BLFS)](https://www.linuxfromscratch.org/blfs/). Every package in the base system — GCC, glibc, systemd, XFCE — was compiled by hand. Chiral was created so that distro could install software without depending on any existing package manager.

The name comes from chirality in chemistry — molecules that are mirror images of each other. The two modes (Dextro and Levo) reflect this.

---

## License

MIT — do whatever you want with it.
