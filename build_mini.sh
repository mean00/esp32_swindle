rm -Rf target .embuild
cargo build -p native_code --config 'env.ln_board="mini"' &&
  cargo build -p extra_code --config 'env.ln_board="mini"' &&
  cargo build -p app && bash flashme_s3.sh
