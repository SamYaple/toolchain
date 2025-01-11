# musl stage2
#set(_SYSROOT /sysroots/musl/llvm)
#set(_TARGET x86_64-unknown-linux-musl)

#set(                    CMAKE_SYSROOT ${_SYSROOT} CACHE STRING "")
#set(RUNTIMES_${_TARGET}_CMAKE_SYSROOT ${_SYSROOT} CACHE STRING "")
#set(BUILTINS_${_TARGET}_CMAKE_SYSROOT ${_SYSROOT} CACHE STRING "")
