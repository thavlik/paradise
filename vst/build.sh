#!/bin/bash
set -euo pipefail
cd $(dirname $0)
target=release
cargo build --$target
rm -rf ../target/$target/paradise.vst || true
mkdir -p ../target/$target/paradise.vst/Contents/MacOS
echo '<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>Paradise</string>
</dict>
</plist>' > ../target/release/paradise.vst/Contents/Info.plist
cp ../target/release/libparadise.dylib ../target/release/paradise.vst/Contents/MacOS/paradise
