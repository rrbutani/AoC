#!/usr/bin/env python3

l = [int(i) for x in open("input") for i in x.split()]
# l = [0, 2, 7, 0]
s = set()#set([tuple(l)])

# print(''.join(str(i) + ' ' for i in l))

count = 0

while((''.join(str(i) + ' ' for i in l) in s) == False):
    s.add(''.join(str(i) + ' ' for i in l))

    idx = l.index(max(l))
    cnt = l[idx]
    l[idx] = 0

    while(cnt > 0):
        idx += 1
        cnt -= 1
        l[idx % len(l)] += 1

    print(l)

    count += 1

print(l)
print(count)

matches = 0
count = 0

ll = [int(i) for x in open("input") for i in x.split()]
# ll = [0, 2, 7, 0]
s = set()#set([tuple(l)])

while((''.join(str(i) + ' ' for i in ll) in s) == False):
    s.add(''.join(str(i) + ' ' for i in ll))

    idx = ll.index(max(ll))
    cnt = ll[idx]
    ll[idx] = 0

    while(cnt > 0):
        idx += 1
        cnt -= 1
        ll[idx % len(ll)] += 1

    print(ll, l)
    # 
    if l == ll:
        print("MATCH")

    if l != ll and matches == 1:
        print("INC")
        count += 1

    if l == ll and matches == 0:
        print("SET") 
        matches = 1

print(count+1)