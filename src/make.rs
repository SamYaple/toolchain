use crate::cmd;
use std::env;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install() -> Result<()> {
    clone_repo("/git_sources/make", "4.4-tarball")?;

    let source_dir = Path::new("/phiban/sources/make");
    env::set_current_dir(source_dir)?;

    let sysroot = "/sysroots/phase1";
    let triple = "x86_64-phiban-linux-musl";
    cmd!{"./configure --prefix={0}/usr --build={1} --host={1}", sysroot, triple};
    cmd!{"make -j64"};
    cmd!{"make install"};
    Ok(())
}
