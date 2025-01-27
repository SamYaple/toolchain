use crate::cmd;
use std::env;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;
use crate::TRIPLE;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo("/git_sources/make", "4.4-tarball")?;

    let source_dir = Path::new("/phiban/sources/make");
    env::set_current_dir(source_dir)?;

    cmd!{"./configure --prefix={0}/usr --build={1} --host={1}", sysroot, TRIPLE};
    cmd!{"make -j64"};
    cmd!{"make install"};
    Ok(())
}
