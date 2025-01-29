use std::env;
use std::fs::remove_file;
use crate::cmd;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo("/git_sources/rust", "1.84.0")?;

    let source_dir = Path::new("/phiban/sources/rust");
    env::set_current_dir(source_dir)?;
    cmd!{"git apply /patches/rust/change-git-submodule-paths.patch"};
    cmd!{"git submodule update --init --recursive"};

    cmd!{"git apply /patches/rust/add-phiban-linux-musl-target.patch"};
    cmd!{"./configure
        --set=build.bootstrap-cache-path=/rustbootstrapdeps
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
        //--set=build.cargo=/toolchain/usr/bin/cargo
        //--set=build.cargo-clippy=/toolchain/usr/bin/cargo-clippy
        //--set=build.rustc=/toolchain/usr/bin/rustc
        //--set=build.rustfmt=/toolchain/usr/bin/rustfmt
    cmd!{"python3 ./x.py build -j64"};
    cmd!{"python3 ./x.py install"};

    Ok(())
}
