if [[ -n "${IDF_PATH}" ]]; then
  echo "Idf env not set! call idf.py"
  exit 1
fi
rm -Rf target .embuild
cargo build -p native_code && cargo build -p extra_code && cargo build -p app && bash flashme_s3.sh
