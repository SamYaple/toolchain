use std::os::unix::fs::symlink;
use std::env;
use crate::cmd;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    //clone_repo("/git_sources/mpdecimal", "4.0.0-tarball")?;
    //let source_dir = Path::new("/phiban/sources/mpdecimal");
    let source_dir = Path::new("/git_sources/mpdecimal");
    env::set_current_dir(source_dir)?;

    cmd!{"./configure --prefix={0}/usr --build={1} --host={1}", sysroot, crate::TRIPLE};
    cmd!{"make -j64"};
    cmd!{"make install"};

    Ok(())
}
