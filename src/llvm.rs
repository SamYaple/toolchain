use crate::clone_repo;
use crate::cmd;
use anyhow::Result;
use std::fs::write;

pub const SOURCE_DIR: &'static str = "/phiban/sources/llvm-project";
pub const SOURCE_URL: &'static str = "file:///git_sources/llvm-project";
// To see the different commits between rust llvm and the original llvm, run:
//
//   $ git --no-pager log llvmorg-19.1.7..rustc-1.84.0 --no-merges --pretty=oneline
//   1268e87bdbaed0693a9d782ccd5a21e2cab2de33 (HEAD, tag: rustc-1.84.0) Revert "[ELF] Fix unnecessary inclusion of unreferenced provide symbols"
//   c36e7699b1fd14e59400e06d894e2ca2c17ec23a Revert "[ELF] PROVIDE: fix spurious "symbol not found""
//   29e82b2592450c43f7d0db2a18a7793c55c4957e [rust] Changes needed for 'x86_64-fortanix-unknown-sgx' nightly target.
//pub const SOURCE_TAG: &'static str = "llvmorg-19.1.7";
pub const SOURCE_TAG: &'static str = "rustc-1.84.0";
pub const RESTORE_METADATA: bool = false;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    write(&format! {"{SOURCE_DIR}/llvm.cmake"}, LLVM_CMAKE)?;

    cmd! {"git apply /patches/llvm-project/disable-sanitizer-tests.patch"};
    cmd! {"git apply /patches/llvm-project/toolchain-prefix.patch"};
    cmd! {"cmake -S llvm -B build -G Ninja
        -D CMAKE_C_COMPILER_LAUNCHER=sccache
        -D CMAKE_CXX_COMPILER_LAUNCHER=sccache
        -D BUILD_SYSROOT={0}
        -D BUILD_TRIPLE={1}
        -D TARGET_SYSROOT={0}
        -D TARGET_TRIPLE={1}
        -C {2}/llvm.cmake", sysroot, crate::TRIPLE, SOURCE_DIR};
    cmd! {"cmake --build build --target stage2-distribution"};
    cmd! {"cmake --build build --target stage2-install-distribution"};

    Ok(())
}

pub fn build_and_install_runtimes(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    write(&format! {"{SOURCE_DIR}/llvm.cmake"}, LLVM_CMAKE)?;

    cmd! {"git apply /patches/llvm-project/disable-sanitizer-tests.patch"};
    cmd! {"git apply /patches/llvm-project/toolchain-prefix.patch"};
    cmd! {"cmake -S llvm -B build -G Ninja
        -D CMAKE_C_COMPILER_LAUNCHER=sccache
        -D CMAKE_CXX_COMPILER_LAUNCHER=sccache
        -D BUILD_SYSROOT=/toolchain
        -D BUILD_TRIPLE={1}
        -D TARGET_SYSROOT={0}
        -D TARGET_TRIPLE={1}
        -C {2}/llvm.cmake", sysroot, crate::TRIPLE, SOURCE_DIR};
    cmd! {"cmake --build build --target runtimes"};
    cmd! {"cmake --build build --target install-runtimes"};

    Ok(())
}

const LLVM_CMAKE: &'static str = r#"
# Added vars to control this build. Pass these to cmake to control the sysroots
# and expected triples. When bootstrapping, it is best to use the STATIC_CORE
# option added in this script to isolate the bootstrapped toolchain and help
# avoid some linking issues. In this cmake file, it will detect different
# triples and automatically set STATIC_CORE.
#
# Example cmake opts if youre trying to reuse this file. After cloning repo:
#   cd /path/to/sources/llvm-project
#   cmake -S llvm -B build -G Ninja \
#     -DBUILD_SYSROOT=/sysroots/glibc \
#     -DBUILD_TRIPLE=x86_64-unknown-linux-gnu \
#     -DTARGET_SYSROOT=/sysroots/musl \
#     -DTARGET_TRIPLE=x86_64-unknown-linux-musl \
#     -C /phiban/llvm.cmake
#        ^^^^^^^ 
#        CLANG_BOOTSTRAP_CMAKE_ARGS below needs this path update if changed

# Exit as early as possible if proper vars are not set.
if ("${BUILD_TRIPLE}" STREQUAL "" OR "${TARGET_TRIPLE}" STREQUAL "")
    message(FATAL_ERROR "BUILD_TRIPLE and TARGET_TRIPLE must be set")
endif()

if (NOT ("${BUILD_SYSROOT}" STREQUAL ""))
    # BUILD_SYSROOT is set; pass it through to CMAKE_SYSROOT
    set(CMAKE_SYSROOT ${BUILD_SYSROOT} CACHE STRING "")
endif()

if ("${TARGET_SYSROOT}" STREQUAL "")
    set(CMAKE_INSTALL_PREFIX /usr CACHE STRING "")
else()
    # TARGET_SYSROOT is set; make sure we 
    set(CMAKE_INSTALL_PREFIX ${TARGET_SYSROOT}/usr CACHE STRING "")

    if (NOT ("${BUILD_SYSROOT}" STREQUAL "${TARGET_SYSROOT}"))
        # BUILD_SYSROOT and TARGET_SYSROOT are different, make sure the
        # runtimes and builtins are setup to use the TARGET_SYSROOT.
        set(RUNTIMES_${TARGET_TRIPLE}_CMAKE_SYSROOT ${TARGET_SYSROOT} CACHE STRING "")
        set(BUILTINS_${TARGET_TRIPLE}_CMAKE_SYSROOT ${TARGET_SYSROOT} CACHE STRING "")
    endif()
endif()

# This accounts for the situation where the BUILD_TRIPLE and TARGET_TRIPLE are
# different, like when moving from glibc to musl libc.
set(LLVM_HOST_TRIPLE     ${BUILD_TRIPLE}  CACHE STRING "")
set(LLVM_BUILTIN_TARGETS ${TARGET_TRIPLE} CACHE STRING "")
set(LLVM_RUNTIME_TARGETS ${TARGET_TRIPLE} CACHE STRING "")
if ("${BUILD_TRIPLE}" STREQUAL "${TARGET_TRIPLE}")
    set(RUNTIMES_${TARGET_TRIPLE}_LLVM_HOST_TRIPLE ${TARGET_TRIPLE} CACHE STRING "")
    set(BUILTINS_${TARGET_TRIPLE}_LLVM_HOST_TRIPLE ${TARGET_TRIPLE} CACHE STRING "")
else()
    set(LLVM_DEFAULT_TARGET_TRIPLE ${TARGET_TRIPLE} CACHE STRING "")
    message(STATUS "BUILD_TRIPLE and TARGET_TRIPLE are different, forcing STATIC_CORE")
    set(STATIC_CORE ON CACHE BOOL "")
endif()

# This is hard to autodetect; matching the user defined triple should be enough
if ("${TARGET_TRIPLE}" MATCHES ".*-musl$")
    set(RUNTIMES_${TARGET_TRIPLE}_LIBCXX_HAS_MUSL_LIBC ON CACHE BOOL "")
endif()

# Statically link as much as possible to detach from the host toolchain. This
# step is not strictly neeed, but it has proven effective at isolating the
# stage1 compiler from the stage2. We get hard failures instead of mis-linked
# toolchains.
set(STATIC_CORE OFF CACHE BOOL "disable shared libs and statically link")
if (STATIC_CORE)
    message(STATUS "STATIC_CORE is True; statically linking as much as possible!")
    set(LLVM_LINK_LLVM_DYLIB        OFF CACHE BOOL "")
    set(LLVM_STATIC_LINK_CXX_STDLIB ON  CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_ENABLE_STATIC_UNWINDER ON CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_STATIC_CXX_LIBRARY     ON CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_LIBCXXABI_ENABLE_STATIC_UNWINDER   ON CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_LIBCXX_ENABLE_STATIC_ABI_LIBRARY   ON CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_SANITIZER_USE_STATIC_CXX_ABI       ON CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_SANITIZER_USE_STATIC_LLVM_UNWINDER ON CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_LIBCXX_STATICALLY_LINK_ABI_IN_STATIC_LIBRARY ON CACHE BOOL "")

    set(LLVM_USE_RELATIVE_PATHS_IN_FILES ON  CACHE BOOL "")
    set(LLVM_ENABLE_ZLIB OFF CACHE BOOL "")
endif()

# Recommended option for build performance (why is it not default?)
# TODO: Maybe reuse the tablegen between stages?
set(LLVM_OPTIMIZED_TABLEGEN ON CACHE BOOL "")

# gotta go fast
set(CMAKE_BUILD_TYPE "Release"           CACHE STRING "")
#set(CMAKE_CXX_FLAGS  "-O3 -march=native" CACHE STRING "")
#set(CMAKE_C_FLAGS    "-O3 -march=native" CACHE STRING "")
set(LLVM_PARALLEL_LINK_JOBS "16"         CACHE STRING "")

# LTO
#set(LLVM_BUILD_LTO "Thin" CACHE STRING "")
# PGO
#set(LLVM_BUILD_INSTRUMENTED "IR"            CACHE STRING "")
#set(LLVM_PROFILE_DATA_DIR   "/llvmprofdata" CACHE STRING "")
#set(PGO_INSTRUMENT_LTO      "Thin"          CACHE STRING "")
#set(PGO_BUILD_CONFIGURATION "/phiban/sources/llvm-project/llvm.cmake" CACHE STRING "")
# Bolt
#set(CLANG_BOLT "INSTRUMENT" CACHE STRING "")
#set(CMAKE_EXE_LINKER_FLAGS "-Wl,--emit-relocs,-znow" CACHE STRING "")

# Set compiler defaults
set(CLANG_DEFAULT_CXX_STDLIB "libc++"      CACHE STRING "")
set(CLANG_DEFAULT_LINKER     "lld"         CACHE STRING "")
set(CLANG_DEFAULT_RTLIB      "compiler-rt" CACHE STRING "")
set(CLANG_DEFAULT_UNWINDLIB  "libunwind"   CACHE STRING "")

# Setup symlinks `cc`, `c++`, `ar`, `nm` to match binutils and historical defaults
set(LLVM_INSTALL_BINUTILS_SYMLINKS ON CACHE BOOL "")
set(LLVM_INSTALL_CCTOOLS_SYMLINKS  ON CACHE BOOL "")
set(LLVM_USE_SYMLINKS              ON CACHE BOOL "")
set(LLVM_INSTALL_UTILS             ON CACHE BOOL "")

# We build Rust with system LLVM and rustc will link against this.
set(LLVM_LINK_LLVM_DYLIB      ON  CACHE BOOL "")
set(LLVM_ENABLE_ZLIB          ON  CACHE BOOL "")
set(LLVM_UNREACHABLE_OPTIMIZE OFF CACHE BOOL "")

# Configure all of our builtins and runtimes link to each other ♥
set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_USE_BUILTINS_LIBRARY ON CACHE BOOL "")
set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_USE_LIBCXX           ON CACHE BOOL "")
set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_USE_LLVM_UNWINDER    ON CACHE BOOL "")
set(RUNTIMES_${TARGET_TRIPLE}_LIBCXXABI_USE_COMPILER_RT        ON CACHE BOOL "")
set(RUNTIMES_${TARGET_TRIPLE}_LIBCXXABI_USE_LLVM_UNWINDER      ON CACHE BOOL "")
set(RUNTIMES_${TARGET_TRIPLE}_LIBCXX_USE_COMPILER_RT           ON CACHE BOOL "")
set(RUNTIMES_${TARGET_TRIPLE}_LIBUNWIND_USE_COMPILER_RT        ON CACHE BOOL "")

# # Need to disable these for musl build (TODO: document the musl sanitizer limitations)
# set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_BUILD_GWP_ASAN OFF CACHE BOOL "")
# set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_BUILD_MEMPROF  OFF CACHE BOOL "")
# set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_BUILD_ORC      OFF CACHE BOOL "")

# Disable what is called "multiarch" support. One sysroot, one target.
set(COMPILER_RT_DEFAULT_TARGET_ONLY    ON  CACHE BOOL "")
set(LLVM_ENABLE_PER_TARGET_RUNTIME_DIR OFF CACHE BOOL "")

# # Disable benchmarks; TODO revisit but it was causing build failures
# set(LLVM_BUILD_BENCHMARKS   OFF CACHE BOOL "")
# set(LLVM_INCLUDE_BENCHMARKS OFF CACHE BOOL "")
# set(RUNTIMES_${TARGET_TRIPLE}_LIBCXX_INCLUDE_BENCHMARKS OFF CACHE BOOL "")

set(LLVM_ENABLE_RUNTIMES
    libunwind
    libcxxabi
    libcxx
    compiler-rt
    CACHE STRING "")

set(LLVM_ENABLE_PROJECTS
    clang
    lld
    bolt
    CACHE STRING "")

set(LLVM_TARGETS_TO_BUILD
    X86
    CACHE STRING "")

## # https://releases.llvm.org/19.1.0/docs/CommandGuide/
## set(LLVM_TOOLCHAIN_TOOLS
##     # binutils alternatives
##     #llvm-addr2line
##     llvm-ar
##     #llvm-cxxfilt
##     llvm-nm
##     #llvm-objcopy
##     #llvm-objdump
##     llvm-ranlib
##     #llvm-readelf
##     #llvm-size
##     #llvm-strings
##     llvm-strip
## 
##     # symlink targets
##     #addr2line
##     ar
##     #c++filt
##     nm
##     #objcopy
##     #objdump
##     ranlib
##     #readelf
##     #size
##     #strings
##     strip
## 
##     # build additional tools
##     llvm-config # rust *needs* this
##     #llvm-cov
##     #llvm-dlltool
##     #llvm-dwp
##     #llvm-lib
##     llvm-lto
##     llvm-mca # rust *needs* this
##     #llvm-ml
##     #llvm-pdbutil
##     #llvm-profdata
##     #llvm-rc
##     #llvm-readobj
##     #llvm-symbolizer
##     CACHE STRING "")
## 
## set(LLVM_TOOLCHAIN_UTILITIES
##   FileCheck # rust *needs* this
##   #obj2yaml
##   #yaml2obj
##   #not
##   #count
##   CACHE STRING "")
## 
## set(LLVM_DISTRIBUTION_COMPONENTS
##   builtins
##   runtimes
## 
##   clang-resource-headers
##   clang-libraries
## 
##   llvm-headers
##   llvm-libraries
## 
##   ${LLVM_TOOLCHAIN_TOOLS}
##   ${LLVM_TOOLCHAIN_UTILITIES}
##   ${LLVM_ENABLE_PROJECTS}
##   CACHE STRING "")

set(COMPILER_RT_SANITIZERS_TO_BUILD
    # These compilers are not going to work on musl due to the need for glibc
    # specific non-C standard functions that musl wont implement because it is
    # out of scope for the musl project.
    # This list is my best effort to sort these for use with musl and ensure the
    # LLVM tests all pass properly.

    # requires glibc interceptors (to my knowledge as of llvmorg-19.1.7)
    #asan
    #asan_abi
    #dfsan
    #gwp_asan
    #hwasan # mainly for aarch64?
    
    # TODO: continue investigating these
    #rtsan
    #nsan
    #rtsan
    #safestack

    # These all work on musl! Probably!
    cfi
    msan
    tsan
    ubsan_minimal
    #scudo_standalone # NOTE: This works, but I don't have a use for it currently
    CACHE STRING "")
set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_SANITIZERS_TO_BUILD
    # These compilers are not going to work on musl due to the need for glibc
    # specific non-C standard functions that musl wont implement because it is
    # out of scope for the musl project.
    # This list is my best effort to sort these for use with musl and ensure the
    # LLVM tests all pass properly.

    # requires glibc interceptors (to my knowledge as of llvmorg-19.1.7)
    #asan
    #asan_abi
    #dfsan
    #gwp_asan
    #hwasan # mainly for aarch64?
    
    # TODO: continue investigating these
    #rtsan
    #nsan
    #rtsan
    #safestack

    # These all work on musl! Probably!
    cfi
    msan
    tsan
    ubsan_minimal
    #scudo_standalone # NOTE: This works, but I don't have a use for it currently
    CACHE STRING "")
set(BUILTINS_${TARGET_TRIPLE}_COMPILER_RT_SANITIZERS_TO_BUILD
    # These compilers are not going to work on musl due to the need for glibc
    # specific non-C standard functions that musl wont implement because it is
    # out of scope for the musl project.
    # This list is my best effort to sort these for use with musl and ensure the
    # LLVM tests all pass properly.

    # requires glibc interceptors (to my knowledge as of llvmorg-19.1.7)
    #asan
    #asan_abi
    #dfsan
    #gwp_asan
    #hwasan # mainly for aarch64?
    
    # TODO: continue investigating these
    #rtsan
    #nsan
    #rtsan
    #safestack

    # These all work on musl! Probably!
    cfi
    msan
    tsan
    ubsan_minimal
    #scudo_standalone # NOTE: This works, but I don't have a use for it currently
    CACHE STRING "")

# The clang bootstrap process works in two stages. `stage1` builds clang and
# llvm, then uses those to build the builtins and runtimes. Only the runtimes
# are installed (compiler-rt, libc++, libunwind). `stage2` builds clang and
# llvm again using the newly installed runtimes, and the compiler from `stage1`
set(CLANG_BOOTSTRAP_TARGETS "distribution;install-distribution" CACHE STRING "")
set(CLANG_ENABLE_BOOTSTRAP ON CACHE BOOL "")
set(CLANG_BOOTSTRAP_CMAKE_ARGS
    -D CLANG_ENABLE_BOOTSTRAP=OFF
    -D STATIC_CORE=${STATIC_CORE}
    -D BUILD_SYSROOT=${TARGET_SYSROOT}
    -D BUILD_TRIPLE=${TARGET_TRIPLE}
    -D TARGET_SYSROOT=${TARGET_SYSROOT}
    -D TARGET_TRIPLE=${TARGET_TRIPLE}
    -C /phiban/sources/llvm-project/llvm.cmake
    CACHE STRING "")
"#;
