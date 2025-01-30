use std::env;
use std::fs::remove_file;
use crate::cmd;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo("/git_sources/cpython", "v3.13.1")?;
    let source_dir = Path::new("/phiban/sources/cpython");
    env::set_current_dir(source_dir)?;

    cmd!{"git apply /patches/python/curses_include_headers.patch"};
    // --with-lto=thin
    // --enable-optimizations
    // --enable-bolt
    cmd!{"./configure --prefix={0}/usr --build={1} --host={1} --enable-shared --without-ensurepip", sysroot, crate::TRIPLE};
    cmd!{"make -j64"};
    cmd!{"make install"};

    Ok(())
}
