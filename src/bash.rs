use std::os::unix::fs::symlink;
use crate::clone_repo;
use crate::cmd;
use anyhow::Result;

pub const SOURCE_DIR: &'static str = "/phiban/sources/bash";
pub const SOURCE_URL: &'static str = "file:///git_sources/bash";
pub const SOURCE_TAG: &'static str = "bash-5.2.37-tarball+gtt";
pub const RESTORE_METADATA: bool = true;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    cmd! {"git apply /patches/bash/fix-missing-header.patch"};
    cmd! {"./configure --prefix={0}/usr --build={1} --host={1} --without-bash-malloc --with-installed-readline --enable-multibyte --disable-install-examples", sysroot, crate::TRIPLE};
    cmd! {"make -j64"};
    cmd! {"make install"};
    symlink("bash", &format!{"{sysroot}/usr/bin/sh"})?;

    Ok(())
}
