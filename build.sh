#!/bin/bash

set -euxEo pipefail

podman build --disable-compression -v ${PWD}/sources:/sources:O --tag toolchain toolchain
