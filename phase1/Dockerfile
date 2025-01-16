# syntax=docker/dockerfile:1
ARG FROM=localhost/toolchain:latest

####
# When building with Docker/Podman the `RUN` statements are executed with
# `/bin/sh -c 'somecommand -with args'`. We need a symlink setup in the root
# `/bin/sh` -> `/toolchain/bin/sh`. We need to setup the appropriate symlinks
# and then `COPY` them into the container before `RUN` will work. We use the
# busybox image because it is tiny, but any image with `mkdir` and `ln` will
# work for this building this compat layer.
#
# /compat/hack/toolchain -> /sysroots/musl
# /compat/bin/sh         -> /sysroots/musl/usr/bin/sh
# /compat/etc/ssl        -> /sysroots/musl/etc/ssl
# /compat/tmp/
#
# TODO: patch openssl build to check /toolchain/etc/ssl
# TODO: patch mktemp to use /toolchain/tmp
#
# we *want* our toolchain image to contain only a symlink and a sysroot dir
# /toolchain -> /sysroots/musl
# /sysroots/musl/...
####
FROM docker.io/library/busybox:stable-musl AS oci-compat
RUN mkdir -p /compat/hack /compat/bin /compat/etc /compat/tmp \
    && ln -sv /sysroots/musl /compat/hack/toolchain \
    && ln -sv /sysroots/musl/usr/bin/sh /compat/bin/sh \
    && ln -sv /sysroots/musl/etc/ssl /compat/etc/ssl

#####
# We have a toolchain with:
#   clang
#   llvm
#   musl
#   libc++
#   python
#   rust
#  
# No gnu libc/libstdc++ bits exist. Rust has the target spec rewrite for musl so
# rust can dylib and dynamic link all of the stuff. We have `bash` available,
# and we have `cargo`, but no other tools.
#
# Where do we go now?
FROM ${FROM}
COPY --from=oci-compat /compat/etc /etc
COPY --from=oci-compat /compat/tmp /tmp
COPY --from=oci-compat /compat/bin /bin
COPY --from=oci-compat /compat/hack/ /

ENV CARGO_HOME=/cargo
ENV MUSL_TRIPLE=x86_64-unknown-linux-musl
ENV STAGE0_SYSROOT=/sysroots/musl
ENV STAGE1_SYSROOT=/sysroots/stage1
ENV PATH="${STAGE0_SYSROOT}/usr/bin"
ENV RUSTFLAGS="--sysroot=${STAGE0_SYSROOT}"

WORKDIR /git_sources/coreutils
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    cargo install --root ${STAGE0_SYSROOT}/usr --path ./ --features unix

WORKDIR /git_sources/findutils
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    cargo install --root ${STAGE0_SYSROOT}/usr --path ./

#WORKDIR /git_sources/diffutils
#RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
#    cargo install --root ${STAGE0_SYSROOT}/usr --path ./

# Setup symlinks to ALL THE THINGS
RUN coreutils ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/ln \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/mkdir \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/head \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/uname

WORKDIR ${STAGE1_SYSROOT}
RUN mkdir usr usr/bin usr/lib etc \
    && ln -sv usr/lib lib \
    && ln -sv usr/bin bin \
    && ln -sv lib usr/lib64 \
    && ln -sv . toolchain

# need sed installed....
RUN sed

# Install linux headers (needed for musl libc)
WORKDIR /sources/linux
RUN make LLVM=1 -j64 headers \
    && find usr/include -type f ! -name '*.h' -delete \
    && cp -rv usr/include ${STAGE1_SYSROOT}/usr/include

# Install libc
WORKDIR /sources/musl
RUN ./configure --prefix="${STAGE1_SYSROOT}/usr" \
    && make -j64 \
    && make install

RUN echo "${STAGE1_SYSROOT}/usr/lib/${MUSL_TRIPLE}" > ${STAGE1_SYSROOT}/etc/ld-musl-x86_64.path \
    && echo "${STAGE1_SYSROOT}/usr/lib"            >> ${STAGE1_SYSROOT}/etc/ld-musl-x86_64.path \
    && ln -sf libc.so ${STAGE1_SYSROOT}/usr/lib/ld-musl-x86_64.so.1 \
    && ln -sf ../lib/libc.so ${STAGE1_SYSROOT}/usr/bin/ldd

# Install compiler-rt, libc++, libc++abi, and libunwind
WORKDIR /sources/llvm-project
COPY configs/common.cmake /
RUN sed -ie '/^  OBJECT_LIBS RTSanitizerCommon RTSanitizerCommonLibc$/d' compiler-rt/lib/ctx_profile/CMakeLists.txt \
    && cmake -S llvm -B build -G Ninja \
        -D CMAKE_SYSROOT=${STAGE1_SYSROOT} \
        -D CMAKE_INSTALL_PREFIX=${STAGE1_SYSROOT}/usr \
        -D LLVM_HOST_TRIPLE=${MUSL_TRIPLE} \
        -D LLVM_BUILTIN_TARGETS=${MUSL_TRIPLE} \
        -D LLVM_RUNTIME_TARGETS=${MUSL_TRIPLE} \
        -D RUNTIMES_${MUSL_TRIPLE}_LIBCXX_HAS_MUSL_LIBC=ON \
        -C /common.cmake \
    && cmake --build build --target runtimes \
    && cmake --build build --target install-runtimes

# NOT UNTIL zlib and bzip2
RUN false

# We can now update our /toolchain symlink to link against stage1 libs. All of
# the binaries still live in stage0 for now.
RUN ln -svnf ${STAGE1_SYSROOT} /toolchain

# TODO: Add all needed libs before the tools that need them
# make/cmake/ninja and other binaries need to be built last-ish

WORKDIR /sources/make
RUN ./configure \
        --prefix=${STAGE1_SYSROOT}/usr \
        --build=${MUSL_TRIPLE} \
        --host=${MUSL_TRIPLE} \
    && make -j64 \
    && make install

WORKDIR /sources/cmake
RUN cmake \
        -B build \
        -D CMAKE_INSTALL_PREFIX=${STAGE1_SYSROOT}/usr \
        -D CMAKE_USE_OPENSSL=OFF \
    && cmake --build build --parallel 64 \
    && cmake --build build --target install

WORKDIR /sources/python
RUN ./configure \
        --prefix=${STAGE1_SYSROOT}/usr \
        --build=${MUSL_TRIPLE} \
        --host=${MUSL_TRIPLE} \
        --enable-shared \
        --without-ensurepip \
    && make -j64 \
    && make install

WORKDIR /sources/ninja
RUN cmake \
        -B build \
        -D CMAKE_INSTALL_PREFIX=${STAGE1_SYSROOT}/usr \
        -D BUILD_TESTING=OFF \
    && cmake --build build --parallel 64 \
    && cmake --build build --target install
