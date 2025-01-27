# syntax=docker/dockerfile:1
ARG FROM=localhost/phase0:latest

FROM ${FROM}

# steal ca-certs from curl image for cargo to use
COPY --from=docker.io/curlimages/curl:latest /etc/ssl /etc/ssl

ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/sccache
ENV CARGO_HOME=/cargo
ENV TRIPLE=x86_64-phiban-linux-musl
ENV PHASE0_SYSROOT=/sysroots/phase0
ENV PHASE1_SYSROOT=/sysroots/phase1
ENV PATH="${PHASE0_SYSROOT}/usr/bin"
ENV   CFLAGS="-O3 -march=native"
ENV CXXFLAGS="-O3 -march=native"

RUN mkdir -p /phiban/sources \
    && git config --global init.defaultBranch main \
    && git config --global advice.detachedHead false \
    && git config --global protocol.file.allow always

WORKDIR ${PHASE1_SYSROOT}
RUN mkdir usr usr/bin usr/lib etc \
    && ln -sv usr/lib lib \
    && ln -sv usr/bin bin \
    && ln -sv lib usr/lib64 \
    && ln -sv . toolchain

WORKDIR /git_sources/git-warp-time
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase1 \
    cargo install --root ${PHASE0_SYSROOT}/usr --path ./

COPY patches /patches
COPY configs /configs

WORKDIR /phiban-bootstrap
COPY Cargo.toml Cargo.toml
COPY src src
RUN --mount=type=cache,target=${CARGO_HOME},id=cargo \
    --mount=type=cache,target=${SCCACHE_DIR},id=sccache-phase1 \
    cargo run || :
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
