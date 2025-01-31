# syntax=docker/dockerfile:1
ARG FROM=localhost/phase0:latest

FROM ${FROM} as phase1

# steal ca-certs from curl image for cargo to use
COPY --from=docker.io/curlimages/curl:latest /etc/ssl /etc/ssl

ENV CARGO_HOME=/cargo
ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/sccache

WORKDIR /git_sources/gtt
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase0 \
    cargo install --root /sysroots/phase0/usr --path ./

# TODO: Remove the protocol allow once mirror is setup
RUN mkdir /root \
    && export HOME=/root \
    && git config --global init.defaultBranch main \
    && git config --global advice.detachedHead false \
    && git config --global protocol.file.allow always

# util-linux has a script expecting /bin/bash
RUN ln -sv sh /bin/bash \
    && ln -sv /toolchain/etc/passwd /etc/passwd \
    && ln -sv /toolchain/etc/group /etc/group \
    && echo /sysroots/phase1/usr/lib >>  /toolchain/etc/ld-musl-x86_64.path
RUN mkdir -p /phiban/sources
WORKDIR /phiban-bootstrap
COPY Cargo.toml Cargo.toml
COPY src src
COPY patches /patches
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase1 \
    --mount=type=cache,target=/phiban/sources,id=phiban-git-sources \
    RUST_BACKTRACE=1 cargo run

# container image compat hack
RUN mkdir -p /compat/hack /compat/bin /compat/tmp \
    && ln -sfv /sysroots/phase1 /compat/hack/toolchain \
    && ln -sfv /sysroots/phase1/usr/bin/sh /compat/bin/sh

FROM scratch
COPY --from=phase1 /sysroots/phase1 /sysroots/phase1
COPY --from=phase1 /compat/hack/ /
COPY --from=phase1 /compat/bin /bin
COPY --from=phase1 /compat/tmp /tmp
ENV PATH="/toolchain/usr/bin"
