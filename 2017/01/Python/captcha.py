#!/usr/bin/env python3

a = input()

print("p1: ", sum( [ int(n) for i, n in enumerate(a) if ( a[i] == a[ (i + 1)             % len(a) ] ) ] ))

print("p2: ", sum( [ int(n) for i, n in enumerate(a) if ( a[i] == a[ (i + int(len(a)/2)) % len(a) ] ) ] ))

##############################
# Author: Rahul Butani       #
# Date:   December 1st, 2017 #
##############################
