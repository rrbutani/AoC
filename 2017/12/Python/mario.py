#!/usr/bin/env python3

pipes =  [ [i.strip(',') for i in x.split()] for x in open("input")]
linked = set()

def addLinked(p, pipes, linked):
    p = int(p)

    if p not in linked:
        linked.add(p)
        for nxt in pipes[p][2:]:
            addLinked(nxt, pipes, linked)

addLinked(0, pipes, linked)

print(f"P1: {len(linked)}")

def lowestMissing(l):
    i = 0
    while i in l:
        i += 1
    return i

groups = 1
while len(linked) != len(pipes):
    addLinked(lowestMissing(linked), pipes, linked)
    groups += 1

print(f"P2: {groups}")
