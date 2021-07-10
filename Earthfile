FROM rust:1.53-bullseye

chef:
    RUN cargo install cargo-chef

    WORKDIR /code

    COPY . .

    RUN cargo chef prepare --recipe-path recipe.json

    SAVE ARTIFACT /usr/local/cargo/bin/cargo-chef
    SAVE ARTIFACT recipe.json

frontend-base:
    RUN rustup target add wasm32-unknown-unknown

    RUN wget -qO- https://github.com/thedodd/trunk/releases/download/v0.11.0/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- && \
        mv /trunk /bin/trunk

    RUN wget -qO- https://github.com/rustwasm/wasm-bindgen/releases/download/0.2.74/wasm-bindgen-0.2.74-x86_64-unknown-linux-musl.tar.gz | tar -xzf- && \
        mv wasm-bindgen-0.2.74-x86_64-unknown-linux-musl/wasm* /bin/ && \
        rm -rf /wasm-bindgen-0.2.74-x86_64-unknown-linux-musl

    RUN apt-get update && \
        apt-get install -y \
            socat \
        && rm -rf /var/lib/apt/lists/*

    SAVE IMAGE frontend-base

frontend-serve:
    LOCALLY
    WITH DOCKER --load=+frontend-base
    RUN docker-compose up frontend
    END

backend-base:
    RUN rustup target add wasm32-unknown-unknown

    RUN wget -q https://github.com/watchexec/cargo-watch/releases/download/v7.8.0/cargo-watch-v7.8.0-x86_64-unknown-linux-gnu.deb && \
        dpkg -i cargo-watch-v7.8.0-x86_64-unknown-linux-gnu.deb && \
        rm cargo-watch-v7.8.0-x86_64-unknown-linux-gnu.deb

    SAVE IMAGE backend-base

backend-serve:
    LOCALLY
    WITH DOCKER --load=+backend-base
    RUN docker-compose up backend
    END

serve:
    BUILD +backend-serve
    BUILD +frontend-serve

backend-build:
    FROM +backend-base

    WORKDIR /code

    COPY +chef/cargo-chef /usr/bin/cargo-chef
    COPY +chef/recipe.json .

    RUN cargo chef cook --release -p train-backend --recipe-path recipe.json

    COPY . /code

    RUN cargo build --release -p train-backend

    SAVE ARTIFACT target/release/train-backend

frontend-build:
    FROM +frontend-base

    WORKDIR /code

    COPY +chef/cargo-chef /usr/bin/cargo-chef
    COPY +chef/recipe.json .

    RUN cargo chef cook --release --target wasm32-unknown-unknown -p train-frontend --recipe-path recipe.json

    COPY Trunk.toml index.html /code/
    COPY static static
    COPY frontend frontend
    COPY common common

    RUN trunk build --release

    SAVE ARTIFACT dist

docker:
    FROM ubuntu:20.10

    RUN apt-get update && \
        apt-get install -y \
            sqlite3 \
        && rm -rf /var/lib/apt/lists/*

    COPY +backend-build/train-backend /usr/bin/train-backend
    COPY +frontend-build/dist/* /var/www/
    COPY +route-database/schedules.db /var/

    ARG EARTHLY_GIT_HASH

    CMD /usr/bin/train-backend

    SAVE IMAGE --push lilymara/train-schedules:$EARTHLY_GIT_HASH
    SAVE IMAGE lilymara/train-schedules

serve-docker:
    LOCALLY
    WITH DOCKER --load=+docker
    RUN docker-compose up prebuilt-image
    END

route-database:
    FROM ubuntu:20.10

    RUN apt-get update && \
        apt-get install -y \
            sqlite3 \
            python3 \
            curl \
            zip \
        && rm -rf /var/lib/apt/lists/*

    COPY backend/import.py backend/migrate.py backend/new-db.sh .

    RUN ./new-db.sh

    SAVE ARTIFACT schedules.db

deploy:
    LOCALLY
    RUN earthly --push +docker

    RUN caprover deploy
