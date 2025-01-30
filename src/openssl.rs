use crate::cmd;
use std::env;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo("/git_sources/openssl", "openssl-3.4.0-tarball")?;
    let source_dir = Path::new("/phiban/sources/openssl");
    env::set_current_dir(source_dir)?;

    // TODO: do *not* set envvar outside of the spawned process
    unsafe {
        env::set_var("CC", "cc");
        env::set_var("CXX", "c++");
    }
    cmd!{"perl ./Configure --prefix={}/usr --libdir=lib shared zlib-dynamic", sysroot};
    unsafe {
        env::remove_var("CC");
        env::remove_var("CXX");
    }
    cmd!{"make -j64"};
    cmd!{"make install_sw"};
    Ok(())
}
