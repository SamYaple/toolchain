use std::env::{set_current_dir, set_var, remove_var};
use std::fs::remove_file;
use crate::cmd;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    //clone_repo("/git_sources/readline", "readline-8.2.13-tarball")?;
    //let source_dir = Path::new("/phiban/sources/readline");
    let source_dir = Path::new("/git_sources/readline");
    set_current_dir(source_dir)?;

    cmd!{"chmod +x support/install.sh"};
    cmd!{"./configure --prefix={0}/usr --build={1} --host={1} --disable-static --with-curses", sysroot, crate::TRIPLE};
    cmd!{"make SHLIB_LIBS='-lncursesw' -j64"};
    cmd!{"make install"};

    Ok(())
}
