#!/usr/bin/env python3.11

from aoc2022 import *
from functools import reduce

input: str = inp()
sacks = [ (l[0:(half := len(l) // 2)], l[half:]) for l in input.splitlines() ]

def score(l: int):
    if ord('a') <= l <= ord('z'):
        return l - ord('a') + 1
    else:
        return l - ord('A') + 26 + 1

p1_ans = sum( score(ord(char)) for left, right in sacks for char in set(left).intersection(right) )
p1(p1_ans)

s = sacks = [ set(l) for l in input.splitlines() ]
groups = [ (s[i], s[i + 1], s[i + 2]) for i in range(0, len(s), 3)]

p2_ans = sum( score(ord(c)) for group in groups for c in reduce((lambda a, b: a.intersection(b)), group) )
p2(p2_ans)
