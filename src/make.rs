use std::env;
use std::process::Command;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install() -> Result<()> {
    clone_repo("/git_sources/make", "4.4-tarball")?;

    let source_dir = Path::new("/phiban/sources/make");
    env::set_current_dir(source_dir)?;

    let status = Command::new("./configure")
        .env("PATH", "/toolchain/bin")
        .arg("--prefix=/sysroots/phase1/usr")
        .arg("--build=x86_64-phiban-linux-musl")
        .arg("--host=x86_64-phiban-linux-musl")
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
