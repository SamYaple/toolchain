use std::os::unix::fs::symlink;
use std::env;
use crate::cmd;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo("/git_sources/shadow", "4.17.2-tarball+gtt")?;
    let source_dir = Path::new("/phiban/sources/shadow");
    env::set_current_dir(source_dir)?;
    cmd!{"gtt restore"};

    cmd!{"./configure --prefix={0}/usr --build={1} --host={1} --disable-static --without-libbsd --without-group-name-max-length --without-tcb --with-bcrypt --with-yescrypt", sysroot, crate::TRIPLE};
    cmd!{"make -j64"};
    cmd!{"make install"};

    Ok(())
}
