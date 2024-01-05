from typing import Dict, List, Set


adj: Dict[str, List[str]] = {}


def add_edge(a: str, b: str):
    adj.setdefault(a, []).append(b)
    adj.setdefault(b, []).append(a)


def rm_edge(a: str, b: str):
    adj[a].remove(b)
    adj[b].remove(a)


for line in open("big.txt"):
    left, *rights = line.split()
    left = left[:-1]
    for right in rights:
        add_edge(left, right)

# visual inspection
rm_edge("xkf", "rcn")
rm_edge("thk", "cms")
rm_edge("dht", "xmv")


def go(seen: Set[str], v: str) -> Set[str]:
    if v in seen:
        return seen
    seen.add(v)
    for vv in adj[v]:
        go(seen, vv)
    return seen


total_nodes = len(adj)
group1_nodes = len(go(set(), "dht"))
group2_nodes = total_nodes - group1_nodes
print("answer: {}", group1_nodes * group2_nodes)
