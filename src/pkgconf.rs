use crate::clone_repo;
use crate::cmd;
use anyhow::Result;
use std::os::unix::fs::symlink;

pub const SOURCE_DIR: &'static str = "/phiban/sources/pkgconf";
pub const SOURCE_URL: &'static str = "file:///git_sources/pkgconf";
pub const SOURCE_TAG: &'static str = "pkgconf-2.3.0-tarball+gtt";
pub const RESTORE_METADATA: bool = true;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    cmd! {"./configure --prefix={0}/usr --build={1} --host={1} --disable-static", sysroot, crate::TRIPLE};
    cmd! {"make -j64"};
    cmd! {"make install"};
    symlink("pkgconf", format! {"{sysroot}/usr/bin/pkg-config"})?;

    Ok(())
}
