#!/usr/bin/env -S poetry run python
import tomli_w
import readline
from pprint import pprint
import tomli
import sys

if len(sys.argv) < 2:
    print("Usage: edit_spheres.py <filename>")
    sys.exit(1)

fname = sys.argv[1]

with open(fname, 'rb') as f:
    data = tomli.load(f)


def add_neighbor(sphere_data, b):
    """ add b as a neighbor of a """
    for neighbor in sphere_data['neighbours']:
        if neighbor['id'] == 'b':
            break
    else:
        sphere_data['neighbours'].append({'id': b, 'angle': -1})


def add_edge(a, b):
    if not any(s['id'] == a for s in data['spheres']):
        data['spheres'].append({'id': a, 'space': -1, 'neighbours': []})

    if not any(s['id'] == b for s in data['spheres']):
        data['spheres'].append({'id': b, 'space': -1, 'neighbours': []})

    for sphere in data['spheres']:
        if sphere['id'] == a:
            add_neighbor(sphere, b)
        elif sphere['id'] == b:
            add_neighbor(sphere, a)


try:
    while (line := input('> ').strip()):
        cmd, *args = line.split(' ')
        match cmd:
            case 'show':
                pprint(data['spheres'])
            case 'add':
                fst, snd = args[0].split('-')
                add_edge(fst, snd)
            case 'write':
                data['spheres'].sort(key=lambda d: int(d['id']))
                with open(args[0], 'wb') as f:
                    tomli_w.dump(data, f)
                print(f'Wrote data to file {args[0]!r}')
            case 'help':
                print("Help:\n\tshow\t\tshow the current data\n\tadd ID1-ID2\tadd edge between ID1 and ID2\n\twrite outfile\twrite output to outfile\n\texit\t\texit")
            case 'exit' | 'quit':
                break
except EOFError:
    pass
