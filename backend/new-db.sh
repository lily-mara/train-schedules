#!/bin/sh

./import.py | sqlite3 new-schedules.db
PYTHONHASHSEED=0 ./migrate.py new-schedules.db

mv new-schedules.db schedules.db