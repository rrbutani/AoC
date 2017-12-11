#!/usr/bin/env python3

path = [x for x in open("input").readline().strip().split(',')]

steps = {
    'n'  : ( 1,  0),
    'ne' : ( 0.5,  1),
    'se' : (-0.5,  1),
    's'  : (-1,  0),
    'sw' : (-0.5, -1),
    'nw' : ( 0.5, -1),
}

def dist(pos):
    return abs(pos[1]) + (abs(pos[0]) - abs(pos[1] * 0.5))

pos = (0, 0)
maxDist = dist(pos)

for mov in path:
    pos = [sum(x) for x in zip(pos, steps[mov])]
    maxDist = max(maxDist, dist(pos))

print(f"P1: {dist(pos)}")
print(f"P2: {maxDist}")