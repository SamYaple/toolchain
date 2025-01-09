#!/bin/bash

set -euxEo pipefail

FILES=(
    https://github.com/llvm/llvm-project/releases/download/llvmorg-19.1.6/llvm-project-19.1.6.src.tar.xz
    https://github.com/llvm/llvm-project/releases/download/llvmorg-19.1.6/llvm-project-19.1.6.src.tar.xz.sig

    https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.12.8.tar.xz
    https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.12.8.tar.sign

    https://musl.libc.org/releases/musl-1.2.5.tar.gz
    https://musl.libc.org/releases/musl-1.2.5.tar.gz.asc

    https://www.python.org/ftp/python/3.13.1/Python-3.13.1.tgz
    https://www.python.org/ftp/python/3.13.1/Python-3.13.1.tgz.asc

    https://github.com/tukaani-project/xz/releases/download/v5.6.3/xz-5.6.3.tar.gz
    https://github.com/tukaani-project/xz/releases/download/v5.6.3/xz-5.6.3.tar.gz.sig

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
)

mkdir -p sources
cd sources
wget -nc "${FILES[@]}"
for pkg in *tar.gz *tar.xz; do
    tar --skip-old-files -xf "${pkg}"
done
