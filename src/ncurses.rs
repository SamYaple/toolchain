use std::env;
use std::fs::remove_file;
use crate::cmd;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo("/git_sources/ncurses", "v6.5-tarball")?;
    let source_dir = Path::new("/phiban/sources/ncurses");
    env::set_current_dir(source_dir)?;

    cmd!{"./configure --prefix={0}/usr --build={1} --host={1} --with-shared --without-debug --without-normal --with-cxx-shared --enable-pc-files --with-pkg-config-libdir={0}/usr/lib/pkgconfig", sysroot, crate::TRIPLE};
    cmd!{"make -j64"};
    cmd!{"make install"};

    Ok(())
}
