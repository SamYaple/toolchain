use std::env;
use std::fs::{soft_link, remove_file, copy};
use std::process::Command;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;
use walkdir::WalkDir;

pub fn build_and_install() -> Result<()> {
    let sysroot = "/sysroots/phase1/usr";
    clone_repo("/git_sources/bzip2", "bzip2-1.0.8-tarball")?;

    let source_dir = Path::new("/phiban/sources/bzip2");
    env::set_current_dir(source_dir)?;


    let makefiles: Vec<String> = WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file() && e.file_name().to_str().unwrap_or_default().starts_with("Makefile")
        })
        .map(|e| e.path().display().to_string())
        .collect();

    let mut binding = Command::new("sed");
    let mut cmd = binding
        .arg("-i")
        .arg("-e")
        .arg(r#"s|CC=gcc|CC=clang|"#)
        .arg("-e")
        .arg(r#"s|\(ln -s -f \)$(PREFIX)/bin/|\1|"#);
    for f in makefiles {
        cmd = cmd.arg(f);
    }

    let status = cmd.status()?;
    if !status.success() {
        dbg![status];
        unimplemented!{"We don't handle this failure yet!"}
    }

    let status = Command::new("make")
        .arg("-f")
        .arg("Makefile-libbz2_so")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"BUILD: We don't handle this failure yet!"}
    }

    let status = Command::new("make")
        .arg("clean")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"BUILD: We don't handle this failure yet!"}
    }

    let status = Command::new("make")
        .arg("-j64")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"BUILD: We don't handle this failure yet!"}
    }

    let status = Command::new("make")
        .arg(format!{"PREFIX={sysroot}"})
        .arg("install")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"INSTALL: We don't handle this failure yet!"}
    }

    // NOTE: If we don't remove the static libbz2.a we will get linking errors
    //       and I have not investigated why this happens.
    remove_file(format!{"{sysroot}/lib/libbz2.a"})?;
    remove_file(format!{"{sysroot}/bin/bzcat"})?;
    remove_file(format!{"{sysroot}/bin/bunzip2"})?;

    copy("libbz2.so.1.0.8", format!{"{sysroot}/lib/libbz2.so.1.0.8"})?;
    soft_link("libbz2.so.1.0.8", format!{"{sysroot}/lib/libbz2.so.1.0"})?;
    soft_link("libbz2.so.1.0.8", format!{"{sysroot}/lib/libbz2.so.1"})?;
    soft_link("libbz2.so.1.0.8", format!{"{sysroot}/lib/libbz2.so"})?;

    copy("bzip2-shared", format!{"{sysroot}/bin/bzip2"})?;
    soft_link("bzip2", format!{"{sysroot}/bin/bzcat"})?;
    soft_link("bzip2", format!{"{sysroot}/bin/bunzip2"})?;

    Ok(())
}
