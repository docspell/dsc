#!/usr/bin/env bash

set -e

export QEMU_OPTS="-m 2048"
export QEMU_NET_OPTS "hostfwd=tcp::7880-:7880"
./result/bin/run-dsctest-vm
