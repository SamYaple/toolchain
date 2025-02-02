# syntax=docker/dockerfile:1
ARG FROM=docker.io/library/debian:bookworm-slim

# During phase0 we go from the host toolchain to an isolated musl sysroot with
# clang, llvm, and libc++ installed. At the very end of this file, the final
# image is crafted `FROM scratch`. None of the host toolchain files end up in
# produced image.
FROM ${FROM} as base

# The buildkit cache used for apt is setup here, and used throughout the
# Dockerfile. The `docker-clean` apt configuration conflicts with this way of
# caching and must be removed. This is the only time `apt-get update`.
COPY debian.sources /etc/apt/sources.list.d/
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    rm /etc/apt/apt.conf.d/docker-clean \
    && apt-get update

ENV TRIPLE=x86_64-phiban-linux-musl
ENV PHASE0_SYSROOT=/sysroots/phase0

FROM base as bootstrap
# These tools are nice to have, but not strictly required to build LLVM
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y git ccache

# The ccache data is stored in a buildkit style volume mount. As long as the
# build directory isn't changing, it will cache both stage1 and stage2 builds.
#
# You'll thank me if you ever have a failed build... otherwise this is just
# burning a bit of extra space. `sccache` is used instead of `ccache` in later
# phases as a unified caching tool for rust and clang.
ENV CCACHE_DIR=/ccache
ENV CCACHE_CPP2=yes
ENV CCACHE_HASHDIR=yes

# Install a base toolchain to build our bootstrap compiler from. We are using
# `gcc` and `g++` for the C compiler and C++ compiler respectively. You could
# swap out this with any other compiler that can build LLVM, at least in theory.
#
# `libc6-dev` is required to bootstrap our initial clang against, since we do
# not have musl installed or built yet.
#
# `cmake` and `python3` are required to build llvm, and `ninja-build` is
# recommended instead of `make` by the LLVM team.
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y \
        gcc \
        g++ \
        libc6-dev \
        cmake \
        python3 \
        ninja-build

# The cmake file `llvm.cmake` is commented for further reading.
WORKDIR /git_sources/llvm-project
COPY llvm.cmake /
RUN --mount=type=cache,target=${CCACHE_DIR},id=ccache-bootstrap \
    cmake -S llvm -B build -G Ninja \
        -D CMAKE_C_COMPILER_LAUNCHER=ccache \
        -D CMAKE_CXX_COMPILER_LAUNCHER=ccache \
        -D BUILD_SYSROOT="" \
        -D BUILD_TRIPLE=x86_64-unknown-linux-gnu \
        -D TARGET_SYSROOT="" \
        -D TARGET_TRIPLE=x86_64-unknown-linux-gnu \
        -D STATIC_CORE=ON \
        -D LLVM_BUILD_LTO=OFF \
        -C /llvm.cmake \
    && cmake --build build --target install-runtimes \
    && cmake --build build --target stage2-install-distribution

# Remove debian host packages that were only used to build llvm. This includes
# binutils as well.
RUN apt-get autoremove --purge -y gcc g++

# Add our new linker to a known path since we no longer have one. `ar` and
# `ranlib` are used during the musl build. Normally the symlink would have been
# created for this by the LLVM install, but at install time there would be a
# conflict with `binutils`. In later steps, we do not create this manually.
RUN ln -sv lld /usr/bin/ld \
    && ln -sv clang /usr/bin/cc \
    && ln -sv clang /usr/bin/c++ \
    && ln -sv llvm-ar /usr/bin/ar \
    && ln -sv llvm-ar /usr/bin/ranlib

# Install make for musl and linux headers bootstrapping
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y make

# Setup base filesystem structure
WORKDIR ${PHASE0_SYSROOT}
RUN mkdir usr usr/bin usr/lib etc \
    && ln -sv usr/lib lib \
    && ln -sv usr/bin bin \
    && ln -sv . toolchain \
    && ln -sv ${PHASE0_SYSROOT} /toolchain

# We need the linux headers to build musl (unlike with glibc). We build them
# using our bootstrap compiler. TODO: detail the what/when/hows of this process
WORKDIR /sources/linux
RUN make LLVM=1 -j64 headers \
    && find usr/include -type f ! -name '*.h' -delete \
    && cp -rv usr/include ${PHASE0_SYSROOT}/usr/include

# Build and install musl libc using our bootstrap tools. A small twist is having
# `/toolchain/etc/ld-musl-x86_64.path` look into `/toolchain/usr` vs `/usr`.
# This further isolates our new toolchain from any host files.
WORKDIR /sources/musl
RUN ./configure --prefix="${PHASE0_SYSROOT}/usr" \
    && make -j64 \
    && make install \
    && echo "/toolchain/usr/lib" > ${PHASE0_SYSROOT}/etc/ld-musl-x86_64.path \
    && ln -svf libc.so ${PHASE0_SYSROOT}/usr/lib/ld-musl-x86_64.so.1 \
    && ln -svf ../lib/libc.so ${PHASE0_SYSROOT}/usr/bin/ldd \
    && rm -v /lib/ld-musl-x86_64.so.1

RUN apt-get autoremove --purge -y make

RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install --no-install-recommends -y patch

# The cmake file `llvm.cmake` is commented for further reading.
WORKDIR /git_sources/llvm-project
COPY llvm.cmake /
COPY patches/llvm-project /patches/llvm-project
RUN --mount=type=cache,target=${CCACHE_DIR},id=ccache-phase0 \
    patch -p1 < /patches/llvm-project/toolchain-prefix.patch \
    && cmake -S llvm -B build -G Ninja \
        -D CMAKE_C_COMPILER_LAUNCHER=ccache \
        -D CMAKE_CXX_COMPILER_LAUNCHER=ccache \
        -D BUILD_SYSROOT="" \
        -D BUILD_TRIPLE=x86_64-unknown-linux-gnu \
        -D TARGET_SYSROOT=${PHASE0_SYSROOT} \
        -D TARGET_TRIPLE=${TRIPLE} \
        -D STATIC_CORE=ON \
        -C /llvm.cmake \
    && cmake --build build --target install-runtimes \
    && cmake --build build --target stage2-install-distribution \
    && ln -sv lld   ${PHASE0_SYSROOT}/usr/bin/ld \
    && ln -sv clang ${PHASE0_SYSROOT}/usr/bin/cc \
    && ln -sv clang ${PHASE0_SYSROOT}/usr/bin/c++ \
    && ln -sv clang ${PHASE0_SYSROOT}/usr/bin/cpp

# Removing libc6-dev should render the `/usr/bin/clang` compiler unusable.
# Additionally, the sysroot arguments are not *required* as the compiler will
# search for files under `/toolchain/usr` now, but for safety we set --sysroot.
RUN apt-get autoremove --purge -y libc6-dev

ENV PATH="${PHASE0_SYSROOT}/usr/bin:${PATH}"
ENV   CFLAGS="--sysroot=${PHASE0_SYSROOT}"
ENV CXXFLAGS="--sysroot=${PHASE0_SYSROOT}"
ENV  LDFLAGS="--sysroot=${PHASE0_SYSROOT}"

WORKDIR /sources/make
COPY patches/make /patches/make
RUN patch -p1 < /patches/make/fix-bootstrap.patch \
    && ./configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --build=${TRIPLE} \
        --host=${TRIPLE} \
    && ./build.sh \
    && ./make install

RUN apt-get autoremove --purge -y patch

WORKDIR /sources/patch
RUN ./configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --build=${TRIPLE} \
        --host=${TRIPLE} \
    && make -j64 \
    && make install

# Reset the build image! Copy only the sysroot over, which is isolated
FROM base as phase0
COPY --from=bootstrap ${PHASE0_SYSROOT} ${PHASE0_SYSROOT}
RUN ln -sv ${PHASE0_SYSROOT} /toolchain
ENV PATH="${PHASE0_SYSROOT}/usr/bin:${PATH}"

ENV   CFLAGS="--sysroot=${PHASE0_SYSROOT} -O3 -march=native"
ENV CXXFLAGS="--sysroot=${PHASE0_SYSROOT} -O3 -march=native"
ENV  LDFLAGS="--sysroot=${PHASE0_SYSROOT}"

WORKDIR /sources/pkgconf
RUN ./configure --prefix=${PHASE0_SYSROOT}/usr --disable-static \
    && make -j64 \
    && make install
RUN ln -sv pkgconf ${PHASE0_SYSROOT}/usr/bin/pkg-config

# rust bootstrap needs to extract tar.gz archives from pythons tarfile
# perl wants system zlib
# openssl wants system zlib
WORKDIR /sources/zlib
COPY patches/zlib /patches/zlib
RUN patch -p1 < /patches/zlib/hack-fix-failed-shared-build.patch \
    && ./configure --prefix=${PHASE0_SYSROOT}/usr \
    && make -j64 \
    && make install \
    && rm -fv ${PHASE0_SYSROOT}/usr/lib/libz.a

# We need a shell. It is not optional if we want to keep building in a container
WORKDIR /sources/bash
COPY patches/bash /patches/bash
RUN patch -p1 < /patches/bash/fix-missing-header.patch \
    && ./configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --build=${TRIPLE} \
        --host=${TRIPLE} \
        --without-bash-malloc \
    && make -j64 \
    && make install

# perl wants system bzip2
WORKDIR /sources/bzip2
RUN sed -i -e 's@CC=gcc@CC=clang@' -e 's@\(ln -s -f \)$(PREFIX)/bin/@\1@' Makefile* \
    && make -f Makefile-libbz2_so \
    && make clean \
    && make -j64 \
    && make PREFIX=${PHASE0_SYSROOT}/usr install \
    && cp -av bzip2-shared ${PHASE0_SYSROOT}/usr/bin/bzip2 \
    && cp -av libbz2.so.* ${PHASE0_SYSROOT}/usr/lib \
    && ln -sv libbz2.so.1.0.8 ${PHASE0_SYSROOT}/usr/lib/libbz2.so \
    && ln -sfv bzip2 ${PHASE0_SYSROOT}/usr/bin/bzcat \
    && ln -sfv bzip2 ${PHASE0_SYSROOT}/usr/bin/bunzip2 \
    && rm -fv ${PHASE0_SYSROOT}/usr/lib/libbz2.a

# openssl needs perl
WORKDIR /sources/perl
RUN export BUILD_ZLIB=False BUILD_BZIP2=0 \
    && sh Configure -des \
        -D libc=__REPLACE_LIBDIR__/libc.so \
        -D prefix=__REPLACE_PREFIX__ \
        -D vendorprefix=__REPLACE_PREFIX__ \
        -D bin=__REPLACE_PREFIX__/bin \
        -D vendorbin=__REPLACE_PREFIX__/bin \
        -D vendorscript=__REPLACE_PREFIX__/bin \
        -D privlib=__REPLACE_LIBDIR__/perl5/5.40/core_perl \
        -D archlib=__REPLACE_LIBDIR__/perl5/5.40/core_perl \
        -D sitelib=__REPLACE_LIBDIR__/perl5/5.40/site_perl \
        -D sitearch=__REPLACE_LIBDIR__/perl5/5.40/site_perl \
        -D vendorlib=__REPLACE_LIBDIR__/perl5/5.40/vendor_perl \
        -D vendorarch=__REPLACE_LIBDIR__/perl5/5.40/vendor_perl \
        -D userelocatableinc \
        -D useshrplib \
        -D usethreads \
    && sed -i \
           -e 's|__REPLACE_PREFIX__|'${PHASE0_SYSROOT}'/usr|g' \
           -e 's|__REPLACE_LIBDIR__|'${PHASE0_SYSROOT}'/usr/lib|g' \
           Makefile \
    && sed -i \
           -e '/^[^=]*install[^=]*=/ s|__REPLACE_PREFIX__|'${PHASE0_SYSROOT}'/usr|g' \
           -e '/^[^=]*install[^=]*=/ s|__REPLACE_LIBDIR__|'${PHASE0_SYSROOT}'/usr/lib|g' \
           Policy.sh \
           config.sh \
    && sed -i \
           -e 's|__REPLACE_PREFIX__|.../..|g' \
           -e 's|__REPLACE_LIBDIR__|.../../lib|g' \
           config.sh \
           Policy.sh \
    && make -j64 \
    && make install

# cargo needs openssl-sys
# cmake wants openssl
WORKDIR /sources/openssl
RUN CC=cc CXX=cxx ${PHASE0_SYSROOT}/usr/bin/perl ./Configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --openssldir=${PHASE0_SYSROOT}/etc/ssl \
        --libdir=lib \
        shared \
        zlib-dynamic \
    && make -j64 \
    && make install_sw

WORKDIR /git_sources/libffi
RUN ./configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --build=${TRIPLE} \
        --host=${TRIPLE} \
    && make -j64 \
    && make install

WORKDIR /git_sources/ncurses
RUN ./configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --build=${TRIPLE} \
        --host=${TRIPLE} \
        --with-shared \
        --without-debug \
        --without-normal \
        --with-cxx-shared \
        --enable-pc-files \
        --with-pkg-config-libdir=${PHASE0_SYSROOT}/usr/lib/pkgconfig \
    && make -j64 \
    && make install

WORKDIR /git_sources/readline
RUN ./configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --build=${TRIPLE} \
        --host=${TRIPLE} \
        --disable-static \
        --with-ncurses \
    && make SHLIB_LIBS='-lncursesw' -j64 \
    && make install

WORKDIR /git_sources/sqlite
RUN ./configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --build=${TRIPLE} \
        --host=${TRIPLE} \
        --disable-static \
        --enable-fts4 \
        --enable-fts5 \
    && make -j64 \
    && make install

WORKDIR /git_sources/shadow
RUN ./configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --build=${TRIPLE} \
        --host=${TRIPLE} \
        --disable-static \
        --without-libbsd \
        --without-group-name-max-length \
        --without-tcp \
        --with-bcrypt \
        --with-yescrypt \
    && make -j64 \
    && make install

WORKDIR /git_sources/util-linux
RUN ./configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --build=${TRIPLE} \
        --host=${TRIPLE} \
    && make -j64 \
    && make install

# rust bootstrap uses python
WORKDIR /sources/python
RUN ./configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --build=${TRIPLE} \
        --host=${TRIPLE} \
        --without-ensurepip \
        --enable-shared \
    && make -j64 \
    && make install

# rust bootstrapping needs this (because of the target json?)
WORKDIR /sources/cmake
RUN ./bootstrap --parallel=64 -- -D CMAKE_INSTALL_PREFIX=${PHASE0_SYSROOT}/usr \
    && make -j64 \
    && make install

WORKDIR /sources/ninja
RUN cmake -B build \
          -D CMAKE_INSTALL_PREFIX=${PHASE0_SYSROOT}/usr \
          -D BUILD_TESTING=OFF \
    && cmake --build build --parallel 64 \
    && cmake --build build --target install

WORKDIR /git_sources/gettext
RUN ./configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --build=${TRIPLE} \
        --host=${TRIPLE} \
    && make -j64 \
    && make install

WORKDIR /git_sources/git
RUN ./configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --build=${TRIPLE} \
        --host=${TRIPLE} \
    && make -j64 \
    && make install

WORKDIR /git_sources/make
RUN ./configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --build=${TRIPLE} \
        --host=${TRIPLE} \
    && make -j64 \
    && make install

WORKDIR /sources/patch
RUN ./configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --build=${TRIPLE} \
        --host=${TRIPLE} \
    && make -j64 \
    && make install

WORKDIR /sources/gawk
RUN ./configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --build=${TRIPLE} \
        --host=${TRIPLE} \
        --disable-extensions \
    && make -j64 \
    && make install

WORKDIR /sources/grep
RUN ./configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --build=${TRIPLE} \
        --host=${TRIPLE} \
    && make -j64 \
    && make install

WORKDIR /sources/sed
RUN ./configure \
        --prefix=${PHASE0_SYSROOT}/usr \
        --build=${TRIPLE} \
        --host=${TRIPLE} \
    && make -j64 \
    && make install

# the stage0 rust compiler is linked against `libgcc_s` which we do not have.
# We can build a compat layer manually. We remove this after bootstrapping rust.
# Similarly, the /etc/ld-musl-x86_64.path and /lib/ld-musl-x86_64.so.1 paths are
# needed for the binary to find libc and other libs.
WORKDIR /git_sources/llvm-project/llvm-libgcc
RUN clang -E -xc gcc_s.ver.in -o gcc_s.ver \
    && clang --sysroot=${PHASE0_SYSROOT} -nostdlib \
        -Wl,--version-script,gcc_s.ver \
        -Wl,--whole-archive \
        ${PHASE0_SYSROOT}/usr/lib/clang/19/lib/${TRIPLE}/libclang_rt.builtins.a \
        ${PHASE0_SYSROOT}/usr/lib/libunwind.a \
        -Wl,-soname,libgcc_s.so.1.0 \
        -lc -shared -o libgcc_s.so.1.0 \
    && install -m 0644 ./libgcc_s.so.1.0 ${PHASE0_SYSROOT}/usr/lib/libgcc_s.so.1.0 \
    && ln -sv libgcc_s.so.1.0 ${PHASE0_SYSROOT}/usr/lib/libgcc_s.so.1 \
    && ln -sv libgcc_s.so.1   ${PHASE0_SYSROOT}/usr/lib/libgcc_s.so \
    && ln -sv ${PHASE0_SYSROOT}/lib/ld-musl-x86_64.so.1 /lib/ld-musl-x86_64.so.1 \
    && ln -sv ${PHASE0_SYSROOT}/etc/ld-musl-x86_64.path /etc/ld-musl-x86_64.path

# FIXME: curl gets installed for the inital rustc bootstrap
# TODO: cache these deps so rust bootstrap doesnt need curl
RUN --mount=type=cache,target=/var/cache/apt,id=apt \
    apt-get install -y curl sccache

ENV CARGO_HOME=/cargo
ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/sccache

WORKDIR /git_sources/rust
COPY patches/rust/add-phiban-linux-musl-target.patch /
COPY x86_64-phiban-linux-musl .
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase0 \
    --mount=type=cache,target=/rustbootstrapdeps,id=rust-bootstrap-cache \
    git apply /add-phiban-linux-musl-target.patch \
    && export BOOTSTRAP_SKIP_TARGET_SANITY=1 OPENSSL_DIR=${PHASE0_SYSROOT}/usr \
    && ./configure \
        --set=build.bootstrap-cache-path=/rustbootstrapdeps \
        --set=build.jobs=64 \
        --set=build.docs=false \
        --set=build.host=${TRIPLE} \
        --set=build.build=x86_64-unknown-linux-musl \
        --set=build.target=${TRIPLE} \
        --set=install.prefix=${PHASE0_SYSROOT}/usr \
        --set=install.sysconfdir=etc \
        --set=llvm.use-libcxx=true \
        --set=rust.llvm-libunwind=system \
        --set=rust.musl-root=${PHASE0_SYSROOT}/usr \
        --set=target.${TRIPLE}.llvm-config=${PHASE0_SYSROOT}/usr/bin/llvm-config \
        --set=target.x86_64-unknown-linux-musl.crt-static=false \
        --set=target.x86_64-unknown-linux-musl.llvm-config=${PHASE0_SYSROOT}/usr/bin/llvm-config \
    && ./x.py build -j64 \
    && ./x.py install

# HACK: Cleanup all our hacked in files. Everything should be linked correctly
#       against libunwind and `/toolchain` now. `libgcc_s` and `curl` are no
#       longer needed.
RUN apt-get autoremove --purge -y curl
RUN rm -v ${PHASE0_SYSROOT}/usr/lib/libgcc_s.so.1.0 \
          ${PHASE0_SYSROOT}/usr/lib/libgcc_s.so.1 \
          ${PHASE0_SYSROOT}/usr/lib/libgcc_s.so \
          /lib/ld-musl-x86_64.so.1 \
          /etc/ld-musl-x86_64.path

# cargo still needs ssl certs to continue... stealing them from curl image
COPY --from=docker.io/curlimages/curl:latest /etc/ssl /etc/ssl

# TODO: show patched sccache which allows sysroots. TooHard or TooEasy...
WORKDIR /git_sources/sccache
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase0 \
    cargo install --root ${PHASE0_SYSROOT}/usr --path ./ --features native-zlib

RUN apt-get autoremove --purge -y sccache

WORKDIR /git_sources/diffutils
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase0 \
    cargo install --root ${PHASE0_SYSROOT}/usr --path ./

WORKDIR /git_sources/findutils
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase0 \
    cargo install --root ${PHASE0_SYSROOT}/usr --path ./

WORKDIR /git_sources/coreutils
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase0 \
    cargo install --root ${PHASE0_SYSROOT}/usr --path ./ --features unix


# Setup symlinks to ALL THE THINGS (or some of them... honestly there are a lot)
WORKDIR ${PHASE0_SYSROOT}/usr/bin
RUN ln -sv bash sh \
    && ln -sv diffutils cmp \
    && ln -sv coreutils basename \
    && ln -sv coreutils cat \
    && ln -sv coreutils chgrp \
    && ln -sv coreutils chmod \
    && ln -sv coreutils chown \
    && ln -sv coreutils comm \
    && ln -sv coreutils cp \
    && ln -sv coreutils cut \
    && ln -sv coreutils date \
    && ln -sv coreutils dirname \
    && ln -sv coreutils echo \
    && ln -sv coreutils env \
    && ln -sv coreutils expr \
    && ln -sv coreutils head \
    && ln -sv coreutils ln \
    && ln -sv coreutils ls \
    && ln -sv coreutils mkdir \
    && ln -sv coreutils mv \
    && ln -sv coreutils pwd \
    && ln -sv coreutils readlink \
    && ln -sv coreutils rm \
    && ln -sv coreutils rmdir \
    && ln -sv coreutils sleep \
    && ln -sv coreutils sort \
    && ln -sv coreutils split \
    && ln -sv coreutils tail \
    && ln -sv coreutils tee \
    && ln -sv coreutils test \
    && ln -sv coreutils touch \
    && ln -sv coreutils tr \
    && ln -sv coreutils true \
    && ln -sv coreutils uname \
    && ln -sv coreutils uniq \
    && ln -sv coreutils wc

RUN echo '#!/bin/sh' > ${PHASE0_SYSROOT}/usr/bin/ldconfig \
    && chmod +x ${PHASE0_SYSROOT}/usr/bin/ldconfig \
    && echo 'root:x:0:0:root:/root:/bin/bash' > ${PHASE0_SYSROOT}/etc/passwd \
    && echo 'root:x:0:' > ${PHASE0_SYSROOT}/etc/group \
    && echo 'bin:x:1:' >> ${PHASE0_SYSROOT}/etc/group \
    && echo 'sys:x:2:' >> ${PHASE0_SYSROOT}/etc/group \
    && echo 'kmem:x:3:' >> ${PHASE0_SYSROOT}/etc/group \
    && echo 'tty:x:4:' >> ${PHASE0_SYSROOT}/etc/group \
    && echo 'tape:x:5:' >> ${PHASE0_SYSROOT}/etc/group \
    && echo 'daemon:x:6:' >> ${PHASE0_SYSROOT}/etc/group \
    && echo 'floppy:x:7:' >> ${PHASE0_SYSROOT}/etc/group \
    && echo 'disk:x:8:' >> ${PHASE0_SYSROOT}/etc/group \
    && echo 'lp:x:9:' >> ${PHASE0_SYSROOT}/etc/group \
    && echo 'dialout:x:10:' >> ${PHASE0_SYSROOT}/etc/group \
    && echo 'audio:x:11:' >> ${PHASE0_SYSROOT}/etc/group \
    && echo 'video:x:12:' >> ${PHASE0_SYSROOT}/etc/group \
    && echo 'utmp:x:13:' >> ${PHASE0_SYSROOT}/etc/group \
    && echo 'usb:x:14:' >> ${PHASE0_SYSROOT}/etc/group \
    && echo 'cdrom:x:15:' >> ${PHASE0_SYSROOT}/etc/group

WORKDIR /git_sources/git-warp-time
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase0 \
    cargo install --root ${PHASE0_SYSROOT}/usr --path ./

WORKDIR /git_sources/gtt
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase0 \
    cargo install --root ${PHASE0_SYSROOT}/usr --path .

# When building with Docker/Podman the `RUN` statements are executed with
# `/bin/sh -c 'somecommand -with args'`. We need a symlink setup in the root
# `/bin/sh` -> `/toolchain/bin/sh`. We need to setup the appropriate symlinks
# and then `COPY` them into the container before `RUN` will work. We use the
# busybox image because it is tiny, but any image with `mkdir` and `ln` will
# work for this building this compat layer.
#
# /compat/hack/toolchain -> ${SYSROOT}
# /compat/bin/sh         -> ${SYSROOT}/usr/bin/sh
# /compat/tmp/
#
# TODO: patch mktemp to use /toolchain/tmp?
RUN mkdir -p /compat/hack /compat/bin /compat/tmp \
    && ln -sv ${PHASE0_SYSROOT} /compat/hack/toolchain \
    && ln -sv ${PHASE0_SYSROOT}/usr/bin/sh /compat/bin/sh

FROM scratch
COPY --from=phase0 /sysroots/phase0 /sysroots/phase0
COPY --from=phase0 /compat/hack/ /
COPY --from=phase0 /compat/bin /bin
COPY --from=phase0 /compat/tmp /tmp
ENV PATH="/toolchain/usr/bin"

# TODO: Remove the protocol allow once mirror is setup
RUN mkdir /root \
    && export HOME=/root \
    && git config --global init.defaultBranch main \
    && git config --global advice.detachedHead false \
    && git config --global protocol.file.allow always
