#!/usr/bin/env python3

from itertools import combinations as cmbs

print(sum( [max(l) -  min(l) for l in [ [int(x) for x in line.split('\t')] for line in open("input")] ] ))

print(sum( [max(t) // min(t) for l in [ [int(x) for x in line.split('\t')] for line in open("input")] for t in cmbs(l, 2) if not max(t) % min(t) ]))

##############################
# Author: Rahul Butani       #
# Date:   December 2nd, 2017 #
##############################
