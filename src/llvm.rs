use std::env;
use std::fs::remove_file;
use crate::cmd;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo("/git_sources/llvm-project", "llvmorg-19.1.7")?;

    let source_dir = Path::new("/phiban/sources/llvm-project");
    env::set_current_dir(source_dir)?;

    cmd!{"git apply /patches/llvm-project/toolchain-prefix.patch"};
    cmd!{"cmake -S llvm -B build -G Ninja
        -D CMAKE_C_COMPILER_LAUNCHER=sccache
        -D CMAKE_CXX_COMPILER_LAUNCHER=sccache
        -D BUILD_SYSROOT=/toolchain
        -D BUILD_TRIPLE={1}
        -D TARGET_SYSROOT={0}
        -D TARGET_TRIPLE={1}
        -C /configs/llvm.cmake", sysroot, crate::TRIPLE};
    cmd!{"cmake --build build --target install-runtimes"};
    cmd!{"cmake --build build --target stage2-install-distribution"};

    Ok(())
}
