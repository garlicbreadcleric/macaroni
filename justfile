check:
    cargo clippy -- -D warnings
    cargo fmt --check

test:
    cargo test

fmt:
    cargo fmt
    npx prettier -w 'macaroni-playground/public/*.{html,css,js,jsx,ts,tsx}'

play:
    cargo run --bin macaroni-playground

docs:
    cargo doc --document-private-items

docs-watch:
    cargo watch -i *.md -s "just docs"

docs-serve:
    http-server target/doc -p 5555 -o /macaroni
