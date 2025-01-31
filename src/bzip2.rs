use crate::clone_repo;
use crate::cmd;
use anyhow::Result;
use std::fs::{copy, remove_file};
use std::os::unix::fs::symlink;
use std::process::Command;
use walkdir::WalkDir;

pub const SOURCE_DIR: &'static str = "/phiban/sources/bzip2";
pub const SOURCE_URL: &'static str = "file:///git_sources/bzip2";
pub const SOURCE_TAG: &'static str = "bzip2-1.0.8-tarball+gtt";
pub const RESTORE_METADATA: bool = true;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    let makefiles: Vec<String> = WalkDir::new(SOURCE_DIR)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file()
                && e.file_name()
                    .to_str()
                    .unwrap_or_default()
                    .starts_with("Makefile")
        })
        .map(|e| e.path().display().to_string())
        .collect();

    //cmd!{"sed -i -e 's|CC=gcc|CC=clang|'"};
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
        unimplemented! {"We don't handle this failure yet!"}
    }

    cmd! {"make -f Makefile-libbz2_so"};
    cmd! {"make clean"};
    cmd! {"make -j64"};
    cmd! {"make PREFIX={}/usr install", sysroot};

    // NOTE: If we don't remove the static libbz2.a we will get linking errors
    //       and I have not investigated why this happens.
    remove_file(format! {"{sysroot}/usr/lib/libbz2.a"})?;
    remove_file(format! {"{sysroot}/usr/bin/bzcat"})?;
    remove_file(format! {"{sysroot}/usr/bin/bunzip2"})?;

    copy("libbz2.so.1.0.8", format! {"{sysroot}/lib/libbz2.so.1.0.8"})?;
    symlink(
        "libbz2.so.1.0.8",
        format! {"{sysroot}/usr/lib/libbz2.so.1.0"},
    )?;
    symlink("libbz2.so.1.0.8", format! {"{sysroot}/usr/lib/libbz2.so.1"})?;
    symlink("libbz2.so.1.0.8", format! {"{sysroot}/usr/lib/libbz2.so"})?;

    copy("bzip2-shared", format! {"{sysroot}/usr/bin/bzip2"})?;
    symlink("bzip2", format! {"{sysroot}/usr/bin/bzcat"})?;
    symlink("bzip2", format! {"{sysroot}/usr/bin/bunzip2"})?;

    Ok(())
}
