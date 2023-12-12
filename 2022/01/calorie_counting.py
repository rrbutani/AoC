#!/usr/bin/env python3.11

from aoc2022 import *

elves = [ sum(int(i) for i in l.split("\n")) for l in inp().split("\n\n") ]

p1(max(elves))
p2(sum(sorted(elves)[-3:]))
