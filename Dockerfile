# syntax=docker/dockerfile:1
ARG FROM=localhost/phase0:latest

FROM ${FROM} as phase1

# steal ca-certs from curl image for cargo to use
COPY --from=docker.io/curlimages/curl:latest /etc/ssl /etc/ssl

ENV CARGO_HOME=/cargo
ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/sccache

# TODO: util-linux has a script expecting /bin/bash
# TODO: /etc/group and /etc/passwd  are not patched either
RUN ln -sv sh /bin/bash \
    && cp -av /toolchain/etc/passwd /etc/passwd \
    && cp -av /toolchain/etc/group /etc/group \
    && echo /sysroots/phase1/usr/lib >>  /toolchain/etc/ld-musl-x86_64.path

COPY patches /patches
WORKDIR /phiban-bootstrap
COPY Cargo.toml Cargo.toml
COPY src src
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase1 \
    --mount=type=cache,target=/phiban/sources,id=phiban-git-sources \
    RUST_BACKTRACE=1 cargo run

# container image compat hack
RUN mkdir -p /compat/bin /compat/tmp \
    && ln -sfv /toolchain/bin/sh /compat/bin/sh

FROM scratch
COPY --from=phase1 /sysroots/phase1 /toolchain
COPY --from=phase1 /compat/bin /bin
COPY --from=phase1 /compat/tmp /tmp
ENV PATH="/toolchain/usr/bin"
