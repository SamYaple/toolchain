use std::env::{set_current_dir, set_var, remove_var};
use std::fs::remove_file;
use crate::cmd;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo("/git_sources/zlib", "v1.3.1-tarball")?;
    let source_dir = Path::new("/phiban/sources/zlib");
    set_current_dir(source_dir)?;

    // The configure script makes the assumption that a Linux (uname -s)
    // host will *always* have a zlib library. Since that is not the case
    // for our bootstrapping process, we override the LDSHARED var so the
    // test passes appropriately.
    // TODO: do *not* set envvar outside of the spawned process
    unsafe {
        set_var("LDSHARED", "cc -shared");
    }
    cmd!{"./configure --prefix={}/usr", sysroot};
    unsafe {
        remove_var("LDSHARED");
    }

    cmd!{"make -j64"};
    cmd!{"make install"};

    // NOTE: If we don't remove the static libz.a we will get linking errors
    //       and I have not investigated why this happens.
    remove_file(format!{"{sysroot}/lib/libz.a"})?;

    Ok(())
}
