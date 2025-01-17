# LLVM build options for glibc base with compiler-rt|libunwind|libc++
# TODO cmake check for GLIBC_SYSROOT and GLIBC_TRIPLE

set(CMAKE_INSTALL_PREFIX ${GLIBC_SYSROOT}/usr CACHE STRING "")

# We set several variables at the start of this file prefixed with `_`. These
# are not able to be configured by the user since they are not specified as
# `CACHE`. They do not conflict with any other options by name. This is a style
# and readability choice on my part that helps keep consistency across the
# BUILTINS and RUNTIMES ontop of the main build.

###VARS###
set(_CLANG_BOOTSTRAP_TARGETS
  distribution
  install-distribution
)

set(_LLVM_ENABLE_PROJECTS
  clang
  lld
)

set(_LLVM_ENABLE_RUNTIMES
  libunwind
  libcxxabi
  libcxx
  compiler-rt
)

set(_LLVM_TOOLCHAIN_TOOLS
  #dsymutil
  llvm-ar
  llvm-nm
  #llvm-objcopy
  #llvm-objdump
  llvm-ranlib
  #llvm-readelf
  #llvm-size
  #llvm-strings
  #llvm-strip
)

set(_LLVM_DISTRIBUTION_COMPONENTS
  clang-resource-headers
  builtins
  runtimes
  ${_LLVM_ENABLE_PROJECTS}
  ${_LLVM_TOOLCHAIN_TOOLS}
)
###VARS###

# Set defaults for the compiler to use when invoked by the user.
#  LLVM       |  GNU
# libc++      | libstdc++
# lld         | ld (from binutils)
# compiler-rt | libgcc_s (glibc)
# libunwind   | libgcc_s (glibc;provides unwinder)
set(CLANG_DEFAULT_CXX_STDLIB "libc++"      CACHE STRING "")
set(CLANG_DEFAULT_LINKER     "lld"         CACHE STRING "")
set(CLANG_DEFAULT_RTLIB      "compiler-rt" CACHE STRING "")
set(CLANG_DEFAULT_UNWINDLIB  "libunwind"   CACHE STRING "")

# Recommended option for build performance (why is it not default?)
set(LLVM_OPTIMIZED_TABLEGEN ON CACHE BOOL "")

# Need to disable these for musl build (TODO: revisit this)
set(LLVM_INCLUDE_TESTS           OFF CACHE BOOL "")
set(CLANG_ENABLE_ARCMT           OFF CACHE BOOL "")
set(CLANG_ENABLE_STATIC_ANALYZER OFF CACHE BOOL "")

# Ensure each target is isolated. Rather than being dumped into `/lib`, we will
# install it into `/lib/x86_64-unknown-linux-musl`, for example.
set(COMPILER_RT_DEFAULT_TARGET_ONLY    OFF CACHE BOOL "")
set(LLVM_ENABLE_PER_TARGET_RUNTIME_DIR ON  CACHE BOOL "")
set(LLVM_USE_RELATIVE_PATHS_IN_FILES   ON  CACHE BOOL "")

# Expose `distribution` and `install-distribution` targets. These will be
# available as `stage2-distribution` and `stage2-install-distribution` targets
set(CLANG_BOOTSTRAP_TARGETS ${_CLANG_BOOTSTRAP_TARGETS} CACHE STRING "")

# Minimize the set of tools and utilities created to a minimal set. This saves
# space and cpu cycles, but is missing some of the cooler tools like bolt and
# even lto currently.
set(LLVM_TARGETS_TO_BUILD "X86" CACHE STRING "")
set(LLVM_TOOLCHAIN_TOOLS ${_LLVM_TOOLCHAIN_TOOLS} CACHE STRING "")
set(LLVM_ENABLE_PROJECTS ${_LLVM_ENABLE_PROJECTS} CACHE STRING "")
set(LLVM_ENABLE_RUNTIMES ${_LLVM_ENABLE_RUNTIMES} CACHE STRING "")
set(LLVM_DISTRIBUTION_COMPONENTS ${_LLVM_DISTRIBUTION_COMPONENTS} CACHE STRING "")

# TODO this seems like a bug that I would need to redeclare this. It might just
#      have been due to the llvm-libgcc cmake checks, but I no longer use cmake
#      for lllvm-libgcc so this might be able to be dropped?
set(RUNTIMES_${GLIBC_TRIPLE}_LLVM_ENABLE_RUNTIMES ${_LLVM_ENABLE_RUNTIMES} CACHE STRING "")

# Disabling sanitizers
set(RUNTIMES_${GLIBC_TRIPLE}_COMPILER_RT_BUILD_GWP_ASAN   OFF CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_COMPILER_RT_BUILD_LIBFUZZER  OFF CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_COMPILER_RT_BUILD_MEMPROF    OFF CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_COMPILER_RT_BUILD_ORC        OFF CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_COMPILER_RT_BUILD_SANITIZERS OFF CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_COMPILER_RT_BUILD_XRAY       OFF CACHE BOOL "")

# TODO: Make sure all builds inherit the proper CMAKE_BUILD_TYPE. This is might
# not be needed but I need to check if CMAKE_BUILD_TYPE gets passed through to
# the BUILTINS, RUNTIMES, and also during the stage2 build before I remove the
# others.
set(_CMAKE_BUILD_TYPE "Release")
set(                         CMAKE_BUILD_TYPE ${_CMAKE_BUILD_TYPE} CACHE STRING "")
set(BUILTINS_${GLIBC_TRIPLE}_CMAKE_BUILD_TYPE ${_CMAKE_BUILD_TYPE} CACHE STRING "")
set(RUNTIMES_${GLIBC_TRIPLE}_CMAKE_BUILD_TYPE ${_CMAKE_BUILD_TYPE} CACHE STRING "")

# Setting target build triples. The LLVM_HOST_TRIPLE var is inherited by all of
# the rest of the `*_TRIPLE` type variables throughout the llvm-project repo.
# Unless we are bootstrapping a cross-compile, this is the only var we set.
set(LLVM_HOST_TRIPLE     ${GLIBC_TRIPLE} CACHE STRING "")
set(LLVM_BUILTIN_TARGETS ${GLIBC_TRIPLE} CACHE STRING "")
set(LLVM_RUNTIME_TARGETS ${GLIBC_TRIPLE} CACHE STRING "")

# Statically link as much as possible to detach from the host toolchain. This
# step is not strictly neeed, but it has helped with troubleshooting inital
# misconfigurations early on.
set(LLVM_STATIC_LINK_CXX_STDLIB ON CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_COMPILER_RT_ENABLE_STATIC_UNWINDER ON  CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_COMPILER_RT_STATIC_CXX_STDLIB      ON  CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_LIBCXXABI_ENABLE_STATIC_UNWINDER   ON  CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_LIBCXX_ENABLE_STATIC_ABI_LIBRARY   ON  CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_LIBUNWIND_ENABLE_SHARED            OFF CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_LIBCXXABI_ENABLE_SHARED            OFF CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_LIBCXX_ENABLE_SHARED               OFF CACHE BOOL "")
set(RUNTIMES_${GLIBC_TRIPLE}_LIBCXXABI_INSTALL_LIBRARY          OFF CACHE BOOL "")
