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

outdir=../../target/$target/plugins/vst2

mkdir -p $outdir
vst_search_dir=$outdir
if [[ "$OSTYPE" == "darwin"* ]]; then
  echo "Packaging MacOS VST2 plugin..."
  ./package_macos.sh paradise ../../target/$target/libparadise_vst2.dylib
  mkdir -p $outdir || true
  rm -rf $outdir/paradise.vst || true
  mv paradise.vst $outdir
  echo "Successfully built VST2 plugin for MacOS"
elif [[ $WSL_DETECTED == "1" ]]; then
  echo "Packaging Windows VST2 plugin..."
  rm $outdir/paradise.dll || true
  cp ../../target/$target/paradise.dll $outdir
  echo "Successfully built VST2 plugin for Windows"
  vst_search_dir=$(wslpath -w $vst_search_dir)
else
  echo "Unknown platform"
  exit 1
fi

echo "Remember to add $vst_search_dir to your VST2 search paths!"
