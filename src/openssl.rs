use std::env;
use std::process::Command;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install() -> Result<()> {
    let sysroot = "/sysroots/phase1/usr";
    clone_repo("/git_sources/openssl", "openssl-3.4.0-tarball")?;

    let source_dir = Path::new("/phiban/sources/openssl");
    env::set_current_dir(source_dir)?;

    let status = Command::new("perl")
        .env("CC", "cc")
        .env("CXX", "c++")
        .arg("./Configure")
        .arg(format!{"--prefix={sysroot}"})
        .arg(format!{"--openssldir={sysroot}/etc/ssl"})
        .arg("--libdir=lib")
        .arg("shared")
        .arg("zlib-dynamic")
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
        .arg("install_sw")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"INSTALL: We don't handle this failure yet!"}
    }

    Ok(())
}
