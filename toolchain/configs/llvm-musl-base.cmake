#
set(CMAKE_BUILD_TYPE Release CACHE STRING "")

#set(LLVM_CCACHE_BUILD ON CACHE BOOL "")
#set(LLVM_CCACHE_DIR   /ccache CACHE STRING "")

set(CLANG_ENABLE_ARCMT           OFF CACHE BOOL "")
set(CLANG_ENABLE_STATIC_ANALYZER OFF CACHE BOOL "")
set(CLANG_PLUGIN_SUPPORT         OFF CACHE BOOL "")
set(CLANG_INCLUDE_DOCS           OFF CACHE BOOL "")
set(CLANG_INCLUDE_TESTS          OFF CACHE BOOL "")
set(LLVM_ENABLE_LIBEDIT          OFF CACHE BOOL "")
set(LLVM_ENABLE_LIBPFM           OFF CACHE BOOL "")
set(LLVM_ENABLE_LIBXML2          OFF CACHE BOOL "")
set(LLVM_ENABLE_ZLIB             OFF CACHE BOOL "")
set(LLVM_ENABLE_ZSTD             OFF CACHE BOOL "")
set(LLVM_INCLUDE_BENCHMARKS      OFF CACHE BOOL "")
set(LLVM_INCLUDE_DOCS            OFF CACHE BOOL "")
set(LLVM_INCLUDE_EXAMPLES        OFF CACHE BOOL "")
set(LLVM_INCLUDE_TESTS           OFF CACHE BOOL "")

set(CLANG_DEFAULT_CXX_STDLIB "libc++"      CACHE STRING "")
set(CLANG_DEFAULT_LINKER     "lld"         CACHE STRING "")
set(CLANG_DEFAULT_RTLIB      "compiler-rt" CACHE STRING "")
set(CLANG_DEFAULT_UNWINDLIB  "libunwind"   CACHE STRING "")

set(COMPILER_RT_DEFAULT_TARGET_ONLY    OFF CACHE BOOL   "")
set(LLVM_ENABLE_PER_TARGET_RUNTIME_DIR ON  CACHE BOOL   "")
set(LLVM_OPTIMIZED_TABLEGEN            ON  CACHE BOOL   "")
set(LLVM_USE_RELATIVE_PATHS_IN_FILES   ON  CACHE BOOL   "")

set(_TOOLCHAIN_TARGET_TRIPLES
  x86_64-unknown-linux-musl
)

set(                                   LLVM_DEFAULT_TARGET_TRIPLE "x86_64-unknown-linux-musl" CACHE STRING "")
set(RUNTIMES_x86_64-unknown-linux-musl_LLVM_DEFAULT_TARGET_TRIPLE "x86_64-unknown-linux-musl" CACHE STRING "")
set(BUILTINS_x86_64-unknown-linux-musl_LLVM_DEFAULT_TARGET_TRIPLE "x86_64-unknown-linux-musl" CACHE STRING "")

set(                                   LIBCXX_HAS_MUSL_LIBC ON CACHE BOOL "")
set(RUNTIMES_x86_64-unknown-linux-musl_LIBCXX_HAS_MUSL_LIBC ON CACHE BOOL "")
set(BUILTINS_x86_64-unknown-linux-musl_LIBCXX_HAS_MUSL_LIBC ON CACHE BOOL "")

foreach(target ${_TOOLCHAIN_TARGET_TRIPLES})
  set(BUILTINS_${target}_CMAKE_SYSTEM_NAME Linux   CACHE STRING "")
  set(BUILTINS_${target}_CMAKE_BUILD_TYPE  Release CACHE STRING "")
  set(BUILTINS_${target}_CMAKE_C_FLAGS   "--target=${target}" CACHE STRING "")
  set(BUILTINS_${target}_CMAKE_CXX_FLAGS "--target=${target}" CACHE STRING "")
  set(BUILTINS_${target}_CMAKE_ASM_FLAGS "--target=${target}" CACHE STRING "")
  set(BUILTINS_${target}_CMAKE_SHARED_LINKER_FLAGS "-fuse-ld=lld" CACHE STRING "")
  set(BUILTINS_${target}_CMAKE_MODULE_LINKER_FLAGS "-fuse-ld=lld" CACHE STRING "")
  set(BUILTINS_${target}_CMAKE_EXE_LINKER_FLAG     "-fuse-ld=lld" CACHE STRING "")
  #set(BUILTINS_${target}_COMPILER_RT_BUILD_STANDALONE_LIBATOMIC ON CACHE BOOL "")

  set(RUNTIMES_${target}_CMAKE_SYSTEM_NAME Linux   CACHE STRING "")
  set(RUNTIMES_${target}_CMAKE_BUILD_TYPE  Release CACHE STRING "")
  set(RUNTIMES_${target}_CMAKE_C_FLAGS   "--target=${target}" CACHE STRING "")
  set(RUNTIMES_${target}_CMAKE_CXX_FLAGS "--target=${target}" CACHE STRING "")
  set(RUNTIMES_${target}_CMAKE_ASM_FLAGS "--target=${target}" CACHE STRING "")
  set(RUNTIMES_${target}_CMAKE_SHARED_LINKER_FLAGS "-fuse-ld=lld" CACHE STRING "")
  set(RUNTIMES_${target}_CMAKE_MODULE_LINKER_FLAGS "-fuse-ld=lld" CACHE STRING "")
  set(RUNTIMES_${target}_CMAKE_EXE_LINKER_FLAGS    "-fuse-ld=lld" CACHE STRING "")
  #set(RUNTIMES_${target}_COMPILER_RT_BUILD_STANDALONE_LIBATOMIC ON CACHE BOOL "")
  #set(RUNTIMES_${target}_COMPILER_RT_BUILD_BUILTINS    ON  CACHE BOOL "")
  set(RUNTIMES_${target}_COMPILER_RT_CXX_LIBRARY "libcxx" CACHE STRING "")
  #set(RUNTIMES_${target}_COMPILER_RT_USE_BUILTINS_LIBRARY ON CACHE BOOL "")
  set(RUNTIMES_${target}_LIBUNWIND_USE_COMPILER_RT     ON  CACHE BOOL "")
  set(RUNTIMES_${target}_LIBCXXABI_USE_COMPILER_RT     ON  CACHE BOOL "")
  set(RUNTIMES_${target}_LIBCXXABI_USE_LLVM_UNWINDER   ON  CACHE BOOL "")
  set(RUNTIMES_${target}_LIBCXX_USE_COMPILER_RT        ON  CACHE BOOL "")
  set(RUNTIMES_${target}_COMPILER_RT_USE_LLVM_UNWINDER ON  CACHE BOOL "")
  set(RUNTIMES_${target}_COMPILER_RT_USE_LIBCXX        ON  CACHE BOOL "")

  set(RUNTIMES_${target}_COMPILER_RT_BUILD_GWP_ASAN    OFF CACHE BOOL "")
  set(RUNTIMES_${target}_COMPILER_RT_BUILD_LIBFUZZER   OFF CACHE BOOL "")
  set(RUNTIMES_${target}_COMPILER_RT_BUILD_MEMPROF     OFF CACHE BOOL "")
  set(RUNTIMES_${target}_COMPILER_RT_BUILD_ORC         OFF CACHE BOOL "")
  set(RUNTIMES_${target}_COMPILER_RT_BUILD_SANITIZERS  OFF CACHE BOOL "")
  set(RUNTIMES_${target}_COMPILER_RT_BUILD_XRAY        OFF CACHE BOOL "")

  set(RUNTIMES_${target}_LLVM_TOOLS_DIR "${CMAKE_BINARY_DIR}/bin" CACHE BOOL "")
  set(RUNTIMES_${target}_LLVM_ENABLE_RUNTIMES "compiler-rt;libcxx;libcxxabi;libunwind" CACHE STRING "")
endforeach()

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
  llvm-size
  llvm-strings
  llvm-strip
)

set(_LLVM_DISTRIBUTION_COMPONENTS
  clang-resource-headers
  builtins
  runtimes
  ${_LLVM_ENABLE_PROJECTS}
  ${_LLVM_TOOLCHAIN_TOOLS}
)

set(LLVM_TOOLCHAIN_TOOLS         ${_LLVM_TOOLCHAIN_TOOLS}         CACHE STRING "")
set(LLVM_DISTRIBUTION_COMPONENTS ${_LLVM_DISTRIBUTION_COMPONENTS} CACHE STRING "")
set(LLVM_ENABLE_PROJECTS         ${_LLVM_ENABLE_PROJECTS}         CACHE STRING "")
set(LLVM_ENABLE_RUNTIMES         ${_LLVM_ENABLE_RUNTIMES}         CACHE STRING "")
set(LLVM_BUILTIN_TARGETS         ${_TOOLCHAIN_TARGET_TRIPLES}     CACHE STRING "")
set(LLVM_RUNTIME_TARGETS         ${_TOOLCHAIN_TARGET_TRIPLES}     CACHE STRING "")
set(LLVM_TARGETS_TO_BUILD        ${_LLVM_TARGETS_TO_BUILD}        CACHE STRING "")

# Expose `distribution` and `install-distribution` targets. These will be
# available as `stage2-distribution` and `stage2-install-distribution` targets
set(_CLANG_BOOTSTRAP_TARGETS
  distribution
  install-distribution
)
set(CLANG_BOOTSTRAP_TARGETS ${_CLANG_BOOTSTRAP_TARGETS} CACHE STRING "")
