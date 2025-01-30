use std::os::unix::fs::symlink;
use std::env;
use crate::cmd;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

// TODO: gdbm wants libiconv and libintl?

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo("/git_sources/gdbm", "v1.24-tarball+gtt")?;
    let source_dir = Path::new("/phiban/sources/gdbm");
    env::set_current_dir(source_dir)?;
    cmd!{"gtt restore"};

    // python _dbm module requires libgdbm-compat, the _gdbm module can be built
    // without the compat flag.
    cmd!{"./configure --prefix={0}/usr --build={1} --host={1} --enable-libgdbm-compat", sysroot, crate::TRIPLE};
    cmd!{"make -j64"};
    cmd!{"make install"};

    Ok(())
}
