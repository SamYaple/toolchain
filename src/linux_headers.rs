use crate::cmd;
use std::env;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo("/git_sources/linux", "v6.13")?;

    let source_dir = Path::new("/phiban/sources/linux");
    env::set_current_dir(source_dir)?;

    // TODO: Does this actually generate headers? or just copy them?
    cmd!{"make LLVM=1 -j64 headers"};

    // `find` and `cp` are how we install the linux kernel headers as the
    // initial files for our sysroot. AFAIK there is no make target that can
    // help us here.
    cmd!{"find usr/local -type f ! -name *.h -delete"};
    cmd!{"cp -rv usr/include {}/usr/include", sysroot};

    println!("Kernel headers install successful");
    Ok(())
}
