#!/bin/sh
echo "ESP_IDF_PATH is {${IDF_PATH}}"

[[ -v ESP_IDF_PATH ]] || {
  echo "ESP_IDF_PATH not  set" >&2
  exit 1
}
rm -Rf target .embuild
cargo build -p native_code --config 'env.ln_board="mini"' &&
  cargo build -p extra_code --config 'env.ln_board="mini"' &&
  cargo build -p app && bash flashme_s3.sh
