use crate::cmd;
use std::env;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo2;
use crate::TRIPLE;

pub const SOURCE_DIR: &'static str = "/phiban/sources/make";
pub const SOURCE_URL: &'static str = "file:///git_sources/make";
pub const SOURCE_TAG: &'static str = "4.4.1-tarball+gtt";
pub const RESTORE_METADATA: bool = true;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo2(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    cmd!{"./configure --prefix={0}/usr --build={1} --host={1}", sysroot, TRIPLE};
    cmd!{"make -j64"};
    cmd!{"make install"};

    Ok(())
}

pub fn bootstrap(sysroot: &str) -> Result<()> {
    clone_repo2(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    // the build.sh calls make during one of the steps :(
    cmd!{"git apply /patches/make/fix-bootstrap.patch"};
    cmd!{"./configure --prefix={0}/usr --build={1} --host={1}", sysroot, TRIPLE};
    cmd!{"./build.sh"};
    cmd!{"./make install"};

    Ok(())
}
