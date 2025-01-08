# syntax=docker/dockerfile:1
ARG FROM=docker.io/library/debian:bookworm-slim


####
# Setup base image
####
FROM ${FROM} AS base
COPY debian.sources /etc/apt/sources.list.d/
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    rm /etc/apt/apt.conf.d/docker-clean && \
    apt-get update
COPY patches /patches
COPY configs /configs


####
# Setup untainted glibc base with no toolchain
####
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


####
# build llvm ontop of glibc
####
FROM base AS llvm-glibc
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y \
        g++ \
        gcc \
        cmake \
	patch \
        ninja-build \
        python3

# copy over glibc base (no toolchain; no c++; libc only)
COPY --from=sysroot-glibc-base /sysroots/glibc/base /sysroots/glibc/llvm

WORKDIR /sources/llvm-project
RUN patch -p1 < /patches/llvm-project/fix-cmake-no-sanitizers-build.patch && \
    cmake -S llvm -B build -G Ninja -C /configs/llvm-glibc-stage1.cmake -C /configs/llvm-glibc-base.cmake && \
    cmake --build build --target runtimes && \
    cmake --build build --target install-runtimes && \
    cmake --build build --target stage2-distribution && \
    cmake --build build --target stage2-install-distribution


####
# setup musl libc base sysroot
####
FROM base AS sysroot-musl-base
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y make

COPY --from=llvm-glibc /sysroots/glibc/llvm /sysroots/glibc/llvm
ENV PATH="/sysroots/glibc/llvm/usr/bin:${PATH}"

ENV SYSROOT=/sysroots/musl/base/toolchain
WORKDIR ${SYSROOT}
RUN mkdir usr usr/bin usr/lib etc
RUN ln -s usr/lib lib
RUN ln -s usr/bin bin
RUN ln -s lib usr/lib64
RUN echo "/toolchain/usr/lib/x86_64-unknown-linux-musl\n/toolchain/usr/lib" > etc/ld-musl-x86_64.path

WORKDIR /sources/linux
RUN make HOSTCFLAGS="--sysroot=/sysroots/glibc/llvm" HOSTLDFLAGS="--sysroot=/sysroots/glibc/llvm" LLVM=1 -j64 headers && \
    find usr/include -type f ! -name '*.h' -delete && \
    cp -rv usr/include ${SYSROOT}/usr/include

WORKDIR /sources/musl
RUN ./configure CC="clang --sysroot=/sysroots/glibc/llvm" AR="llvm-ar" RANLIB="llvm-ranlib" --target="x86_64-unknown-linux-musl" --prefix="${SYSROOT}/usr" && \
    make -j64 && \
    make install
RUN ln -sf libc.so "${SYSROOT}/usr/lib/ld-musl-x86_64.so.1"
RUN ln -sf ../lib/libc.so "${SYSROOT}/usr/bin/ldd"
RUN echo "/toolchain/lib/x86_64-unknown-linux-musl" > "${SYSROOT}/etc/ld-musl-x86_64.path"


####
# build llvm based on musl
####
FROM base AS llvm-musl
COPY --from=llvm-glibc /sysroots/glibc/llvm /sysroots/glibc/llvm
ENV PATH="/sysroots/glibc/llvm/usr/bin:${PATH}"

RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y \
        cmake \
        ninja-build \
        python3 \
	patch

COPY --from=sysroot-musl-base /sysroots/musl/base /sysroots/musl/llvm
RUN ln -sv /sysroots/musl/llvm/toolchain /toolchain

WORKDIR /sources/llvm-project
RUN patch -p1 < /patches/llvm-project/toolchain-prefix.patch && \
    patch -p1 < /patches/llvm-project/fix-cmake-no-sanitizers-build.patch && \
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
    apt-get install --no-install-recommends -y make

COPY --from=llvm-musl /sysroots/musl/llvm /sysroots/musl/llvm
RUN ln -sv /sysroots/musl/llvm/toolchain /toolchain
ENV PATH="/toolchain/usr/bin:${PATH}"

WORKDIR /sources/make
RUN LDFLAGS="-static" STRIP=llvm-strip RANLIB=llvm-ranlib AS=llvm-as AR=llvm-ar LD=lld CC=clang CXX=clang++ \
    ./configure --prefix=/toolchain/usr --host=x86_64-unknown-linux-musl && \
    make -j64 && \
    make install

####
# build bash
####
FROM base AS bash
COPY --from=toolchain /sysroots/musl/llvm /sysroots/musl/llvm
RUN ln -sv /sysroots/musl/llvm/toolchain /toolchain
ENV PATH="/toolchain/usr/bin:${PATH}"

WORKDIR /sources/bash
RUN sed -ie '/#include "ltcap.h"/i #include <unistd.h>' lib/termcap/tparam.c && \
    STRIP=llvm-strip RANLIB=llvm-ranlib AS=llvm-as AR=llvm-ar LD=lld CC=clang CXX=clang++ \
    ./configure --prefix=/toolchain/usr --host=x86_64-unknown-linux-musl --without-bash-malloc && \
    make -j64 && \
    make install

####
# final
####
FROM ${FROM}
COPY --from=bash /sysroots/musl/llvm /sysroots/musl/llvm
RUN ln -sv /sysroots/musl/llvm/toolchain /toolchain
ENV PATH="/toolchain/usr/bin:${PATH}"
