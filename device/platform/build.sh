#!/bin/bash
# This script is intended to be called from `cargo make` within cli/
# Generally, you will not invoke it manually.
set -euo pipefail
cd $(dirname $0)
if [ $CARGO_MAKE_RUST_TARGET_OS == "macos" ]; then
  macOS/build.sh
fi
