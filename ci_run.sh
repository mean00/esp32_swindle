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
  local cpu="${1}"
  local sz="${2}"
  rm -Rf target/
  bash build_all.sh ${cpu} ${sz} || fail build_failed
  cp target/swindle_${cpu}_${sz} artefacts/
  gzip artefacts/swindle_${cpu}_${sz} || fail gzip
}

mkdir -p artefacts
rm -Rf artefacts
mkdir -p artefacts
runbuild esp32s3 full
runbuild esp32s3 zero
runbuild esp32c3 zero
runbuild esp32c6 zero
