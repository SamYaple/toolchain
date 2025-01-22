# LLVM build for `phiban`

# Exit as early as possible if proper vars are not set.
if ("${TARGET_SYSROOT}" STREQUAL "" OR 
    "${TARGET_TRIPLE}"  STREQUAL "" OR
    "${BUILD_TRIPLE}"   STREQUAL "")
    message(FATAL_ERROR "BUILD_TRIPLE, TARGET_TRIPLE, and TARGET_SYSROOT must be set")
endif()

# Setup install directory for final stage builds
set(CMAKE_INSTALL_PREFIX ${TARGET_SYSROOT}/usr CACHE STRING "")

# Setup CMAKE_SYSROOT for all stages, runtimes, and builtins.
if ("${BUILD_SYSROOT}" STREQUAL "")
    message(STATUS "BUILD_SYSROOT is empty or unset; using host for `stage1` build.")
    set(RUNTIMES_${TARGET_TRIPLE}_CMAKE_SYSROOT ${TARGET_SYSROOT} CACHE STRING "")
    set(BUILTINS_${TARGET_TRIPLE}_CMAKE_SYSROOT ${TARGET_SYSROOT} CACHE STRING "")
else()
    set(CMAKE_SYSROOT ${BUILD_SYSROOT} CACHE STRING "")
endif()

# This accounts for the situation where the BUILD_TRIPLE and TARGET_TRIPLE are
# different, like when moving from glibc to musl libc.
set(LLVM_HOST_TRIPLE     ${BUILD_TRIPLE}  CACHE STRING "")
set(LLVM_BUILTIN_TARGETS ${TARGET_TRIPLE} CACHE STRING "")
set(LLVM_RUNTIME_TARGETS ${TARGET_TRIPLE} CACHE STRING "")
if (NOT ("${BUILD_TRIPLE}" STREQUAL "${TARGET_TRIPLE}"))
    set(LLVM_DEFAULT_TARGET_TRIPLE ${TARGET_TRIPLE} CACHE STRING "")
endif()

set(STATIC_CORE OFF CACHE BOOL "disable shared libs and statically link")
if (STATIC_CORE)
    # Statically link as much as possible to detach from the host toolchain.
    # This step is not strictly neeed, but it has proven effective at isolating
    # the stage1 compiler from the stage2. We get hard failures instead of
    # mis-linked toolchains.
    set(LLVM_LINK_LLVM_DYLIB        OFF CACHE BOOL "")
    set(LLVM_STATIC_LINK_CXX_STDLIB ON  CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_ENABLE_STATIC_UNWINDER ON CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_STATIC_CXX_LIBRARY     ON CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_LIBCXXABI_ENABLE_STATIC_UNWINDER   ON CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_LIBCXX_ENABLE_STATIC_ABI_LIBRARY   ON CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_SANITIZER_USE_STATIC_CXX_ABI       ON CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_SANITIZER_USE_STATIC_LLVM_UNWINDER ON CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_LIBCXX_STATICALLY_LINK_ABI_IN_STATIC_LIBRARY ON CACHE BOOL "")

    # Even less neccesary, we distable the shared libraries entirely. This has
    # the added value that the first time we install a shared lib, it will have
    # been built with the proper, up-to-date compiler.
    set(RUNTIMES_${TARGET_TRIPLE}_LIBUNWIND_ENABLE_SHARED   OFF CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_LIBCXXABI_ENABLE_SHARED   OFF CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_LIBCXX_ENABLE_SHARED      OFF CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_LIBCXXABI_INSTALL_LIBRARY OFF CACHE BOOL "")
    set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_SCUDO_STANDALONE_BUILD_SHARED   OFF CACHE BOOL "")
endif()

# gotta go fast
set(CMAKE_BUILD_TYPE "Release"           CACHE STRING "")
set(CMAKE_CXX_FLAGS  "-O3 -march=native" CACHE STRING "")
set(CMAKE_C_FLAGS    "-O3 -march=native" CACHE STRING "")

# Recommended option for build performance (why is it not default?)
set(LLVM_OPTIMIZED_TABLEGEN ON CACHE BOOL "")

# Set compiler defaults
set(CLANG_DEFAULT_CXX_STDLIB "libc++"      CACHE STRING "")
set(CLANG_DEFAULT_LINKER     "lld"         CACHE STRING "")
set(CLANG_DEFAULT_RTLIB      "compiler-rt" CACHE STRING "")
set(CLANG_DEFAULT_UNWINDLIB  "libunwind"   CACHE STRING "")

# LTO
set(LLVM_BUILD_LTO "Thin" CACHE BOOL "")
# PGO
#set(LLVM_BUILD_INSTRUMENTED ON CACHE BOOL "")
# Bolt
#set(CLANG_BOLT ON CACHE BOOL "")

# Using this library for all the shared code between tools can greatly reduce
# the size of the final toolchain.
set(LLVM_LINK_LLVM_DYLIB ON CACHE BOOL "")
set(LLVM_INSTALL_UTILS   ON CACHE BOOL "")
set(LLVM_USE_RELATIVE_PATHS_IN_FILES ON CACHE BOOL "")

# Setup symlinks `cc`, `c++`, `ar`, `nm` to match binutils and historical defaults
set(LLVM_INSTALL_BINUTILS_SYMLINKS ON CACHE BOOL "")
set(LLVM_INSTALL_CCTOOLS_SYMLINKS  ON CACHE BOOL "")
set(LLVM_USE_SYMLINKS              ON CACHE BOOL "")

# Configure all of our builtins and runtimes link to each other ♥
set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_USE_BUILTINS_LIBRARY ON CACHE BOOL "")
set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_USE_LIBCXX           ON CACHE BOOL "")
set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_USE_LLVM_UNWINDER    ON CACHE BOOL "")
set(RUNTIMES_${TARGET_TRIPLE}_LIBCXXABI_USE_COMPILER_RT        ON CACHE BOOL "")
set(RUNTIMES_${TARGET_TRIPLE}_LIBCXXABI_USE_LLVM_UNWINDER      ON CACHE BOOL "")
set(RUNTIMES_${TARGET_TRIPLE}_LIBCXX_USE_COMPILER_RT           ON CACHE BOOL "")
set(RUNTIMES_${TARGET_TRIPLE}_LIBUNWIND_USE_COMPILER_RT        ON CACHE BOOL "")

# Need to disable these for musl build (TODO: document the musl sanitizer limitations)
set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_BUILD_GWP_ASAN OFF CACHE BOOL "")
set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_BUILD_MEMPROF  OFF CACHE BOOL "")
set(RUNTIMES_${TARGET_TRIPLE}_COMPILER_RT_BUILD_ORC      OFF CACHE BOOL "")

# Disable what is called "multiarch" support. One sysroot, one target.
set(COMPILER_RT_DEFAULT_TARGET_ONLY    ON  CACHE BOOL "")
set(LLVM_ENABLE_PER_TARGET_RUNTIME_DIR OFF CACHE BOOL "")
set(LLVM_USE_RELATIVE_PATHS_IN_FILES   ON  CACHE BOOL "")

# Disable benchmarks; TODO revisit but it was causing build failures
set(LLVM_BUILD_BENCHMARKS   OFF CACHE BOOL "")
set(LLVM_INCLUDE_BENCHMARKS OFF CACHE BOOL "")
set(RUNTIMES_${TARGET_TRIPLE}_LIBCXX_INCLUDE_BENCHMARKS OFF CACHE BOOL "")

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
    -C /llvm.cmake
    CACHE STRING "")

set(LLVM_ENABLE_RUNTIMES
    libunwind
    libcxxabi
    libcxx
    compiler-rt
    CACHE STRING "")

set(LLVM_ENABLE_PROJECTS
    clang
    lld
    CACHE STRING "")

set(LLVM_TARGETS_TO_BUILD
    X86
    CACHE STRING "")

# https://releases.llvm.org/19.1.0/docs/CommandGuide/
set(LLVM_TOOLCHAIN_TOOLS
    # binutils alternatives
    #llvm-addr2line
    llvm-ar
    #llvm-cxxfilt
    llvm-nm
    #llvm-objcopy
    #llvm-objdump
    llvm-ranlib
    #llvm-readelf
    #llvm-size
    #llvm-strings
    llvm-strip

    # symlink targets
    #addr2line
    ar
    #c++filt
    nm
    #objcopy
    #objdump
    ranlib
    #readelf
    #size
    #strings
    strip

    # build additional tools
    llvm-config # rust *needs* this
    #llvm-cov
    #llvm-dlltool
    #llvm-dwp
    #llvm-lib
    llvm-lto
    llvm-mca # rust *needs* this
    #llvm-ml
    #llvm-pdbutil
    #llvm-profdata
    #llvm-rc
    #llvm-readobj
    #llvm-symbolizer
    CACHE STRING "")

set(LLVM_TOOLCHAIN_UTILITIES
  FileCheck # rust *needs* this
  #obj2yaml
  #yaml2obj
  #not
  #count
  CACHE STRING "")

set(LLVM_DISTRIBUTION_COMPONENTS
  builtins
  runtimes

  clang-resource-headers
  llvm-headers # rust *needs* this

  ${LLVM_TOOLCHAIN_TOOLS}
  ${LLVM_TOOLCHAIN_UTILITIES}
  ${LLVM_ENABLE_PROJECTS}
  CACHE STRING "")
