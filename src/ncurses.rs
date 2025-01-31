use crate::clone_repo;
use crate::cmd;
use anyhow::Result;

pub const SOURCE_DIR: &'static str = "/phiban/sources/ncurses";
pub const SOURCE_URL: &'static str = "file:///git_sources/ncurses";
pub const SOURCE_TAG: &'static str = "v6.5-tarball+gtt";
pub const RESTORE_METADATA: bool = true;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    cmd! {"./configure --prefix={0}/usr --build={1} --host={1} --with-shared --without-debug --without-normal --with-cxx-shared --enable-pc-files --with-pkg-config-libdir={0}/usr/lib/pkgconfig", sysroot, crate::TRIPLE};
    cmd! {"make -j64"};
    cmd! {"make install"};

    Ok(())
}
