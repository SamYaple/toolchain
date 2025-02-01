use crate::clone_repo;
use crate::cmd;
use anyhow::Result;

pub const SOURCE_DIR: &'static str = "/phiban/sources/linux";
pub const SOURCE_URL: &'static str = "file:///git_sources/linux";
pub const SOURCE_TAG: &'static str = "v6.13";
pub const RESTORE_METADATA: bool = false;

pub fn build_and_install_headers(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    // TODO: Does this actually generate headers? or just copy them?
    cmd! {"make LLVM=1 -j64 headers"};

    // `find` and `cp` are how we install the linux kernel headers as the
    // initial files for our sysroot. AFAIK there is no make target that can
    // help us here.
    cmd! {"find usr/include -type f ! -name *.h -delete"};
    cmd! {"cp -rv usr/include {}/usr/include", sysroot};

    println!("Kernel headers install successful");
    Ok(())
}
