#!/bin/bash
set -euo pipefail
cd $(dirname $0)
PROFILE="${PROFILE:-debug}"
if [ $PROFILE == "debug" ]; then
  CONFIGURATION="Debug"
elif [ $PROFILE == "release" ]; then
  CONFIGURATION="Release"
else
  echo "Unknown profile '${PROFILE}'"
  exit 1
fi
xcodebuild -verbose \
  -configuration $CONFIGURATION \
  -scheme ${SCHEME:-ProxyAudioDevice} \
  -target ${TARGET:-ProxyAudioDevice}
