# syntax=docker/dockerfile:1
ARG FROM=localhost/phase0:latest

FROM ${FROM}

# steal ca-certs from curl image for cargo to use
COPY --from=docker.io/curlimages/curl:latest /etc/ssl /etc/ssl

# TODO: Remove the protocol allow once mirror is setup
RUN git config --global protocol.file.allow always \
    && git config --global init.defaultBranch main \
    && git config --global advice.detachedHead false

ENV CARGO_HOME=/cargo
ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/sccache

WORKDIR /git_sources/gtt
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase0 \
    cargo install --root /sysroots/phase0/usr --path ./

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
    RUST_BACKTRACE=1 cargo run || :
RUN false
# # container image compat hack
# RUN mkdir -p /compat/hack /compat/bin /compat/tmp \
#     && ln -sfv ${PHASE1_SYSROOT} /compat/hack/toolchain \
#     && ln -sfv ${PHASE1_SYSROOT}/usr/bin/sh /compat/bin/sh
# 
# FROM scratch
# COPY --from=phase1 /sysroots/phase1 /sysroots/phase1
# COPY --from=phase1 /compat/hack/ /
# COPY --from=phase1 /compat/bin /bin
# COPY --from=phase1 /compat/tmp /tmp
# ENV PATH="/toolchain/usr/bin"
