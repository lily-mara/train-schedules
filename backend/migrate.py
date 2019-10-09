#!/usr/bin/env python3

import sqlite3


def main():
    connection = sqlite3.connect('schedules.db')
    c = connection.cursor()

    rows = list(c.execute(
        'select stop_id, stop_name from stops'
    ))

    hashes = {}

    for row in rows:
        stop_id, stop_name = row

        station_id = hashes.get(stop_name)
        if station_id is None:
            station_id = hash(stop_name) % 1024
            while station_id in hashes:
                station_id += 1

            hashes[stop_name] = station_id

        print(stop_id, stop_name, station_id)

        c.execute(
            'update stops set station_id=? where stop_id=?',
            (
                station_id, stop_id,
            ),
        )

    connection.commit()
    connection.close()


if __name__ == "__main__":
    main()
