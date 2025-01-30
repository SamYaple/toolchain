use std::os::unix::fs::symlink;
use std::env;
use crate::cmd;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo("/git_sources/pkgconf", "pkgconf-2.3.0-tarball")?;
    let source_dir = Path::new("/phiban/sources/pkgconf");
    env::set_current_dir(source_dir)?;
    cmd!{"git-warp-time"};

    cmd!{"./configure --prefix={0}/usr --build={1} --host={1} --disable-static", sysroot, crate::TRIPLE};
    cmd!{"make -j64"};
    cmd!{"make install"};
    symlink("pkgconf", format!{"{sysroot}/usr/bin/pkg-config"})?;

    Ok(())
}
