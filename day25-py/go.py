fout = open("big.viz", "w")

print("graph G {", file=fout)
print("  nodesep = 1;", file=fout)

for line in open("big.txt"):
    left, *rights = line.split()
    left = left[:-1]
    for right in rights:
        print(f"  {left} -- {right};", file=fout)

print("}", file=fout)
