FROM alpine:latest

ARG version=
ARG dsc_url=
ARG TARGETPLATFORM

WORKDIR /opt
RUN binary=""; \
    if [ "$TARGETPLATFORM" = "linux/amd64" ]; then binary="dsc_amd64-musl-$version"; fi; \
    if [ "$TARGETPLATFORM" = "linux/arm/v7" ]; then binary="dsc_armv7-$version"; fi; \
    if [ "$TARGETPLATFORM" = "linux/aarch64" ]; then binary="dsc_aarch64-$version"; fi; \
    wget ${dsc_url:-https://github.com/docspell/dsc/releases/download/v$version/$binary} && \
    mv "$binary" /usr/local/bin/dsc && \
    chmod 755 /usr/local/bin/dsc

RUN dsc --help > /dev/null
