#!/usr/bin/env bash
set -e

base=$(dirname "$(readlink -f "$0")")

start_docker() {
    cd $base/ci
    docker-compose -f docker-compose.yml up -d
    sleep 5
}

stop_docker() {
    cd $base/ci
    docker-compose -f docker-compose.yml down
    docker-compose -f docker-compose.yml kill
}

trap "{ stop_docker ; }" EXIT

start_docker

cargo test --test login
cargo test
