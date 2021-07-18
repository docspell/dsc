#!/usr/bin/env bash

if [ -z "$1" ]; then
    echo "Please specify a version"
    exit 1
fi

version="$1"
if [[ $version == v* ]]; then
    version="${version:1}"
fi

push=""
if [ -z "$2" ] || [ "$2" == "--push" ]; then
    push="$2"
    if [ ! -z "$push" ]; then
        echo "Running with $push !"
    fi
else
    echo "Don't understand second argument: $2"
    exit 1
fi

if ! docker buildx version > /dev/null; then
    echo "The docker buildx command is required."
    echo "See: https://github.com/docker/buildx#binary-release"
    exit 1
fi

set -e
cd "$(dirname "$0")"

trap "{ docker buildx rm dsc-builder; }" EXIT

platforms="linux/amd64"
docker buildx create --name dsc-builder --use

if [[ $version == *SNAPSHOT* ]]; then
    echo ">>>> Building nightly images for $version <<<<<"
    url_base="https://github.com/eikek/docspell/releases/download/nightly"

    echo "============ Building dsc.dockerfile ============"
    docker buildx build \
           --platform="$platforms" $push \
           --build-arg version=$version \
           --tag dsc:nightly \
           -f dsc.dockerfile .
else
    echo ">>>> Building release images for $version <<<<<"
    echo "============ Building dsc ============"
    docker buildx build \
           --platform="$platforms" $push \
           --build-arg version=$version \
           --tag dsc:v$version \
           --tag dsc:latest \
           -f dsc.dockerfile .

fi
