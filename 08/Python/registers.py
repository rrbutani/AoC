#!/usr/bin/env python3

import operator

OPS = {
    '<': operator.lt,
    '<=': operator.le,
    '>': operator.gt,
    '>=': operator.ge,
    '==': operator.eq,
    '!=': operator.ne,
}

ins = [i for i in [ x.split() for x in open("input")] ]

regs = {}

def reg(name):
    if name not in regs:
        regs[name] = 0
    return regs[name]

def condition(name, cond, val):
    print(reg(name), cond, int(val), " :: ", OPS[cond](reg(name), int(val)))
    return OPS[cond](reg(name), int(val))

maxv = 0

for i in ins:

    reg(i[0])

    if condition(i[4], i[5], i[6]):
        if(i[1] == 'inc'):
            regs[i[0]] += int(i[2])
        if(i[1] == 'dec'):
            regs[i[0]] -= int(i[2])

    if regs[max(regs, key=regs.get)] > maxv:
        maxv = regs[max(regs, key=regs.get)]


print("P1: ", regs[max(regs, key=regs.get)])
print("P2: ", maxv)
