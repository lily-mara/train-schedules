#!/bin/bash

set -e

mkdir -p data
pushd data
curl -Lo caltrain-ca-us.zip http://data.trilliumtransit.com/gtfs/caltrain-ca-us/caltrain-ca-us.zip
unzip caltrain-ca-us.zip
popd

./import.py | sqlite3 new-schedules.db
PYTHONHASHSEED=0 ./migrate.py new-schedules.db

mv new-schedules.db schedules.db
