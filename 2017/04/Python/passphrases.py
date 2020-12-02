#!/usr/bin/env python3

# print ([x for line in open("input") for x in line.split()])

print(sum([1 for line in open("input") if len(set(line.split())) == len(line.split())]))




print(sum([ 1 for l in [ [ [''.join(sorted(x)) for x in line.split()] for line in open("input") ] ] for j in l if len(set(j)) == len(j)] ))