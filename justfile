test:
    diesel database reset
    sh .env.test
    cargo test
run:
    diesel database reset
    sh .env
    cargo run
release:
    diesel database reset
    sh .env
    cargo run --release
