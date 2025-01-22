# syntax=docker/dockerfile:1
ARG FROM=localhost/phase0:latest

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
FROM ${FROM} AS phase1

ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/sccache
ENV CARGO_HOME=/cargo
ENV MUSL_TRIPLE=x86_64-unknown-linux-musl
ENV STAGE0_SYSROOT=/sysroots/musl
ENV STAGE1_SYSROOT=/sysroots/stage1
ENV PATH="${STAGE0_SYSROOT}/usr/bin"

WORKDIR ${STAGE1_SYSROOT}
RUN mkdir usr usr/bin usr/lib etc \
    && ln -sv usr/lib lib \
    && ln -sv usr/bin bin \
    && ln -sv lib usr/lib64 \
    && ln -sv . toolchain

# Install linux headers (needed for musl libc)
# llvm sanitizers need linux headers for asm/param.h
WORKDIR /sources/linux
RUN make LLVM=1 -j64 headers \
    && find usr/include -type f ! -name '*.h' -delete \
    && cp -rv usr/include ${STAGE1_SYSROOT}/usr/include

# Install libc
WORKDIR /sources/musl
RUN ./configure --prefix="${STAGE1_SYSROOT}/usr" \
    && make -j64 \
    && make install

RUN echo "/toolchain/usr/lib" >> ${STAGE1_SYSROOT}/etc/ld-musl-x86_64.path \
    && ln -sf libc.so ${STAGE1_SYSROOT}/usr/lib/ld-musl-x86_64.so.1 \
    && ln -sf ../lib/libc.so ${STAGE1_SYSROOT}/usr/bin/ldd

# perl wants system zlib
# openssl wants system zlib
WORKDIR /sources/zlib
RUN ./configure --prefix=${STAGE1_SYSROOT}/usr \
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

# cargo needs openssl-sys
# cmake wants openssl
WORKDIR /sources/openssl
RUN CC=cc CXX=c++ perl ./Configure \
        --prefix=${STAGE1_SYSROOT}/usr \
        --openssldir=${STAGE1_SYSROOT}/etc/ssl \
        --libdir=lib \
        shared \
        zlib-dynamic \
    && make -j64 \
    && make install_sw

#RUN ln -sv . ${STAGE1_SYSROOT}/usr/include/${MUSL_TRIPLE} \
#    && ln -sv . ${STAGE1_SYSROOT}/usr/lib/${MUSL_TRIPLE}

WORKDIR /sources/llvm-project
COPY musl-llvm.cmake llvm-toolchain-prefix.patch /
RUN --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase1 \
    patch -p1 < /llvm-toolchain-prefix.patch \
    && cmake -S llvm -B /opt/llvm-build -G Ninja \
        -D CMAKE_C_COMPILER_LAUNCHER=sccache \
        -D CMAKE_CXX_COMPILER_LAUNCHER=sccache \
        -D CMAKE_SYSROOT=${STAGE1_SYSROOT} \
        -D DEFAULT_SYSROOT=${STAGE1_SYSROOT} \
        -D CMAKE_INSTALL_PREFIX=${STAGE1_SYSROOT}/usr \
        -C /musl-llvm.cmake \
    && cmake --build /opt/llvm-build --target runtimes \
    && cmake --build /opt/llvm-build --target install-runtimes \
    && cmake --build /opt/llvm-build --target stage2-distribution \
    || cat /git_sources/llvm-project/build/tools/clang/stage2-bins/CMakeFiles/CMakeError.log && false \
    && cmake --build /opt/llvm-build --target stage2-install-distribution

COPY --from=docker.io/curlimages/curl:latest /etc/ssl /etc/ssl
#ENV RUSTFLAGS="--sysroot=${STAGE0_SYSROOT}"
RUN ln -sv ../../${MUSL_TRIPLE}/c++/v1/__config_site ${STAGE0_SYSROOT}/usr/include/c++/v1/__config_site \
    && cd ${STAGE0_SYSROOT}/usr/lib/${MUSL_TRIPLE} \
    && for f in *; do ln -sv ${MUSL_TRIPLE}/${f} ../${f}; done

# Build rust again with the patched musl-dynamic target
WORKDIR /sources/rustc
COPY rust-dynamic-musl-target.patch /
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase1 \
    patch -p1 < /rust-dynamic-musl-target.patch \
    && ./configure \
        --set=build.jobs=64 \
        --set=build.cargo=${STAGE0_SYSROOT}/usr/bin/cargo \
        --set=build.cargo-clippy=${STAGE0_SYSROOT}/usr/bin/cargo-clippy \
        --set=build.rustc=${STAGE0_SYSROOT}/usr/bin/rustc \
        --set=build.rustfmt=${STAGE0_SYSROOT}/usr/bin/rustfmt \
        --set=build.host=${MUSL_TRIPLE} \
        --set=build.build=${MUSL_TRIPLE} \
        --set=build.target=${MUSL_TRIPLE} \
        --set=install.prefix=${STAGE1_SYSROOT}/usr \
        --set=install.sysconfdir=etc \
        --set=llvm.use-libcxx=true \
        --set=rust.llvm-libunwind=system \
        --set=target.${MUSL_TRIPLE}.llvm-config=${STAGE1_SYSROOT}/usr/bin/llvm-config \
    && python3 x.py build -j64 \
    && python3 x.py install

WORKDIR /sources/python
RUN ./configure \
        --prefix=${STAGE1_SYSROOT}/usr \
        --build=${MUSL_TRIPLE} \
        --host=${MUSL_TRIPLE} \
        --enable-optimizations \
        --with-lto \
        --enable-shared \
        --without-ensurepip \
    && make -j64 \
    && make install

WORKDIR /git_sources/sccache
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase1 \
    cargo install --root ${STAGE1_SYSROOT}/usr --path ./ --features native-zlib

WORKDIR /git_sources/coreutils
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase1 \
    cargo install --root ${STAGE1_SYSROOT}/usr --path ./ --features unix

WORKDIR /git_sources/findutils
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase1 \
    cargo install --root ${STAGE1_SYSROOT}/usr --path ./

WORKDIR /git_sources/diffutils
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase1 \
    cargo install --root ${STAGE1_SYSROOT}/usr --path ./

RUN mkdir -p /compat/hack /compat/bin /compat/tmp \
    && ln -sv ${STAGE1_SYSROOT}/usr/bin/sh /compat/bin/sh \
    && ln -sv ${STAGE1_SYSROOT} /compat/hack/toolchain

# ###
# # GNU tools
# WORKDIR /sources/make
# RUN ./configure \
#         --prefix=${STAGE1_SYSROOT}/usr \
#         --build=${MUSL_TRIPLE} \
#         --host=${MUSL_TRIPLE} \
#     && make -j64 \
#     && make install
# 
# WORKDIR /sources/gawk
# RUN ./configure \
#         --prefix=${STAGE1_SYSROOT}/usr \
#         --build=${MUSL_TRIPLE} \
#         --host=${MUSL_TRIPLE} \
#         --disable-extensions \
#     && make -j64 \
#     && make install
# 
# WORKDIR /sources/grep
# RUN ./configure \
#         --prefix=${STAGE1_SYSROOT}/usr \
#         --build=${MUSL_TRIPLE} \
#         --host=${MUSL_TRIPLE} \
#     && make -j64 \
#     && make install
# 
# WORKDIR /sources/sed
# RUN ./configure \
#         --prefix=${STAGE1_SYSROOT}/usr \
#         --build=${MUSL_TRIPLE} \
#         --host=${MUSL_TRIPLE} \
#     && make -j64 \
#     && make install
# 
# # TODO: Drop this once all sources exist in the git mirror. Use git apply after
# WORKDIR /sources/patch
# RUN ./configure \
#         --prefix=${STAGE1_SYSROOT}/usr \
#         --build=${MUSL_TRIPLE} \
#         --host=${MUSL_TRIPLE} \
#     && make -j64 \
#     && make install
# 
# # We need a shell. It is not optional if we want to keep building in a container
# WORKDIR /sources/bash
# COPY bash-fix-missing-header.patch /
# RUN patch -p1 < /bash-fix-missing-header.patch && \
#     ./configure \
#         --prefix=${STAGE1_SYSROOT}/usr \
#         --build=${MUSL_TRIPLE} \
#         --host=${MUSL_TRIPLE} \
#         --without-bash-malloc \
#     && make -j64 \
#     && make install
# RUN ln -sv bash ${STAGE1_SYSROOT}/usr/bin/sh
# # END GNU tools
# ###
# 
# WORKDIR /sources/perl
# RUN export BUILD_ZLIB=False BUILD_BZIP2=0 \
#     && sh Configure -des \
#         -D libc=__REPLACE_LIBDIR__/${MUSL_TRIPLE}/libc.so \
#         -D prefix=__REPLACE_PREFIX__ \
#         -D vendorprefix=__REPLACE_PREFIX__ \
#         -D bin=__REPLACE_PREFIX__/bin \
#         -D vendorbin=__REPLACE_PREFIX__/bin \
#         -D vendorscript=__REPLACE_PREFIX__/bin \
#         -D privlib=__REPLACE_LIBDIR__/perl5/5.40/core_perl \
#         -D archlib=__REPLACE_LIBDIR__/perl5/5.40/core_perl \
#         -D sitelib=__REPLACE_LIBDIR__/perl5/5.40/site_perl \
#         -D sitearch=__REPLACE_LIBDIR__/perl5/5.40/site_perl \
#         -D vendorlib=__REPLACE_LIBDIR__/perl5/5.40/vendor_perl \
#         -D vendorarch=__REPLACE_LIBDIR__/perl5/5.40/vendor_perl \
#         -D userelocatableinc \
#         -D useshrplib \
#         -D usethreads \
#     && sed -i \
#            -e 's|__REPLACE_PREFIX__|'${STAGE1_SYSROOT}'/usr|g' \
#            -e 's|__REPLACE_LIBDIR__|'${STAGE1_SYSROOT}'/usr/lib|g' \
#            Makefile \
#     && sed -i \
#            -e '/^[^=]*install[^=]*=/ s|__REPLACE_PREFIX__|'${STAGE1_SYSROOT}'/usr|g' \
#            -e '/^[^=]*install[^=]*=/ s|__REPLACE_LIBDIR__|'${STAGE1_SYSROOT}'/usr/lib|g' \
#            Policy.sh \
#            config.sh \
#     && sed -i \
#            -e 's|__REPLACE_PREFIX__|.../..|g' \
#            -e 's|__REPLACE_LIBDIR__|.../../lib|g' \
#            config.sh \
#            Policy.sh \
#     && make -j64 \
#     && make install
# 
# WORKDIR /sources/cmake
# RUN cmake \
#         -B build \
#         -D CMAKE_INSTALL_PREFIX=${STAGE1_SYSROOT}/usr \
#     && cmake --build build --parallel 64 \
#     && cmake --build build --target install
# 
# WORKDIR /sources/ninja
# RUN cmake \
#         -B build \
#         -D CMAKE_INSTALL_PREFIX=${STAGE1_SYSROOT}/usr \
#         -D BUILD_TESTING=OFF \
#     && cmake --build build --parallel 64 \
#     && cmake --build build --target install

FROM scratch
COPY --from=phase1 /sysroots/stage1 /sysroots/stage1
COPY --from=phase1 /compat/tmp /tmp
COPY --from=phase1 /compat/bin /bin
COPY --from=phase1 /compat/hack/ /
