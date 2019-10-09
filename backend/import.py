#!/usr/bin/env python3

from glob import glob
import os.path

print('.mode csv')

for file in glob('data/*.txt'):
    print(f'.import {file} {os.path.splitext(os.path.basename(file))[0]}')

print("""
alter table stops add column station_id int;
""")
