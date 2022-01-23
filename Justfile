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
    find frontend/ backend/ -exec md5sum {} \; 2>/dev/null | md5sum | cut -d' ' -f1

deploy:
    cargo build --release -p train-backend
    trunk build --release
    docker build -t lilymara/train-schedules:$(just md5) .
    docker push lilymara/train-schedules:$(just md5)
    echo '{ "imageName": "lilymara/train-schedules:'$(just md5)'", "schemaVersion": 2 }' > captain-definition
    caprover deploy -c captain-definition

clean:
    cargo clean
    rm -rf dist
