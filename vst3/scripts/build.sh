#!/bin/bash
set -euo pipefail
cd $(dirname $0)
target=${target:-debug}

if grep -qEi "(Microsoft|WSL)" /proc/version &>/dev/null; then
  WSL_DETECTED=1
  echo "WSL detected"
else
  WSL_DETECTED=0
fi

extra_args=""
if [ $target == "release" ]; then
  extra_args="--$target"
fi
if [[ $WSL_DETECTED == "1" ]]; then
  set +e
  win_userprofile="$(cmd.exe /c "<nul set /p=%UserProfile%" 2>/dev/null)"
  set -euo pipefail
  win_userprofile=$(wslpath -u "$win_userprofile")
  cargo_bin="$win_userprofile/.cargo/bin/cargo.exe"
  echo "Using cargo.exe at $cargo_bin"
else
  cargo_bin=$(which cargo)
fi

$cargo_bin build $extra_args

outdir=../../target/$target/plugins/vst3

mkdir -p $outdir
vst_search_dir=$outdir
if [[ "$OSTYPE" == "darwin"* ]]; then
  echo "Packaging MacOS VST3 plugin..."
  ./osx-bundler.sh paradise ../target/$target/libparadise.dylib
  mkdir -p $outdir || true
  rm -rf $outdir/paradise.vst || true
  mv paradise.vst3 $outdir
  echo "Successfully VST3 built plugin for MacOS"
elif [[ $WSL_DETECTED == "1" ]]; then
  echo "Packaging Windows VST3 plugin..."
  rm $outdir/paradise.vst3 || true
  cp ../../target/$target/paradise_vst3.dll $outdir/paradise.vst3
  echo "Successfully built VST3 plugin for Windows"
  vst_search_dir=$(wslpath -w $vst_search_dir)
else
  echo "Unknown platform"
  exit 1
fi

echo "Remember to add $vst_search_dir to your VST3 search paths!"
