use crate::clone_repo;
use crate::cmd;
use anyhow::Result;

pub const SOURCE_DIR: &'static str = "/phiban/sources/xz";
pub const SOURCE_URL: &'static str = "file:///git_sources/xz";
pub const SOURCE_TAG: &'static str = "v5.6.4-tarball+gtt";
pub const RESTORE_METADATA: bool = true;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    cmd! {"./configure --prefix={0}/usr --build={1} --host={1} --disable-static", sysroot, crate::TRIPLE};
    cmd! {"make -j64"};
    cmd! {"make install"};

    Ok(())
}
