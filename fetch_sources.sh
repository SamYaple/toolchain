#!/bin/bash

set -euxEo pipefail

FILES=(
    https://www.cpan.org/src/5.0/perl-5.40.0.tar.gz
    https://www.cpan.org/src/5.0/perl-5.40.0.tar.gz.sha256.txt
    # no sig file; expected sha256 is c740348f357396327a9795d3e8323bafd0fe8a5c7835fc1cbaba0cc8dfe7161f

    https://github.com/lz4/lz4/releases/download/v1.10.0/lz4-1.10.0.tar.gz
    https://github.com/lz4/lz4/releases/download/v1.10.0/lz4-1.10.0.tar.gz.sha256
    # no sig file; expected sha256 is 537512904744b35e232912055ccf8ec66d768639ff3abe5788d90d792ec5f48b

    https://github.com/Kitware/CMake/releases/download/v3.31.3/cmake-3.31.3.tar.gz
    https://github.com/Kitware/CMake/releases/download/v3.31.3/cmake-3.31.3-SHA-256.txt
    https://github.com/Kitware/CMake/releases/download/v3.31.3/cmake-3.31.3-SHA-256.txt.asc

    https://github.com/llvm/llvm-project/releases/download/llvmorg-19.1.6/llvm-project-19.1.6.src.tar.xz
    https://github.com/llvm/llvm-project/releases/download/llvmorg-19.1.6/llvm-project-19.1.6.src.tar.xz.sig

    https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.12.8.tar.xz
    https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.12.8.tar.sign

    https://www.kernel.org/pub/software/scm/git/git-2.48.1.tar.xz
    https://www.kernel.org/pub/software/scm/git/git-2.48.1.tar.sign

    https://musl.libc.org/releases/musl-1.2.5.tar.gz
    https://musl.libc.org/releases/musl-1.2.5.tar.gz.asc

    https://www.python.org/ftp/python/3.13.1/Python-3.13.1.tgz
    https://www.python.org/ftp/python/3.13.1/Python-3.13.1.tgz.asc

    https://static.rust-lang.org/dist/rustc-1.84.0-src.tar.xz
    https://static.rust-lang.org/dist/rustc-1.84.0-src.tar.xz.asc

    https://github.com/ccache/ccache/releases/download/v4.10.2/ccache-4.10.2.tar.gz
    https://github.com/ccache/ccache/releases/download/v4.10.2/ccache-4.10.2.tar.gz.asc

    https://github.com/gavinhoward/bc/releases/download/7.0.3/bc-7.0.3.tar.gz
    https://github.com/gavinhoward/bc/releases/download/7.0.3/bc-7.0.3.tar.gz.sig

    https://zlib.net/zlib-1.3.1.tar.gz
    https://zlib.net/zlib-1.3.1.tar.gz.asc

    https://github.com/tukaani-project/xz/releases/download/v5.6.3/xz-5.6.3.tar.gz
    https://github.com/tukaani-project/xz/releases/download/v5.6.3/xz-5.6.3.tar.gz.sig

    https://www.sourceware.org/pub/bzip2/bzip2-1.0.8.tar.gz
    https://www.sourceware.org/pub/bzip2/bzip2-1.0.8.tar.gz.sig

    https://ftpmirror.gnu.org/bash/bash-5.2.37.tar.gz
    https://ftpmirror.gnu.org/bash/bash-5.2.37.tar.gz.sig

    https://ftpmirror.gnu.org/make/make-4.4.1.tar.gz
    https://ftpmirror.gnu.org/make/make-4.4.1.tar.gz.sig

    https://ftpmirror.gnu.org/m4/m4-1.4.19.tar.gz
    https://ftpmirror.gnu.org/m4/m4-1.4.19.tar.gz.sig

    https://ftpmirror.gnu.org/gawk/gawk-5.3.1.tar.gz
    https://ftpmirror.gnu.org/gawk/gawk-5.3.1.tar.gz.sig

    https://ftpmirror.gnu.org/grep/grep-3.11.tar.gz
    https://ftpmirror.gnu.org/grep/grep-3.11.tar.gz.sig

    https://ftpmirror.gnu.org/gzip/gzip-1.13.tar.gz
    https://ftpmirror.gnu.org/gzip/gzip-1.13.tar.gz.sig

    https://ftpmirror.gnu.org/patch/patch-2.7.6.tar.gz
    https://ftpmirror.gnu.org/patch/patch-2.7.6.tar.gz.sig

    https://ftpmirror.gnu.org/sed/sed-4.9.tar.gz
    https://ftpmirror.gnu.org/sed/sed-4.9.tar.gz.sig

    https://ftpmirror.gnu.org/tar/tar-1.35.tar.gz
    https://ftpmirror.gnu.org/tar/tar-1.35.tar.gz.sig

    https://ftpmirror.gnu.org/ncurses/ncurses-6.5.tar.gz
    https://ftpmirror.gnu.org/ncurses/ncurses-6.5.tar.gz.sig

    https://ftpmirror.gnu.org/gettext/gettext-0.23.tar.gz
    https://ftpmirror.gnu.org/gettext/gettext-0.23.tar.gz.sig

    https://ftpmirror.gnu.org/bison/bison-3.8.2.tar.gz
    https://ftpmirror.gnu.org/bison/bison-3.8.2.tar.gz.sig

    https://ftpmirror.gnu.org/texinfo/texinfo-7.2.tar.gz
    https://ftpmirror.gnu.org/texinfo/texinfo-7.2.tar.gz.sig

    https://ftpmirror.gnu.org/readline/readline-8.2.13.tar.gz
    https://ftpmirror.gnu.org/readline/readline-8.2.13.tar.gz.sig

    https://ftpmirror.gnu.org/libtool/libtool-2.5.4.tar.gz
    https://ftpmirror.gnu.org/libtool/libtool-2.5.4.tar.gz.sig

    https://ftpmirror.gnu.org/coreutils/coreutils-9.5.tar.gz
    https://ftpmirror.gnu.org/coreutils/coreutils-9.5.tar.gz.sig

    https://ftpmirror.gnu.org/findutils/findutils-4.10.0.tar.xz
    https://ftpmirror.gnu.org/findutils/findutils-4.10.0.tar.xz.sig
)

mkdir -p sources
cd sources
wget -nc "${FILES[@]}"
for pkg in *.tar.gz *.tar.xz *.tgz; do
    tar --skip-old-files -xf "${pkg}"
done
