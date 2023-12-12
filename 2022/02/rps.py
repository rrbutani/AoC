#!/usr/bin/env python3.11

from aoc2022 import *


input = """A Y
B X
C Z
"""
input: str = inp()

games = [  l.split(" ") for l in input.splitlines() ]

R = 0
P = 1
S = 2

# P > R
# R > S
# S > P

def score(opp, you):
    print(opp, you)
    you = ord(you) - ord('X')
    opp = ord(opp) - ord('A')
    score = 1 + you
    s = score
    # print(score)

    if opp == you:
        # draw
        score += 3
        # print(f"P:{s} D")
    elif (you == P and opp == R) or (you == R and opp == S) or (you == S and opp == P):
        score += 6
        # print(f"P:{s} W")
    else:
        # print(f"P:{s} L")
        pass

    print(score)
    print()

    return score

# p1_ans = sum(score(*g) for g in games)
# print(p1_ans)
# p1(p1_ans)

def pick(opp_m, result):
    print(opp_m, result)
    opp = ord(opp_m) - ord('A')
    res = ord(result) - ord('X')

    move = ""
    if res == 0:
        # Lose:
        t = {}
        t[R] = S
        t[P] = R
        t[S] = P

        move = t[opp]
    elif res == 1:
        # Draw:
        move = opp
    else:
        # Win:
        t = {}
        t[S] = R
        t[R] = P
        t[P] = S

        move = t[opp]

    return score(opp_m, chr(move + ord('X')))

p2_ans = sum(pick(*g) for g in games)
print(p2_ans)
p2(p2_ans)

# p1(max(elves))
# p2(sum(sorted(elves)[-3:]))
