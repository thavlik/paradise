#!/bin/bash
set -euo pipefail
cd $(dirname $0)
../vst2/scripts/build.sh
../vst3/scripts/build.sh
