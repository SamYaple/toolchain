#!/bin/bash

set -euxEo pipefail

FILES=(
    https://github.com/llvm/llvm-project/releases/download/llvmorg-19.1.6/llvm-project-19.1.6.src.tar.xz
    https://github.com/llvm/llvm-project/releases/download/llvmorg-19.1.6/llvm-project-19.1.6.src.tar.xz.sig

    https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.12.8.tar.xz
    https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.12.8.tar.sign

    https://ftpmirror.gnu.org/bash/bash-5.2.37.tar.gz
    https://ftpmirror.gnu.org/bash/bash-5.2.37.tar.gz.sig

    https://ftpmirror.gnu.org/make/make-4.4.1.tar.gz
    https://ftpmirror.gnu.org/make/make-4.4.1.tar.gz.sig

    https://musl.libc.org/releases/musl-1.2.5.tar.gz
    https://musl.libc.org/releases/musl-1.2.5.tar.gz.asc
)

mkdir -p sources
cd sources
wget -nc "${FILES[@]}"
