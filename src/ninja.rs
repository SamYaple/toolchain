use crate::clone_repo;
use crate::cmd;
use anyhow::Result;

pub const SOURCE_DIR: &'static str = "/phiban/sources/ninja";
pub const SOURCE_URL: &'static str = "file:///git_sources/ninja";
pub const SOURCE_TAG: &'static str = "v1.12.1";
pub const RESTORE_METADATA: bool = false;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    cmd! {"cmake -B build -G Ninja -D CMAKE_INSTALL_PREFIX={0}/usr -D BUILD_TESTING=OFF", sysroot};
    cmd! {"cmake --build build"};
    cmd! {"cmake --build build --target install"};

    Ok(())
}

pub fn bootstrap(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    cmd! {"cmake -B build -D CMAKE_INSTALL_PREFIX={0}/usr -D BUILD_TESTING=OFF", sysroot};
    cmd! {"cmake --build build --parallel 64"};
    cmd! {"cmake --build build --target install"};

    Ok(())
}
