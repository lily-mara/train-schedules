FROM ubuntu:22.04

RUN apt-get update && \
    apt-get install -y \
    sqlite3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY ./backend/schedules.db /var/
COPY ./backend/target/release/train-backend /usr/bin/train-backend
COPY ./dist /var/www

EXPOSE 8088

ENV STATIC_FILE_PATH=/var/www/

RUN echo 'vm.overcommit_memory=1' >> /etc/sysctl.conf

CMD /usr/bin/train-backend
