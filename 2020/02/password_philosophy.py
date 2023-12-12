#!/usr/bin/env python3.8

# 11:09AM
# 11:26AM
# 11:31AM

from aoc2020 import *


def line(l) -> Tuple[int, int, str, str]:
    parts = l.split(" ")
    expected_range = parts[0]
    lower, upper = expected_range.split("-")
    char = parts[1][0]
    string = parts[2]

    return (int(lower), int(upper), char, string)


i = list(map(line, inp().split("\n")))

check = lambda l, u, c, p: p.count(c) in range(l, u + 1)
p1(count(filter(lambda l: check(*l), i)))

check = lambda l, u, c, p: (p[l - 1] is c) ^ (p[u - 1] is c)
p2(count(filter(lambda l: check(*l), i)))
