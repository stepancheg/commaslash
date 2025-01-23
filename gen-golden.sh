#!/bin/sh -e

cd $(dirname $0)

cargo run -- \
  --macos-aarch64='
    url=https://github.com/protocolbuffers/protobuf/releases/download/v29.3/protoc-29.3-osx-aarch_64.zip
    path=bin/protoc size=2290929 sha256=2b8a3403cd097f95f3ba656e14b76c732b6b26d7f183330b11e36ef2bc028765
  ' \
  --linux-x86-64='
    url=https://github.com/protocolbuffers/protobuf/releases/download/v29.3/protoc-29.3-linux-x86_64.zip
    path=bin/protoc size=3288836 sha256=3e866620c5be27664f3d2fa2d656b5f3e09b5152b42f1bedbf427b333e90021a
  ' \
  --output=golden/protoc