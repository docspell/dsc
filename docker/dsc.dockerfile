FROM alpine:latest

ARG version=
ARG dsc_url=
ARG TARGETPLATFORM

WORKDIR /opt
RUN apk add --no-cache curl jq
RUN binary=""; release=""; \
    if [ "$TARGETPLATFORM" = "linux/amd64" ]; then binary="dsc_amd64-musl-$version"; fi; \
    if [ "$TARGETPLATFORM" = "linux/arm/v7" ]; then binary="dsc_armv7-$version"; fi; \
    if [ "$TARGETPLATFORM" = "linux/aarch64" ]; then binary="dsc_aarch64-musl-$version"; fi; \
    if [ "$TARGETPLATFORM" = "linux/arm64" ]; then binary="dsc_aarch64-musl-$version"; fi; \
    if [[ $version == "nightly" ]]; then release="nightly"; else release="v$version"; fi; \
    echo "Downloading ${dsc_url:-https://github.com/docspell/dsc/releases/download/$release/$binary} ..." && \
    curl --fail -o dsc -L ${dsc_url:-https://github.com/docspell/dsc/releases/download/$release/$binary} && \
    mv dsc /usr/local/bin/ && \
    chmod 755 /usr/local/bin/dsc
