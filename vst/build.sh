#!/bin/bash
set -euo pipefail
cd $(dirname $0)
target=release

if grep -qEi "(Microsoft|WSL)" /proc/version &>/dev/null; then
  WSL_DETECTED=1
  echo "WSL detected"
else
  WSL_DETECTED=0
fi

if [[ $WSL_DETECTED == "1" ]]; then
  set +e
  win_userprofile="$(cmd.exe /c "<nul set /p=%UserProfile%" 2>/dev/null)"
  set -e
  win_userprofile=$(wslpath -u "$win_userprofile")
  cargo_bin="$win_userprofile/.cargo/bin/cargo.exe"
  echo "Using cargo.exe at $cargo_bin"
  $cargo_bin build --$target
else
  cargo build --$target
fi

outdir=../target/$target/plugins
mkdir -p $outdir

if [[ "$OSTYPE" == "darwin"* ]]; then
  echo "Packaging MacOS plugin..."
  ./scripts/package_macos.sh paradise ../target/$target/libparadise.dylib
  mkdir -p $outdir || true
  rm -rf $outdir/paradise.vst || true
  mv paradise.vst $outdir
elif [[ $WSL_DETECTED == "1" ]]; then
  echo "Packaging Windows plugin..."
  rm $outdir/paradise.dll || true
  cp ../target/$target/paradise.dll $outdir
else
  echo "Unknown platform"
  exit 1
fi

#mkdir -p ../target/$target/paradise.vst/Contents/MacOS
#echo '<?xml version="1.0" encoding="UTF-8"?>
#<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
#<plist version="1.0">
#<dict>
#    <key>CFBundleName</key>
#    <string>Paradise</string>
#</dict>
#</plist>' > ../target/release/paradise.vst/Contents/Info.plist
#cp ../target/release/libparadise.dylib ../target/release/paradise.vst/Contents/MacOS/paradise
