# syntax=docker/dockerfile:1
ARG FROM=localhost/phase0:latest

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
ENV PATH="${STAGE1_SYSROOT}/usr/bin:${STAGE0_SYSROOT}/usr/bin"

ENV    CFLAGS="--sysroot=${STAGE0_SYSROOT}"
ENV  CXXFLAGS="--sysroot=${STAGE0_SYSROOT}"
ENV   LDFLAGS="--sysroot=${STAGE0_SYSROOT}"
ENV RUSTFLAGS="--sysroot=${STAGE0_SYSROOT}"

# Setup symlinks to ALL THE THINGS
RUN coreutils ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/ln \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/basename \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/cat \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/chmod \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/cp \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/cut \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/date \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/dirname \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/expr \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/head \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/ls \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/mkdir \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/mv \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/pwd \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/rm \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/rmdir \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/sleep \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/sort\
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/tee \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/test \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/touch \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/tr \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/true \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/uname \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/wc \
    && ln -sv diffutils ${STAGE0_SYSROOT}/usr/bin/cmp

WORKDIR ${STAGE1_SYSROOT}
RUN mkdir usr usr/bin usr/lib etc \
    && ln -sv usr/lib lib \
    && ln -sv usr/bin bin \
    && ln -sv lib usr/lib64 \
    && ln -sv . toolchain

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
COPY musl-llvm.cmake /
RUN sed -ie '/^  OBJECT_LIBS RTSanitizerCommon RTSanitizerCommonLibc$/d' compiler-rt/lib/ctx_profile/CMakeLists.txt \
    && cmake -S llvm -B build -G "Unix Makefiles" \
        -D MUSL_TRIPLE=${MUSL_TRIPLE} \
	-D MUSL_SYSROOT=${STAGE1_SYSROOT} \
        -C /musl-llvm.cmake \
    && cmake --build build --target runtimes --parallel 64 \
    && cmake --build build --target install-runtimes

ENV    CFLAGS="--sysroot=${STAGE1_SYSROOT}"
ENV  CXXFLAGS="--sysroot=${STAGE1_SYSROOT}"
ENV   LDFLAGS="--sysroot=${STAGE1_SYSROOT}"
ENV RUSTFLAGS="--sysroot=${STAGE1_SYSROOT}"

# perl wants system zlib
# openssl wants system zlib
WORKDIR /sources/zlib
RUN patch -p1 < /patches/zlib/hack-fix-failed-shared-build.patch \
    && ./configure --prefix=${STAGE1_SYSROOT}/usr \
    && make -j64 \
    && make install \
    && rm -v ${STAGE1_SYSROOT}/lib/libz.a

# perl wants system bzip2
WORKDIR /sources/bzip2
RUN sed -i -e 's@CC=gcc@CC=clang@' -e 's@\(ln -s -f \)$(PREFIX)/bin/@\1@' Makefile* \
    && make -f Makefile-libbz2_so \
    && make clean \
    && make -j64 \
    && make PREFIX=${STAGE1_SYSROOT}/usr install \
    && cp -av bzip2-shared ${STAGE1_SYSROOT}/usr/bin/bzip2 \
    && ln -sfv bzip2       ${STAGE1_SYSROOT}/usr/bin/bzcat \
    && ln -sfv bzip2       ${STAGE1_SYSROOT}/usr/bin/bunzip2 \
    && cp -av libbz2.so.1.0.8 ${STAGE1_SYSROOT}/usr/lib/libbz2.so.1.0.8 \
    && ln -sv libbz2.so.1.0.8 ${STAGE1_SYSROOT}/usr/lib/libbz2.so.1.0 \
    && ln -sv libbz2.so.1.0   ${STAGE1_SYSROOT}/usr/lib/libbz2.so.1 \
    && ln -sv libbz2.so.1     ${STAGE1_SYSROOT}/usr/lib/libbz2.so \
    && rm -v ${STAGE1_SYSROOT}/lib/libbz2.a


# openssl needs perl
WORKDIR /sources/perl
RUN export BUILD_ZLIB=False BUILD_BZIP2=0 \
    && sh Configure -des \
        -D libc=${STAGE1_SYSROOT}/usr/lib/${MUSL_TRIPLE}/libc.so \
        -D prefix=/toolchain/usr \
        -D vendorprefix=/toolchain/usr \
        -D privlib=/toolchain/usr/lib/perl5/5.40/core_perl \
        -D archlib=/toolchain/usr/lib/perl5/5.40/core_perl \
        -D sitelib=/toolchain/usr/lib/perl5/5.40/site_perl \
        -D sitearch=/toolchain/usr/lib/perl5/5.40/site_perl \
        -D vendorlib=/toolchain/usr/lib/perl5/5.40/vendor_perl \
        -D vendorarch=/toolchain/usr/lib/perl5/5.40/vendor_perl \
        -D pager="/toolchain/usr/bin/less -isR" \
        -D useshrplib \
        -D usethreads \
    && make -j64 \
    && make install

# cargo needs openssl-sys
# cmake wants openssl
WORKDIR /sources/openssl
RUN CC=cc CXX=cxx ./config \
        --prefix=${STAGE1_SYSROOT}/usr \
        --openssldir=${STAGE1_SYSROOT}/etc/ssl \
        --libdir=lib \
        shared \
        zlib-dynamic \
    && make -j64 \
    && make install_sw

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

RUN ln -svnf ${STAGE1_SYSROOT} /toolchain
RUN rm -rf ${STAGE0_SYSROOT}
