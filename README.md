===

LLVM Musl toolchain build

```bash
./fetch_sources.sh
./build.sh
podman run --rm -it toolchain
bash-5.2# which ldd
/toolchain/usr/bin/ldd
bash-5.2# which bash
/toolchain/usr/bin/bash
bash-5.2# ldd `which bash`
        /toolchain/lib/ld-musl-x86_64.so.1 (0x7f9115360000)
        libc.so => /toolchain/lib/ld-musl-x86_64.so.1 (0x7f9115360000)
bash-5.2# ldd `which clang`
        /toolchain/lib/ld-musl-x86_64.so.1 (0x7f2abd1ff000)
        libc++.so.1 => /toolchain/lib/x86_64-unknown-linux-musl/libc++.so.1 (0x7f2ab7c0d000)
        libc++abi.so.1 => /toolchain/lib/x86_64-unknown-linux-musl/libc++abi.so.1 (0x7f2ab7bc6000)
        libunwind.so.1 => /toolchain/lib/x86_64-unknown-linux-musl/libunwind.so.1 (0x7f2ab7bb7000)
        libc.so => /toolchain/lib/ld-musl-x86_64.so.1 (0x7f2abd1ff000)
```
