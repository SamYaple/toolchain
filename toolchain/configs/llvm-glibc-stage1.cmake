# stage1 overrides

# We do not yet have a sysroot that is capable of building our toolchain. The
# provided sysroot only contains libc currently. Because of this, we need to set
# the install location directly.
set(CMAKE_INSTALL_PREFIX /sysroots/glibc/llvm/usr CACHE STRING "")

set(RUNTIMES_x86_64-unknown-linux-gnu_CMAKE_SYSROOT /sysroots/glibc/llvm     CACHE STRING "")
set(BUILTINS_x86_64-unknown-linux-gnu_CMAKE_SYSROOT /sysroots/glibc/llvm     CACHE STRING "")

set(LLVM_STATIC_LINK_CXX_STDLIB ON CACHE BOOL "")
foreach(target x86_64-unknown-linux-gnu)
  set(RUNTIMES_${target}_COMPILER_RT_ENABLE_STATIC_UNWINDER ON  CACHE BOOL "")
  set(RUNTIMES_${target}_COMPILER_RT_STATIC_CXX_STDLIB      ON  CACHE BOOL "")
  set(RUNTIMES_${target}_LIBCXXABI_ENABLE_STATIC_UNWINDER   ON  CACHE BOOL "")
  set(RUNTIMES_${target}_LIBCXX_ENABLE_STATIC_ABI_LIBRARY   ON  CACHE BOOL "")
  set(RUNTIMES_${target}_LIBUNWIND_ENABLE_SHARED            OFF CACHE BOOL "")
  set(RUNTIMES_${target}_LIBCXXABI_ENABLE_SHARED            OFF CACHE BOOL "")
  set(RUNTIMES_${target}_LIBCXX_ENABLE_SHARED               OFF CACHE BOOL "")
  set(RUNTIMES_${target}_LIBCXXABI_INSTALL_LIBRARY          OFF CACHE BOOL "")
endforeach()

set(CLANG_ENABLE_BOOTSTRAP ON CACHE BOOL "")
set(CLANG_BOOTSTRAP_CMAKE_ARGS -C /configs/llvm-glibc-stage2.cmake -C /configs/llvm-glibc-base.cmake -C /configs/common.cmake CACHE STRING "")
