# i = input()
# print(sum([int(i[idx]) for idx in range(len(inp)) if (inp[i] == inp[(i+1) % len(inp)])]))

# print(sum( [ int(inp[i]) for i in range(len(inp)) if (inp[i] == inp[(i+(int(len(inp)/2))) % len(inp)] ) ] ))


# print(sum([int(i[idx]) for idx in range(len(inp)) if (inp[i] == inp[(i+1) % len(inp)])]))
a = input()

print("p1: ", sum( [ int(n) for i, n in enumerate(a) if ( a[i] == a[ (i + 1)             % len(a) ] ) ] ))

print("p2: ", sum( [ int(n) for i, n in enumerate(a) if ( a[i] == a[ (i + int(len(a)/2)) % len(a) ] ) ] ))
