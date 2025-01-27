use crate::cmd;
use std::env;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install() -> Result<()> {
    clone_repo("/git_sources/musl", "v1.2.5")?;

    let source_dir = Path::new("/phiban/sources/musl");
    env::set_current_dir(source_dir)?;

    let sysroot = "/sysroots/phase1";
    cmd!{"./configure --prefix={}/usr", sysroot};
    cmd!{"make -j64"};
    cmd!{"make install"};

    Ok(())
}
