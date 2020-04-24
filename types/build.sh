#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"
pushd definitions
  ./build.sh
popd
cargo build
