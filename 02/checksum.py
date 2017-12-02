#!/usr/bin/python3
# import sys


# sum = 0

# inp = sys.stdin.read()

# print(raw_input())

# while(sys.stdin.read()):
#     l = raw_input().split()
#     print(l)
#     sum += max(l) - min(l)
#     inp = sys.stdin.read()

#     print(sum)

# print(sum)
# 
#

import itertools

# with open("in.txt") ad f:

# print([ [ max(t) // min(t) for t in itertools.combinations(l, 2) if not max(t) % min(t) ] for l in [ [int(x) for x in line.split('\t')] for line in open("in.txt") ] ])

# [   ] 
print(sum( [max(l) -  min(l) for l in [ [int(x) for x in line.split('\t')] for line in open("in.txt")] ] ))

print(sum( [max(t) // min(t) for l in [ [int(x) for x in line.split('\t')] for line in open("in.txt")] for t in itertools.combinations(l, 2) if not max(t) % min(t) ]))



# sum('a')


# print(sum( [max(t) // min(t) for t in [itertools.combinations(l, 2) for l in [ [int(x) for x in line.split('\t')] for line in open("in.txt")] ] if not max(t) % min(t)] ))
# [ for t in [itertools.combinations(l, 2) for l in [ [int(x) for x in line.split('\t')] for line in open("in.txt")] ] ]

# max(line.split('\t')) - min(line.split('\t'))

# sum = 0

# while(True):
#     # the_string = input()
#     time.sleep(0.2)

#     lo = input()

#     # print(lo)

#     if(lo == ''):
#         print("YO")
#         break

#     # l = map(int, lo.split('\t'))#lo.split('\t')

#     l = [ int(s) for s in lo.split('\t') ]

#     # print(itertools.combinations(l, 2))

#     for subset in itertools.combinations(l, 2):
#         # print("ay", subset[0], subset[1], subset[0] / subset[1])
#         if (float(max(subset)) / min(subset)) % 1 == 0.0:
#             sum += float(max(subset)) / min(subset)
#             # print("ay", subset[0], subset[1], float(max(subset)) / min(subset))
#             break
#         # print(subset[0], subset[1])
    
#     l/2 * l 

#     # print(l)

#     # print(l, "max: ", max(l), "min: ", min(l))

#     # print("max is ", max(l), " min is ", min(l))

#     # sum += int(max(l)) - int(min(l))


#     # print("NEW ", sum)
# # print(min(l))
# # print(max(l))

# print(sum)

# # print(name)
# # print(age)


# # 5 9 2 8
# # 9 4 7 3
# # 3 8 6 5