#!/usr/bin/env python3

l = [ int(x) for x in open("input")]
i = 0
c = 0

while(i < len(l)):
    jmp = l[i]
    l[i] = l[i] + 1
    i += jmp
    c += 1

print("P1: ", c)

l = [ int(x) for x in open("input")]
i = 0
c = 0

while(i < len(l)):
    jmp = l[i]
    if jmp >= 3:
        l[i] = l[i] - 1
    else:
        l[i] = l[i] + 1
    i += jmp
    c += 1

# (i += l[i]++) < len(l)

print("P2: ", c)