use crate::clone_repo;
use crate::cmd;
use anyhow::Result;
use std::env::{remove_var, set_var};
use std::fs::remove_file;

pub const SOURCE_DIR: &'static str = "/phiban/sources/zlib";
pub const SOURCE_URL: &'static str = "file:///git_sources/zlib";
pub const SOURCE_TAG: &'static str = "v1.3.1-tarball+gtt";
pub const RESTORE_METADATA: bool = true;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    // The configure script makes the assumption that a Linux (uname -s)
    // host will *always* have a zlib library. Since that is not the case
    // for our bootstrapping process, we override the LDSHARED var so the
    // test passes appropriately.
    // TODO: do *not* set envvar outside of the spawned process
    unsafe {
        set_var("LDSHARED", "cc -shared");
    }
    cmd! {"./configure --prefix={}/usr", sysroot};
    unsafe {
        remove_var("LDSHARED");
    }

    cmd! {"make -j64"};
    cmd! {"make install"};

    // NOTE: If we don't remove the static libz.a we will get linking errors
    //       and I have not investigated why this happens.
    remove_file(format! {"{sysroot}/lib/libz.a"})?;

    Ok(())
}
