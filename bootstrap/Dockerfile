# syntax=docker/dockerfile:1
ARG FROM=docker.io/library/debian:bookworm-slim

# The buildkit cache used for apt is setup here, and used throughout the
# Dockerfile. The `docker-clean` apt configuration conflicts with this way of
# caching and must be removed. This is the only time `apt-get update`.
FROM ${FROM}
COPY debian.sources /etc/apt/sources.list.d/
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    rm /etc/apt/apt.conf.d/docker-clean \
    && apt-get update

# Setup some common variables used throughout the build process, but not in the
# final produced image. No trickery going on here, just some helper vars.
ENV GLIBC_SYSROOT=/sysroots/glibc
ENV GLIBC_TRIPLE=x86_64-unknown-linux-gnu

# Create a new sysroot and extract glibc with relevant kernel header files. This
# gets us a minimal glibc sysroot with no c++ support.
WORKDIR ${GLIBC_SYSROOT}
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get download libc6 libc6-dev linux-libc-dev

# extract a pre-built glibc and headers. this step should work from a non-debian
# based system if you extract the deb files in the same way.
RUN for deb in *.deb; do dpkg-deb -x ${deb} .; done \
    && mv lib/x86_64-linux-gnu/* usr/lib/x86_64-linux-gnu/ \
    && rm -rf lib* *.deb etc/ld.so.conf.d/* \
    && mkdir usr/bin \
    && ln -sv bin usr/sbin \
    && ln -sv lib usr/lib64 \
    && ln -sv usr/lib lib \
    && ln -sv usr/lib lib64 \
    && ln -sv usr/bin bin \
    && ln -sv usr/bin sbin \
    && ln -sv ../etc usr/etc \
    && ln -sv x86_64-linux-gnu/libc.so.6 lib/libc.so.6 \
    && ln -sv x86_64-linux-gnu/libm.so.6 lib/libm.so.6 \
    && ln -sv x86_64-linux-gnu/ld-linux-x86-64.so.2 lib/ld-linux-x86-64.so.2 \
    && cd lib/x86_64-linux-gnu \
    && find . -type l -exec sh -c 'ln -svf "$1".* "$1"' _ {} \;

# We need these utilities to build LLVM. We don't have to use ninja, but we
# aren't trying to hit issues with make builds and LLVM defaults to ninja
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y \
        cmake \
        python3 \
        ninja-build

# Install a base toolchain to build our bootstrap compiler from. We are using
# `gcc` and `g++` for the C compiler and C++ compiler respectively. You could
# swap out this with any other compiler that can build LLVM, at least in theory.
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y gcc g++

# Build LLVM, Clang, libc++, compiler-rt
#
# The referenced cmake config files contain all the configuration options set
# to ensure this bootstrap compiler will be a minimal distribution of tools.
# Only the tools needed to bootstrap the musl toolchain are kept.
#
# The cmake files are also commented for additional reading.
WORKDIR /sources/llvm-project
COPY common.cmake glibc-base.cmake glibc-stage1.cmake /
RUN sed -ie '/^  OBJECT_LIBS RTSanitizerCommon RTSanitizerCommonLibc$/d' compiler-rt/lib/ctx_profile/CMakeLists.txt \
    && cmake -S llvm -B build -G Ninja \
        -D GLIBC_SYSROOT=${GLIBC_SYSROOT} \
        -D GLIBC_TRIPLE=${GLIBC_TRIPLE} \
        -C /glibc-stage1.cmake \
        -C /glibc-base.cmake \
        -C /common.cmake \
    && cmake --build build --target runtimes \
    && cmake --build build --target install-runtimes \
    && cmake --build build --target stage2-distribution \
    && cmake --build build --target stage2-install-distribution
