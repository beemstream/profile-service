test:
    diesel migration run
    sh .env.test
    cargo test && cargo test -- --ignored --test-threads=1
run:
    diesel migration run
    sh .env
    cargo run
release:
    diesel migration run
    sh .env
    cargo run --release
