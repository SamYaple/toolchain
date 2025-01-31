use crate::clone_repo;
use crate::cmd;
use anyhow::Result;

// TODO: gdbm wants libiconv and libintl?

pub const SOURCE_DIR: &'static str = "/phiban/sources/gdbm";
pub const SOURCE_URL: &'static str = "file:///git_sources/gdbm";
pub const SOURCE_TAG: &'static str = "v1.24-tarball+gtt";
pub const RESTORE_METADATA: bool = true;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    // python _dbm module requires libgdbm-compat, the _gdbm module can be built
    // without the compat flag.
    cmd! {"./configure --prefix={0}/usr --build={1} --host={1} --enable-libgdbm-compat", sysroot, crate::TRIPLE};
    cmd! {"make -j64"};
    cmd! {"make install"};

    Ok(())
}
