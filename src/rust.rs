use crate::clone_repo;
use crate::cmd;
use anyhow::Result;

pub const SOURCE_DIR: &'static str = "/phiban/sources/rust";
pub const SOURCE_URL: &'static str = "file:///git_sources/rust";
pub const SOURCE_TAG: &'static str = "1.84.0";
pub const RESTORE_METADATA: bool = false;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;
    // TODO: I dont like this and also its unneeded to clone `gcc` and
    // `llvm-project` submodules since they go unused...
    cmd! {"git apply /patches/rust/change-git-submodule-paths.patch"};
    cmd! {"git submodule update --init --recursive"};

    cmd! {"git apply /patches/rust/add-phiban-linux-musl-target.patch"};
    cmd! {"./configure
        --set=build.cargo=/sysroots/phase0/usr/bin/cargo
        --set=build.cargo-clippy=/sysroots/phase0/usr/bin/cargo-clippy
        --set=build.rustc=/sysroots/phase0/usr/bin/rustc
        --set=build.rustfmt=/sysroots/phase0/usr/bin/rustfmt
        --set=build.jobs=64
        --set=build.host={1}
        --set=build.build={1}
        --set=build.target={1}
        --set=install.prefix={0}/usr
        --set=install.sysconfdir=etc
        --set=llvm.use-libcxx=true
        --set=rust.llvm-libunwind=system
        --set=rust.musl-root={0}/usr
        --set=target.{1}.llvm-config={0}/usr/bin/llvm-config",
    sysroot,
    crate::TRIPLE};
    cmd! {"python3 ./x.py build -j64"};
    cmd! {"python3 ./x.py install"};

    Ok(())
}
