#!/usr/bin/env python3

from functools import reduce as red
from itertools import takewhile as tW
from itertools import count as ct

fw =  [ [int(i.strip()) for i in x.split(':')] for x in open("input")]

# c = lambda p, r: abs(p - ((p+(r-2))//(2*(r-1))) * (2*(r-1)))
c = lambda p, r: p % ((r-1)*2)
t = lambda d,z:red(lambda sv,i:sv+((c(i[0]+d,i[1]) is 0)*(i[0]*i[1]+z)), fw, 0)

print(f"P1: {t(0,0)}")
print(f"P2: {[i for i in tW(lambda x: t(x,1) is not 0, ct())][-1]+1}")
