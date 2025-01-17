# LLVM build common cmake options
#
# We set several variables at the start of this file prefixed with `_`. These
# are not able to be configured by the user since they are not specified as
# `CACHE`. They do not conflict with any other options by name. This is a style
# and readability choice on my part that helps keep consistency across the
# BUILTINS and RUNTIMES ontop of the main build.

###VARS###
set(_CMAKE_BUILD_TYPE
  Release
)

set(_TOOLCHAIN_TARGET_TRIPLES
  x86_64-unknown-linux-gnu
  x86_64-unknown-linux-musl
)

set(_CLANG_BOOTSTRAP_TARGETS
  distribution
  install-distribution
)

set(_LLVM_TARGETS_TO_BUILD
  X86
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
  dsymutil
  llvm-ar
  llvm-nm
  llvm-objcopy
  llvm-objdump
  llvm-ranlib
  llvm-readelf
  llvm-size
  llvm-strings
  llvm-strip

  # needs to be built for rust, but this list also gets installed
  # this is technically undesired TODO
  FileCheck
  llvm-mca
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
set(LLVM_OPTIMIZED_TABLEGEN      ON  CACHE BOOL "")

# Need to disable these for musl build (TODO: revisit this)
set(LLVM_INCLUDE_TESTS           OFF CACHE BOOL "")
set(CLANG_ENABLE_ARCMT           OFF CACHE BOOL "")
set(CLANG_ENABLE_STATIC_ANALYZER OFF CACHE BOOL "")

# Ensure each target is isolated. Rather than being dumped into `/lib`, we will
# install it into `/lib/x86_64-unknown-linux-musl`, for example.
set(COMPILER_RT_DEFAULT_TARGET_ONLY    OFF CACHE BOOL "")
set(LLVM_ENABLE_PER_TARGET_RUNTIME_DIR ON  CACHE BOOL "")
set(LLVM_USE_RELATIVE_PATHS_IN_FILES   ON  CACHE BOOL "")
set(LLVM_INSTALL_UTILS                 ON  CACHE BOOL "")
set(LLVM_INSTALL_CCTOOLS_SYMLINKS      ON  CACHE BOOL "")
set(LLVM_INSTALL_BINUTILS_SYMLINKS     ON  CACHE BOOL "")

# Expose `distribution` and `install-distribution` targets. These will be
# available as `stage2-distribution` and `stage2-install-distribution` targets
set(CLANG_BOOTSTRAP_TARGETS ${_CLANG_BOOTSTRAP_TARGETS} CACHE STRING "")

# Minimize the set of tools and utilities created to a minimal set. This saves
# space and cpu cycles, but is missing some of the cooler tools like bolt and
# even lto currently.
set(LLVM_TOOLCHAIN_TOOLS         ${_LLVM_TOOLCHAIN_TOOLS}         CACHE STRING "")
set(LLVM_DISTRIBUTION_COMPONENTS ${_LLVM_DISTRIBUTION_COMPONENTS} CACHE STRING "")
set(LLVM_ENABLE_PROJECTS         ${_LLVM_ENABLE_PROJECTS}         CACHE STRING "")
set(LLVM_ENABLE_RUNTIMES         ${_LLVM_ENABLE_RUNTIMES}         CACHE STRING "")
set(LLVM_TARGETS_TO_BUILD        ${_LLVM_TARGETS_TO_BUILD}        CACHE STRING "")

set(CMAKE_BUILD_TYPE ${_CMAKE_BUILD_TYPE} CACHE STRING "")
foreach(target ${_TOOLCHAIN_TARGET_TRIPLES})
  # Inheriting options from the main build. For more complex usage, you might
  # want to select different components or debug options per target.
  set(BUILTINS_${target}_CMAKE_BUILD_TYPE     ${_CMAKE_BUILD_TYPE}     CACHE STRING "")
  set(RUNTIMES_${target}_CMAKE_BUILD_TYPE     ${_CMAKE_BUILD_TYPE}     CACHE STRING "")
  set(RUNTIMES_${target}_LLVM_ENABLE_RUNTIMES ${_LLVM_ENABLE_RUNTIMES} CACHE STRING "")

  # Disabling sanitizers
  set(RUNTIMES_${target}_COMPILER_RT_BUILD_GWP_ASAN   OFF CACHE BOOL "")
  set(RUNTIMES_${target}_COMPILER_RT_BUILD_LIBFUZZER  OFF CACHE BOOL "")
  set(RUNTIMES_${target}_COMPILER_RT_BUILD_MEMPROF    OFF CACHE BOOL "")
  set(RUNTIMES_${target}_COMPILER_RT_BUILD_ORC        OFF CACHE BOOL "")
  set(RUNTIMES_${target}_COMPILER_RT_BUILD_SANITIZERS OFF CACHE BOOL "")
  set(RUNTIMES_${target}_COMPILER_RT_BUILD_XRAY       OFF CACHE BOOL "")
endforeach()
# musl base config options
set(LLVM_HOST_TRIPLE     ${MUSL_TRIPLE} CACHE STRING "")
set(LLVM_BUILTIN_TARGETS ${MUSL_TRIPLE} CACHE STRING "")
set(LLVM_RUNTIME_TARGETS ${MUSL_TRIPLE} CACHE STRING "")
set(RUNTIMES_${MUSL_TRIPLE}_LIBCXX_HAS_MUSL_LIBC ON CACHE BOOL "")

set(CMAKE_INSTALL_PREFIX ${MUSL_SYSROOT}/usr CACHE STRING "")
set(CMAKE_SYSROOT ${MUSL_SYSROOT} CACHE STRING "")

# The clang bootstrap process works in two stages. `stage1` builds clang and
# llvm, then uses those to build the builtins and runtimes. Only the runtimes
# are installed (compiler-rt, libc++, libunwind). `stage2` builds clang and
# llvm again using the newly installed runtimes, and the compiler from `stage1`
set(CLANG_ENABLE_BOOTSTRAP ON CACHE BOOL "")
set(CLANG_BOOTSTRAP_CMAKE_ARGS
    -D MUSL_TRIPLE=${MUSL_TRIPLE}
    -D CLANG_ENABLE_BOOTSTRAP=OFF
    -C /musl-llvm.cmake
    CACHE STRING "")
