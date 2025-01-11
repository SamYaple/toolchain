# stage2 overrides

# During stage1 we populated the sysroot with a minimal runtime consisting of
# compiler-rt and libc++. These components were built with the current version
# of LLVM and are tied to the libc in the associated sysroot. Now we can set the
# proper sysroot and rebuild during the stage2 compilation.
set(                                  CMAKE_SYSROOT /sysroots/glibc/llvm CACHE STRING "")
set(RUNTIMES_x86_64-unknown-linux-gnu_CMAKE_SYSROOT /sysroots/glibc/llvm CACHE STRING "")
set(BUILTINS_x86_64-unknown-linux-gnu_CMAKE_SYSROOT /sysroots/glibc/llvm CACHE STRING "")

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
