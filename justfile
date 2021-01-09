test:
    diesel migration redo
    sh .env.test
    cargo test
run:
    diesel migration redo
    sh .env
    cargo run
release:
    diesel migration redo
    sh .env
    cargo run --release
