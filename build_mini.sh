if [[ -n "${IDF_PATH}" ]]; then
  echo "Idf env not set! call idf.py"
  exit 1
fi
rm -Rf target .embuild
cargo build -p native_code --config 'env.ln_board="mini"' &&
  cargo build -p extra_code --config 'env.ln_board="mini"' &&
  cargo build -p app && bash flashme_s3.sh
