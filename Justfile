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

md5:
    @find frontend/ backend/ -exec md5sum {} \; 2>/dev/null | md5sum | cut -d' ' -f1

deploy:
    rm -rf dist
    cargo build --release -p train-backend
    trunk build --release
    docker build -t registry.fly.io/trains:$(just md5) .
    docker push registry.fly.io/trains:$(just md5)
    flyctl deploy --image=registry.fly.io/trains:$(just md5)

clean:
    cargo clean
    rm -rf dist
