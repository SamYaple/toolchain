# pass1 overrides
set(_GLIBC_SYSROOT /sysroots/glibc/llvm)
set(_MUSL_SYSROOT /sysroots/musl/llvm)

# our musl sysroot only contains libc so it cannot be used as a sysroot until
# we bootstrap clang stage2
set(CMAKE_INSTALL_PREFIX ${_MUSL_SYSROOT}/toolchain/usr CACHE STRING "")

set(CMAKE_SYSROOT ${_GLIBC_SYSROOT} CACHE STRING "")
set(RUNTIMES_x86_64-unknown-linux-musl_CMAKE_SYSROOT ${_MUSL_SYSROOT} CACHE STRING "")
set(BUILTINS_x86_64-unknown-linux-musl_CMAKE_SYSROOT ${_MUSL_SYSROOT} CACHE STRING "")

set(CLANG_ENABLE_BOOTSTRAP ON CACHE BOOL "")
set(CLANG_BOOTSTRAP_CMAKE_ARGS -D CMAKE_SYSROOT=${_MUSL_SYSROOT} -C /configs/llvm-musl-base.cmake CACHE STRING "")
