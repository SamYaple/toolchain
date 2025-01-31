use crate::clone_repo;
use crate::cmd;
use anyhow::Result;

pub const SOURCE_DIR: &'static str = "/phiban/sources/cpython";
pub const SOURCE_URL: &'static str = "file:///git_sources/cpython";
pub const SOURCE_TAG: &'static str = "v3.13.1";
pub const RESTORE_METADATA: bool = false;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    cmd! {"git apply /patches/python/curses_include_headers.patch"};
    // --with-lto=thin
    // --enable-optimizations
    // --enable-bolt
    cmd! {"./configure --prefix={0}/usr --build={1} --host={1} --enable-shared --without-ensurepip", sysroot, crate::TRIPLE};
    cmd! {"make -j64"};
    cmd! {"make install"};

    Ok(())
}
