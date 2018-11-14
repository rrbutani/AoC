#!/usr/bin/env python3

def program(line):
    name = line[0]
    weight = int(line[1].strip("()"))
    above = []
    
    if(len(line) > 3):
        for i in range(3, len(line)):
            above.append(line[i].strip(','))

    return (name, weight, above)


pgms = [program(i) for i in [ x.split() for x in open("input.txt")] ]

root = 0
for p in pgms:
    root = 0
    pname = p[0]

    for s in pgms:
        aboves = s[2]
        if pname in aboves:
            root = 1
            break
        # print(aboves)
        # print(pname)

    if root is 0:
        root = pname
        print(root)
        break
    # print(pname)

pmap = {}
pint = {}

for p in pgms:
    pmap[p[0]] = (p[1], p[2])
    pint[p[0]] = (p[1], p[2])

# print(root)
# print(pmap[root], "YO")



# throw
def recurseWeights(pname):
    # print(pname)
    if(len(pmap[pname][1]) == 0):
        return

    if(len(pmap[pname][1]) > 0):
        for i in pmap[pname][1]:
            recurseWeights(i)

    # Now that we have flat weights:
    childrenW = []
    for i in pmap[pname][1]:
        # Check that they're all equal:
        # print(i)
        childrenW.append(pmap[i][0])

    if min(childrenW) != max(childrenW):
        print("ON: ", pname)
        print("CHILD: ", pmap[pname][1])
        print(childrenW)
        exit()

    pmap.update({pname : (pmap[pname][0] + sum(childrenW), pmap[pname][1])})

print(pmap['vmttcwe'])

for i in pmap['vmttcwe'][1]:
    # print(i, pmap[i])
    recurseWeights(i)
    print(i, pmap[i])


# recurseWeights(root)

# print(l)
