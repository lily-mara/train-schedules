FROM ubuntu:22.04

RUN apt-get update && \
    apt-get install -y \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

COPY ./backend/schedules.db /var/
COPY ./target/release/train-backend /usr/bin/train-backend
COPY ./dist /var/www

CMD /usr/bin/train-backend
