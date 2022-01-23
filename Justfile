default:
    @just --list

backend:
    cd backend && cargo watch -x run

frontend:
    trunk serve

bootstrap:
    cargo install trunk cargo-watch
    rustup target add wasm32-unknown-unknown
    cd backend && ./new-db.sh

deploy:
    rm -rf dist
    cargo build --release -p train-backend
    trunk build --release
    flyctl deploy

clean:
    cargo clean
    rm -rf dist
