# musl stage2

# At this point in our build, the ${_SYSROOT} has the following components
# installed: libc, libc++, libunwind, compiler-rt. Additionally, we can reuse
# the Clang and LLVM tools compiled in stage1 to rebuild Clang and LLVM.
#
# The output of the stage2 build is a toolchain that was built with the current
# version of LLVM and Clang in it's own contained sysroot. In our case, we have
# switched from a glibc based toolchain to a musl base toolchain.
set(CMAKE_SYSROOT ${MUSL_SYSROOT} CACHE STRING "")
