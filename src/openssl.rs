use crate::clone_repo;
use crate::cmd;
use anyhow::Result;
use std::env;

pub const SOURCE_DIR: &'static str = "/phiban/sources/openssl";
pub const SOURCE_URL: &'static str = "file:///git_sources/openssl";
pub const SOURCE_TAG: &'static str = "openssl-3.4.0-tarball+gtt";
pub const RESTORE_METADATA: bool = true;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    // TODO: do *not* set envvar outside of the spawned process
    unsafe {
        env::set_var("CC", "cc");
        env::set_var("CXX", "c++");
    }
    cmd! {"perl ./Configure --prefix={}/usr --libdir=lib shared zlib-dynamic", sysroot};
    unsafe {
        env::remove_var("CC");
        env::remove_var("CXX");
    }
    cmd! {"make -j64"};
    cmd! {"make install_sw"};
    Ok(())
}
