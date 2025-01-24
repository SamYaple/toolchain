#!/bin/bash

set -euxEo pipefail

build() {
    local phase=$1
    shift

    podman build \
        --disable-compression \
        --volume ${PWD}/sources:/sources:O \
        --volume ${PWD}/git_sources:/git_sources:O \
        --tag phase0 \
        --file Dockerfile-${phase} \
        phases
}

build phase0
#build phase1
