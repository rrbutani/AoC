#!/usr/bin/env python3.8

# 11:36AM
# 11:45AM

from aoc2020 import *

grid = [[True if spot == "#" else False for spot in row] for row in inp().split("\n")]


def run(
    step: Tuple[int, int],
    grid: List[List[bool]] = grid,
    start: Tuple[int, int] = (0, 0),
) -> int:
    x, y = start
    dx, dy = step
    height, width = len(grid), len(grid[0])

    tree_count = 0
    while y < height:
        if grid[y][x % width]:
            tree_count += 1

        x += dx
        y += dy

    return tree_count


p1(run((3, 1)))
p2(prod(run(s) for s in [(1, 1), (3, 1), (5, 1), (7, 1), (1, 2)]))
