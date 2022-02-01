FROM ubuntu:22.04

RUN apt-get update && \
    apt-get install -y \
    sqlite3 \
    wget \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && wget -v -O otel.deb https://github.com/open-telemetry/opentelemetry-collector-releases/releases/download/v0.43.0/otelcol_0.43.0_linux_$(uname -m).deb \
    && dpkg -i otel.deb

COPY ./backend/schedules.db /var/
COPY ./backend/target/release/train-backend /usr/bin/train-backend
COPY ./dist /var/www
COPY opentelemetry-collector.yml /etc/otel-collector/config.yaml

EXPOSE 8088

ENV STATIC_FILE_PATH=/var/www/

RUN echo 'vm.overcommit_memory=1' >> /etc/sysctl.conf

CMD ["bash", "-c", "otelcol & train-backend"]
