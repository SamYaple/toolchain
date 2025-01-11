# musl stage1
set(_GLIBC_SYSROOT /sysroots/glibc/llvm)
set(_MUSL_SYSROOT /sysroots/musl/llvm)

# our musl sysroot only contains libc so it cannot be used as a sysroot until
# we bootstrap clang stage2
set(CMAKE_INSTALL_PREFIX ${_MUSL_SYSROOT}/usr CACHE STRING "")

set(                    CMAKE_SYSROOT ${_GLIBC_SYSROOT} CACHE STRING "")
set(RUNTIMES_${_TARGET}_CMAKE_SYSROOT ${_MUSL_SYSROOT} CACHE STRING "")
set(BUILTINS_${_TARGET}_CMAKE_SYSROOT ${_MUSL_SYSROOT} CACHE STRING "")

set(CLANG_ENABLE_BOOTSTRAP ON CACHE BOOL "")
set(CLANG_BOOTSTRAP_CMAKE_ARGS -C /musl-stage2.cmake -C /musl-base.cmake -C /common.cmake CACHE STRING "")
