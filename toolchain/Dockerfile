# syntax=docker/dockerfile:1
ARG FROM=docker.io/library/debian:bookworm-slim


#####
# BASE
#
# The buildkit cache used for apt is setup here, and used throughout the
# project. The `docker-clean` apt configuration conflicts with this way of
# caching and must be removed. This is the only time `apt-get update` is called
# throughout the Dockerfile.
FROM ${FROM} AS base
COPY debian.sources /etc/apt/sources.list.d/
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    rm /etc/apt/apt.conf.d/docker-clean && \
    apt-get update
#
# End BASE build
#####


#####
# SYSROOT-GLIBC-BASE
#
# Extract the packaged version of glibc and headers. This will not contain any
# c++ library, nor any tooling. It exists as minimal sysroot which we will use
# to bootstrap LLVM against during our STAGE0 build.
FROM base AS sysroot-glibc-base
WORKDIR /sysroots/glibc/base
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get download libc6 libc6-dev linux-libc-dev
RUN for deb in *.deb; do dpkg-deb -x ${deb} .; done
RUN mv lib/x86_64-linux-gnu/* usr/lib/x86_64-linux-gnu/
RUN rm -rf lib* *.deb etc/ld.so.conf.d/*
RUN mkdir usr/bin
RUN ln -sv bin usr/sbin
RUN ln -sv lib usr/lib64
RUN ln -sv usr/lib lib
RUN ln -sv usr/lib lib64
RUN ln -sv usr/bin bin
RUN ln -sv usr/bin sbin
RUN ln -sv ../etc usr/etc
RUN ln -sv x86_64-linux-gnu/libc.so.6 lib/libc.so.6
RUN ln -sv x86_64-linux-gnu/libm.so.6 lib/libm.so.6
RUN ln -sv x86_64-linux-gnu/ld-linux-x86-64.so.2 lib/ld-linux-x86-64.so.2
RUN cd lib/x86_64-linux-gnu && find . -type l -printf '%P\0' | xargs -0 -r -n1 bash -c 'ln -svf ${1}.* ${1}' _
#
# End SYSROOT-GLIBC-BASE build
#####


#####
# STAGE0
#
# Produces a working toolchain with LLVM and Clang that is still linked against
# `glibc`. This will be the base compiler we use to bootstrap the Musl sysroot
FROM base AS stage0
# Install a base toolchain to build our stage0 compiler from. We are using `gcc`
# and `g++` for the C compiler and C++ compiler respectively. You could swap out
# this with any other compiler that can build LLVM, at least in theory.
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y \
        g++ \
        gcc \
        cmake \
	patch \
        ninja-build \
        python3
# Build llvm on top of the pristine glibc environment we unpacked in the
# `sysroot-glibc-base` image. This sysroot contains only libc and its headers
# without any c++.
COPY --from=sysroot-glibc-base /sysroots/glibc/base /sysroots/glibc/llvm
# Build LLVM
#
# The referenced cmake config files contain all the configuration options set
# to ensure this stage0 compiler will be a minimal distribution of tools. Only
# the tools needed to bootstrap the musl toolchain are kept.
#
# The cmake files are also commented for additional reading.
#
# NOTE: fix-cmake-no-sanitizers-build.patch is a hotfix to allow a no-sanitizers
#       build to work. This was not a problem in llvm 18.x but I haven't looked
#       at the issue beyond making this quick patch.
WORKDIR /sources/llvm-project
COPY configs /configs
COPY patches/llvm-project /patches/llvm-project
RUN patch -p1 < /patches/llvm-project/fix-cmake-no-sanitizers-build.patch && \
    cmake -S llvm -B build -G Ninja -C /configs/llvm-glibc-stage1.cmake -C /configs/llvm-glibc-base.cmake && \
    cmake --build build --target runtimes && \
    cmake --build build --target install-runtimes && \
    cmake --build build --target stage2-distribution && \
    cmake --build build --target stage2-install-distribution
#
# End STAGE0 build
#####


#####
# SYSROOT-MUSL-BASE
#
FROM base AS sysroot-musl-base
# We reuse the `base` image here, so we have no tools. gnumake and a
# functioning compiler is needed. We copy in our stage0 compiler and install
# make from a packaged source.
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y make
COPY --from=stage0 /sysroots/glibc/llvm /sysroots/glibc/llvm
ENV PATH="/sysroots/glibc/llvm/usr/bin:${PATH}"
# Setup our sysroot structure. It is more or less the merged-usr approach.
RUN mkdir -p /sysroots/musl/base/toolchain
RUN ln -sv /sysroots/musl/base/toolchain /toolchain
WORKDIR /toolchain
RUN mkdir usr usr/bin usr/lib etc
RUN ln -s usr/lib lib
RUN ln -s usr/bin bin
RUN ln -s lib usr/lib64
# We need the linux headers to build musl (unlike with glibc). We build them
# using our stage0 compiler. TODO: detail the what/when/hows of this process
WORKDIR /sources/linux
RUN make HOSTCFLAGS="--sysroot=/sysroots/glibc/llvm" HOSTLDFLAGS="--sysroot=/sysroots/glibc/llvm" LLVM=1 -j64 headers && \
    find usr/include -type f ! -name '*.h' -delete && \
    cp -rv usr/include /toolchain/usr/include
# Build and install musl libc using our stage0 tools. A small twist is having
# `/toolchain/etc/ld-musl-x86_64.path` look into `/toolchain/usr` vs `/usr`.
# This further isolates our new toolchain from any host files.
WORKDIR /sources/musl
RUN ./configure CC="clang --sysroot=/sysroots/glibc/llvm" AR="llvm-ar" RANLIB="llvm-ranlib" --target="x86_64-unknown-linux-musl" --prefix="/toolchain/usr" && \
    make -j64 && \
    make install
RUN ln -sf libc.so /toolchain/usr/lib/ld-musl-x86_64.so.1
RUN ln -sf ../lib/libc.so /toolchain/usr/bin/ldd
RUN echo "/toolchain/usr/lib/x86_64-unknown-linux-musl\n/toolchain/usr/lib" > /toolchain/etc/ld-musl-x86_64.path
#
# End SYSROOT-MUSL-BASE build
#####


####
# build llvm based on musl
####
FROM base AS llvm-musl

RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y \
        cmake \
        ninja-build \
        python3 \
        patch

COPY --from=sysroot-musl-base /sysroots/musl/base /sysroots/musl/llvm
RUN ln -sv /sysroots/musl/llvm/toolchain /toolchain
COPY --from=stage0 /sysroots/glibc/llvm /sysroots/glibc/llvm
ENV PATH="/sysroots/glibc/llvm/usr/bin:${PATH}"

WORKDIR /sources/llvm-project
COPY configs /configs
COPY patches/llvm-project /patches/llvm-project
RUN patch -p1 < /patches/llvm-project/fix-cmake-no-sanitizers-build.patch && \
    patch -p1 < /patches/llvm-project/toolchain-prefix.patch && \
    cmake -S llvm -B build -G Ninja -C /configs/llvm-musl-stage1.cmake -C /configs/llvm-musl-base.cmake && \
    cmake --build build --target runtimes && \
    cmake --build build --target install-runtimes && \
    cmake --build build --target stage2-distribution && \
    cmake --build build --target stage2-install-distribution


#### 
# build static gnumake
####
FROM base AS toolchain
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y patch

COPY --from=llvm-musl /sysroots/musl/llvm/toolchain /toolchain
RUN ln -sv . /toolchain/toolchain
ENV PATH="/toolchain/usr/bin:${PATH}"

ENV STRIP=llvm-strip
ENV RANLIB=llvm-ranlib
ENV AS=llvm-as
ENV AR=llvm-ar
ENV LD=lld
ENV CC=clang
ENV CXX=clang++
ENV CFLAGS="--sysroot=/toolchain"
ENV CXXFLAGS="${CFLAGS}"

WORKDIR /sources/make
COPY patches/make /patches/make
RUN patch -p1 < /patches/make/fix-bootstrap.patch && \
    ./configure --prefix=/toolchain/usr --host=x86_64-unknown-linux-musl && \
    ./build.sh && \
    ./make install

WORKDIR /sources/bash
COPY patches/bash /patches/bash
RUN patch -p1 < /patches/bash/fix-missing-header.patch && \
    ./configure --prefix=/toolchain/usr --host=x86_64-unknown-linux-musl --without-bash-malloc && \
    make -j64 && \
    make install
RUN ln -sv bash /toolchain/bin/sh
# Force all new docker shell invocations to use our just-build bash shell. This
# has no affect on the /toolchain sysroot and only persists inside of this image
# build.
RUN ln -svf /toolchain/bin/sh /bin/sh
RUN ln -svf /toolchain/bin/bash /bin/bash

WORKDIR /sources/python
RUN ./configure --prefix=/toolchain/usr --build=x86_64-unknown-linux-musl --host=x86_64-unknown-linux-musl --enable-shared --without-ensurepip && \
    make -j64 && \
    make install

WORKDIR /sources/cmake
COPY patches/cmake /patches/cmake
RUN patch -p1 < /patches/cmake/disable-openssl-during-bootstrap.patch && \
    ./bootstrap && \
    make -j64 && \
    make install

WORKDIR /sources/ninja
RUN ./configure.py --bootstrap && \
    ./ninja all && \
    install -m 0755 ./ninja /toolchain/usr/bin/ninja

#WORKDIR /sources/m4
#RUN ./configure --prefix=/toolchain/usr --host=x86_64-unknown-linux-musl && \
#    make -j64 && \
#    make install
#
#WORKDIR /sources/gawk
## TODO: convert this to a patch
#RUN sed -ie 's|extras||' Makefile.in && \
#    ./configure --prefix=/toolchain/usr --host=x86_64-unknown-linux-musl --disable-extensions && \
#    make -j64 && \
#    make install
#
#WORKDIR /sources/grep
#RUN ./configure --prefix=/toolchain/usr --host=x86_64-unknown-linux-musl && \
#    make -j64 && \
#    make install
#
#WORKDIR /sources/gzip
#RUN ./configure --prefix=/toolchain/usr --host=x86_64-unknown-linux-musl && \
#    make -j64 && \
#    make install
#
#WORKDIR /sources/patch
#RUN ./configure --prefix=/toolchain/usr --host=x86_64-unknown-linux-musl && \
#    make -j64 && \
#    make install

WORKDIR /sources/sed
RUN ./configure --prefix=/toolchain/usr --host=x86_64-unknown-linux-musl && \
    make -j64 && \
    make install

## TODO: remove FORCE_UNSAFE_CONFIGURE=1. this is in place because we are
##       building as root currently
WORKDIR /sources/coreutils
RUN FORCE_UNSAFE_CONFIGURE=1 ./configure --prefix=/toolchain/usr --host=x86_64-unknown-linux-musl && \
    make -j64 && \
    make install

## TODO: remove FORCE_UNSAFE_CONFIGURE=1. this is in place because we are
##       building as root currently
#WORKDIR /sources/tar
#RUN FORCE_UNSAFE_CONFIGURE=1 ./configure --prefix=/toolchain/usr --host=x86_64-unknown-linux-musl && \
#    make -j64 && \
#    make install

#WORKDIR /sources/xz
#RUN ./configure --prefix=/toolchain/usr --host=x86_64-unknown-linux-musl --disable-static && \
#    make -j64 && \
#    make install
#RUN rm /toolchain/usr/lib/liblzma.la

####
# OCI/docker/podman need /bin/sh available, but only /toolchain will exist in
# the root of our final FROM scratch image.
####
FROM base AS oci-compat
RUN mkdir -p /compat/bin
RUN ln -s /toolchain/bin/sh /compat/bin/sh

####
# final
####
FROM scratch
COPY --from=oci-compat /compat/bin /bin
COPY --from=toolchain /toolchain /toolchain
ENV PATH="/toolchain/usr/bin"
