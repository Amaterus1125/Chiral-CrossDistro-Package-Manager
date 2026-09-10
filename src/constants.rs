//! Shared constants used across the crate.

pub const SERVER: &str       = "https://raw.githubusercontent.com/Amaterus1125/chpm/main/packages";
pub const ARCH_MIRROR: &str  = "https://mirror.rackspace.com/archlinux";
pub const RELEASES_API: &str = "https://api.github.com/repos/Amaterus1125/Chiral-CrossDistro-Package-Manager/releases/latest";

/// Packages that are fundamental to every Linux system and must NEVER be
/// downloaded from Arch/Debian — they conflict with LFS filesystem layou
pub const NEVER_INSTALL: &[&str] = &[
    "filesystem", "linux-api-headers", "iana-etc", "tzdata",
    "glibc", "sh", "bash", "coreutils", "util-linux", "systemd",
    "gcc-libs", "libgcc", "glibc-locales", "shadow", "pam",
    "linux", "linux-firmware", "grub", "efibootmgr",
];
