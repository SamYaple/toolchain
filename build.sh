#!/bin/bash

set -euxEo pipefail

podman build \
    --disable-compression \
    --volume ${PWD}/sources:/sources:O \
    --tag toolchain \
    toolchain
