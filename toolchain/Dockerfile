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
    rm /etc/apt/apt.conf.d/docker-clean \
    && apt-get update

# Setup some common variables used throughout the build process, but not in the
# final produced image. No trickery going on here, just some helper vars.
ENV GLIBC_SYSROOT=/sysroots/glibc
ENV GLIBC_TRIPLE=x86_64-unknown-linux-gnu
ENV MUSL_SYSROOT=/sysroots/musl
ENV MUSL_TRIPLE=x86_64-unknown-linux-musl
ENV CCACHE_DIR=/ccache
ARG CCACHE_BUILD=OFF
ENV CCACHE_BUILD=${CCACHE_BUILD}

# End BASE build
#####


#####
# BOOTSTRAP

FROM base AS bootstrap

# Install a base toolchain to build our bootstrap compiler from. We are using
# `gcc` and `g++` for the C compiler and C++ compiler respectively. You could
# swap out this with any other compiler that can build LLVM, at least in theory.
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y \
        ccache \
        cmake \
        g++ \
        gcc \
        ninja-build \
        patch \
        python3

# Create a new sysroot with glibc installed
WORKDIR ${GLIBC_SYSROOT}
# extract a pre-built glibc and headers. this step should work from a non-debian
# based system if you extract the deb files in the same way.
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get download libc6 libc6-dev linux-libc-dev
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

# Build LLVM
#
# The referenced cmake config files contain all the configuration options set
# to ensure this bootstrap compiler will be a minimal distribution of tools.
# Only the tools needed to bootstrap the musl toolchain are kept.
#
# The cmake files are also commented for additional reading.
#
# NOTE: fix-cmake-no-sanitizers-build.patch is a hotfix to allow a no-sanitizers
#       build to work. This was not a problem in llvm 18.x but I haven't looked
#       at the issue beyond making this quick patch.
WORKDIR /sources/llvm-project
COPY configs/common.cmake configs/glibc-base.cmake configs/glibc-stage1.cmake configs/glibc-stage2.cmake /
COPY patches/llvm-project /patches/llvm-project
RUN --mount=type=cache,target=${CCACHE_DIR},id=ccache-bootstrap \
    patch -p1 < /patches/llvm-project/fix-cmake-no-sanitizers-build.patch \
    && cmake -S llvm -B build -G Ninja \
        -D LLVM_CCACHE_BUILD=${CCACHE_BUILD} \
        -D LLVM_CCACHE_DIR=${CCACHE_DIR} \
        -D GLIBC_SYSROOT=${GLIBC_SYSROOT} \
        -D GLIBC_TRIPLE=${GLIBC_TRIPLE} \
        -C /glibc-stage1.cmake \
        -C /glibc-base.cmake \
        -C /common.cmake \
    && cmake --build build --target runtimes \
    && cmake --build build --target install-runtimes \
    && cmake --build build --target stage2-distribution \
    && cmake --build build --target stage2-install-distribution

# I could use the cmake build to setup these symlinks, but for now I am doing it
# directly. This will allow most of the remaining builds to detect which tools
# to use.
WORKDIR ${GLIBC_SYSROOT}
RUN ln -sv clang           bin/cc \
    && ln -sv clang++      bin/c++ \
    && ln -sv lld          bin/ld \
    && ln -sv llvm-ar      bin/ar \
    && ln -sv llvm-nm      bin/nm \
    && ln -sv llvm-objcopy bin/objcopy \
    && ln -sv llvm-objdump bin/objdump \
    && ln -sv llvm-ranlib  bin/ranlib \
    && ln -sv llvm-size    bin/size \
    && ln -sv llvm-strings bin/strings \
    && ln -sv llvm-strip   bin/strip

# End BOOTSTRAP build
#####


#####
# STAGE0

FROM base AS stage0

# We reuse the `base` image here, so we have no tools. gnumake and a
# functioning compiler is needed. We copy in our stage0 compiler and install
# make from a packaged source.
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y \
        cmake \
        ccache \
        ninja-build \
        python3 \
        patch \
        make

COPY --from=bootstrap ${GLIBC_SYSROOT} ${GLIBC_SYSROOT}
ENV PATH="${GLIBC_SYSROOT}/usr/bin:${PATH}"

WORKDIR ${MUSL_SYSROOT}
RUN mkdir usr usr/bin usr/lib etc \
    && ln -sv usr/lib lib \
    && ln -sv usr/bin bin \
    && ln -sv lib usr/lib64 \
    && ln -sv . toolchain \
    && ln -sv ${MUSL_SYSROOT} /toolchain

# We need the linux headers to build musl (unlike with glibc). We build them
# using our bootstrap compiler. TODO: detail the what/when/hows of this process
WORKDIR /sources/linux
RUN make HOSTCFLAGS="--sysroot=${GLIBC_SYSROOT}" HOSTLDFLAGS="--sysroot=${GLIBC_SYSROOT}" LLVM=1 -j64 headers \
    && find usr/include -type f ! -name '*.h' -delete \
    && cp -rv usr/include ${MUSL_SYSROOT}/usr/include

# Build and install musl libc using our bootstrap tools. A small twist is having
# `/toolchain/etc/ld-musl-x86_64.path` look into `/toolchain/usr` vs `/usr`.
# This further isolates our new toolchain from any host files.
WORKDIR /sources/musl
RUN ./configure CFLAGS="--sysroot=${GLIBC_SYSROOT}" --prefix="${MUSL_SYSROOT}/usr" \
    && make -j64 \
    && make install
RUN echo "${MUSL_SYSROOT}/usr/lib"                  >> ${MUSL_SYSROOT}/etc/ld-musl-x86_64.path \
    && echo "${MUSL_SYSROOT}/usr/lib/${MUSL_TRIPLE}" > ${MUSL_SYSROOT}/etc/ld-musl-x86_64.path \
    && ln -svf libc.so ${MUSL_SYSROOT}/usr/lib/ld-musl-x86_64.so.1 \
    && ln -svf ../lib/libc.so ${MUSL_SYSROOT}/usr/bin/ldd

WORKDIR /sources/llvm-project
COPY configs/common.cmake configs/musl-base.cmake configs/musl-stage1.cmake configs/musl-stage2.cmake /
COPY patches/llvm-project /patches/llvm-project
RUN --mount=type=cache,target=${CCACHE_DIR},id=ccache-stage0 \
    patch -p1 < /patches/llvm-project/fix-cmake-no-sanitizers-build.patch \
    && patch -p1 < /patches/llvm-project/toolchain-prefix.patch \
    && cmake -S llvm -B build -G Ninja \
        -D LLVM_CCACHE_BUILD=${CCACHE_BUILD} \
        -D LLVM_CCACHE_DIR=${CCACHE_DIR} \
        -D GLIBC_SYSROOT=${GLIBC_SYSROOT} \
        -D GLIBC_TRIPLE=${GLIBC_TRIPLE} \
        -D MUSL_SYSROOT=${MUSL_SYSROOT} \
        -D MUSL_TRIPLE=${MUSL_TRIPLE} \
        -C /musl-stage1.cmake \
        -C /musl-base.cmake \
        -C /common.cmake \
    && cmake --build build --target runtimes \
    && cmake --build build --target install-runtimes \
    && cmake --build build --target stage2-distribution \
    && cmake --build build --target stage2-install-distribution

# I could use the cmake build to setup these symlinks, but for now I am doing it
# directly. This will allow most of the remaining builds to detect which tools
# to use.
WORKDIR ${MUSL_SYSROOT}
RUN ln -sv clang           bin/cc \
    && ln -sv clang++      bin/c++ \
    && ln -sv lld          bin/ld \
    && ln -sv llvm-ar      bin/ar \
    && ln -sv llvm-nm      bin/nm \
    && ln -sv llvm-objcopy bin/objcopy \
    && ln -sv llvm-objdump bin/objdump \
    && ln -sv llvm-ranlib  bin/ranlib \
    && ln -sv llvm-size    bin/size \
    && ln -sv llvm-strings bin/strings \
    && ln -sv llvm-strip   bin/strip

# only needed because im applying patches to some of these source trees
WORKDIR /sources/patch
ENV PATH="${MUSL_SYSROOT}/usr/bin:${PATH}"
ENV CFLAGS="--sysroot=${MUSL_SYSROOT}"
ENV CXXFLAGS="--sysroot=${MUSL_SYSROOT}"
ENV LDFLAGS="--sysroot=${MUSL_SYSROOT}"
RUN ./configure \
        --prefix=${MUSL_SYSROOT}/usr \
        --build=${MUSL_TRIPLE} \
        --host=${MUSL_TRIPLE} \
    && make -j64 \
    && make install

# END STAGE0
####

#### 
# TOOLCHAIN

FROM base AS toolchain
COPY --from=stage0 ${MUSL_SYSROOT} ${MUSL_SYSROOT}
RUN ln -sv ${MUSL_SYSROOT} /toolchain
ENV PATH="${MUSL_SYSROOT}/usr/bin:${PATH}"
ENV CFLAGS="--sysroot=${MUSL_SYSROOT}"
ENV CXXFLAGS="--sysroot=${MUSL_SYSROOT}"
ENV LDFLAGS="--sysroot=${MUSL_SYSROOT}"

WORKDIR /sources/make
COPY patches/make /patches/make
RUN patch -p1 < /patches/make/fix-bootstrap.patch \
    && ./configure \
        --prefix=${MUSL_SYSROOT}/usr \
        --build=${MUSL_TRIPLE} \
        --host=${MUSL_TRIPLE} \
    && ./build.sh \
    && ./make install

WORKDIR /sources/zlib
COPY patches/zlib /patches/zlib
RUN patch -p1 < /patches/zlib/hack-fix-failed-shared-build.patch \
    && ./configure --prefix=${MUSL_SYSROOT}/usr \
    && make -j64 \
    && make install
RUN rm -fv ${MUSL_SYSROOT}/usr/lib/libz.a

WORKDIR /sources/python
RUN ./configure \
        --prefix=${MUSL_SYSROOT}/usr \
        --build=${MUSL_TRIPLE} \
        --host=${MUSL_TRIPLE} \
        --enable-shared \
        --without-ensurepip \
    && make -j64 \
    && make install

WORKDIR /sources/cmake
RUN ./bootstrap --parallel=64 -- \
        -D CMAKE_INSTALL_PREFIX=${MUSL_SYSROOT}/usr \
        -D CMAKE_USE_OPENSSL=OFF \
    && make -j64 \
    && make install

WORKDIR /sources/ninja
RUN ./configure.py --bootstrap \
    && ./ninja all \
    && install -m 0755 ./ninja ${MUSL_SYSROOT}/usr/bin/ninja

WORKDIR /sources/ccache/build
# TODO: I need to find a better way to provide these deps to ccahe during build
COPY ccachedeps /ccachedeps
RUN ln -s /ccachedeps/_deps _deps \
    && cmake \
        -D CMAKE_BUILD_TYPE=Release \
        -D CMAKE_INSTALL_PREFIX=${MUSL_SYSROOT}/usr \
        -D DOCTEST_INCLUDE_DIR=/ccachedeps/_deps .. \
    && make -j64 \
    && make install

WORKDIR /sources/llvm-project/llvm-libgcc
RUN clang -E -xc gcc_s.ver.in -o gcc_s.ver \
    && clang \
        -nostdlib \
	--sysroot=${MUSL_SYSROOT} \
        -Wl,--version-script,gcc_s.ver \
	-Wl,--whole-archive \
	${MUSL_SYSROOT}/usr/lib/clang/19/lib/${MUSL_TRIPLE}/libclang_rt.builtins.a \
	${MUSL_SYSROOT}/usr/lib/${MUSL_TRIPLE}/libunwind.a \
	-Wl,-soname,libgcc_s.so.1.0 \
	-lc \
	-shared \
	-o libgcc_s.so.1.0 \
    && install -m 0644 ./libgcc_s.so.1.0 ${MUSL_SYSROOT}/usr/lib/${MUSL_TRIPLE}/libgcc_s.so.1.0 \
    && ln -sv libgcc_s.so.1.0 ${MUSL_SYSROOT}/usr/lib/${MUSL_TRIPLE}/libgcc_s.so.1 \
    && ln -sv libgcc_s.so.1   ${MUSL_SYSROOT}/usr/lib/${MUSL_TRIPLE}/libgcc_s.so

RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install -y curl

WORKDIR /sources/rustc
COPY configs/rustc-config.toml /config.toml
RUN cp -av ${MUSL_SYSROOT}/etc/ld-musl-x86_64.path /etc/ld-musl-x86_64.path \
    && ln -sv ${MUSL_SYSROOT}/usr/lib/crt1.o  /usr/lib/ \
    && ln -sv ${MUSL_SYSROOT}/usr/lib/Scrt1.o /usr/lib/ \
    && ln -sv ${MUSL_SYSROOT}/usr/lib/rcrt1.o /usr/lib/ \
    && ln -sv ${MUSL_SYSROOT}/usr/lib/crti.o  /usr/lib/ \
    && ln -sv ${MUSL_SYSROOT}/usr/lib/crtn.o  /usr/lib/ \
    && ln -sv ${MUSL_SYSROOT}/usr/lib/libc.a  /usr/lib/ \
    && ln -sv ${MUSL_SYSROOT}/usr/lib/ld-musl-x86_64.so.1 /lib/ld-musl-x86_64.so.1
RUN --mount=type=cache,target=/rustbootstrapdeps,id=rust-bootstrap-cache \
    ./configure \
	--bootstrap-cache-path=/rustbootstrapdeps \
        --build=${MUSL_TRIPLE} \
        --host=${MUSL_TRIPLE} \
        --target=${MUSL_TRIPLE} \
        --prefix=${MUSL_SYSROOT}/usr \
        --llvm-cflags=${CFLAGS} \
        --llvm-cxxflags=${CXXFLAGS} \
        --llvm-ldflags=${CXXFLAGS} \
    && sed \
        -e "s|\${MUSL_TRIPLE}|${MUSL_TRIPLE}|g" \
        -e "s|\${MUSL_SYSROOT}|${MUSL_SYSROOT}|g" \
	/config.toml > config.toml \
    && make -j64 \
    && make install
RUN rm -fv /etc/ld-musl-x86_64.path /lib/ld-musl-x86_64.so.1 /usr/lib/crt1.o /usr/lib/Scrt1.o /usr/lib/rcrt1.o /usr/lib/crti.o /usr/lib/crtn.o /usr/lib/libc.a

#WORKDIR /sources/bash
#COPY patches/bash /patches/bash
#RUN patch -p1 < /patches/bash/fix-missing-header.patch && \
#    ./configure \
#        --prefix=/toolchain/usr \
#        --build=${MUSL_TRIPLE} \
#        --host=${MUSL_TRIPLE} \
#        --without-bash-malloc \
#    && make -j64 \
#    && make install
#RUN ln -sv bash /toolchain/bin/sh


#WORKDIR /sources/m4
#RUN ./configure
#        --prefix=/toolchain/usr
#        --host=${MUSL_TRIPLE} \
#    && make -j64 \
#    && make install

#WORKDIR /sources/gawk
## TODO: convert this to a patch
#RUN sed -ie 's|extras||' Makefile.in \
#    && ./configure \
#        --prefix=/toolchain/usr \
#        --build=${MUSL_TRIPLE} \
#        --host=${MUSL_TRIPLE} \
#        --disable-extensions \
#    && make -j64 \
#    && make install

#WORKDIR /sources/grep
#RUN ./configure \
#        --prefix=/toolchain/usr \
#        --build=${MUSL_TRIPLE} \
#        --host=${MUSL_TRIPLE} \
#    && make -j64 \
#    && make install

#WORKDIR /sources/gzip
#RUN ./configure \
#        --prefix=/toolchain/usr \
#        --host=${MUSL_TRIPLE} \
#    && make -j64 \
#    && make install

#WORKDIR /sources/patch
#RUN ./configure \
#        --prefix=/toolchain/usr \
#        --build=${MUSL_TRIPLE} \
#        --host=${MUSL_TRIPLE} \
#    && make -j64 \
#    && make install

#WORKDIR /sources/sed
#RUN ./configure \
#        --prefix=/toolchain/usr \
#        --build=${MUSL_TRIPLE} \
#        --host=${MUSL_TRIPLE} \
#    && make -j64 \
#    && make install

## TODO: remove FORCE_UNSAFE_CONFIGURE=1. this is in place because we are
##       building as root currently
#WORKDIR /sources/coreutils
#RUN FORCE_UNSAFE_CONFIGURE=1 ./configure \
#        --prefix=/toolchain/usr \
#        --build=${MUSL_TRIPLE} \
#        --host=${MUSL_TRIPLE} \
#    && make -j64 \
#    && make install

#WORKDIR /sources/findutils
#RUN ./configure \
#        --prefix=/toolchain/usr \
#        --build=${MUSL_TRIPLE} \
#        --host=${MUSL_TRIPLE} \
#    && make -j64 \
#    && make install

## TODO: remove FORCE_UNSAFE_CONFIGURE=1. this is in place because we are
##       building as root currently
#WORKDIR /sources/tar
#RUN FORCE_UNSAFE_CONFIGURE=1 ./configure \
#        --prefix=/toolchain/usr \
#        --host=${MUSL_TRIPLE} \
#    && make -j64 \
#    && make install

#WORKDIR /sources/xz
#RUN ./configure \
#        --prefix=/toolchain/usr \
#        --host=${MUSL_TRIPLE} \
#        --disable-static \
#    && make -j64 \
#    && make install
#RUN rm /toolchain/usr/lib/liblzma.la

#WORKDIR /sources/ccache/build
## TODO: I need to find a better way to provide these deps to ccahe during build
#COPY ccachedeps /ccachedeps
#RUN ln -s /ccachedeps/_deps _deps \
#    && cmake \
#        -D CMAKE_BUILD_TYPE=Release \
#        -D CMAKE_INSTALL_PREFIX=/toolchain/usr \
#        -D DOCTEST_INCLUDE_DIR=/ccachedeps/_deps .. \
#    && make -j64 \
#    && make install

####
# OCI/docker/podman need /bin/sh available, but only /toolchain will exist in
# the root of our final FROM scratch image.
####
FROM base AS oci-compat
RUN mkdir -p /compat/hack /compat/bin
RUN ln -sv /sysroots/stage0/usr/bin/sh /compat/bin/sh
RUN ln -sv /sysroots/stage0 /compat/hack/toolchain

####
# final
####
FROM scratch
ARG CCACHE_BUILD=ON
ENV CCACHE_DIR=/ccache
ENV MUSL_TRIPLE=x86_64-unknown-linux-musl
ENV STAGE0_SYSROOT=/sysroots/stage0
ENV STAGE1_SYSROOT=/sysroots/stage1

COPY --from=toolchain /sysroots/musl ${STAGE0_SYSROOT}
COPY --from=oci-compat /compat/bin /bin
COPY --from=oci-compat /compat/hack/ /
ENV PATH="${STAGE0_SYSROOT}/usr/bin"

RUN mkdir /tmp

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
COPY configs/common.cmake /
COPY patches/llvm-project /patches/llvm-project
RUN --mount=type=cache,target=${CCACHE_DIR},id=ccache-toolchain \
    patch -p1 < /patches/llvm-project/fix-cmake-no-sanitizers-build.patch \
    && cmake -S llvm -B build -G Ninja \
        -D LLVM_CCACHE_BUILD=${CCACHE_BUILD} \
        -D LLVM_CCACHE_DIR=${CCACHE_DIR} \
        -D CMAKE_SYSROOT=${STAGE1_SYSROOT} \
        -D CMAKE_INSTALL_PREFIX=${STAGE1_SYSROOT}/usr \
        -D LLVM_HOST_TRIPLE=${MUSL_TRIPLE} \
        -D LLVM_BUILTIN_TARGETS=${MUSL_TRIPLE} \
        -D LLVM_RUNTIME_TARGETS=${MUSL_TRIPLE} \
        -D RUNTIMES_${MUSL_TRIPLE}_LIBCXX_HAS_MUSL_LIBC=ON \
        -C /common.cmake \
    && cmake --build build --target runtimes \
    && cmake --build build --target install-runtimes

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
