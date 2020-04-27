#!/bin/bash
## NOTE: if on Windows, run this script in WSL. It'll use cargo.exe
##
set -euo pipefail
cd $(dirname $0)
target=${target:-debug}
rm -rf ../target/$target/plugins &>/dev/null || true
../vst2/scripts/build.sh
../vst3/scripts/build.sh
echo "Successfully built all plugins"