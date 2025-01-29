use std::env::{set_current_dir, set_var, remove_var};
use std::fs::remove_file;
use crate::cmd;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    //clone_repo("/git_sources/sqlite", "version-3.48.0-tarball")?;
    //let source_dir = Path::new("/phiban/sources/sqlite");
    let source_dir = Path::new("/git_sources/sqlite");
    set_current_dir(source_dir)?;

    unsafe {
        set_var("CPPFLAGS", "-D SQLITE_ENABLE_COLUMN_METADATA=1 -D SQLITE_ENABLE_UNLOCK_NOTIFY=1 -D SQLITE_ENABLE_DBSTAT_VTAB=1 -D SQLITE_SECURE_DELETE=1");
    }
    cmd!{"./configure --prefix={0}/usr --build={1} --host={1} --disable-static --enable-fts4 --enable-fts5", sysroot, crate::TRIPLE};
    unsafe {
        remove_var("CPPFLAGS");
    }
    cmd!{"make -j64"};
    cmd!{"make install"};

    Ok(())
}
