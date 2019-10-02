#!/usr/bin/env python3

from glob import glob
import os.path

print('.mode csv')

for file in glob('data/*.txt'):
    print(f'.import {file} {os.path.splitext(os.path.basename(file))[0]}')

print('alter table stop_times add column departure_minute integer;')
print('alter table stop_times add column arrival_minute integer;')