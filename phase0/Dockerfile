# syntax=docker/dockerfile:1
ARG FROM=docker.io/library/debian:bookworm-slim
ARG BOOTSTRAP_FROM=localhost/bootstrap:latest

FROM ${FROM}
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

COPY --from=${BOOTSTRAP_FROM} ${GLIBC_SYSROOT} ${GLIBC_SYSROOT}
ENV PATH="${GLIBC_SYSROOT}/usr/bin:${PATH}"

# Setup base filesystem structure
WORKDIR ${MUSL_SYSROOT}
RUN mkdir usr usr/bin usr/lib etc \
    && ln -sv usr/lib lib \
    && ln -sv usr/bin bin \
    && ln -sv . toolchain \
    && ln -sv ${MUSL_SYSROOT} /toolchain

# We need `make` for the linux headers and musl build
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y make

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

# We use the following utils for building LLVM
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y \
        patch \
        cmake \
        python3 \
        ninja-build

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

ENV PATH="${MUSL_SYSROOT}/usr/bin:${PATH}"
ENV CFLAGS="--sysroot=${MUSL_SYSROOT}"
ENV CXXFLAGS="--sysroot=${MUSL_SYSROOT}"
ENV LDFLAGS="--sysroot=${MUSL_SYSROOT}"

# We no longer use anything glibc in the build; trash it!
RUN rm -rf ${GLIBC_SYSROOT}

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
# FIXME: curl gets installed for the inital rustc bootstrap
# FIXME: we steal debians cacerts from /etc/ssl
FROM base AS toolchain
COPY --from=stage0 ${MUSL_SYSROOT} ${MUSL_SYSROOT}
RUN ln -sv ${MUSL_SYSROOT} /toolchain
ENV PATH="${MUSL_SYSROOT}/usr/bin:${PATH}"
ENV CFLAGS="--sysroot=${MUSL_SYSROOT}"
ENV CXXFLAGS="--sysroot=${MUSL_SYSROOT}"
ENV LDFLAGS="--sysroot=${MUSL_SYSROOT}"

# Bootstrap GNU `make`, using the `./build.sh` bootstrap script
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
# perl wants system zlib
# openssl wants system zlib
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
# FIXME: make perl search dir relative OR /toolchain
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

# cargo tries to compile openssl-sys. Install it early so python can also use
# it during its build for consistency throughout.
WORKDIR /sources/openssl
RUN CC=cc CXX=cxx ./config \
        --prefix=${MUSL_SYSROOT}/usr \
        --openssldir=${MUSL_SYSROOT}/etc/ssl \
        --libdir=lib \
        shared \
        zlib-dynamic \
    && make -j64 \
    && sed -i '/INSTALL_LIBS/s/libcrypto.a libssl.a//' Makefile \
    && make install_sw

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

# cargo dependency openssl-sys needs pkg-config
WORKDIR /sources/pkgconf
RUN ./configure --prefix=${MUSL_SYSROOT}/usr --disable-static \
    && make -j64 \
    && make install
RUN ln -sv pkgconf ${MUSL_SYSROOT}/usr/bin/pkg-config

#### RUST BOOTSTRAPPING start
# FIXME: copy over LLVM build source for rust to reference
COPY --from=stage0 /opt/build /opt/build

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
#       against libunwind and `/toolchain` now. `libgcc_s` and `curl` are no
#       longer needed.
RUN rm -fv \
        ${MUSL_SYSROOT}/usr/lib/${MUSL_TRIPLE}/libgcc_s.so.1.0 \
        ${MUSL_SYSROOT}/usr/lib/${MUSL_TRIPLE}/libgcc_s.so.1 \
        ${MUSL_SYSROOT}/usr/lib/${MUSL_TRIPLE}/libgcc_s.so \
        /lib/ld-musl-x86_64.so.1 \
        /etc/ld-musl-x86_64.path \
        apt-get autoremove --purge -y curl
#### RUST BOOTSTRAPPING end

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

RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install -y ca-certificates
RUN cp -av /etc/ssl /sysroots/musl/etc/ssl

####
# final
####
FROM scratch
COPY --from=toolchain /sysroots/musl /sysroots/musl
ENV PATH="/sysroots/musl/usr/bin"
