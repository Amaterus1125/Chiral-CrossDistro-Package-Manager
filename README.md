<div align="center">

# 🧪 Chiral Package Manager

**A fast, minimal binary package manager for Linux — built in Rust.**  
Designed for custom Linux distros. Inspired by pacman. No source compilation needed.

![Version](https://img.shields.io/badge/version-2.0-orange)
![Platform](https://img.shields.io/badge/platform-x86__64%20Linux-blue)
![License](https://img.shields.io/badge/license-MIT-green)

</div>

---

## 📦 What is Chiral?

Chiral is a binary package manager — you install pre-compiled programs with a single command, just like `pacman -S` on Arch Linux. No compiling, no build tools, no dependencies to chase. Packages are hosted on GitHub and downloaded directly.

It handles:
- Single binaries (`hello`, `bat`, `ripgrep`)
- Complex packages with libs, headers, man pages (`alsa-lib`, `mesa`, `gtk`)
- Symlinks for shared libraries (`libfoo.so → libfoo.so.1.2.3`)
- Automatic `ldconfig` after install/remove (when run as root)

---

## ⚡ Install Chiral

```bash
curl -sSL https://raw.githubusercontent.com/Amaterus1125/chpm/main/install.sh | bash
```

Then add to PATH (only needed once):
```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

For packages with shared libraries, also add:
```bash
echo 'export LD_LIBRARY_PATH="$HOME/.local/lib:$LD_LIBRARY_PATH"' >> ~/.bashrc
source ~/.bashrc
```

---

## 🚀 Usage

```bash
chiral install <package>    # Install a package
chiral remove  <package>    # Remove a package and all its files
chiral update  <package>    # Update a package to latest version
chiral upgrade              # Update every installed package
chiral search  <query>      # Search available packages
chiral list                 # List all installed packages
```

### Examples
```bash
chiral install hello
chiral install alsa-lib
chiral search audio
chiral list
chiral remove hello
chiral upgrade
```

---

## 📁 Install Locations

| Running as | Binaries | Libraries | Headers | DB |
|---|---|---|---|---|
| normal user | `~/.local/bin` | `~/.local/lib` | `~/.local/include` | `~/.local/share/chiral/` |
| root (sudo) | `/usr/local/bin` | `/usr/local/lib` | `/usr/local/include` | `/var/lib/chiral/` |

---

## 🗂 Repository Structure

```
chpm/
├── src/
│   ├── lib.rs          # Core package manager logic
│   ├── main.rs         # CLI entry point
│   └── ui.rs           # Terminal UI / progress bar
├── packages/           # All .tar.gz package files live here
│   ├── hello.tar.gz
│   ├── alsa-lib.tar.gz
│   └── ...
├── Cargo.toml          # Rust dependencies
├── Cargo.lock
├── install.sh          # One-liner installer script
└── README.md
```

---

## 📦 How to Add a Package

Packages are pre-compiled binaries packed into `.tar.gz` files.  
The tarball must follow this directory structure:

```
usr/
├── bin/        → executables
├── lib/        → shared libraries (.so files)
├── include/    → header files (.h files)
└── share/
    ├── man/    → man pages
    └── doc/    → documentation
```

### Step-by-step: packaging from your BLFS system

```bash
# 1. Compile the package normally on your BLFS system
./configure --prefix=/usr
make

# 2. Install into a staging directory (not live system)
make DESTDIR=/tmp/staging install

# 3. Check what was installed
find /tmp/staging -type f

# 4. Repack cleanly for chiral
cd /tmp/staging
tar -czf alsa-lib.tar.gz usr/

# 5. Upload alsa-lib.tar.gz to the packages/ folder in this repo
# 6. Anyone can now run:
chiral install alsa-lib
```

### Simple single-binary package
```bash
# For a single binary like 'hello'
mkdir -p /tmp/staging/usr/bin
cp hello /tmp/staging/usr/bin/
cd /tmp/staging
tar -czf hello.tar.gz usr/
```

---

## 🔧 Build from Source

Requires Rust. Install it with:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then build chiral:
```bash
git clone https://github.com/Amaterus1125/chpm
cd chpm
cargo build --release

# Install system-wide
sudo cp target/release/chiral /usr/local/bin/chiral

# Or user-only
cp target/release/chiral ~/.local/bin/chiral
```

---

## 🖥 Using on Your Own LFS Distro

Copy the chiral binary into your LFS system during build:
```bash
# During LFS chroot or post-install
cp chiral-x86_64-linux /usr/local/bin/chiral
chmod +x /usr/local/bin/chiral
```

After booting your distro, users can immediately run:
```bash
chiral install <anything-in-your-packages-folder>
```

---

## 🗃 How the Package Database Works

Chiral tracks every single file each package installs.  
Example entry in `installed.db`:
```
[alsa-lib=latest]
/usr/local/lib/libasound.so.2.0.0
/usr/local/lib/libasound.so.2
/usr/local/lib/libasound.so
/usr/local/include/alsa/asoundlib.h
/usr/local/share/man/man3/snd_pcm_open.3
```

When you run `chiral remove alsa-lib`, every one of those files gets deleted cleanly — no leftover files ever.

---

## 📋 Roadmap

- [ ] Package versioning (replace `latest` with real versions)
- [ ] Dependency resolution
- [ ] Package signing / verification
- [ ] Multiple repository support
- [ ] `chiral info <package>` command

---

## 🤝 Contributing a Package

1. Fork this repo
2. Compile the package on an LFS/BLFS system
3. Create the tarball following the structure above
4. Add it to `packages/`
5. Open a pull request

---

<div align="center">
Built with ❤️ for custom Linux distros
</div>
