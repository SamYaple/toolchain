use std::env;
use std::process::Command;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install() -> Result<()> {
    clone_repo("/git_sources/musl", "v1.2.5")?;

    let source_dir = Path::new("/phiban/sources/musl");
    env::set_current_dir(source_dir)?;

    let status = Command::new("./configure")
        .arg("--prefix=/sysroots/phase1/usr")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"We don't handle this failure yet!"}
    }

    let status = Command::new("make")
        .arg("-j64")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"We don't handle this failure yet!"}
    }

    let status = Command::new("make")
        .arg("install")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"We don't handle this failure yet!"}
    }

    Ok(())
}
