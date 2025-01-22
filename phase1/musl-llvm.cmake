set(TRIPLE "x86_64-unknown-linux-musl" CACHE STRING "")

set(CMAKE_C_FLAGS   "-O3 -march=native" CACHE STRING "")
set(CMAKE_CXX_FLAGS "-O3 -march=native" CACHE STRING "")

set(CMAKE_BUILD_TYPE      "Release"   CACHE STRING "")
set(LLVM_TARGETS_TO_BUILD "X86"       CACHE STRING "")
set(LLVM_ENABLE_PROJECTS  "clang;lld" CACHE STRING "")
set(LLVM_ENABLE_RUNTIMES  "libunwind;libcxxabi;libcxx;compiler-rt" CACHE STRING "")
set(CLANG_BOOTSTRAP_TARGETS "distribution;install-distribution" CACHE STRING "")

# Set compiler defaults
set(CLANG_DEFAULT_CXX_STDLIB "libc++"      CACHE STRING "")
set(CLANG_DEFAULT_LINKER     "lld"         CACHE STRING "")
set(CLANG_DEFAULT_RTLIB      "compiler-rt" CACHE STRING "")
set(CLANG_DEFAULT_UNWINDLIB  "libunwind"   CACHE STRING "")

# Recommended option for build performance (why is it not default?)
set(LLVM_OPTIMIZED_TABLEGEN ON CACHE BOOL "")

#set(LLVM_ENABLE_LTO "Thin" CACHE STRING "")
#set(LLVM_BUILD_INSTRUMENTED OFF CACHE BOOL "")
set(LLVM_LINK_LLVM_DYLIB ON CACHE BOOL "")
set(LLVM_INSTALL_UTILS   ON CACHE BOOL "")
set(LLVM_USE_RELATIVE_PATHS_IN_FILES ON CACHE BOOL "")
set(LLVM_INSTALL_BINUTILS_SYMLINKS   ON CACHE BOOL "")
set(LLVM_INSTALL_CCTOOLS_SYMLINKS    ON CACHE BOOL "")
set(CLANG_LINKS_TO_CREATE "clang++;cc;c++" CACHE STRING "")

set(LLVM_BUILTIN_TARGETS ${TRIPLE} CACHE STRING "")
set(LLVM_RUNTIME_TARGETS ${TRIPLE} CACHE STRING "")
set(LLVM_HOST_TRIPLE     ${TRIPLE} CACHE STRING "")

set(RUNTIMES_${TRIPLE}_LIBCXX_HAS_MUSL_LIBC             ON CACHE BOOL "")
set(RUNTIMES_${TRIPLE}_COMPILER_RT_USE_BUILTINS_LIBRARY ON CACHE BOOL "")
set(RUNTIMES_${TRIPLE}_COMPILER_RT_USE_LLVM_UNWINDER    ON CACHE BOOL "")
set(RUNTIMES_${TRIPLE}_LIBCXXABI_USE_COMPILER_RT        ON CACHE BOOL "")
set(RUNTIMES_${TRIPLE}_LIBCXX_USE_COMPILER_RT           ON CACHE BOOL "")
set(RUNTIMES_${TRIPLE}_LIBUNWIND_USE_COMPILER_RT        ON CACHE BOOL "")

# Need to disable these for musl build (TODO: revisit this)
set(RUNTIMES_${TRIPLE}_COMPILER_RT_BUILD_GWP_ASAN OFF CACHE BOOL "")
set(RUNTIMES_${TRIPLE}_COMPILER_RT_BUILD_MEMPROF  OFF CACHE BOOL "")
set(RUNTIMES_${TRIPLE}_COMPILER_RT_BUILD_ORC      OFF CACHE BOOL "")

set(COMPILER_RT_DEFAULT_TARGET_ONLY    ON  CACHE BOOL "")
set(LLVM_ENABLE_PER_TARGET_RUNTIME_DIR OFF CACHE BOOL "")
set(LLVM_USE_RELATIVE_PATHS_IN_FILES   ON  CACHE BOOL "")

# The clang bootstrap process works in two stages. `stage1` builds clang and
# llvm, then uses those to build the builtins and runtimes. Only the runtimes
# are installed (compiler-rt, libc++, libunwind). `stage2` builds clang and
# llvm again using the newly installed runtimes, and the compiler from `stage1`
set(CLANG_ENABLE_BOOTSTRAP ON CACHE BOOL "")
set(CLANG_BOOTSTRAP_CMAKE_ARGS
    -D CMAKE_SYSROOT=${CMAKE_SYSROOT}
    -D DEFAULT_SYSROOT=${DEFAULT_SYSROOT}
    -D CLANG_ENABLE_BOOTSTRAP=OFF
    -C /musl-llvm.cmake
    CACHE STRING "")

set(LLVM_TOOLCHAIN_TOOLS
    # binutils alternatives
    llvm-addr2line
    llvm-ar
    llvm-cxxfilt
    llvm-nm
    llvm-objcopy
    llvm-objdump
    llvm-ranlib
    llvm-readelf
    llvm-size
    llvm-strings
    llvm-strip

    # symlink targets
    addr2line
    ar
    c++filt
    nm
    objcopy
    objdump
    ranlib
    readelf
    size
    strings
    strip

    # build additional tools
    llvm-config # rust *needs* this
    llvm-cov
    llvm-dlltool
    llvm-dwp
    llvm-lib
    llvm-lto
    llvm-mca # rust *needs* this
    llvm-ml
    llvm-pdbutil
    llvm-profdata
    llvm-rc
    llvm-readobj
    llvm-symbolizer
    CACHE STRING "")

set(LLVM_TOOLCHAIN_UTILITIES
  FileCheck
  obj2yaml
  yaml2obj
  #not
  #count
  CACHE STRING "")

set(LLVM_DISTRIBUTION_COMPONENTS
  # compiler-rt, libc++, libunwind
  builtins
  runtimes

  # linker
  lld

  # c/c++ compiler
  clang
  clang-format
  clang-resource-headers

  # llvm components
  llvm-headers
  LLVM
  LTO
  Remarks
  ${LLVM_TOOLCHAIN_TOOLS}
  ${LLVM_TOOLCHAIN_UTILITIES}
  CACHE STRING "")
