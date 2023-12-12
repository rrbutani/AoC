#!/usr/bin/env python3.10

from aoc2021 import *
from itertools import pairwise

p1(len([
    a for a, b in pairwise(
        int(l.strip()) for l in inp()
    ) if b > a
]))

def window(iter, n):
    for i in range(0, len(iter) - n + 1): yield iter[i:i+n]

p2(len([a for a, b, c, d in window(list(int(l.strip()) for l in inp()), 4) if (d) > (a)]))
