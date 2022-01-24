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

dist:
    cargo build --release -p train-backend
    trunk build --release

deploy:
    just dist
    flyctl deploy

serve-docker:
    just dist
    docker build -t trains .
    docker run --rm -it -eAPI_KEY=local -p8088:8088 trains

clean:
    cargo clean
    rm -rf dist
