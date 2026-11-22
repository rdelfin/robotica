FROM ubuntu:26.04

ARG TARGETARCH
ARG BAZELISK_VERSION=v1.30.0

RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential ca-certificates curl git \
    && rm -rf /var/lib/apt/lists/*

# TARGETARCH is "amd64" or "arm64", the same names Bazelisk uses in its releases.
RUN curl -fsSL -o /usr/local/bin/bazel \
    "https://github.com/bazelbuild/bazelisk/releases/download/${BAZELISK_VERSION}/bazelisk-linux-${TARGETARCH}" \
    && chmod +x /usr/local/bin/bazel
