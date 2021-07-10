FROM rust:1.53

RUN rustup target add wasm32-unknown-unknown

RUN wget -qO- https://github.com/thedodd/trunk/releases/download/v0.11.0/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- && \
    mv /trunk /bin/trunk

RUN wget -qO- https://github.com/rustwasm/wasm-bindgen/releases/download/0.2.74/wasm-bindgen-0.2.74-x86_64-unknown-linux-musl.tar.gz | tar -xzf- && \
    mv wasm-bindgen-0.2.74-x86_64-unknown-linux-musl/wasm* /bin/ && \
    rm -rf /wasm-bindgen-0.2.74-x86_64-unknown-linux-musl
