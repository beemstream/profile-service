test:
    diesel migration run
    sh .env.test
    cargo test
run:
    diesel migration run
    sh .env
    cargo run
release:
    diesel migration run
    sh .env
    cargo run --release
