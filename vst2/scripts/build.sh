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
  set -euo pipefail
  win_userprofile=$(wslpath -u "$win_userprofile")
  cargo_bin="$win_userprofile/.cargo/bin/cargo.exe"
  echo "Using cargo.exe at $cargo_bin"
  $cargo_bin build --$target
else
  cargo build --$target
fi

outdir=../../target/$target/plugins

mkdir -p $outdir
vst_search_dir=$outdir
if [[ "$OSTYPE" == "darwin"* ]]; then
  echo "Packaging MacOS plugin..."
  ./package_macos.sh paradise ../target/$target/libparadise.dylib
  mkdir -p $outdir || true
  rm -rf $outdir/paradise.vst || true
  mv paradise.vst $outdir
  echo "Successfully built plugin for MacOS"
elif [[ $WSL_DETECTED == "1" ]]; then
  echo "Packaging Windows plugin..."
  rm $outdir/paradise.dll || true
  cp ../../target/$target/paradise.dll $outdir
  echo "Successfully built plugin for Windows"
  vst_search_dir=$(wslpath -w $vst_search_dir)
else
  echo "Unknown platform"
  exit 1
fi

echo "Remember to add $vst_search_dir to your VST search paths!"
