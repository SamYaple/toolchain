# musl base config options
set(LLVM_HOST_TRIPLE     ${MUSL_TRIPLE} CACHE STRING "")
set(LLVM_BUILTIN_TARGETS ${MUSL_TRIPLE} CACHE STRING "")
set(LLVM_RUNTIME_TARGETS ${MUSL_TRIPLE} CACHE STRING "")

# NOTE: This seems like an option that should inherit but maybe that is
#       stepping a can of worms to try to figure out
set(                        LIBCXX_HAS_MUSL_LIBC ON CACHE BOOL "")
set(RUNTIMES_${MUSL_TRIPLE}_LIBCXX_HAS_MUSL_LIBC ON CACHE BOOL "")
set(BUILTINS_${MUSL_TRIPLE}_LIBCXX_HAS_MUSL_LIBC ON CACHE BOOL "")
