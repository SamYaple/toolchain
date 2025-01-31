use crate::clone_repo;
use crate::cmd;
use anyhow::Result;
use std::env::{remove_var, set_var};

pub const SOURCE_DIR: &'static str = "/phiban/sources/sqlite";
pub const SOURCE_URL: &'static str = "file:///git_sources/sqlite";
pub const SOURCE_TAG: &'static str = "sqlite-autoconf-3480000-tarball+gtt";
pub const RESTORE_METADATA: bool = true;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    unsafe {
        set_var("CPPFLAGS", "-D SQLITE_ENABLE_COLUMN_METADATA=1 -D SQLITE_ENABLE_UNLOCK_NOTIFY=1 -D SQLITE_ENABLE_DBSTAT_VTAB=1 -D SQLITE_SECURE_DELETE=1");
    }
    cmd! {"./configure --prefix={0}/usr --build={1} --host={1} --disable-static --enable-fts4 --enable-fts5", sysroot, crate::TRIPLE};
    unsafe {
        remove_var("CPPFLAGS");
    }
    cmd! {"make -j64"};
    cmd! {"make install"};

    Ok(())
}
