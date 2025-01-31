use crate::clone_repo;
use crate::cmd;
use anyhow::Result;
use std::fs::write;
use std::os::unix::fs::symlink;

pub const SOURCE_DIR: &'static str = "/phiban/sources/musl";
pub const SOURCE_URL: &'static str = "file:///git_sources/musl";
pub const SOURCE_TAG: &'static str = "v1.2.5";
pub const RESTORE_METADATA: bool = false;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    cmd! {"./configure --prefix={}/usr", sysroot};
    cmd! {"make -j64"};
    cmd! {"make install"};

    let ldd_path = &format! {"{sysroot}/usr/lib/ldd"};
    symlink("../lib/libc.so", ldd_path)?;

    let lib_path = &format! {"{sysroot}/usr/lib/ld-musl-x86_64.so.1"};
    symlink("libc.so", lib_path)?;

    // These are the search paths musl will use to find libraries. Currently, we
    // only use one directory, `/toolchain/usr/lib`, but this is where you would
    // want to add additional search paths if you have multiple paths to search.
    let library_search_paths = vec![
        // NOTE: we don't currently use seperate directories per target.
        //format!{"/toolchain/usr/lib/{}", crate::TRIPLE},
        "/toolchain/usr/lib",
    ];

    let musl_ld_path = &format! {"{sysroot}/etc/ld-musl-x86_64.path"};
    let musl_ld_content = library_search_paths.join("\n");
    write(musl_ld_path, musl_ld_content)?;
    Ok(())
}
