#!/bin/bash

docker-compose run --rm backend cargo build --release
container=$(docker-compose run -d backend sleep 10000000)

docker cp $container:/target/release/train-schedules .
docker rm -f $container