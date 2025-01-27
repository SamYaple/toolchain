use std::env;
use std::process::Command;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install() -> Result<()> {
    clone_repo("/git_sources/linux", "v6.13")?;

    let source_dir = Path::new("/phiban/sources/linux");
    env::set_current_dir(source_dir)?;

    let status = Command::new("make")
        .arg("LLVM=1")
        .arg("-j64")
        .arg("headers")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"We don't handle this failure yet!"}
    }

    // `find` and `cp` are how we install the linux kernel headers as the
    // initial files for our sysroot. AFAIK there is no make target that can
    // help us here.

    // TODO: Refactor into rust (maybe get fancy and reuse uutils/findutils
    let status = Command::new("find")
        .arg("usr/include")
        .arg("-type").arg("f")
        .arg("!")
        .arg("-name")
        .arg("*.h")
        .arg("-delete")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"We don't handle this failure yet!"}
    }

    // TODO: Refactor into rust
    let status = Command::new("cp")
        .arg("-rv")
        .arg("usr/include")
        .arg("/sysroots/phase1/usr/include")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"We don't handle this failure yet!"}
    }

    println!("Kernel headers install successful");
    Ok(())
}
