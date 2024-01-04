use smallvec::SmallVec;

#[derive(Debug, Clone)]
struct SparseMap {
    start: u32,
    end: u32,
    in_edges: std::collections::HashMap<u32, SmallVec<[(u32, u32); 2]>>,
    out_edges: std::collections::HashMap<u32, SmallVec<[(u32, u32); 2]>>,
}

impl SparseMap {
    fn add_edge(&mut self, from: u32, to: u32, weight: u32) {
        self.out_edges.entry(from).or_default().push((to, weight));
        self.out_edges.entry(to).or_default();
        self.in_edges.entry(to).or_default().push((from, weight));
        self.in_edges.entry(from).or_default();
    }

    #[allow(dead_code)]
    fn remove_edge(&mut self, from: u32, to: u32) {
        self.out_edges
            .get_mut(&from)
            .unwrap()
            .retain(|(id, _)| *id != to);
        self.in_edges
            .get_mut(&to)
            .unwrap()
            .retain(|(id, _)| *id != from);
    }

    fn get_edge(&self, from: u32, to: u32) -> Option<u32> {
        self.out_edges[&from]
            .iter()
            .find(|(id, _)| *id == to)
            .map(|(_, w)| *w)
    }

    fn from_str(s: &str) -> Self {
        let grid = s
            .lines()
            .map(|l| l.trim().as_bytes().to_vec())
            .collect::<Vec<Vec<u8>>>();
        let nrows = grid.len();
        let ncols = grid[0].len();
        let mut map = Self {
            start: u32::MAX,
            end: u32::MAX,
            in_edges: std::collections::HashMap::new(),
            out_edges: std::collections::HashMap::new(),
        };
        let directions = [(0, 1, b'>'), (0, -1, b'<'), (1, 0, b'v'), (-1, 0, b'^')];
        let iter_dirs = |(r, c): (usize, usize)| {
            directions
                .into_iter()
                .filter_map(move |(dr, dc, ch)| -> Option<(usize, usize, u8)> {
                    let (dr, dc): (i64, i64) = (dr.into(), dc.into());
                    let (r, c): (i64, i64) = (r.try_into().unwrap(), c.try_into().unwrap());
                    let (nrows, ncols): (i64, i64) =
                        (nrows.try_into().unwrap(), ncols.try_into().unwrap());
                    let (nr, nc) = (r + dr, c + dc);
                    if 0 <= nr && nr < nrows && 0 <= nc && nc < ncols {
                        Some((nr.try_into().unwrap(), nc.try_into().unwrap(), ch))
                    } else {
                        None
                    }
                })
        };
        for r in 0..grid.len() {
            for c in 0..grid[r].len() {
                let id: u32 = (c + r * ncols).try_into().unwrap();
                if grid[r][c] != b'#' {
                    if r == 0 {
                        map.start = id;
                    }
                    if r == grid.len() - 1 {
                        map.end = id;
                    }
                    for (nr, nc, ch) in iter_dirs((r, c)) {
                        let nid: u32 = (nc + nr * ncols).try_into().unwrap();
                        if grid[r][c] == ch || grid[r][c] == b'.' {
                            map.add_edge(id, nid, 1);
                        }
                    }
                }
            }
        }
        assert!(map.start != u32::MAX);
        assert!(map.end != u32::MAX);
        map
    }

    fn remove_vertex(&mut self, id_to_rm: u32) {
        for (outgoing_id, _) in self.out_edges[&id_to_rm].iter() {
            self.in_edges
                .get_mut(outgoing_id)
                .unwrap()
                .retain(|(id, _)| *id != id_to_rm);
        }
        for (incoming_id, _) in self.in_edges[&id_to_rm].iter() {
            self.out_edges
                .get_mut(incoming_id)
                .unwrap()
                .retain(|(id, _)| *id != id_to_rm);
        }
        self.out_edges.remove(&id_to_rm);
        self.in_edges.remove(&id_to_rm);
    }

    fn maybe_remove_empty(&mut self, id: u32) -> bool {
        if id == self.start || id == self.end {
            return false;
        }
        if self.out_edges[&id].is_empty() {
            self.remove_vertex(id);
            true
        } else {
            false
        }
    }

    fn maybe_simplify_to_edge(&mut self, id: u32) -> Option<(u32, u32)> {
        if id == self.start || id == self.end {
            return None;
        }
        let mut neighs = self.in_edges[&id]
            .iter()
            .chain(self.out_edges[&id].iter())
            .map(|(id, _)| *id)
            .collect::<SmallVec<[u32; 8]>>();
        neighs.sort();
        neighs.dedup();
        match neighs.as_slice() {
            &[id2, id3] => {
                let weight_12 = self.get_edge(id, id2);
                let weight_21 = self.get_edge(id2, id);
                let weight_13 = self.get_edge(id, id3);
                let weight_31 = self.get_edge(id3, id);
                if let (Some(w1), Some(w2)) = (weight_21, weight_13) {
                    self.add_edge(id2, id3, w1 + w2);
                }
                if let (Some(w1), Some(w2)) = (weight_31, weight_12) {
                    self.add_edge(id3, id2, w1 + w2);
                }
                self.remove_vertex(id);
                Some((id2, id3))
            }
            _ => None,
        }
    }

    fn simplify(&mut self) -> bool {
        let mut remaining_ids = self.in_edges.keys().cloned().collect::<Vec<_>>();
        let mut overall_changed = false;
        loop {
            let mut next_remaining_ids = Vec::new();
            let mut changed = false;
            for id in remaining_ids {
                if self.maybe_remove_empty(id) || self.maybe_simplify_to_edge(id).is_some() {
                    changed = true;
                } else {
                    next_remaining_ids.push(id);
                }
            }
            overall_changed |= changed;
            remaining_ids = next_remaining_ids;
            if !changed {
                break;
            }
        }
        overall_changed
    }

    #[allow(dead_code)]
    fn to_graphviz(&self) -> String {
        let mut s = String::new();
        s.push_str("digraph {\n");
        for (id, edges) in self.out_edges.iter() {
            for (id2, weight) in edges.iter() {
                s.push_str(&format!("  N{} -> N{} [label={}]\n", id, id2, weight));
            }
        }
        s.push_str(&format!("  N{} [shape=Mdiamond]\n", self.start));
        s.push_str(&format!("  N{} [shape=Msquare]\n", self.end));
        s.push_str("}\n");
        s
    }

    fn longest_path(&self) -> u32 {
        let mut memo = std::collections::HashMap::<u32, Option<u32>>::new();
        fn go(
            map: &SparseMap,
            memo: &mut std::collections::HashMap<u32, Option<u32>>,
            id: u32,
        ) -> Option<u32> {
            if let Some(&res) = memo.get(&id) {
                return res;
            }
            let res = if id == map.end {
                Some(0)
            } else {
                map.out_edges[&id]
                    .iter()
                    .filter_map(|(id2, weight)| go(map, memo, *id2).map(|w2| weight + w2))
                    .max()
            };
            memo.insert(id, res);
            res
        }
        go(self, &mut memo, self.start).unwrap()
    }
}

struct CompactMap {
    start: u8,
    end: u8,
    nverts: u8,
    in_edges: Vec<SmallVec<[(u8, u32); 4]>>,
    #[allow(dead_code)]
    out_edges: Vec<SmallVec<[(u8, u32); 4]>>,
}

impl CompactMap {
    fn from_sparse_map(map: &SparseMap) -> Self {
        let mut nverts = 0;
        let old_to_new_id_map = map
            .out_edges
            .keys()
            .map(|old_id| {
                let new_id = nverts;
                nverts += 1;
                (*old_id, new_id)
            })
            .collect::<std::collections::HashMap<u32, u8>>();
        let mut in_edges = map
            .in_edges
            .iter()
            .map(|(id, edges)| {
                let new_id = old_to_new_id_map[id];
                let edges = edges
                    .iter()
                    .map(|(id, weight)| (old_to_new_id_map[id], *weight))
                    .collect::<SmallVec<[_; 4]>>();
                (new_id, edges)
            })
            .collect::<Vec<_>>();
        in_edges.sort();
        assert_eq!(
            in_edges.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            (0..nverts).collect::<Vec<_>>()
        );
        let in_edges = in_edges.into_iter().map(|(_, edges)| edges).collect();
        let mut out_edges = map
            .out_edges
            .iter()
            .map(|(id, edges)| {
                let new_id = old_to_new_id_map[id];
                let edges = edges
                    .iter()
                    .map(|(id, weight)| (old_to_new_id_map[id], *weight))
                    .collect::<SmallVec<[_; 4]>>();
                (new_id, edges)
            })
            .collect::<Vec<_>>();
        out_edges.sort();
        assert_eq!(
            out_edges.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            (0..nverts).collect::<Vec<_>>()
        );
        let out_edges = out_edges.into_iter().map(|(_, edges)| edges).collect();
        Self {
            start: old_to_new_id_map[&map.start],
            end: old_to_new_id_map[&map.end],
            nverts,
            in_edges,
            out_edges,
        }
    }

    #[allow(dead_code)]
    fn to_graphviz(&self) -> String {
        let mut s = String::new();
        s.push_str("digraph {\n");
        for (id, edges) in self.out_edges.iter().enumerate() {
            for (id2, weight) in edges.iter() {
                s.push_str(&format!("  N{} -> N{} [label={}]\n", id, id2, weight));
            }
        }
        s.push_str(&format!("  N{} [shape=Mdiamond]\n", self.start));
        s.push_str(&format!("  N{} [shape=Msquare]\n", self.end));
        s.push_str("}\n");
        s
    }

    fn longest_path(&self) -> u32 {
        let mut memo = std::collections::HashMap::<(u64, u8), Option<u32>>::new();
        fn go(
            memo: &mut std::collections::HashMap<(u64, u8), Option<u32>>,
            in_edges: &Vec<SmallVec<[(u8, u32); 4]>>,
            start: u8,
            allowed: u64,
            id: u8,
        ) -> Option<u32> {
            if let Some(&res) = memo.get(&(allowed, id)) {
                return res;
            }
            let res = {
                if id == start {
                    Some(0)
                } else {
                    let mut best = None;
                    let edges: &SmallVec<[(u8, u32); 4]> = &in_edges[usize::from(id)];
                    for (id2, weight) in edges.iter() {
                        let bit = 1u64 << id2;
                        if (allowed & bit) != 0 {
                            if let Some(subanswer) =
                                go(memo, in_edges, start, allowed & !(1u64 << id), *id2)
                            {
                                let candidate = subanswer + *weight;
                                best = match best {
                                    None => Some(candidate),
                                    Some(current) => Some(current.max(candidate)),
                                };
                            }
                        }
                    }
                    best
                }
            };
            memo.insert((allowed, id), res);
            res
        }
        go(
            &mut memo,
            &self.in_edges,
            self.start,
            (1u64 << self.nverts) - 1,
            self.end,
        )
        .unwrap()
    }
}

pub fn part1(s: &str) -> usize {
    let mut map = SparseMap::from_str(s);
    map.simplify();
    // println!("{}", map.to_graphviz());
    map.longest_path().try_into().unwrap()
}

pub fn part2(s: &str) -> usize {
    let mut map = SparseMap::from_str(s);
    map.simplify();
    // println!("{}", map.to_graphviz());
    let mut map = SparseMap::from_str(s.replace(['<', '>', '^', 'v'], ".").as_str());
    map.simplify();
    let map = CompactMap::from_sparse_map(&map);
    // println!("{}", map.to_graphviz());
    map.longest_path().try_into().unwrap()
}

#[cfg(test)]
mod tests {
    const EXAMPLE1: &str = "\
#.#####################
#.......#########...###
#######.#########.#.###
###.....#.>.>.###.#.###
###v#####.#v#.###.#.###
###.>...#.#.#.....#...#
###v###.#.#.#########.#
###...#.#.#.......#...#
#####.#.#.#######.#.###
#.....#.#.#.......#...#
#.#####.#.#.#########v#
#.#...#...#...###...>.#
#.#.#v#######v###.###v#
#...#.>.#...>.>.#.###.#
#####v#.#.###v#.#.###.#
#.....#...#...#.#.#...#
#.#########.###.#.#.###
#...###...#...#...#.###
###.###.#.###v#####v###
#...#...#.#.>.>.#.>.###
#.###.###.#.###.#.#v###
#.....###...###...#...#
#####################.#
";

    #[test]
    fn part1_example() {
        assert_eq!(super::part1(EXAMPLE1), 94);
    }

    #[test]
    fn part1_big() {
        assert_eq!(super::part1(include_str!("big.txt")), 2070);
    }

    #[test]
    fn part2_example() {
        assert_eq!(super::part2(EXAMPLE1), 154);
    }

    #[test]
    fn part2_big() {
        // not 6715
        assert_eq!(super::part2(include_str!("big.txt")), 6498);
    }
}
