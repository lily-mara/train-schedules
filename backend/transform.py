#!/usr/bin/env python3

import sqlite3


def main():
    connection = sqlite3.connect('schedules.db')
    c = connection.cursor()

    rows = list(c.execute('select trip_id, stop_id, arrival_time, departure_time from stop_times'))

    for row in rows:
        trip_id, stop_id, arrival_time, departure_time = row

        placeholders = (
            time_string_to_minutes(departure_time),
            time_string_to_minutes(arrival_time),
            trip_id,
            stop_id,
        )

        c.execute(
            'update stop_times set departure_minute=?, arrival_minute=? where trip_id=? and stop_id=?',
            placeholders,
        )

    connection.commit()
    connection.close()




def time_string_to_minutes(time_str):
    parts = time_str.split(':')

    return int(parts[0]) * 60 + int(parts[1])


if __name__ == "__main__":
    main()