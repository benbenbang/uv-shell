#!/bin/bash

docker buildx build \
              --platform linux/amd64,linux/arm64 \
              --push \
              --rm \
              -t ghcr.io/benbenbang/uv-shell/prek \
              -f ./scripts/image/Dockerfile \
              .
