# syntax=docker/dockerfile:1
ARG FROM=localhost/phase0:latest

FROM ${FROM}

# steal ca-certs from curl image for cargo to use
COPY --from=docker.io/curlimages/curl:latest /etc/ssl /etc/ssl

ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/sccache
ENV CARGO_HOME=/cargo

# TODO: Remove after next rebuild....
WORKDIR /git_sources/git-warp-time
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase1 \
    cargo install --root /toolchain/usr --path ./

# TODO: Remove the protocol allow once mirror is setup
RUN git config --global protocol.file.allow always \
    && git config --global init.defaultBranch main \
    && git config --global advice.detachedHead false

RUN mkdir -p /phiban/sources
COPY patches /patches
WORKDIR /phiban-bootstrap
COPY Cargo.toml Cargo.toml
COPY src src
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase1 \
    RUST_BACKTRACE=1 cargo run

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
