#!/usr/bin/bash
export COMMON="-DUSE_NO_DEFAULT=TRUE  -DUSE_CLANG=True   "
export PATH=/home/${USER}/.cargo/bin:/home/${USER}/.local/bin:$PATH
export PICO_SDK=/opt/pico/pico-sdk
export ROOT=$PWD

fail() {
  echo "**** FAILURE ${1} ***"
  exit -1
}

runbuild() {
  local cur_dir=$PWD
  local preset="${1}"
  rm -Rf target/
  bash build.sh --preset ${preset} || fail build_failed
  cp target/swindle_${preset} artefacts/
  gzip artefacts/swindle_${preset} || fail gzip
}

mkdir -p artefacts
rm -Rf artefacts
mkdir -p artefacts
runbuild esp32s3_dev
runbuild esp32s3_zero
runbuild esp32c3_zero
runbuild esp32c6_zero
runbuild esp32c6_alternatezero
