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
ENV CARGO_HOME=/cargo

# End BASE build
#####


#####
# BOOTSTRAP
FROM base AS stage0

# Create a new sysroot with glibc installed
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
        make \
        cmake \
        patch \
        python3 \
        ninja-build

# Install a base toolchain to build our bootstrap compiler from. We are using
# `gcc` and `g++` for the C compiler and C++ compiler respectively. You could
# swap out this with any other compiler that can build LLVM, at least in theory.
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y gcc g++

# Build LLVM
#
# The referenced cmake config files contain all the configuration options set
# to ensure this bootstrap compiler will be a minimal distribution of tools.
# Only the tools needed to bootstrap the musl toolchain are kept.
#
# The cmake files are also commented for additional reading.
WORKDIR /sources/llvm-project
COPY configs/common.cmake configs/glibc-base.cmake configs/glibc-stage1.cmake configs/glibc-stage2.cmake /
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

# Remove the host compiler
RUN apt-get autoremove --purge -y gcc g++

# Add the newly built sysroot to our PATH, no compiler or tools remain otherwise
ENV PATH="${GLIBC_SYSROOT}/usr/bin:${PATH}"

# Setup base filesystem structure
WORKDIR ${MUSL_SYSROOT}
RUN mkdir usr usr/bin usr/lib etc \
    && ln -sv usr/lib lib \
    && ln -sv usr/bin bin \
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
RUN ./configure CFLAGS="--sysroot=${GLIBC_SYSROOT}" CC=clang CXX=clang++ AR=llvm-ar RANLIB=llvm-ranlib --prefix="${MUSL_SYSROOT}/usr" \
    && make -j64 \
    && make install
RUN echo "${MUSL_SYSROOT}/usr/lib/${MUSL_TRIPLE}" > ${MUSL_SYSROOT}/etc/ld-musl-x86_64.path \
    && echo "${MUSL_SYSROOT}/usr/lib"            >> ${MUSL_SYSROOT}/etc/ld-musl-x86_64.path \
    && ln -svf libc.so ${MUSL_SYSROOT}/usr/lib/ld-musl-x86_64.so.1 \
    && ln -svf ../lib/libc.so ${MUSL_SYSROOT}/usr/bin/ldd

WORKDIR /sources/llvm-project
COPY configs/common.cmake configs/musl-base.cmake configs/musl-stage1.cmake configs/musl-stage2.cmake /
COPY patches/llvm-project /patches/llvm-project
RUN sed -ie '/^  OBJECT_LIBS RTSanitizerCommon RTSanitizerCommonLibc$/d' compiler-rt/lib/ctx_profile/CMakeLists.txt \
    && patch -p1 < /patches/llvm-project/toolchain-prefix.patch \
    && cmake -S llvm -B /opt/build -G Ninja \
        -D GLIBC_SYSROOT=${GLIBC_SYSROOT} \
        -D GLIBC_TRIPLE=${GLIBC_TRIPLE} \
        -D MUSL_SYSROOT=${MUSL_SYSROOT} \
        -D MUSL_TRIPLE=${MUSL_TRIPLE} \
        -C /musl-stage1.cmake \
        -C /musl-base.cmake \
        -C /common.cmake \
    && cmake --build /opt/build --target runtimes \
    && cmake --build /opt/build --target install-runtimes \
    && cmake --build /opt/build --target stage2-distribution \
    && cmake --build /opt/build --target stage2-install-distribution

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
    && ln -sv llvm-readelf bin/readelf \
    && ln -sv llvm-size    bin/size \
    && ln -sv llvm-strings bin/strings \
    && ln -sv llvm-strip   bin/strip

RUN rm -rf ${GLIBC_SYSROOT}
ENV PATH="${MUSL_SYSROOT}/usr/bin:${PATH}"
ENV CFLAGS="--sysroot=${MUSL_SYSROOT}"
ENV CXXFLAGS="--sysroot=${MUSL_SYSROOT}"
ENV LDFLAGS="--sysroot=${MUSL_SYSROOT}"

# only needed because im applying a patch before building `patch` after this
WORKDIR /sources/patch
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
#
# No additional host packages are used, but some host tooling is still used for
# the configuration step of building packages.
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

# rust bootstrap needs to extract tar.gz archives from pythons tarfile
# perl also needs this (to link against the system zlib)
WORKDIR /sources/zlib
COPY patches/zlib /patches/zlib
RUN patch -p1 < /patches/zlib/hack-fix-failed-shared-build.patch \
    && ./configure --prefix=${MUSL_SYSROOT}/usr \
    && make -j64 \
    && make install
RUN rm -fv ${MUSL_SYSROOT}/usr/lib/libz.a

# perl wants system bzip2
WORKDIR /sources/bzip2
RUN sed -i -e 's@CC=gcc@CC=clang@' -e 's@\(ln -s -f \)$(PREFIX)/bin/@\1@' Makefile* \
    && make -f Makefile-libbz2_so \
    && make clean \
    && make -j64 \
    && make PREFIX=${MUSL_SYSROOT}/usr install \
    && cp -av libbz2.so.* ${MUSL_SYSROOT}/usr/lib \
    && ln -sv libbz2.so.1.0.8 ${MUSL_SYSROOT}/usr/lib/libbz2.so \
    && cp -v bzip2-shared /usr/bin/bzip2 \
    && ln -sfv bzip2 ${MUSL_SYSROOT}/usr/bin/bzcat \
    && ln -sfv bzip2 ${MUSL_SYSROOT}/usr/bin/bunzip2 \
    && rm -fv ${MUSL_SYSROOT}/usr/lib/libbz2.a

# openssl needs perl
WORKDIR /sources/perl
RUN export BUILD_ZLIB=False BUILD_BZIP2=0 \
    && sh Configure -des \
        -D libc="${MUSL_SYSROOT}/usr/lib/${MUSL_TRIPLE}/libc.so" \
        -D prefix=${MUSL_SYSROOT}/usr \
        -D vendorprefix=${MUSL_SYSROOT}/usr \
        -D privlib=${MUSL_SYSROOT}/usr/lib/perl5/5.40/core_perl \
        -D archlib=${MUSL_SYSROOT}/usr/lib/perl5/5.40/core_perl \
        -D sitelib=${MUSL_SYSROOT}/usr/lib/perl5/5.40/site_perl \
        -D sitearch=${MUSL_SYSROOT}/usr/lib/perl5/5.40/site_perl \
        -D vendorlib=${MUSL_SYSROOT}/usr/lib/perl5/5.40/vendor_perl \
        -D vendorarch=${MUSL_SYSROOT}/usr/lib/perl5/5.40/vendor_perl \
        -D pager="${MUSL_SYSROOT}/usr/bin/less -isR" \
        -D useshrplib \
        -D usethreads \
    && make -j64 \
    && make install

# rust stage3 tries to compile openssl-sys. Install it early so python
WORKDIR /sources/openssl
RUN CC=cc CXX=cxx ./config --prefix=${MUSL_SYSROOT}/usr --openssldir=${MUSL_SYSROOT}/etc/ssl --libdir=lib shared zlib-dynamic \
    && make -j64 \
    && sed -i '/INSTALL_LIBS/s/libcrypto.a libssl.a//' Makefile \
    && make install_sw \
    && make install_ssldirs

# rust bootstrap uses python
WORKDIR /sources/python
RUN ./configure \
        --prefix=${MUSL_SYSROOT}/usr \
        --build=${MUSL_TRIPLE} \
        --host=${MUSL_TRIPLE} \
        --enable-shared \
        --without-ensurepip \
    && make -j64 \
    && make install

# rust stage3 needs pkg-config during openssl-sys install
WORKDIR /sources/pkgconf
RUN ./configure --prefix=${MUSL_SYSROOT}/usr --disable-static \
    && make -j64 \
    && make install
RUN ln -sv pkgconf ${MUSL_SYSROOT}/usr/bin/pkg-config

# TODO: cache these deps so rust bootstrap doesnt need curl
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install -y curl

# the stage0 compiler downloaded is linked against `libgcc_s` which we do not
# have. We can build a compat layer manually. We removed this after building
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

# HACK: downloaded stage0 is linked against the /lib instead of /toolchain/lib
RUN cp -av ${MUSL_SYSROOT}/etc/ld-musl-x86_64.path /etc/ld-musl-x86_64.path \
    && ln -sv ${MUSL_SYSROOT}/usr/lib/ld-musl-x86_64.so.1 /lib/ld-musl-x86_64.so.1

# FIXME: copy over LLVM build source for rust to reference
COPY --from=stage0 /opt/build /opt/build

WORKDIR /sources/rustc
COPY patches/rust /patches/rust
RUN --mount=type=cache,target=/rustbootstrapdeps,id=rust-bootstrap-cache \
    patch -p1 < /patches/rust/remove-dead-musl-static-code.patch \
    && patch -p1 < /patches/rust/musl-target-dynamic.patch \
    && ./configure \
        --tools=src,cargo,clippy,rustfmt \
        --set=build.bootstrap-cache-path=/rustbootstrapdeps \
        --set=build.jobs=64 \
        --set=build.docs=false \
        --set=build.host=${MUSL_TRIPLE} \
        --set=build.build=${MUSL_TRIPLE} \
        --set=build.target=${MUSL_TRIPLE} \
        --set=install.prefix=${MUSL_SYSROOT}/usr \
        --set=install.sysconfdir=etc \
        --set=llvm.download-ci-llvm=false \
        --set=llvm.use-libcxx=true \
        --set=rust.llvm-libunwind=system \
        --set=rust.lto=off \
        --set=rust.musl-root=${MUSL_SYSROOT}/usr \
        --set=target.${MUSL_TRIPLE}.crt-static=false \
        --set=target.${MUSL_TRIPLE}.llvm-config=/opt/build/tools/clang/stage2-bins/bin/llvm-config \
    && make -j64 \
    && make install

# HACK: Cleanup all our hacked in files. Everything should be linked correctly
#       against libunwind and `/toolchain` now. `libgcc_s` is no longer needed.
RUN rm -fv \
        ${MUSL_SYSROOT}/usr/lib/${MUSL_TRIPLE}/libgcc_s.so.1.0 \
        ${MUSL_SYSROOT}/usr/lib/${MUSL_TRIPLE}/libgcc_s.so.1 \
        ${MUSL_SYSROOT}/usr/lib/${MUSL_TRIPLE}/libgcc_s.so \
        /lib/ld-musl-x86_64.so.1 \
        /etc/ld-musl-x86_64.path
#        /usr/lib/Scrt1.o \
#        /usr/lib/rcrt1.o \
#        /usr/lib/crt1.o \
#        /usr/lib/crti.o \
#        /usr/lib/crtn.o \
#        /usr/lib/libc.a

# Build rust again with the patched musl-dynamic target
# HACK this is a nasty thing that needs to be called out. It changes the default
# musl target to be dynamically linked. Traditionally the rust target named
# `x86_64-unknown-linux-musl` would be statically linked, but we want dylib
# stuff. I could do a custom target, but then the names won't match for other
# tools. Given how this distro is being built up, this is a compromise for
# internal consistency vs consistency in the rust ecosystem.
WORKDIR /sources/rustc
COPY patches/rust /patches/rust
RUN --mount=type=cache,target=/rustbootstrapdeps,id=rust-bootstrap-cache \
    patch -p1 < /patches/rust/remove-dead-musl-static-code.patch \
    && patch -p1 < /patches/rust/musl-target-dynamic.patch \
    && ./configure \
        --tools=src,cargo,clippy,rustfmt \
        --set=build.bootstrap-cache-path=/rustbootstrapdeps \
        --set=build.jobs=64 \
        --set=build.docs=false \
        --set=build.cargo=${MUSL_SYSROOT}/usr/bin/cargo \
        --set=build.cargo-clippy=${MUSL_SYSROOT}/usr/bin/cargo-clippy \
        --set=build.rustc=${MUSL_SYSROOT}/usr/bin/rustc \
        --set=build.rustfmt=${MUSL_SYSROOT}/usr/bin/rustfmt \
        --set=build.host=${MUSL_TRIPLE} \
        --set=build.build=${MUSL_TRIPLE} \
        --set=build.target=${MUSL_TRIPLE} \
        --set=install.prefix=${MUSL_SYSROOT}/usr \
        --set=install.sysconfdir=etc \
        --set=llvm.download-ci-llvm=false \
        --set=llvm.use-libcxx=true \
        --set=rust.llvm-libunwind=system \
        --set=rust.lto=off \
        --set=rust.download-rustc=false \
        --set=rust.musl-root=${MUSL_SYSROOT}/usr \
        --set=target.${MUSL_TRIPLE}.llvm-config=/opt/build/tools/clang/stage2-bins/bin/llvm-config \
    && make -j64 \
    && make install

WORKDIR /sources/bash
COPY patches/bash /patches/bash
RUN patch -p1 < /patches/bash/fix-missing-header.patch && \
    ./configure \
        --prefix=${MUSL_SYSROOT}/usr \
        --build=${MUSL_TRIPLE} \
        --host=${MUSL_TRIPLE} \
        --without-bash-malloc \
    && make -j64 \
    && make install
RUN ln -sv bash ${MUSL_SYSROOT}/usr/bin/sh

ENV RUSTFLAGS="--sysroot=${MUSL_SYSROOT}"
# TODO setup cargo cache (and commit sccache build)
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    cargo install --root ${MUSL_SYSROOT}/usr --features unix \
        coreutils
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    cargo install --root ${MUSL_SYSROOT}/usr \
        diffutils \
        findutils

####
# OCI/docker/podman need /bin/sh available, but only /toolchain will exist in
# the root of our final FROM scratch image.
####
FROM base AS oci-compat
RUN mkdir -p /compat/hack /compat/bin
RUN ln -sv /sysroots/musl/usr/bin/sh /compat/bin/sh
RUN ln -sv /sysroots/musl /compat/hack/toolchain

####
# final
####
FROM scratch
ENV MUSL_TRIPLE=x86_64-unknown-linux-musl
ENV STAGE0_SYSROOT=/sysroots/musl
ENV STAGE1_SYSROOT=/sysroots/stage1

COPY --from=toolchain /sysroots/musl ${STAGE0_SYSROOT}
COPY --from=oci-compat /compat/bin /bin
COPY --from=oci-compat /compat/hack/ /
ENV PATH="${STAGE0_SYSROOT}/usr/bin"

# Setup symlinks to ALL THE THINGS
RUN coreutils ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/ln \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/ls \
    && ln -sv coreutils ${STAGE0_SYSROOT}/usr/bin/mkdir

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
