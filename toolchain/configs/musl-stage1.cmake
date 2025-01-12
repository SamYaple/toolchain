#
set(CMAKE_INSTALL_PREFIX ${MUSL_SYSROOT}/usr CACHE STRING "")

# Our host is technically glibc based. The sysroot only contains musl libc, but
# we cannot use musl for our HOST_TRIPLE until stage2 when we have installed
# the runtimes and builtins.
set(LLVM_HOST_TRIPLE ${GLIBC_TRIPLE} CACHE STRING "")
set(LLVM_DEFAULT_TARGET_TRIPLE ${MUSL_TRIPLE} CACHE STRING "")

# The musl sysroot only contains libc. To properly build clang and llvm we need
# to have our stage1 build from the glibc sysroot. The builtins/runtimes are
# installed into the musl sysroot for the stage2 build. During the stage2 build
# the glibc sysroot is no longer used.
set(                        CMAKE_SYSROOT ${GLIBC_SYSROOT} CACHE STRING "")
set(RUNTIMES_${MUSL_TRIPLE}_CMAKE_SYSROOT ${MUSL_SYSROOT} CACHE STRING "")
set(BUILTINS_${MUSL_TRIPLE}_CMAKE_SYSROOT ${MUSL_SYSROOT} CACHE STRING "")

# The clang bootstrap process works in two stages. `stage1` builds clang and
# llvm, then uses those to build the builtins and runtimes. Only the runtimes
# are installed (compiler-rt, libc++, libunwind). `stage2` builds clang and
# llvm again using the newly installed runtimes, and the compiler from `stage1`
set(CLANG_ENABLE_BOOTSTRAP ON CACHE BOOL "")
set(CLANG_BOOTSTRAP_CMAKE_ARGS
    -D MUSL_SYSROOT=${MUSL_SYSROOT}
    -D MUSL_TRIPLE=${MUSL_TRIPLE}
    -C /musl-stage2.cmake
    -C /musl-base.cmake
    -C /common.cmake
    CACHE STRING "")
