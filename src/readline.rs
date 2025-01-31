use crate::clone_repo;
use crate::cmd;
use anyhow::Result;

pub const SOURCE_DIR: &'static str = "/phiban/sources/readline";
pub const SOURCE_URL: &'static str = "file:///git_sources/readline";
pub const SOURCE_TAG: &'static str = "readline-8.2.13-tarball+gtt";
pub const RESTORE_METADATA: bool = true;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    cmd! {"chmod +x support/install.sh"};
    cmd! {"./configure --prefix={0}/usr --build={1} --host={1} --disable-static --with-curses", sysroot, crate::TRIPLE};
    cmd! {"make SHLIB_LIBS='-lncursesw' -j64"};
    cmd! {"make install"};

    Ok(())
}
