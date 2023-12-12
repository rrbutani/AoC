#!/usr/bin/env python3.8

from aoc2020 import *

expenses = set(map(int, inp().split("\n")))

for e in sorted(expenses):
    if (compl := 2020 - e) in expenses:
        p1(compl * e)
        break

for t in combinations(expenses, 3):
    if sum(t) == 2020:
        p2(prod(t))
        break
