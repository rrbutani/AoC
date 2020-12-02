#!/usr/bin/env python3

# algo: find nearest odd square root
# any odd square root has prop dist = (s//2, s//2)

# input as file (input)

import itertools

input = 361527

maxSq = [i for i in itertools.takewhile(lambda x: x*x<input, range(1, 1000001, 2))][-1]

dist_x = -(maxSq // 2)
dist_y = -(maxSq // 2)

remain = input - (maxSq * maxSq)

print(remain)

index = 0
dists = [-(maxSq // 2), (maxSq // 2)] # y, x
signs = [+1, -1, -1, +1]

print(dists)


while(remain > 0):
    dists[index % 2] += signs[index % 4] * (min(remain, maxSq))
    remain -= (min(remain, maxSq))

print(dists)
print(sum(map(abs, dists)))

# print(input - (maxSq * maxSq))


# def generateIndexes(i):


def maxSquare(inp):
    return [i for i in itertools.takewhile(lambda x: x*x<inp, range(1, 1000001, 2))][-1]


mp = [[0 for x in range(1000)] for y in range(1000)] #[[0]*1000]*1000

x = 500
y = 500
i = 1
r = 1
step = 0
down = 2

mp[x][y] = 1

# for xd in range(-1, 2):
#     for yd in range(-1, 2):
#         # print(xd)
#         # if not (xd == 0 and yd == 0):
#             print("    ", x+xd, y+yd)
#             # print(len(mp), len((mp[0])))
#             print("        ", mp[x+xd][y+yd])
#             # mp[x][y] = (mp[x][y] + mp[x+xd][y+yd])

# wreck()

v = 1

rot = [[1, 0], [0, 1], [-1, 0], [0, -1]]

# print(mp)

while v <= input:
    # mp[dists[r%2]]
    for i in range(0, 2):
        for j in range(0, r):
            x = x + rot[step % 4][0]
            y = y + rot[step % 4][1]

            print(x, y)

            for xd in range(-1, 2):
                for yd in range(-1, 2):
                    # print(xd)
                    if not (xd == 0 and yd == 0):
                        print("    ", x+xd, y+yd)
                        # print(len(mp), len((mp[0])))
                        print("        ", mp[x+xd][y+yd])
                        mp[x][y] = (mp[x][y] + mp[x+xd][y+yd])
            v = mp[x][y]

            print(mp[x][y])
        step = step + 1
    r = r + 1

# 1
# 1 2
# 1 3
# 1 4
# 1 4 5
# 1 6
# 1 6 7
# 1 2 8
# 1 9
# 2 9 10
# 2 3 11
# 
# 
# x1
# x2
# +1
# x2
# +1
# x2+1
# +2
# +1
# x2+2
# +3
# 