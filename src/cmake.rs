use std::os::unix::fs::symlink;
use std::env;
use crate::cmd;
use std::path::Path;
use anyhow::Result;
use crate::{clone_repo, clone_repo2};

// NOTES:
//   dependencies:
//     musl
//     llvm
//     openssl
//     cmake | make

pub const SOURCE_DIR: &'static str = "/phiban/sources/cmake";
pub const SOURCE_URL: &'static str = "file:///git_sources/cmake";
pub const SOURCE_TAG: &'static str = "v3.31.5-tarball+gtt";
pub const RESTORE_METADATA: bool = true;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo2(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    cmd!{"cmake -B build -G Ninja -D CMAKE_INSTALL_PREFIX={0}/usr", sysroot};
    cmd!{"cmake --build build"};
    cmd!{"cmake --build build --target install"};

    Ok(())
}

/// `cmake` has a dependency on itself to build, but it does provide a way to
/// bootstrap with `make`. This fn exposes the bootstrap build path
pub fn bootstrap(sysroot: &str) -> Result<()> {
    clone_repo2(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    cmd!{"./bootstrap --parallel=64 -- -D CMAKE_INSTALL_PREFIX={}/usr", sysroot};
    cmd!{"make -j64"};
    cmd!{"make install"};

    Ok(())
}
