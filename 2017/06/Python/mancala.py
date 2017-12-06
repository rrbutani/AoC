#!/usr/bin/env python3

mbs = [int(i) for x in open("input") for i in x.split()]

configs = {}
rcs = 0

while tuple(mbs) not in configs:
    configs[tuple(mbs)] = rcs

    indx = mbs.index(max(mbs))
    blks = mbs[indx]
    mbs[indx] = 0

    for i in range(blks):
        mbs[(indx + i + 1) % len(mbs)] += 1

    rcs += 1

print(f"P1: {rcs}")
print(f"P2: {rcs - configs[tuple(mbs)]}")
