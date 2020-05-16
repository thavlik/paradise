#!/bin/bash
set -euo pipefail
cd $(dirname $0)
if [ $CARGO_MAKE_RUST_TARGET_OS == "macos" ]; then
  macOS/build.sh
fi
