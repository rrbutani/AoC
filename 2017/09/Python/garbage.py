#!/usr/bin/env python3

# ins = [ x.split() for x in open("input")]
# inp = [ print(c) for c in ins[0] ]

for x in open("input"):
    trash = (list(x))
# 
# throw

score = 0
level = 0

garbage = False
skip = False
# startCount = False
count = 0

for i in trash:
    # print(i, 'a')
    if skip:
        skip = False
        continue

    if i is '!':
        skip = True
        continue

    if i is '>':
        garbage = False

    if garbage:
        print(i)
        count += 1
        continue

    if i is '<':
        garbage = True
        continue


    if i is '{':
        level += 1

    if i is '}':
        score += level
        level -= 1

print(score)
print(count)