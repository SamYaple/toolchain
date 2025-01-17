# glibc-stage1-only.cmake
#
# This file overrides some cmake options during the stage1 toolchain build. The
# overrides are *not* used during the stage2 compilation.
#
# See the option `CLANG_BOOTSTRAP_CMAKE_ARGS` in this file for how cmake is
# invoked during the stage2 build.

# We do not yet have a sysroot that is capable of building our toolchain. The
# provided sysroot only contains libc currently. Because of this, we cannot
# build the main build with a SYSROOT until stage2.
set(RUNTIMES_${GLIBC_TRIPLE}_CMAKE_SYSROOT ${GLIBC_SYSROOT} CACHE STRING "")
set(BUILTINS_${GLIBC_TRIPLE}_CMAKE_SYSROOT ${GLIBC_SYSROOT} CACHE STRING "")

# Force using all of the projects and runtimes from LLVM vs the host system
set(RUNTIMES_${GLIBC_TRIPLE}_LIBUNWIND_USE_COMPILER_RT     ON CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_LIBCXXABI_USE_COMPILER_RT     ON CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_LIBCXXABI_USE_LLVM_UNWINDER   ON CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_LIBCXX_USE_COMPILER_RT        ON CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_COMPILER_RT_USE_LLVM_UNWINDER ON CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_COMPILER_RT_USE_LIBCXX        ON CACHE BOOL "")

# Disable libraries which may be present on the host. We do not want to link
# against them
set(LLVM_ENABLE_LIBEDIT OFF CACHE BOOL "")
set(LLVM_ENABLE_LIBPFM  OFF CACHE BOOL "")
set(LLVM_ENABLE_LIBXML2 OFF CACHE BOOL "")
set(LLVM_ENABLE_ZLIB    OFF CACHE BOOL "")
set(LLVM_ENABLE_ZSTD    OFF CACHE BOOL "")

# The clang bootstrap process works in two stages. `stage1` builds clang and
# llvm, then uses those to build the builtins and runtimes. Only the runtimes
# are installed (compiler-rt, libc++, libunwind). `stage2` builds clang and
# llvm again using the newly installed runtimes, and the compiler from `stage1
set(CLANG_ENABLE_BOOTSTRAP ON CACHE BOOL "")
set(CLANG_BOOTSTRAP_CMAKE_ARGS
    -D GLIBC_TRIPLE=${GLIBC_TRIPLE}
    -D CMAKE_SYSROOT=${GLIBC_SYSROOT}
    -C /glibc-llvm.cmake
    CACHE STRING "")
