#!/bin/bash

set -euxEo pipefail

CCACHE_BUILD="${CCACHE_BUILD:-ON}"

podman build \
    --disable-compression \
    --volume ${PWD}/sources:/sources:O \
    --build-arg CCACHE_BUILD="${CCACHE_BUILD}" \
    --tag toolchain \
    toolchain
