dev-api:
  cargo run -- api

dev-ingester:
  cargo run -- ingester

lexgen-rs:
  esquema-cli generate local -l lexicons -o crates/libs/lexicons/src

lexgen: lexgen-rs
