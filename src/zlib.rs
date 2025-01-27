use std::env;
use std::process::Command;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install() -> Result<()> {
    let sysroot = "/sysroots/phase1/usr";
    clone_repo("/git_sources/zlib", "v1.3.1-tarball")?;

    let source_dir = Path::new("/phiban/sources/zlib");
    env::set_current_dir(source_dir)?;

    let status = Command::new("./configure")
        // The configure script makes the assumption that a Linux (uname -s)
        // host will *always* have a zlib library. Since that is not the case
        // for our bootstrapping process, we override the LDSHARED var so the
        // test passes appropriately.
        .env("LDSHARED", "cc -shared")
        .arg(format!{"--prefix={sysroot}"})
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"CONFIGURE: We don't handle this failure yet!"}
    }

    let status = Command::new("make")
        .arg("-j64")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"BUILD: We don't handle this failure yet!"}
    }

    let status = Command::new("make")
        .arg("install")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"INSTALL: We don't handle this failure yet!"}
    }

    // NOTE: If we don't remove the static libz.a we will get linking errors
    //       and I have not investigated why this happens.
    std::fs::remove_file(format!{"{sysroot}/lib/libz.a"})?;

    Ok(())
}
