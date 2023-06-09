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
    cd backend && cargo build --release
    trunk build --release

deploy:
    just dist
    flyctl deploy

otel-collector:
    docker run \
    --env-file=otel.env \
    -p 14268:14268 \
    -p 4317-4318:4317-4318 \
    -v $(pwd)/opentelemetry-collector.yml:/etc/otel/config.yaml \
    otel/opentelemetry-collector-contrib:latest

serve-docker:
    just dist
    docker build -t trains .
    docker run --rm -it --env-file otel.env -eAPI_KEY=local -p8088:8088 trains

clean:
    cargo clean
    rm -rf dist
