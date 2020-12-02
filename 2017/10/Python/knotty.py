#!/usr/bin/env python3

from functools import reduce as red
from operator import xor as xor

iLens = [int(x) for x in open("input").readline().split(',')]
bLens = [ord(x) for x in open("input").readline().strip()]+[17, 31, 73, 47, 23]

def twist(m, s, l, sz):
    for i in range(l//2):
        m[(s+i) % sz], m[(s+l-1-i) % sz] = m[(s+l-1-i) % sz], m[(s+i) % sz]

def hashRound(pos, ss, circ, lenl):
    for l in lenl:
        twist(circ, pos, l, len(circ))

        pos += l + ss
        ss  += 1

    return (pos, ss)

# Part 1:
marks = list(range(256))

hashRound(0, 0, marks, iLens)
print(f"P1: {marks[0] * marks[1]}")

# Part 2:
pos, ss, sparse = 0, 0, list(range(256))

for i in range(64):
    pos, ss = hashRound(pos, ss, sparse, bLens)

dense = [red(xor, sparse[i:(i+16)], 0) for i in range(0, len(sparse), 16)]
kHash = ''.join(["%0.2x" % dd for dd in dense])

print(f"P2: {kHash}")
